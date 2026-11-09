//! ST-04 (base de `androidx.security:security-crypto`): provedor
//! `AndroidKeyStore`, `KeyGenerator` AES, `Cipher` `AES/GCM/NoPadding`,
//! `GCMParameterSpec` e a chave não exportável `AndroidKeyStoreSecretKey`.
//!
//! - As chaves ficam só no host (`Tables::keys`, uma tabela por engine, por
//!   alias). `getEncoded()` devolve null, como no Android para chaves do
//!   AndroidKeyStore: o app não lê o material da chave.
//! - AES-128/256 em GCM com tag de 128 bits e IV de 96 bits, via `aes-gcm`
//!   (criptografia real, não simulada).
//! - Aleatoriedade (chave e IV) do host, `/dev/urandom`. Não é reprodutível de
//!   propósito: chave e IV não aparecem em log nem em saída de ferramenta.
//! - Fora deste subconjunto há falha tipada (`NoSuchAlgorithmException`,
//!   `InvalidKeyException`…), nunca sucesso fingido: `KeyGenerator` de AES sem
//!   AndroidKeyStore e outros transformations não são modelados.

use std::io::Read;

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes128Gcm, Aes256Gcm, Nonce};

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::heap::{ElemKind, ObjRef};
use crate::value::Value;

const KEYSTORE: &str = "Ljava/security/KeyStore;";
const KEY_GENERATOR: &str = "Ljavax/crypto/KeyGenerator;";
const CIPHER: &str = "Ljavax/crypto/Cipher;";
const GCM_SPEC: &str = "Ljavax/crypto/spec/GCMParameterSpec;";
const ANDROID_KEY: &str = "Landroid/security/keystore/AndroidKeyStoreSecretKey;";
const KEY_GEN_SPEC: &str = "Landroid/security/keystore/KeyGenParameterSpec;";

const PROVIDER: &str = "AndroidKeyStore";
const TRANSFORMATION: &str = "AES/GCM/NoPadding";
const GCM_IV_LEN: usize = 12;
const GCM_TAG_BITS: i32 = 128;
const ENCRYPT_MODE: i32 = 1;
const DECRYPT_MODE: i32 = 2;
const DEFAULT_KEY_BITS: i32 = 256;

/// Estado do provedor AndroidKeyStore por engine (ADR 024, passo 3).
#[derive(Debug, Default)]
pub(crate) struct Tables {
    /// Material de chave por alias. Nunca sai do host.
    pub(crate) keys: std::collections::BTreeMap<String, Vec<u8>>,
}

fn throw(class: &str, msg: impl Into<String>) -> VmExit {
    VmExit::Exception(Throwable::new(class, msg))
}

pub(crate) fn random_bytes(n: usize) -> Result<Vec<u8>, VmExit> {
    let mut buf = vec![0u8; n];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut buf))
        .map_err(|e| {
            throw(
                "Ljava/lang/IllegalStateException;",
                format!("sem aleatoriedade do host: {e}"),
            )
        })?;
    Ok(buf)
}

// ── persistência entre sessões (VFS app-private) ─────────────────────────
//
// O AndroidKeyStore real sobrevive ao processo. Aqui cada chave vai para o VFS
// da sessão (`Engine::set_vfs`), em `files/rd_keystore/<alias em hex>.key`, e
// é recarregada sob demanda por outra Engine com o mesmo VFS. Desvio
// declarado: o material fica em claro no VFS, sem TEE/StrongBox.

/// Hex minúsculo, usado para nomes de arquivo e campos de texto do VFS.
pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 0xF) as usize] as char);
    }
    out
}

/// Inverso de `hex_encode`. `None` para comprimento ímpar ou dígito inválido.
pub(crate) fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.is_ascii() || !s.len().is_multiple_of(2) {
        return None;
    }
    s.as_bytes()
        .chunks(2)
        .map(|p| {
            let hi = (p[0] as char).to_digit(16)?;
            let lo = (p[1] as char).to_digit(16)?;
            Some((hi * 16 + lo) as u8)
        })
        .collect()
}

/// Diretório privado do pacote (mesma raiz de `prefs::path`).
pub(crate) fn app_files_dir(package: &str) -> String {
    let package = if package.is_empty() { "app" } else { package };
    format!("/data/user/0/{package}/files")
}

