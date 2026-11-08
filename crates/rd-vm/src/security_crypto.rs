//! Apêndice D — `androidx.security:security-crypto` (host-shim, ST-04).
//!
//! Modela `EncryptedSharedPreferences.create` e a superfície de
//! `SharedPreferences`/`Editor` que o app usa. Os valores são **cifrados de
//! verdade**: AES-GCM com a chave do AndroidKeyStore (`jkeystore.rs`), IV
//! aleatório por valor e AAD = arquivo + nome da chave, então um valor
//! copiado para outra chave ou arquivo não decifra.
//!
//! Desvios documentados (nunca silenciosos):
//! - O formato não é o do Tink. O que a biblioteca real grava (keyset em
//!   SharedPreferences, AES-SIV para nomes) não é produzido, e os dados não
//!   são compatíveis com um dispositivo real nos dois sentidos.
//! - Só os VALORES são cifrados. Os nomes das chaves ficam em claro na
//!   memória do host; o AES-SIV do Tink cifraria também os nomes.
//! - O estado dura só a sessão do Engine, como o próprio AndroidKeyStore do
//!   host. Nada é gravado no VFS.
//! - `create` é interceptado mesmo com o DEX da biblioteca presente: o
//!   caminho Tink exige JCA/keyset fora do modelo.
//! - A biblioteca tem um único esquema por parâmetro (AES256_SIV para chave,
//!   AES256_GCM para valor). O host aceita qualquer enum não nulo.
//! - O build da chave mestra (`MasterKey.Builder`) roda no DEX sobre o
//!   KeyGenerator/KeyStore do ST-04. `create` exige que o alias exista no
//!   provedor; sem chave, falha tipado (`GeneralSecurityException`).
//! - Editor: `commit`/`apply` aplicam as edições pendentes em ordem (`clear`
//!   antes dos `put` seguintes). Listeners são notificados por chave alterada.
//!
//! Falhas são tipadas: null → NPE, tipo errado → `ClassCastException`
//! (como a biblioteca), AEAD que não confere → `SecurityException`.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::framework::{
    extract_pref_string_set, pref_materialize, pref_string_set_obj, pref_wrong_type, HostObj,
};
use crate::heap::ObjRef;
use crate::intrinsics;
use crate::jkeystore;
use crate::prefs::PrefValue;
use crate::value::Value;

pub const ENCRYPTED_PREFS: &str = "Landroidx/security/crypto/EncryptedSharedPreferences;";
pub const ENCRYPTED_EDITOR: &str = "Landroidx/security/crypto/EncryptedSharedPreferences$Editor;";
pub const MASTER_KEY: &str = "Landroidx/security/crypto/MasterKey;";
/// Alias padrão da biblioteca para a chave mestra (`MasterKey.DEFAULT_MASTER_KEY_ALIAS`).
pub const DEFAULT_MASTER_KEY_ALIAS: &str = "_androidx_security_master_key_";

const CREATE_WITH_KEY: &str = "(Landroid/content/Context;Ljava/lang/String;Landroidx/security/crypto/MasterKey;Landroidx/security/crypto/EncryptedSharedPreferences$PrefKeyEncryptionScheme;Landroidx/security/crypto/EncryptedSharedPreferences$PrefValueEncryptionScheme;)Landroid/content/SharedPreferences;";
const CREATE_WITH_ALIAS: &str = "(Ljava/lang/String;Ljava/lang/String;Landroid/content/Context;Landroidx/security/crypto/EncryptedSharedPreferences$PrefKeyEncryptionScheme;Landroidx/security/crypto/EncryptedSharedPreferences$PrefValueEncryptionScheme;)Landroid/content/SharedPreferences;";
const LISTENER_SIG: &str =
    "(Landroid/content/SharedPreferences$OnSharedPreferenceChangeListener;)V";
const ON_CHANGED_SIG: &str = "(Landroid/content/SharedPreferences;Ljava/lang/String;)V";

const GSE: &str = "Ljava/security/GeneralSecurityException;";
const SEC: &str = "Ljava/lang/SecurityException;";
const NPE: &str = "Ljava/lang/NullPointerException;";
const ISE: &str = "Ljava/lang/IllegalStateException;";

/// Byte de tipo para `null` armazenado (`putString(k, null)`).
const NULL_TAG: u8 = 0xFF;

/// Estado do host por Engine. Tudo aqui é dado puro (sem `ObjRef`), exceto
/// as tabelas de editor e listener, que o GC purga/marca em `gc_host_ephemerons`.
#[derive(Debug, Clone, Default)]
pub struct Tables {
    /// Arquivos por nome: alias da chave mestra e valores cifrados.
    pub(crate) files: BTreeMap<String, EncFile>,
    /// Edições pendentes (antes de `commit`/`apply`) por objeto editor.
    pub(crate) editors: HashMap<ObjRef, Vec<Op>>,
    /// Listeners por objeto `EncryptedSharedPreferences` (raízes do GC).
    pub(crate) listeners: HashMap<ObjRef, Vec<ObjRef>>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct EncFile {
    alias: String,
    entries: BTreeMap<String, Sealed>,
}

/// Valor cifrado: IV de 96 bits + `ciphertext || tag` (AES-GCM).
#[derive(Debug, Clone)]
pub(crate) struct Sealed {
    iv: [u8; 12],
    ct: Vec<u8>,
}

#[derive(Debug, Clone)]
pub(crate) enum Op {
    Put(String, Sealed),
    Remove(String),
    Clear,
}

fn exc(class: &str, msg: impl Into<String>) -> VmExit {
    VmExit::Exception(Throwable::new(class, msg))
}

fn obj_arg(args: &[Value], i: usize, what: &str) -> Result<ObjRef, VmExit> {
    match args.get(i) {
        Some(Value::Obj(r)) => Ok(*r),
        _ => Err(exc(NPE, format!("{what} null"))),
    }
}

fn str_of(vm: &Engine, args: &[Value], i: usize, what: &str) -> Result<String, VmExit> {
    let r = obj_arg(args, i, what)?;
    Ok(vm.heap.as_str(r)?.to_string())
}

/// Nome do arquivo guardado no campo `rd_file` do objeto.
fn file_of(vm: &Engine, prefs: ObjRef) -> Result<String, VmExit> {
    match vm.heap.get_field(prefs, "rd_file") {
        Ok(Value::Obj(s)) => Ok(vm.heap.as_str(s)?.to_string()),
        _ => Err(exc(ISE, "EncryptedSharedPreferences sem arquivo")),
    }
}

fn editor_prefs(vm: &Engine, editor: ObjRef) -> Result<ObjRef, VmExit> {
    match vm.heap.get_field(editor, "rd_prefs") {
        Ok(Value::Obj(p)) => Ok(p),
        _ => Err(exc(ISE, "Editor sem SharedPreferences de origem")),
    }
}

// ── codec dos valores (tipo + payload) ──────────────────────────────────

fn put_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u32).to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}