fn key_path(vm: &Engine, alias: &str) -> String {
    format!(
        "{}/rd_keystore/{}.key",
        app_files_dir(&vm.fw.package_name),
        hex_encode(alias.as_bytes())
    )
}

/// Grava a chave recém-gerada no VFS.
fn persist_key(vm: &Engine, alias: &str) {
    if let Some(material) = vm.fw.keystore.keys.get(alias) {
        crate::jfile::write_bytes(&vm.fw.vfs, &key_path(vm, alias), material.clone());
    }
}

/// `deleteEntry`: a chave sai da memória e do VFS.
fn forget_key(vm: &Engine, alias: &str) {
    crate::jfile::delete_file(&vm.fw.vfs, &key_path(vm, alias));
}

/// Recarrega a chave do VFS se esta Engine ainda não a tem em memória. Um
/// material de tamanho fora de AES-128/256 é tratado como ausente.
pub(crate) fn ensure_key_loaded(vm: &mut Engine, alias: &str) {
    if vm.fw.keystore.keys.contains_key(alias) {
        return;
    }
    let path = key_path(vm, alias);
    if let Some(bytes) = crate::jfile::read_bytes(&vm.fw.vfs, &path) {
        if matches!(bytes.len(), 16 | 32) {
            vm.fw.keystore.keys.insert(alias.to_string(), bytes);
        }
    }
}

fn str_arg(vm: &Engine, v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::Obj(r)) => vm.heap.as_str(*r).ok().map(str::to_string),
        _ => None,
    }
}

fn bytes_of(vm: &Engine, v: Option<&Value>) -> Option<Vec<u8>> {
    match v {
        Some(Value::Obj(r)) => vm
            .heap
            .as_array_elems(*r)
            .ok()
            .map(|(e, _)| e.iter().map(|x| x.as_int().unwrap_or(0) as u8).collect()),
        _ => None,
    }
}

fn byte_array(vm: &mut Engine, bytes: &[u8]) -> Result<Value, VmExit> {
    crate::arrays::new_array(
        vm,
        "[B",
        ElemKind::Int,
        bytes.iter().map(|b| Value::Int(*b as i8 as i32)).collect(),
    )
}

fn field(vm: &Engine, r: ObjRef, name: &str) -> Value {
    vm.heap.get_field(r, name).unwrap_or(Value::Null)
}

fn valid_key_bits(bits: i32) -> bool {
    bits == 128 || bits == 256
}

/// Chave do AndroidKeyStore (sem material exportável) para `alias`.
fn new_key(vm: &mut Engine, alias: &str) -> Result<Value, VmExit> {
    let s = crate::intrinsics::alloc_string(vm, alias.to_string())?;
    let r = vm
        .heap
        .alloc_instance(
            ANDROID_KEY.to_string(),
            vec![("rd_alias".into(), Value::Obj(s))],
        )
        .map_err(crate::err::oom_throwable)?;
    Ok(Value::Obj(r))
}

fn alloc(vm: &mut Engine, class: &str, fields: Vec<(String, Value)>) -> Result<Value, VmExit> {
    let r = vm
        .heap
        .alloc_instance(class.to_string(), fields)
        .map_err(crate::err::oom_throwable)?;
    Ok(Value::Obj(r))
}

/// AES-GCM. Devolve `ciphertext || tag`. `None` se a chave ou o IV têm tamanho
/// inválido (já validados antes; aqui é defesa).
pub(crate) fn gcm_seal(key: &[u8], iv: &[u8], aad: &[u8], msg: &[u8]) -> Option<Vec<u8>> {
    let nonce = Nonce::try_from(iv).ok()?;
    let p = Payload { msg, aad };
    match key.len() {
        16 => Aes128Gcm::new_from_slice(key).ok()?.encrypt(&nonce, p).ok(),
        32 => Aes256Gcm::new_from_slice(key).ok()?.encrypt(&nonce, p).ok(),
        _ => None,
    }
}

/// Inverso de `gcm_seal`. `None` = tag não confere (AEADBadTagException).
pub(crate) fn gcm_open(key: &[u8], iv: &[u8], aad: &[u8], sealed: &[u8]) -> Option<Vec<u8>> {
    let nonce = Nonce::try_from(iv).ok()?;
    let p = Payload { msg: sealed, aad };
    match key.len() {
        16 => Aes128Gcm::new_from_slice(key).ok()?.decrypt(&nonce, p).ok(),
        32 => Aes256Gcm::new_from_slice(key).ok()?.decrypt(&nonce, p).ok(),
        _ => None,
    }
}

pub fn static_intrinsic(
    vm: &mut Engine,
    class: &str,
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match (class, name) {
        (KEYSTORE, "getInstance") => {
            let ty = str_arg(vm, args.first());
            if ty.as_deref() != Some(PROVIDER) {
                return Err(throw(
                    "Ljava/security/KeyStoreException;",
                    format!("No provider found for {}", ty.unwrap_or_default()),
                ));
            }
            Ok(Some(alloc(vm, KEYSTORE, vec![])?))
        }
        (KEY_GENERATOR, "getInstance") => {
            let alg = str_arg(vm, args.first());
            let provider = str_arg(vm, args.get(1));
            if alg.as_deref() != Some("AES") {
                return Err(throw(
                    "Ljava/security/NoSuchAlgorithmException;",
                    format!("{} KeyGenerator não é modelado", alg.unwrap_or_default()),
                ));
            }
            if provider.as_deref() != Some(PROVIDER) {
                return Err(throw(
                    "Ljava/security/NoSuchAlgorithmException;",
                    "KeyGenerator AES só é modelado no AndroidKeyStore",
                ));
            }
            Ok(Some(alloc(
                vm,
                KEY_GENERATOR,
                vec![
                    ("rd_alias".into(), Value::Null),
                    ("rd_bits".into(), Value::Int(DEFAULT_KEY_BITS)),
                ],
            )?))
        }
        (CIPHER, "getInstance") => {
            let t = str_arg(vm, args.first());
            if t.as_deref() != Some(TRANSFORMATION) {
                return Err(throw(
                    "Ljava/security/NoSuchAlgorithmException;",
                    format!("Cannot find transformation {}", t.unwrap_or_default()),
                ));
            }
            Ok(Some(alloc(
                vm,
                CIPHER,
                vec![
                    ("rd_mode".into(), Value::Int(0)),
                    ("rd_alias".into(), Value::Null),
                    ("rd_iv".into(), Value::Null),
                    ("rd_buf".into(), Value::Null),
                    ("rd_aad".into(), Value::Null),
                ],
            )?))
        }
        _ => Ok(None),
    }
}

pub fn instance_intrinsic(
    vm: &mut Engine,
    recv_class: &str,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match recv_class {
        KEYSTORE => keystore_method(vm, name, args),
        KEY_GENERATOR => key_generator_method(vm, recv, name, sig, args),
        CIPHER => cipher_method(vm, recv, name, sig, args),
        GCM_SPEC => gcm_spec_method(vm, recv, name, args),
        ANDROID_KEY => Ok(match name {
            "getAlgorithm" => Some(Value::Obj(crate::intrinsics::alloc_string(
                vm,
                "AES".into(),
            )?)),
            "getFormat" | "getEncoded" => Some(Value::Null),
            _ => None,
        }),
        _ => Ok(None),
    }
}

fn keystore_method(vm: &mut Engine, name: &str, args: &[Value]) -> Result<Option<Value>, VmExit> {
    let alias = str_arg(vm, args.first());
    if let Some(a) = &alias {
        ensure_key_loaded(vm, a);
    }
    Ok(Some(match name {
        "load" => Value::Null,
        "containsAlias" => {
            let has = alias.is_some_and(|a| vm.fw.keystore.keys.contains_key(&a));
            Value::Int(has as i32)
        }
        "getKey" => match alias {
            Some(a) if vm.fw.keystore.keys.contains_key(&a) => new_key(vm, &a)?,
            _ => Value::Null,
        },
        "deleteEntry" => {
            if let Some(a) = alias {
                vm.fw.keystore.keys.remove(&a);
                forget_key(vm, &a);
            }
            Value::Null
        }
        _ => return Ok(None),
    }))
}