/// Serializa um valor (`None` = null). O tipo viaja junto, para que um
/// `getInt` sobre um `String` falhe com `ClassCastException`.
fn encode(value: Option<&PrefValue>) -> Vec<u8> {
    let mut out = Vec::new();
    match value {
        None => out.push(NULL_TAG),
        Some(PrefValue::String(s)) => {
            out.push(0);
            put_str(&mut out, s);
        }
        Some(PrefValue::Int(i)) => {
            out.push(1);
            out.extend_from_slice(&i.to_be_bytes());
        }
        Some(PrefValue::Long(l)) => {
            out.push(2);
            out.extend_from_slice(&l.to_be_bytes());
        }
        Some(PrefValue::Float(bits)) => {
            out.push(3);
            out.extend_from_slice(&bits.to_be_bytes());
        }
        Some(PrefValue::Bool(b)) => {
            out.push(4);
            out.push(*b as u8);
        }
        Some(PrefValue::StringSet(set)) => {
            out.push(5);
            out.extend_from_slice(&(set.len() as u32).to_be_bytes());
            for item in set {
                match item {
                    Some(s) => {
                        out.push(1);
                        put_str(&mut out, s);
                    }
                    None => out.push(0),
                }
            }
        }
    }
    out
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn take(&mut self, n: usize) -> Option<&[u8]> {
        let end = self.at.checked_add(n)?;
        let s = self.b.get(self.at..end)?;
        self.at = end;
        Some(s)
    }
    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
    fn array<const N: usize>(&mut self) -> Option<[u8; N]> {
        let mut a = [0u8; N];
        a.copy_from_slice(self.take(N)?);
        Some(a)
    }
    fn string(&mut self) -> Option<String> {
        let n = u32::from_be_bytes(self.array()?) as usize;
        String::from_utf8(self.take(n)?.to_vec()).ok()
    }
}

/// Inverso de `encode`. Externo `None` = payload corrompido; interno `None` = null.
fn decode(bytes: &[u8]) -> Option<Option<PrefValue>> {
    let mut r = Reader { b: bytes, at: 0 };
    let value = match r.u8()? {
        NULL_TAG => None,
        0 => Some(PrefValue::String(r.string()?)),
        1 => Some(PrefValue::Int(i32::from_be_bytes(r.array()?))),
        2 => Some(PrefValue::Long(i64::from_be_bytes(r.array()?))),
        3 => Some(PrefValue::Float(u32::from_be_bytes(r.array()?))),
        4 => Some(PrefValue::Bool(r.u8()? != 0)),
        5 => {
            let n = u32::from_be_bytes(r.array()?);
            let mut set = BTreeSet::new();
            for _ in 0..n {
                set.insert(match r.u8()? {
                    0 => None,
                    1 => Some(r.string()?),
                    _ => return None,
                });
            }
            Some(PrefValue::StringSet(set))
        }
        _ => return None,
    };
    (r.at == bytes.len()).then_some(value)
}

// ── cifragem (AES-GCM via AndroidKeyStore) ──────────────────────────────

/// AAD = arquivo (com prefixo de tamanho) + nome da chave.
fn aad(file: &str, key: &str) -> Vec<u8> {
    let mut a = Vec::with_capacity(4 + file.len() + key.len());
    a.extend_from_slice(&(file.len() as u32).to_be_bytes());
    a.extend_from_slice(file.as_bytes());
    a.extend_from_slice(key.as_bytes());
    a
}

fn seal(vm: &Engine, alias: &str, file: &str, key: &str, plain: &[u8]) -> Result<Sealed, VmExit> {
    let material = vm.fw.keystore.keys.get(alias).ok_or_else(|| {
        exc(
            GSE,
            format!("chave mestra '{alias}' ausente no AndroidKeyStore"),
        )
    })?;
    let mut iv = [0u8; 12];
    iv.copy_from_slice(&jkeystore::random_bytes(12)?);
    let ct = jkeystore::gcm_seal(material, &iv, &aad(file, key), plain)
        .ok_or_else(|| exc(GSE, format!("falha ao cifrar '{key}'")))?;
    Ok(Sealed { iv, ct })
}

fn open(
    vm: &Engine,
    alias: &str,
    file: &str,
    key: &str,
    sealed: &Sealed,
) -> Result<Vec<u8>, VmExit> {
    let material = vm.fw.keystore.keys.get(alias).ok_or_else(|| {
        exc(
            SEC,
            format!("chave mestra '{alias}' ausente; '{key}' indecifrável"),
        )
    })?;
    jkeystore::gcm_open(material, &sealed.iv, &aad(file, key), &sealed.ct)
        .ok_or_else(|| exc(SEC, format!("falha de autenticação ao decifrar '{key}'")))
}

// ── create ──────────────────────────────────────────────────────────────

/// Só as duas assinaturas públicas de `EncryptedSharedPreferences.create`.
pub fn is_create_sig(sig: &str) -> bool {
    sig == CREATE_WITH_KEY || sig == CREATE_WITH_ALIAS
}