fn key_generator_method(
    vm: &mut Engine,
    recv: ObjRef,
    name: &str,
    sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match (name, sig) {
        ("init", "(I)V") | ("init", "(ILjava/security/SecureRandom;)V") => {
            // Sem KeyGenParameterSpec não há alias: o AndroidKeyStore exige a spec.
            Err(throw(
                "Ljava/security/InvalidAlgorithmParameterException;",
                "AndroidKeyStore exige KeyGenParameterSpec",
            ))
        }
        ("init", "(Ljava/security/spec/AlgorithmParameterSpec;)V") => {
            let spec = match args.first() {
                Some(Value::Obj(r)) if vm.heap.class_of(*r).ok() == Some(KEY_GEN_SPEC) => *r,
                _ => {
                    return Err(throw(
                        "Ljava/security/InvalidAlgorithmParameterException;",
                        "KeyGenParameterSpec esperado",
                    ))
                }
            };
            let alias = match field(vm, spec, "mKeystoreAlias") {
                Value::Obj(s) => vm.heap.as_str(s).ok().map(str::to_string),
                _ => None,
            };
            let Some(alias) = alias else {
                return Err(throw(
                    "Ljava/security/InvalidAlgorithmParameterException;",
                    "KeyGenParameterSpec sem alias",
                ));
            };
            let bits = match field(vm, spec, "mKeySize") {
                Value::Int(-1) | Value::Null => DEFAULT_KEY_BITS,
                Value::Int(b) => b,
                _ => DEFAULT_KEY_BITS,
            };
            if !valid_key_bits(bits) {
                return Err(throw(
                    "Ljava/security/InvalidAlgorithmParameterException;",
                    format!("Unsupported key size {bits}"),
                ));
            }
            let s = crate::intrinsics::alloc_string(vm, alias)?;
            let _ = vm.heap.put_field(recv, "rd_alias", Value::Obj(s));
            let _ = vm.heap.put_field(recv, "rd_bits", Value::Int(bits));
            Ok(Some(Value::Null))
        }
        ("generateKey", "()Ljavax/crypto/SecretKey;") => {
            let alias = match field(vm, recv, "rd_alias") {
                Value::Obj(s) => vm.heap.as_str(s).ok().map(str::to_string),
                _ => None,
            };
            let Some(alias) = alias else {
                return Err(throw(
                    "Ljava/lang/IllegalStateException;",
                    "KeyGenerator não inicializado",
                ));
            };
            let bits = match field(vm, recv, "rd_bits") {
                Value::Int(b) => b,
                _ => DEFAULT_KEY_BITS,
            };
            let material = random_bytes((bits / 8) as usize)?;
            vm.fw.keystore.keys.insert(alias.clone(), material);
            persist_key(vm, &alias);
            Ok(Some(new_key(vm, &alias)?))
        }
        _ => Ok(None),
    }
}

fn cipher_method(
    vm: &mut Engine,
    recv: ObjRef,
    name: &str,
    sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match (name, sig) {
        (
            "init",
            "(ILjava/security/Key;)V"
            | "(ILjava/security/Key;Ljava/security/SecureRandom;)V"
            | "(ILjava/security/Key;Ljava/security/spec/AlgorithmParameterSpec;)V"
            | "(ILjava/security/Key;Ljava/security/spec/AlgorithmParameterSpec;Ljava/security/SecureRandom;)V",
        ) => {
            cipher_init(vm, recv, args)?;
            Ok(Some(Value::Null))
        }
        ("getIV", "()[B") => Ok(Some(field(vm, recv, "rd_iv"))),
        ("updateAAD", "([B)V") => {
            let mut aad = bytes_of(vm, Some(&field(vm, recv, "rd_aad"))).unwrap_or_default();
            aad.extend(bytes_of(vm, args.first()).unwrap_or_default());
            let v = byte_array(vm, &aad)?;
            let _ = vm.heap.put_field(recv, "rd_aad", v);
            Ok(Some(Value::Null))
        }
        ("update", "([B)[B") => {
            let mut buf = bytes_of(vm, Some(&field(vm, recv, "rd_buf"))).unwrap_or_default();
            buf.extend(bytes_of(vm, args.first()).unwrap_or_default());
            let v = byte_array(vm, &buf)?;
            let _ = vm.heap.put_field(recv, "rd_buf", v);
            Ok(Some(byte_array(vm, &[])?))
        }
        ("doFinal", "([B)[B") => {
            let mut msg = bytes_of(vm, Some(&field(vm, recv, "rd_buf"))).unwrap_or_default();
            msg.extend(bytes_of(vm, args.first()).unwrap_or_default());
            Ok(Some(cipher_finish(vm, recv, &msg)?))
        }
        ("doFinal", "()[B") => {
            let msg = bytes_of(vm, Some(&field(vm, recv, "rd_buf"))).unwrap_or_default();
            Ok(Some(cipher_finish(vm, recv, &msg)?))
        }
        _ => Ok(None),
    }
}

fn cipher_init(vm: &mut Engine, recv: ObjRef, args: &[Value]) -> Result<(), VmExit> {
    let mode = match args.first() {
        Some(Value::Int(m)) if *m == ENCRYPT_MODE || *m == DECRYPT_MODE => *m,
        _ => {
            return Err(throw(
                "Ljava/lang/IllegalArgumentException;",
                "modo de Cipher inválido",
            ))
        }
    };
    let key = match args.get(1) {
        Some(Value::Obj(k)) if vm.heap.class_of(*k).ok() == Some(ANDROID_KEY) => *k,
        _ => {
            return Err(throw(
                "Ljava/security/InvalidKeyException;",
                "chave não suportada",
            ))
        }
    };
    let alias = match field(vm, key, "rd_alias") {
        Value::Obj(s) => vm
            .heap
            .as_str(s)
            .ok()
            .map(str::to_string)
            .unwrap_or_default(),
        _ => String::new(),
    };
    ensure_key_loaded(vm, &alias);
    if !vm.fw.keystore.keys.contains_key(&alias) {
        return Err(throw(
            "Ljava/security/InvalidKeyException;",
            "chave ausente no AndroidKeyStore",
        ));
    }
    // O spec, quando há, é o 3.º argumento. SecureRandom (4.º ou 3.º) é ignorado:
    // a aleatoriedade vem do host.
    let spec = match args.get(2) {
        Some(Value::Obj(s)) if vm.heap.class_of(*s).ok() == Some(GCM_SPEC) => Some(*s),
        _ => None,
    };
    let iv = match spec {
        Some(s) => {
            let tlen = match field(vm, s, "rd_tlen") {
                Value::Int(t) => t,
                _ => 0,
            };
            if tlen != GCM_TAG_BITS {
                return Err(throw(
                    "Ljava/security/InvalidAlgorithmParameterException;",
                    format!("Unsupported GCM tag length {tlen}"),
                ));
            }
            bytes_of(vm, Some(&field(vm, s, "rd_iv"))).unwrap_or_default()
        }
        None if mode == ENCRYPT_MODE => random_bytes(GCM_IV_LEN)?,
        None => {
            return Err(throw(
                "Ljava/security/InvalidKeyException;",
                "IV obrigatório ao decifrar (GCMParameterSpec)",
            ))
        }
    };
    if iv.len() != GCM_IV_LEN {
        return Err(throw(
            "Ljava/security/InvalidAlgorithmParameterException;",
            format!("IV de {} bytes; GCM aqui usa {GCM_IV_LEN}", iv.len()),
        ));
    }
    let iv_v = byte_array(vm, &iv)?;
    let alias_v = crate::intrinsics::alloc_string(vm, alias)?;
    let _ = vm.heap.put_field(recv, "rd_mode", Value::Int(mode));
    let _ = vm.heap.put_field(recv, "rd_alias", Value::Obj(alias_v));
    let _ = vm.heap.put_field(recv, "rd_iv", iv_v);
    let _ = vm.heap.put_field(recv, "rd_buf", Value::Null);
    let _ = vm.heap.put_field(recv, "rd_aad", Value::Null);
    Ok(())
}

fn cipher_finish(vm: &mut Engine, recv: ObjRef, msg: &[u8]) -> Result<Value, VmExit> {
    let mode = match field(vm, recv, "rd_mode") {
        Value::Int(m) if m == ENCRYPT_MODE || m == DECRYPT_MODE => m,
        _ => {
            return Err(throw(
                "Ljava/lang/IllegalStateException;",
                "Cipher não inicializado",
            ))
        }
    };
    let alias = match field(vm, recv, "rd_alias") {
        Value::Obj(s) => vm
            .heap
            .as_str(s)
            .ok()
            .map(str::to_string)
            .unwrap_or_default(),
        _ => String::new(),
    };
    ensure_key_loaded(vm, &alias);
    let Some(key) = vm.fw.keystore.keys.get(&alias).cloned() else {
        return Err(throw(
            "Ljava/lang/IllegalStateException;",
            "chave removida do AndroidKeyStore",
        ));
    };
    let iv = bytes_of(vm, Some(&field(vm, recv, "rd_iv"))).unwrap_or_default();
    let aad = bytes_of(vm, Some(&field(vm, recv, "rd_aad"))).unwrap_or_default();
    let out = if mode == ENCRYPT_MODE {
        gcm_seal(&key, &iv, &aad, msg).ok_or_else(|| {
            throw(
                "Ljava/lang/IllegalStateException;",
                "falha interna do AES-GCM",
            )
        })?
    } else {
        gcm_open(&key, &iv, &aad, msg)
            .ok_or_else(|| throw("Ljavax/crypto/AEADBadTagException;", "Tag mismatch"))?
    };
    // doFinal zera o buffer e o AAD, como no JDK.
    let empty = byte_array(vm, &[])?;
    let _ = vm.heap.put_field(recv, "rd_buf", empty);
    let _ = vm.heap.put_field(recv, "rd_aad", Value::Null);
    byte_array(vm, &out)
}