/// `create(Context, fileName, MasterKey, keyScheme, valueScheme)` ou o
/// deprecado `create(fileName, masterKeyAlias, Context, keyScheme, valueScheme)`.
pub fn create(vm: &mut Engine, sig: &str, args: &[Value]) -> Result<Value, VmExit> {
    let (file, alias) = if sig == CREATE_WITH_KEY {
        obj_arg(args, 0, "context")?;
        let file = str_of(vm, args, 1, "fileName")?;
        let mk = obj_arg(args, 2, "masterKey")?;
        let alias = match vm.heap.get_field(mk, "mKeyAlias") {
            Ok(Value::Obj(s)) => vm.heap.as_str(s)?.to_string(),
            _ => return Err(exc(NPE, "masterKey.mKeyAlias null")),
        };
        (file, alias)
    } else {
        let file = str_of(vm, args, 0, "fileName")?;
        let alias = str_of(vm, args, 1, "masterKeyAlias")?;
        obj_arg(args, 2, "context")?;
        (file, alias)
    };
    obj_arg(args, 3, "prefKeyEncryptionScheme")?;
    obj_arg(args, 4, "prefValueEncryptionScheme")?;

    if !vm.fw.keystore.keys.contains_key(&alias) {
        return Err(exc(
            GSE,
            format!("chave mestra '{alias}' ausente no AndroidKeyStore"),
        ));
    }
    match vm.fw.androidx.enc.files.get(&file) {
        Some(f) if f.alias != alias => {
            return Err(exc(
                GSE,
                format!("arquivo '{file}' foi criado com outra chave mestra"),
            ))
        }
        Some(_) => {}
        None => {
            vm.fw.androidx.enc.files.insert(
                file.clone(),
                EncFile {
                    alias,
                    entries: BTreeMap::new(),
                },
            );
        }
    }
    let name = intrinsics::alloc_string(vm, file)?;
    let r = vm
        .heap
        .alloc_instance(
            ENCRYPTED_PREFS.to_string(),
            vec![("rd_file".into(), Value::Obj(name))],
        )
        .map_err(crate::err::oom_throwable)?;
    Ok(Value::Obj(r))
}

// ── leitura ─────────────────────────────────────────────────────────────

/// `None` = chave ausente; `Some(None)` = null armazenado.
fn read(vm: &Engine, prefs: ObjRef, key: &str) -> Result<Option<Option<PrefValue>>, VmExit> {
    let file = file_of(vm, prefs)?;
    let Some(f) = vm.fw.androidx.enc.files.get(&file) else {
        return Ok(None);
    };
    let Some(sealed) = f.entries.get(key) else {
        return Ok(None);
    };
    let plain = open(vm, &f.alias, &file, key, sealed)?;
    decode(&plain)
        .map(Some)
        .ok_or_else(|| exc(SEC, format!("valor de '{key}' corrompido")))
}

fn lookup(
    vm: &Engine,
    recv: ObjRef,
    args: &[Value],
) -> Result<(String, Option<Option<PrefValue>>), VmExit> {
    let key = str_of(vm, args, 0, "key")?;
    let value = read(vm, recv, &key)?;
    Ok((key, value))
}

fn default_of(args: &[Value]) -> Value {
    args.get(1).cloned().unwrap_or(Value::Null)
}

fn get_all(vm: &mut Engine, recv: ObjRef) -> Result<Value, VmExit> {
    let file = file_of(vm, recv)?;
    let keys: Vec<String> = vm
        .fw
        .androidx
        .enc
        .files
        .get(&file)
        .map(|f| f.entries.keys().cloned().collect())
        .unwrap_or_default();
    let mut entries = Vec::with_capacity(keys.len());
    for key in keys {
        let Some(value) = read(vm, recv, &key)? else {
            continue;
        };
        let key_obj = intrinsics::alloc_string(vm, key)?;
        let value = match value {
            None => Value::Null,
            Some(pv) => pref_materialize(vm, &pv)?,
        };
        entries.push((Value::Obj(key_obj), value));
    }
    let map = vm
        .heap
        .alloc_instance("Ljava/util/HashMap;".to_string(), Vec::new())
        .map_err(crate::err::oom_throwable)?;
    vm.fw.objects.insert(map, HostObj::Map { entries });
    Ok(Value::Obj(map))
}

fn prefs_call(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match (name, sig) {
        ("getAll", "()Ljava/util/Map;") => get_all(vm, recv).map(Some),
        ("getString", "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;") => {
            let (key, v) = lookup(vm, recv, args)?;
            Ok(Some(match v {
                None => default_of(args),
                Some(None) => Value::Null,
                Some(Some(PrefValue::String(s))) => Value::Obj(intrinsics::alloc_string(vm, s)?),
                Some(Some(other)) => return Err(pref_wrong_type(&key, "String", &other)),
            }))
        }
        ("getStringSet", "(Ljava/lang/String;Ljava/util/Set;)Ljava/util/Set;") => {
            let (key, v) = lookup(vm, recv, args)?;
            Ok(Some(match v {
                None => default_of(args),
                Some(None) => Value::Null,
                Some(Some(PrefValue::StringSet(set))) => Value::Obj(pref_string_set_obj(vm, &set)?),
                Some(Some(other)) => return Err(pref_wrong_type(&key, "Set<String>", &other)),
            }))
        }
        ("getInt", "(Ljava/lang/String;I)I") => {
            let (key, v) = lookup(vm, recv, args)?;
            Ok(Some(match v {
                None | Some(None) => default_of(args),
                Some(Some(PrefValue::Int(i))) => Value::Int(i),
                Some(Some(other)) => return Err(pref_wrong_type(&key, "Integer", &other)),
            }))
        }
        ("getLong", "(Ljava/lang/String;J)J") => {
            let (key, v) = lookup(vm, recv, args)?;
            Ok(Some(match v {
                None | Some(None) => default_of(args),
                Some(Some(PrefValue::Long(l))) => Value::Long(l),
                Some(Some(other)) => return Err(pref_wrong_type(&key, "Long", &other)),
            }))
        }
        ("getFloat", "(Ljava/lang/String;F)F") => {
            let (key, v) = lookup(vm, recv, args)?;
            Ok(Some(match v {
                None | Some(None) => default_of(args),
                Some(Some(PrefValue::Float(bits))) => Value::Float(f32::from_bits(bits)),
                Some(Some(other)) => return Err(pref_wrong_type(&key, "Float", &other)),
            }))
        }
        ("getBoolean", "(Ljava/lang/String;Z)Z") => {
            let (key, v) = lookup(vm, recv, args)?;
            Ok(Some(match v {
                None | Some(None) => default_of(args),
                Some(Some(PrefValue::Bool(b))) => Value::Int(b as i32),
                Some(Some(other)) => return Err(pref_wrong_type(&key, "Boolean", &other)),
            }))
        }
        ("contains", "(Ljava/lang/String;)Z") => {
            let key = str_of(vm, args, 0, "key")?;
            let file = file_of(vm, recv)?;
            let has = vm
                .fw
                .androidx
                .enc
                .files
                .get(&file)
                .is_some_and(|f| f.entries.contains_key(&key));
            Ok(Some(Value::Int(has as i32)))
        }
        ("edit", "()Landroid/content/SharedPreferences$Editor;") => {
            let editor = vm
                .heap
                .alloc_instance(
                    ENCRYPTED_EDITOR.to_string(),
                    vec![("rd_prefs".into(), Value::Obj(recv))],
                )
                .map_err(crate::err::oom_throwable)?;
            vm.fw.androidx.enc.editors.insert(editor, Vec::new());
            Ok(Some(Value::Obj(editor)))
        }
        ("registerOnSharedPreferenceChangeListener", LISTENER_SIG) => {
            let listener = obj_arg(args, 0, "listener")?;
            let list = vm.fw.androidx.enc.listeners.entry(recv).or_default();
            if !list.contains(&listener) {
                list.push(listener);
            }
            Ok(Some(Value::Null))
        }
        ("unregisterOnSharedPreferenceChangeListener", LISTENER_SIG) => {
            let listener = obj_arg(args, 0, "listener")?;
            if let Some(list) = vm.fw.androidx.enc.listeners.get_mut(&recv) {
                list.retain(|l| *l != listener);
            }
            Ok(Some(Value::Null))
        }
        _ => Ok(None),
    }
}