fn gcm_spec_method(
    vm: &mut Engine,
    recv: ObjRef,
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    Ok(Some(match name {
        "<init>" => {
            let tlen = match args.first() {
                Some(Value::Int(t)) => *t,
                _ => 0,
            };
            let iv = bytes_of(vm, args.get(1)).unwrap_or_default();
            let iv_v = byte_array(vm, &iv)?;
            let _ = vm.heap.put_field(recv, "rd_tlen", Value::Int(tlen));
            let _ = vm.heap.put_field(recv, "rd_iv", iv_v);
            Value::Null
        }
        "getIV" => field(vm, recv, "rd_iv"),
        "getTLen" => field(vm, recv, "rd_tlen"),
        _ => return Ok(None),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcm_roundtrip_both_key_sizes_and_tag_check() {
        for key in [vec![7u8; 16], vec![9u8; 32]] {
            let iv = [1u8; 12];
            let sealed = gcm_seal(&key, &iv, b"aad", b"segredo").unwrap();
            assert_eq!(sealed.len(), b"segredo".len() + 16, "ciphertext || tag");
            assert_eq!(gcm_open(&key, &iv, b"aad", &sealed).unwrap(), b"segredo");
            // AAD diferente, IV diferente ou byte alterado: a tag não confere.
            assert!(gcm_open(&key, &iv, b"outro", &sealed).is_none());
            assert!(gcm_open(&key, &[2u8; 12], b"aad", &sealed).is_none());
            let mut bad = sealed.clone();
            bad[0] ^= 1;
            assert!(gcm_open(&key, &iv, b"aad", &bad).is_none());
        }
    }

    #[test]
    fn gcm_vector_matches_reference_aes128_gcm() {
        // NIST SP 800-38D, caso de teste 2 (AES-128, plaintext de 16 zeros):
        // key = 0^128, iv = 0^96 → ciphertext = 0388dace60b6a392f328c2b971b2fe78,
        // tag = ab6e47d42cec13bdf53a67b21257bddf.
        let key = [0u8; 16];
        let iv = [0u8; 12];
        let sealed = gcm_seal(&key, &iv, &[], &[0u8; 16]).unwrap();
        let hex: String = sealed.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex,
            "0388dace60b6a392f328c2b971b2fe78ab6e47d42cec13bdf53a67b21257bddf"
        );
    }

    #[test]
    fn key_sizes_are_limited_to_aes_128_and_256() {
        assert!(valid_key_bits(128) && valid_key_bits(256));
        assert!(!valid_key_bits(192) && !valid_key_bits(512));
    }

    fn js(vm: &mut Engine, s: &str) -> Value {
        Value::Obj(crate::intrinsics::alloc_string(vm, s.to_string()).unwrap())
    }

    fn obj(v: Option<Value>) -> ObjRef {
        match v {
            Some(Value::Obj(r)) => r,
            other => panic!("esperava objeto, veio {other:?}"),
        }
    }

    #[test]
    fn keystore_key_is_not_exportable_and_round_trips_through_cipher() {
        let mut vm = Engine::new(vec![], crate::VmConfig::default());
        let provider = js(&mut vm, PROVIDER);
        let ks = obj(static_intrinsic(&mut vm, KEYSTORE, "getInstance", &[provider]).unwrap());

        // KeyGenParameterSpec com alias e 256 bits (o builder real já modela os campos).
        let alias_v = js(&mut vm, "chave.teste");
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

        let aes = js(&mut vm, "AES");
        let provider2 = js(&mut vm, PROVIDER);
        let kg = obj(
            static_intrinsic(&mut vm, KEY_GENERATOR, "getInstance", &[aes, provider2]).unwrap(),
        );
        instance_intrinsic(
            &mut vm,
            KEY_GENERATOR,
            "init",
            "(Ljava/security/spec/AlgorithmParameterSpec;)V",
            kg,
            &[Value::Obj(spec)],
        )
        .unwrap();
        let key = obj(instance_intrinsic(
            &mut vm,
            KEY_GENERATOR,
            "generateKey",
            "()Ljavax/crypto/SecretKey;",
            kg,
            &[],
        )
        .unwrap());
        assert!(vm.fw.keystore.keys.contains_key("chave.teste"));
        assert!(
            matches!(
                instance_intrinsic(&mut vm, ANDROID_KEY, "getEncoded", "()[B", key, &[]).unwrap(),
                Some(Value::Null)
            ),
            "material da chave não é exportável"
        );
        let alias_q = js(&mut vm, "chave.teste");
        let contains = instance_intrinsic(
            &mut vm,
            KEYSTORE,
            "containsAlias",
            "(Ljava/lang/String;)Z",
            ks,
            &[alias_q],
        )
        .unwrap();
        assert!(matches!(contains, Some(Value::Int(1))));

        // Cipher AES/GCM: cifra com IV gerado, decifra com GCMParameterSpec.
        let t1 = js(&mut vm, TRANSFORMATION);
        let c = obj(static_intrinsic(&mut vm, CIPHER, "getInstance", &[t1]).unwrap());
        let plain_items: Vec<Value> = b"lista".iter().map(|b| Value::Int(*b as i32)).collect();
        let plain = crate::arrays::new_array(&mut vm, "[B", ElemKind::Int, plain_items).unwrap();
        instance_intrinsic(
            &mut vm,
            CIPHER,
            "init",
            "(ILjava/security/Key;)V",
            c,
            &[Value::Int(ENCRYPT_MODE), Value::Obj(key)],
        )
        .unwrap();
        let iv = instance_intrinsic(&mut vm, CIPHER, "getIV", "()[B", c, &[])
            .unwrap()
            .unwrap();
        let iv_bytes = bytes_of(&vm, Some(&iv)).unwrap();
        assert_eq!(iv_bytes.len(), 12);
        let sealed = instance_intrinsic(&mut vm, CIPHER, "doFinal", "([B)[B", c, &[plain])
            .unwrap()
            .unwrap();
        assert_eq!(
            bytes_of(&vm, Some(&sealed)).unwrap().len(),
            5 + 16,
            "ciphertext || tag"
        );

        let spec_iv = byte_array(&mut vm, &iv_bytes).unwrap();
        let gcm = obj(Some(alloc(&mut vm, GCM_SPEC, vec![]).unwrap()));
        instance_intrinsic(
            &mut vm,
            GCM_SPEC,
            "<init>",
            "(I[B)V",
            gcm,
            &[Value::Int(GCM_TAG_BITS), spec_iv],
        )
        .unwrap();
        let t2 = js(&mut vm, TRANSFORMATION);
        let d = obj(static_intrinsic(&mut vm, CIPHER, "getInstance", &[t2]).unwrap());
        instance_intrinsic(
            &mut vm,
            CIPHER,
            "init",
            "(ILjava/security/Key;Ljava/security/spec/AlgorithmParameterSpec;)V",
            d,
            &[Value::Int(DECRYPT_MODE), Value::Obj(key), Value::Obj(gcm)],
        )
        .unwrap();
        let back = instance_intrinsic(&mut vm, CIPHER, "doFinal", "([B)[B", d, &[sealed])
            .unwrap()
            .unwrap();
        assert_eq!(bytes_of(&vm, Some(&back)).unwrap(), b"lista");

        // Chave removida: não há mais alias no keystore.
        let alias_d = js(&mut vm, "chave.teste");
        instance_intrinsic(
            &mut vm,
            KEYSTORE,
            "deleteEntry",
            "(Ljava/lang/String;)V",
            ks,
            &[alias_d],
        )
        .unwrap();
        assert!(!vm.fw.keystore.keys.contains_key("chave.teste"));
    }
}