// ── editor ──────────────────────────────────────────────────────────────

fn stage(vm: &mut Engine, editor: ObjRef, op: Op) {
    vm.fw
        .androidx
        .enc
        .editors
        .entry(editor)
        .or_default()
        .push(op);
}

/// Sela `plain` e deixa a escrita pendente até `commit`/`apply`.
fn put(
    vm: &mut Engine,
    editor: ObjRef,
    key: String,
    plain: Vec<u8>,
) -> Result<Option<Value>, VmExit> {
    let prefs = editor_prefs(vm, editor)?;
    let file = file_of(vm, prefs)?;
    let alias = vm
        .fw
        .androidx
        .enc
        .files
        .get(&file)
        .map(|f| f.alias.clone())
        .ok_or_else(|| exc(ISE, format!("arquivo '{file}' ausente")))?;
    let sealed = seal(vm, &alias, &file, &key, &plain)?;
    stage(vm, editor, Op::Put(key, sealed));
    Ok(Some(Value::Obj(editor)))
}

/// Aplica as edições pendentes e notifica listeners por chave alterada.
fn apply(vm: &mut Engine, editor: ObjRef) -> Result<(), VmExit> {
    let prefs = editor_prefs(vm, editor)?;
    let file = file_of(vm, prefs)?;
    let ops = std::mem::take(vm.fw.androidx.enc.editors.entry(editor).or_default());
    let mut changed: BTreeSet<String> = BTreeSet::new();
    {
        let f = vm
            .fw
            .androidx
            .enc
            .files
            .get_mut(&file)
            .ok_or_else(|| exc(ISE, format!("arquivo '{file}' ausente")))?;
        for op in ops {
            match op {
                Op::Put(k, sealed) => {
                    f.entries.insert(k.clone(), sealed);
                    changed.insert(k);
                }
                Op::Remove(k) => {
                    if f.entries.remove(&k).is_some() {
                        changed.insert(k);
                    }
                }
                Op::Clear => {
                    changed.extend(f.entries.keys().cloned());
                    f.entries.clear();
                }
            }
        }
    }
    let listeners = vm
        .fw
        .androidx
        .enc
        .listeners
        .get(&prefs)
        .cloned()
        .unwrap_or_default();
    for key in changed {
        for listener in &listeners {
            let k = intrinsics::alloc_string(vm, key.clone())?;
            crate::m8::call_dex(
                vm,
                *listener,
                "onSharedPreferenceChanged",
                ON_CHANGED_SIG,
                vec![Value::Obj(prefs), Value::Obj(k)],
            )?;
        }
    }
    Ok(())
}

fn editor_call(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match (name, sig) {
        (
            "putString",
            "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;",
        ) => {
            let key = str_of(vm, args, 0, "key")?;
            let value = match args.get(1) {
                Some(Value::Obj(s)) => Some(PrefValue::String(vm.heap.as_str(*s)?.to_string())),
                _ => None,
            };
            put(vm, recv, key, encode(value.as_ref()))
        }
        (
            "putStringSet",
            "(Ljava/lang/String;Ljava/util/Set;)Landroid/content/SharedPreferences$Editor;",
        ) => {
            let key = str_of(vm, args, 0, "key")?;
            let value = extract_pref_string_set(vm, args.get(1))?;
            put(vm, recv, key, encode(value.as_ref()))
        }
        ("putInt", "(Ljava/lang/String;I)Landroid/content/SharedPreferences$Editor;") => {
            let key = str_of(vm, args, 0, "key")?;
            let v = args.get(1).map(|v| v.as_int()).transpose()?.unwrap_or(0);
            put(vm, recv, key, encode(Some(&PrefValue::Int(v))))
        }
        ("putLong", "(Ljava/lang/String;J)Landroid/content/SharedPreferences$Editor;") => {
            let key = str_of(vm, args, 0, "key")?;
            let v = args.get(1).map(|v| v.as_long()).transpose()?.unwrap_or(0);
            put(vm, recv, key, encode(Some(&PrefValue::Long(v))))
        }
        ("putFloat", "(Ljava/lang/String;F)Landroid/content/SharedPreferences$Editor;") => {
            let key = str_of(vm, args, 0, "key")?;
            let v = args
                .get(1)
                .map(|v| v.as_float())
                .transpose()?
                .unwrap_or(0.0);
            put(vm, recv, key, encode(Some(&PrefValue::Float(v.to_bits()))))
        }
        ("putBoolean", "(Ljava/lang/String;Z)Landroid/content/SharedPreferences$Editor;") => {
            let key = str_of(vm, args, 0, "key")?;
            let v = args.get(1).map(|v| v.as_int()).transpose()?.unwrap_or(0) != 0;
            put(vm, recv, key, encode(Some(&PrefValue::Bool(v))))
        }
        ("remove", "(Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;") => {
            let key = str_of(vm, args, 0, "key")?;
            stage(vm, recv, Op::Remove(key));
            Ok(Some(Value::Obj(recv)))
        }
        ("clear", "()Landroid/content/SharedPreferences$Editor;") => {
            stage(vm, recv, Op::Clear);
            Ok(Some(Value::Obj(recv)))
        }
        ("commit", "()Z") => {
            apply(vm, recv)?;
            Ok(Some(Value::Int(1)))
        }
        ("apply", "()V") => {
            apply(vm, recv)?;
            Ok(Some(Value::Null))
        }
        _ => Ok(None),
    }
}

/// Dispatch de instância: `recv_class` é a classe do objeto host
/// (`EncryptedSharedPreferences` ou o seu `Editor`).
pub fn call_instance(
    vm: &mut Engine,
    recv_class: &str,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if recv_class == ENCRYPTED_EDITOR {
        editor_call(vm, name, sig, recv, args)
    } else {
        prefs_call(vm, name, sig, recv, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::HostObj;

    const PREFS_IFACE: &str = "Landroid/content/SharedPreferences;";
    const EDITOR_IFACE: &str = "Landroid/content/SharedPreferences$Editor;";
    const KEYSTORE: &str = "Ljava/security/KeyStore;";
    const KEY_GENERATOR: &str = "Ljavax/crypto/KeyGenerator;";
    const KEY_GEN_SPEC: &str = "Landroid/security/keystore/KeyGenParameterSpec;";
    const PROVIDER: &str = "AndroidKeyStore";
    const FILE: &str = "segredos";

    fn vm() -> Engine {
        Engine::new(vec![], crate::VmConfig::default())
    }

    fn js(vm: &mut Engine, s: &str) -> Value {
        Value::Obj(intrinsics::alloc_string(vm, s.to_string()).unwrap())
    }

    fn obj(v: Option<Value>) -> ObjRef {
        match v {
            Some(Value::Obj(r)) => r,
            other => panic!("esperava objeto, veio {other:?}"),
        }
    }

    fn text(vm: &Engine, v: Option<Value>) -> Option<String> {
        match v {
            Some(Value::Obj(r)) => Some(vm.heap.as_str(r).unwrap().to_string()),
            Some(Value::Null) | None => None,
            other => panic!("esperava String, veio {other:?}"),
        }
    }

    /// Chave do AndroidKeyStore pelo mesmo caminho do ST-04: KeyStore, spec,
    /// KeyGenerator AES e generateKey. Nada é inserido direto na tabela.
    fn make_key(vm: &mut Engine, alias: &str) {
        let p = js(vm, PROVIDER);
        let _ks = obj(jkeystore::static_intrinsic(vm, KEYSTORE, "getInstance", &[p]).unwrap());
        let alias_v = js(vm, alias);
        let spec = vm
            .heap
            .alloc_instance(
                KEY_GEN_SPEC.to_string(),
                vec![
                    ("mKeystoreAlias".into(), alias_v),
                    ("mKeySize".into(), Value::Int(256)),
                ],
            )
            .unwrap();
        let aes = js(vm, "AES");
        let p2 = js(vm, PROVIDER);
        let kg =
            obj(jkeystore::static_intrinsic(vm, KEY_GENERATOR, "getInstance", &[aes, p2]).unwrap());
        jkeystore::instance_intrinsic(
            vm,
            KEY_GENERATOR,
            "init",
            "(Ljava/security/spec/AlgorithmParameterSpec;)V",
            kg,
            &[Value::Obj(spec)],
        )
        .unwrap();
        jkeystore::instance_intrinsic(
            vm,
            KEY_GENERATOR,
            "generateKey",
            "()Ljavax/crypto/SecretKey;",
            kg,
            &[],
        )
        .unwrap();
        assert!(vm.fw.keystore.keys.contains_key(alias));
    }

    /// `create` pela API pública de dispatch (`androidx::call_static`),
    /// com a assinatura deprecada (alias em string).
    fn create_file(vm: &mut Engine, file: &str, alias: &str) -> ObjRef {
        let f = js(vm, file);
        let a = js(vm, alias);
        let ctx = obj(Some(Value::Obj(
            vm.heap
                .alloc_instance("Landroid/content/Context;".into(), vec![])
                .unwrap(),
        )));
        let k_scheme = Value::Obj(
            vm.heap
                .alloc_instance("Ljava/lang/Object;".into(), vec![])
                .unwrap(),
        );
        let v_scheme = Value::Obj(
            vm.heap
                .alloc_instance("Ljava/lang/Object;".into(), vec![])
                .unwrap(),
        );
        let r = crate::androidx::call_static(
            vm,
            ENCRYPTED_PREFS,
            "create",
            CREATE_WITH_ALIAS,
            &[f, a, Value::Obj(ctx), k_scheme, v_scheme],
        );
        obj(r.unwrap())
    }

    fn get(
        vm: &mut Engine,
        prefs: ObjRef,
        name: &str,
        sig: &str,
        args: &[Value],
    ) -> Result<Option<Value>, VmExit> {
        crate::androidx::call_instance(vm, PREFS_IFACE, name, sig, prefs, args)
    }

    fn edit(vm: &mut Engine, prefs: ObjRef) -> ObjRef {
        obj(get(
            vm,
            prefs,
            "edit",
            "()Landroid/content/SharedPreferences$Editor;",
            &[],
        )
        .unwrap())
    }

    fn editor(
        vm: &mut Engine,
        e: ObjRef,
        name: &str,
        sig: &str,
        args: &[Value],
    ) -> Result<Option<Value>, VmExit> {
        crate::androidx::call_instance(vm, EDITOR_IFACE, name, sig, e, args)
    }

    fn setup() -> (Engine, ObjRef) {
        let mut vm = vm();
        make_key(&mut vm, "chave.mestra");
        let prefs = create_file(&mut vm, FILE, "chave.mestra");
        (vm, prefs)
    }

    fn put_string(vm: &mut Engine, e: ObjRef, key: &str, value: &str) {
        let k = js(vm, key);
        let v = js(vm, value);
        editor(
            vm,
            e,
            "putString",
            "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;",
            &[k, v],
        )
        .unwrap();
    }

    fn put_int(vm: &mut Engine, e: ObjRef, key: &str, value: i32) {
        let k = js(vm, key);
        editor(
            vm,
            e,
            "putInt",
            "(Ljava/lang/String;I)Landroid/content/SharedPreferences$Editor;",
            &[k, Value::Int(value)],
        )
        .unwrap();
    }

    fn get_string(vm: &mut Engine, prefs: ObjRef, key: &str, default: &str) -> Option<String> {
        let k = js(vm, key);
        let d = js(vm, default);
        let v = get(
            vm,
            prefs,
            "getString",
            "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            &[k, d],
        )
        .unwrap();
        text(vm, v)
    }

    fn map_len(vm: &Engine, map: ObjRef) -> usize {
        match vm.fw.objects.get(&map) {
            Some(HostObj::Map { entries }) => entries.len(),
            other => panic!("esperava HostObj::Map, veio {other:?}"),
        }
    }

    // ── codec puro ──────────────────────────────────────────────────────

    #[test]
    fn codec_roundtrips_every_type_and_rejects_garbage() {
        let set: BTreeSet<Option<String>> = [Some("a".to_string()), None, Some("é".to_string())]
            .into_iter()
            .collect();
        let cases = vec![
            None,
            Some(PrefValue::String("olá".into())),
            Some(PrefValue::Int(-7)),
            Some(PrefValue::Long(i64::MIN)),
            Some(PrefValue::Float(1.5f32.to_bits())),
            Some(PrefValue::Bool(true)),
            Some(PrefValue::StringSet(set)),
        ];
        for case in cases {
            let bytes = encode(case.as_ref());
            assert_eq!(decode(&bytes), Some(case.clone()), "caso {case:?}");
        }
        assert_eq!(decode(&[]), None);
        assert_eq!(decode(&[0x09]), None, "tipo desconhecido");
        let mut ok = encode(Some(&PrefValue::Int(1)));
        ok.push(0);
        assert_eq!(decode(&ok), None, "bytes sobrando");
        assert_eq!(
            decode(&encode(Some(&PrefValue::String("abc".into())))[..3]),
            None,
            "truncado"
        );
    }

    // ── create ──────────────────────────────────────────────────────────

    #[test]
    fn create_fails_typed_when_master_key_is_missing() {
        let mut vm = vm();
        let f = js(&mut vm, FILE);
        let a = js(&mut vm, "nao.existe");
        let ctx = Value::Obj(
            vm.heap
                .alloc_instance("Landroid/content/Context;".into(), vec![])
                .unwrap(),
        );
        let s = Value::Obj(
            vm.heap
                .alloc_instance("Ljava/lang/Object;".into(), vec![])
                .unwrap(),
        );
        let err = crate::androidx::call_static(
            &mut vm,
            ENCRYPTED_PREFS,
            "create",
            CREATE_WITH_ALIAS,
            &[f, a, ctx, s.clone(), s],
        )
        .unwrap_err();
        match err {
            VmExit::Exception(t) => assert_eq!(t.class, GSE),
            other => panic!("esperava GeneralSecurityException, veio {other:?}"),
        }
    }

    #[test]
    fn create_rejects_null_arguments_with_npe() {
        let mut vm = vm();
        make_key(&mut vm, "chave.mestra");
        let a = js(&mut vm, "chave.mestra");
        let s = Value::Obj(
            vm.heap
                .alloc_instance("Ljava/lang/Object;".into(), vec![])
                .unwrap(),
        );
        let err = crate::androidx::call_static(
            &mut vm,
            ENCRYPTED_PREFS,
            "create",
            CREATE_WITH_ALIAS,
            &[Value::Null, a, s.clone(), s.clone(), s],
        )
        .unwrap_err();
        match err {
            VmExit::Exception(t) => assert_eq!(t.class, NPE),
            other => panic!("esperava NPE, veio {other:?}"),
        }
    }

    #[test]
    fn file_stays_bound_to_its_master_key_and_is_shared_by_instances() {
        let (mut vm, prefs) = setup();
        let e0 = edit(&mut vm, prefs);
        put_string(&mut vm, e0, "token", "abc");
        editor(&mut vm, e0, "commit", "()Z", &[]).unwrap();

        // Outra instância do mesmo arquivo com a mesma chave enxerga o valor.
        let again = create_file(&mut vm, FILE, "chave.mestra");
        assert_eq!(
            get_string(&mut vm, again, "token", "x").as_deref(),
            Some("abc")
        );

        // Outra chave mestra no mesmo arquivo é recusada.
        make_key(&mut vm, "outra.chave");
        let f = js(&mut vm, FILE);
        let a = js(&mut vm, "outra.chave");
        let ctx = Value::Obj(
            vm.heap
                .alloc_instance("Landroid/content/Context;".into(), vec![])
                .unwrap(),
        );
        let s = Value::Obj(
            vm.heap
                .alloc_instance("Ljava/lang/Object;".into(), vec![])
                .unwrap(),
        );
        let err = crate::androidx::call_static(
            &mut vm,
            ENCRYPTED_PREFS,
            "create",
            CREATE_WITH_ALIAS,
            &[f, a, ctx, s.clone(), s],
        )
        .unwrap_err();
        assert!(matches!(err, VmExit::Exception(t) if t.class == GSE));
    }

    // ── escrita, commit e leitura ───────────────────────────────────────

    #[test]
    fn typed_values_round_trip_through_commit() {
        let (mut vm, prefs) = setup();
        let e = edit(&mut vm, prefs);
        put_string(&mut vm, e, "nome", "RustyDroid");
        put_int(&mut vm, e, "n", 42);
        let k = js(&mut vm, "longo");
        editor(
            &mut vm,
            e,
            "putLong",
            "(Ljava/lang/String;J)Landroid/content/SharedPreferences$Editor;",
            &[k, Value::Long(-9_000_000_000)],
        )
        .unwrap();
        let k = js(&mut vm, "pi");
        editor(
            &mut vm,
            e,
            "putFloat",
            "(Ljava/lang/String;F)Landroid/content/SharedPreferences$Editor;",
            &[k, Value::Float(3.25)],
        )
        .unwrap();
        let k = js(&mut vm, "ativo");
        editor(
            &mut vm,
            e,
            "putBoolean",
            "(Ljava/lang/String;Z)Landroid/content/SharedPreferences$Editor;",
            &[k, Value::Int(1)],
        )
        .unwrap();
        let k = js(&mut vm, "tags");
        let set = obj(Some(
            crate::framework::pref_string_set_obj(
                &mut vm,
                &[Some("a".to_string()), None].into_iter().collect(),
            )
            .map(Value::Obj)
            .unwrap(),
        ));
        editor(
            &mut vm,
            e,
            "putStringSet",
            "(Ljava/lang/String;Ljava/util/Set;)Landroid/content/SharedPreferences$Editor;",
            &[k, Value::Obj(set)],
        )
        .unwrap();
        let committed = editor(&mut vm, e, "commit", "()Z", &[]).unwrap();
        assert_eq!(committed, Some(Value::Int(1)));

        assert_eq!(
            get_string(&mut vm, prefs, "nome", "x").as_deref(),
            Some("RustyDroid")
        );
        let key_arg = js_(&mut vm, "n");
        let d = get(
            &mut vm,
            prefs,
            "getInt",
            "(Ljava/lang/String;I)I",
            &[key_arg, Value::Int(0)],
        )
        .unwrap();
        assert_eq!(d, Some(Value::Int(42)));
        let key_arg = js_(&mut vm, "longo");
        let d = get(
            &mut vm,
            prefs,
            "getLong",
            "(Ljava/lang/String;J)J",
            &[key_arg, Value::Long(0)],
        )
        .unwrap();
        assert_eq!(d, Some(Value::Long(-9_000_000_000)));
        let key_arg = js_(&mut vm, "pi");
        let d = get(
            &mut vm,
            prefs,
            "getFloat",
            "(Ljava/lang/String;F)F",
            &[key_arg, Value::Float(0.0)],
        )
        .unwrap();
        assert_eq!(d, Some(Value::Float(3.25)));
        let key_arg = js_(&mut vm, "ativo");
        let d = get(
            &mut vm,
            prefs,
            "getBoolean",
            "(Ljava/lang/String;Z)Z",
            &[key_arg, Value::Int(0)],
        )
        .unwrap();
        assert_eq!(d, Some(Value::Int(1)));
        let key_arg = js_(&mut vm, "tags");
        let set_v = get(
            &mut vm,
            prefs,
            "getStringSet",
            "(Ljava/lang/String;Ljava/util/Set;)Ljava/util/Set;",
            &[key_arg, Value::Null],
        )
        .unwrap();
        let set_ref = obj(set_v);
        let items =
            crate::framework::extract_pref_string_set(&mut vm, Some(&Value::Obj(set_ref))).unwrap();
        assert_eq!(
            items,
            Some(PrefValue::StringSet(
                [Some("a".to_string()), None].into_iter().collect()
            ))
        );
    }

    fn js_(vm: &mut Engine, s: &str) -> Value {
        js(vm, s)
    }

    #[test]
    fn values_are_ciphertext_in_host_state_and_iv_is_fresh() {
        let (mut vm, prefs) = setup();
        let e = edit(&mut vm, prefs);
        put_string(&mut vm, e, "senha", "segredo-muito-especifico");
        put_string(&mut vm, e, "copia", "segredo-muito-especifico");
        editor(&mut vm, e, "commit", "()Z", &[]).unwrap();
        let f = vm.fw.androidx.enc.files.get(FILE).unwrap();
        let a = &f.entries["senha"];
        let b = &f.entries["copia"];
        let needle = b"segredo-muito-especifico";
        assert!(
            !a.ct.windows(needle.len()).any(|w| w == needle),
            "texto claro no estado"
        );
        assert_ne!(a.iv, b.iv, "IV repetido");
        assert_ne!(a.ct, b.ct, "mesmo texto cifrado para valores iguais");
    }

    #[test]
    fn uncommitted_edits_are_invisible_until_commit() {
        let (mut vm, prefs) = setup();
        let e = edit(&mut vm, prefs);
        put_int(&mut vm, e, "contador", 5);
        let key_arg = js_(&mut vm, "contador");
        let d = get(
            &mut vm,
            prefs,
            "getInt",
            "(Ljava/lang/String;I)I",
            &[key_arg, Value::Int(-1)],
        )
        .unwrap();
        assert_eq!(
            d,
            Some(Value::Int(-1)),
            "edição pendente visível antes do commit"
        );
        editor(&mut vm, e, "apply", "()V", &[]).unwrap();
        let key_arg = js_(&mut vm, "contador");
        let d = get(
            &mut vm,
            prefs,
            "getInt",
            "(Ljava/lang/String;I)I",
            &[key_arg, Value::Int(-1)],
        )
        .unwrap();
        assert_eq!(d, Some(Value::Int(5)));
    }

    #[test]
    fn clear_then_put_in_one_editor_keeps_only_the_new_value() {
        let (mut vm, prefs) = setup();
        let e = edit(&mut vm, prefs);
        put_string(&mut vm, e, "antigo", "v1");
        editor(&mut vm, e, "commit", "()Z", &[]).unwrap();

        let e = edit(&mut vm, prefs);
        editor(
            &mut vm,
            e,
            "clear",
            "()Landroid/content/SharedPreferences$Editor;",
            &[],
        )
        .unwrap();
        put_string(&mut vm, e, "novo", "v2");
        editor(&mut vm, e, "commit", "()Z", &[]).unwrap();

        assert_eq!(
            get_string(&mut vm, prefs, "antigo", "gone").as_deref(),
            Some("gone")
        );
        assert_eq!(
            get_string(&mut vm, prefs, "novo", "gone").as_deref(),
            Some("v2")
        );
        let all = get(&mut vm, prefs, "getAll", "()Ljava/util/Map;", &[]).unwrap();
        assert_eq!(map_len(&vm, obj(all)), 1);
    }

    #[test]
    fn remove_contains_and_get_all() {
        let (mut vm, prefs) = setup();
        let e = edit(&mut vm, prefs);
        put_string(&mut vm, e, "a", "1");
        put_string(&mut vm, e, "b", "2");
        editor(&mut vm, e, "commit", "()Z", &[]).unwrap();
        let all = get(&mut vm, prefs, "getAll", "()Ljava/util/Map;", &[]).unwrap();
        assert_eq!(map_len(&vm, obj(all)), 2);

        let e = edit(&mut vm, prefs);
        let k = js(&mut vm, "a");
        editor(
            &mut vm,
            e,
            "remove",
            "(Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;",
            &[k],
        )
        .unwrap();
        editor(&mut vm, e, "commit", "()Z", &[]).unwrap();
        let key_arg = js_(&mut vm, "a");
        let has = get(
            &mut vm,
            prefs,
            "contains",
            "(Ljava/lang/String;)Z",
            &[key_arg],
        )
        .unwrap();
        assert_eq!(has, Some(Value::Int(0)));
        let key_arg = js_(&mut vm, "b");
        let has = get(
            &mut vm,
            prefs,
            "contains",
            "(Ljava/lang/String;)Z",
            &[key_arg],
        )
        .unwrap();
        assert_eq!(has, Some(Value::Int(1)));
    }

    // ── erros tipados ───────────────────────────────────────────────────

    #[test]
    fn wrong_type_null_key_and_defaults_are_typed() {
        let (mut vm, prefs) = setup();
        let e = edit(&mut vm, prefs);
        put_string(&mut vm, e, "texto", "x");
        editor(&mut vm, e, "commit", "()Z", &[]).unwrap();

        // getInt sobre String: ClassCastException, como a biblioteca.
        let key_arg = js_(&mut vm, "texto");
        let err = get(
            &mut vm,
            prefs,
            "getInt",
            "(Ljava/lang/String;I)I",
            &[key_arg, Value::Int(0)],
        )
        .unwrap_err();
        assert!(matches!(err, VmExit::Exception(t) if t.class == "Ljava/lang/ClassCastException;"));

        // Chave null: NPE.
        let err = get(
            &mut vm,
            prefs,
            "getInt",
            "(Ljava/lang/String;I)I",
            &[Value::Null, Value::Int(0)],
        )
        .unwrap_err();
        assert!(matches!(err, VmExit::Exception(t) if t.class == NPE));

        // Chave ausente: devolve o default da chamada.
        let key_arg = js_(&mut vm, "nada");
        let d = get(
            &mut vm,
            prefs,
            "getInt",
            "(Ljava/lang/String;I)I",
            &[key_arg, Value::Int(9)],
        )
        .unwrap();
        assert_eq!(d, Some(Value::Int(9)));
    }

    #[test]
    fn tampered_ciphertext_and_swapped_entries_fail_authentication() {
        let (mut vm, prefs) = setup();
        let e = edit(&mut vm, prefs);
        put_string(&mut vm, e, "a", "valor-a");
        put_string(&mut vm, e, "b", "valor-b");
        editor(&mut vm, e, "commit", "()Z", &[]).unwrap();

        // Troca de entradas: o AAD (arquivo + chave) não confere.
        {
            let f = vm.fw.androidx.enc.files.get_mut(FILE).unwrap();
            let a = f.entries["a"].clone();
            f.entries.insert("b".to_string(), a);
        }
        let key_arg = js_(&mut vm, "b");
        let err = get(
            &mut vm,
            prefs,
            "getString",
            "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            &[key_arg, Value::Null],
        )
        .unwrap_err();
        assert!(matches!(err, VmExit::Exception(t) if t.class == SEC));

        // Um bit alterado no texto cifrado também é rejeitado.
        {
            let f = vm.fw.androidx.enc.files.get_mut(FILE).unwrap();
            f.entries.get_mut("a").unwrap().ct[0] ^= 1;
        }
        let key_arg = js_(&mut vm, "a");
        let err = get(
            &mut vm,
            prefs,
            "getString",
            "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            &[key_arg, Value::Null],
        )
        .unwrap_err();
        assert!(matches!(err, VmExit::Exception(t) if t.class == SEC));
    }

    #[test]
    fn value_copied_to_another_file_with_same_key_fails_authentication() {
        // Mesma chave mestra, dois arquivos: o AAD inclui o nome do arquivo.
        let (mut vm, prefs) = setup();
        let e = edit(&mut vm, prefs);
        put_string(&mut vm, e, "token", "do-arquivo-original");
        editor(&mut vm, e, "commit", "()Z", &[]).unwrap();
        let other = create_file(&mut vm, "outro-arquivo", "chave.mestra");
        let copied = vm.fw.androidx.enc.files[FILE].entries["token"].clone();
        vm.fw
            .androidx
            .enc
            .files
            .get_mut("outro-arquivo")
            .unwrap()
            .entries
            .insert("token".to_string(), copied);
        let key_arg = js_(&mut vm, "token");
        let err = get(
            &mut vm,
            other,
            "getString",
            "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            &[key_arg, Value::Null],
        )
        .unwrap_err();
        assert!(matches!(err, VmExit::Exception(t) if t.class == SEC));
    }

    #[test]
    fn keystore_key_removed_makes_reads_fail_closed() {
        let (mut vm, prefs) = setup();
        let e = edit(&mut vm, prefs);
        put_string(&mut vm, e, "a", "v");
        editor(&mut vm, e, "commit", "()Z", &[]).unwrap();
        vm.fw.keystore.keys.remove("chave.mestra");
        let key_arg = js_(&mut vm, "a");
        let err = get(
            &mut vm,
            prefs,
            "getString",
            "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            &[key_arg, Value::Null],
        )
        .unwrap_err();
        assert!(matches!(err, VmExit::Exception(t) if t.class == SEC));
    }

    // ── listeners ───────────────────────────────────────────────────────

    #[test]
    fn listeners_register_once_and_unregister() {
        let (mut vm, prefs) = setup();
        let l = Value::Obj(
            vm.heap
                .alloc_instance("Lcom/example/L;".into(), vec![])
                .unwrap(),
        );
        let sig = "(Landroid/content/SharedPreferences$OnSharedPreferenceChangeListener;)V";
        get(
            &mut vm,
            prefs,
            "registerOnSharedPreferenceChangeListener",
            sig,
            std::slice::from_ref(&l),
        )
        .unwrap();
        get(
            &mut vm,
            prefs,
            "registerOnSharedPreferenceChangeListener",
            sig,
            std::slice::from_ref(&l),
        )
        .unwrap();
        assert_eq!(vm.fw.androidx.enc.listeners[&prefs].len(), 1);
        // Apply notifica sem falhar (listener sem corpo DEX não recebe o callback).
        let e = edit(&mut vm, prefs);
        put_string(&mut vm, e, "k", "v");
        editor(&mut vm, e, "commit", "()Z", &[]).unwrap();
        get(
            &mut vm,
            prefs,
            "unregisterOnSharedPreferenceChangeListener",
            sig,
            &[l],
        )
        .unwrap();
        assert!(vm.fw.androidx.enc.listeners[&prefs].is_empty());
    }
}
