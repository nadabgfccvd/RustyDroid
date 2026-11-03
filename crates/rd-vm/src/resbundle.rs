//! `java.util.ResourceBundle` — *class-based* bundles (`ListResourceBundle`).
//!
//! Real APK driver (NewPipe 0.27): `PrettyTime` localizes its time units with
//! `ResourceBundle.getBundle("org.ocpsoft.prettytime.i18n.Resources", locale)`,
//! where `Resources` is a DEX class extending `java.util.ListResourceBundle`
//! (and `Resources_ar`, `Resources_pt`, … are the locale variants). Without the
//! bundle the whole `Application.onCreate` dies, because the preference screens
//! and the relative-time formatter are built there.
//!
//! What is modelled:
//!
//! * `getBundle(baseName[, locale[, loader]])` → resolve `base_lang_COUNTRY`,
//!   `base_lang`, `base` **in that order** (the JDK's candidate list, minus the
//!   script/variant permutations) and instantiate the guest class;
//! * `getObject`/`handleGetObject`/`getString`/`getStringArray`/`containsKey`
//!   over the pairs returned by the guest `getContents()`.
//!
//! What is *not* modelled — and says so instead of guessing: `.properties`
//! bundles (there is no `PropertyResourceBundle` file loader here), candidate
//! parents (`base_en_US` falling back to `base_en`'s *contents*), and
//! `getKeys`/`keySet` enumeration.

use crate::engine::Engine;
use crate::err::Throwable;
use crate::err::{vm_error, VmExit};
use crate::heap::ObjRef;
use crate::value::Value;

pub const RESOURCE_BUNDLE: &str = "Ljava/util/ResourceBundle;";
pub const LIST_BUNDLE: &str = "Ljava/util/ListResourceBundle;";
const MISSING: &str = "Ljava/util/MissingResourceException;";

fn missing(key: &str, base: &str) -> VmExit {
    VmExit::Exception(Throwable::new(
        MISSING,
        format!("Can't find resource for bundle {base}, key {key}"),
    ))
}

fn heap_str(vm: &Engine, value: &Value) -> Option<String> {
    match value {
        Value::Obj(r) => vm.heap.as_str(*r).ok().map(str::to_string),
        _ => None,
    }
}

/// Chave/valor do bundle como `String` de Rust. Strings literais vivem no heap
/// como `HeapObj::Str`; as construídas em runtime são instâncias de
/// `Ljava/lang/String;`, e aí o `toString` do convidado é a resposta certa.
fn text_of(vm: &mut Engine, value: &Value) -> Option<String> {
    if let Some(s) = heap_str(vm, value) {
        return Some(s);
    }
    match value {
        Value::Obj(r) => crate::intrinsics::object_to_string(vm, *r).ok(),
        _ => None,
    }
}

fn field_str(vm: &Engine, obj: ObjRef, field: &str) -> String {
    vm.heap
        .get_field(obj, field)
        .ok()
        .and_then(|v| heap_str(vm, &v))
        .unwrap_or_default()
}

/// Nome da classe do bundle a partir do nome-base (pontos → barras).
fn descriptor_of(base_name: &str) -> String {
    format!("L{};", base_name.replace('.', "/"))
}

/// `ResourceBundle.getBundle(baseName[, locale])`.
pub fn get_bundle(
    vm: &mut Engine,
    base_name: &str,
    locale: Option<ObjRef>,
) -> Result<ObjRef, VmExit> {
    let (lang, country) = match locale {
        Some(l) => (field_str(vm, l, "rd_lang"), field_str(vm, l, "rd_country")),
        None => (String::new(), String::new()),
    };
    let mut candidates = Vec::new();
    if !lang.is_empty() && !country.is_empty() {
        candidates.push(format!("{base_name}_{lang}_{country}"));
    }
    if !lang.is_empty() {
        candidates.push(format!("{base_name}_{lang}"));
    }
    candidates.push(base_name.to_string());

    for candidate in &candidates {
        let descriptor = descriptor_of(candidate);
        if vm.cp.find_class(&descriptor).is_none() {
            continue;
        }
        vm.ensure_initialized(&descriptor)?;
        let obj = vm.new_dex_instance_pub(&descriptor)?;
        // Nome-base e locale efetivo ficam no bundle, como no JDK (o nome é o
        // que aparece na mensagem da `MissingResourceException`).
        if let Ok(base) = vm.heap.alloc_string(candidate.clone()) {
            let _ = vm.heap.put_field(obj, "rd_base", Value::Obj(base));
        }
        if let Some(l) = locale {
            let _ = vm.heap.put_field(obj, "rd_locale", Value::Obj(l));
        }
        return Ok(obj);
    }
    Err(VmExit::Exception(Throwable::new(
        MISSING,
        format!(
            "Can't find bundle for base name {base_name}, locale {}_{}",
            if lang.is_empty() { "und" } else { &lang },
            country
        ),
    )))
}

/// Pares `chave, valor` do `getContents()` do bundle convidado. O array
/// externo (`Object[][]`) fica cacheado no objeto — `getContents` aloca arrays
/// novos a cada chamada no código gerado pelo R8.
fn contents(vm: &mut Engine, recv: ObjRef) -> Result<Vec<Value>, VmExit> {
    let outer = match vm.heap.get_field(recv, "rd_contents") {
        Ok(Value::Obj(cached)) if vm.heap.as_array_elems(cached).is_ok() => cached,
        _ => {
            let contents =
                vm.invoke_instance(recv, "getContents", "()[[Ljava/lang/Object;", &[])?;
            let Value::Obj(array) = contents else {
                return Err(vm_error(
                    "VM_TYPE_ERROR",
                    "ListResourceBundle.getContents não devolveu Object[][]",
                )
                .into());
            };
            let _ = vm.heap.put_field(recv, "rd_contents", Value::Obj(array));
            array
        }
    };
    // Object[][] → pares achatados `[chave, valor, chave, valor, …]`.
    let (rows, _) = vm
        .heap
        .as_array_elems(outer)
        .map_err(|e| vm_error("VM_TYPE_ERROR", e))?;
    let rows = rows.to_vec();
    let mut flat = Vec::with_capacity(rows.len() * 2);
    for row in rows {
        let Value::Obj(pair) = row else { continue };
        let (cells, _) = vm
            .heap
            .as_array_elems(pair)
            .map_err(|e| vm_error("VM_TYPE_ERROR", e))?;
        flat.extend(cells.iter().take(2).cloned());
    }
    Ok(flat)
}

fn find(vm: &mut Engine, recv: ObjRef, key: &str) -> Result<Option<Value>, VmExit> {
    let pairs = contents(vm, recv)?;
    let mut index = 0;
    while index + 1 < pairs.len() {
        if text_of(vm, &pairs[index]).as_deref() == Some(key) {
            return Ok(Some(pairs[index + 1].clone()));
        }
        index += 2;
    }
    Ok(None)
}

/// `getBundle` estático (todas as aridades usadas por APKs reais).
pub fn static_intrinsic(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if name != "getBundle" {
        return Ok(None);
    }
    let base = match sig {
        "(Ljava/lang/String;)Ljava/util/ResourceBundle;"
        | "(Ljava/lang/String;Ljava/util/Locale;)Ljava/util/ResourceBundle;"
        | "(Ljava/lang/String;Ljava/util/Locale;Ljava/lang/ClassLoader;)Ljava/util/ResourceBundle;"
        | "(Ljava/lang/String;Ljava/util/Locale;Ldalvik/system/BaseDexClassLoader;)Ljava/util/ResourceBundle;" => {
            heap_str(vm, args.first().unwrap_or(&Value::Null))
        }
        _ => return Ok(None),
    };
    let Some(base) = base else {
        return Err(VmExit::Exception(Throwable::new(
            "Ljava/lang/NullPointerException;",
            "baseName == null",
        )));
    };
    let locale = match args.get(1) {
        Some(Value::Obj(l)) => Some(*l),
        _ => None,
    };
    Ok(Some(Value::Obj(get_bundle(vm, &base, locale)?)))
}

/// Métodos de instância do `ResourceBundle` (e do `ListResourceBundle`).
pub fn instance_intrinsic(
    vm: &mut Engine,
    recv: ObjRef,
    name: &str,
    sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match (name, sig) {
        ("getObject", "(Ljava/lang/String;)Ljava/lang/Object;") => {
            let key = heap_str(vm, args.first().unwrap_or(&Value::Null))
                .unwrap_or_else(|| "null".to_string());
            match find(vm, recv, &key)? {
                Some(value) => Ok(Some(value)),
                None => Err(missing(&key, &field_str(vm, recv, "rd_base"))),
            }
        }
        ("handleGetObject", "(Ljava/lang/String;)Ljava/lang/Object;") => {
            let key = heap_str(vm, args.first().unwrap_or(&Value::Null))
                .unwrap_or_else(|| "null".to_string());
            Ok(Some(find(vm, recv, &key)?.unwrap_or(Value::Null)))
        }
        ("containsKey", "(Ljava/lang/String;)Z") => {
            let key = heap_str(vm, args.first().unwrap_or(&Value::Null)).unwrap_or_default();
            Ok(Some(Value::Int(i32::from(find(vm, recv, &key)?.is_some()))))
        }
        ("getString", "(Ljava/lang/String;)Ljava/lang/String;") => {
            let key = heap_str(vm, args.first().unwrap_or(&Value::Null))
                .unwrap_or_else(|| "null".to_string());
            match find(vm, recv, &key)? {
                Some(Value::Obj(r)) if vm.heap.as_str(r).is_ok() => Ok(Some(Value::Obj(r))),
                Some(other) => Err(vm_error(
                    "VM_TYPE_ERROR",
                    format!("ResourceBundle.getString({key}) não é String: {other:?}"),
                )
                .into()),
                None => Err(missing(&key, &field_str(vm, recv, "rd_base"))),
            }
        }
        ("getStringArray", "(Ljava/lang/String;)[Ljava/lang/String;") => {
            let key = heap_str(vm, args.first().unwrap_or(&Value::Null))
                .unwrap_or_else(|| "null".to_string());
            match find(vm, recv, &key)? {
                Some(value @ Value::Obj(_)) => Ok(Some(value)),
                Some(other) => Err(vm_error(
                    "VM_TYPE_ERROR",
                    format!("ResourceBundle.getStringArray({key}) não é String[]: {other:?}"),
                )
                .into()),
                None => Err(missing(&key, &field_str(vm, recv, "rd_base"))),
            }
        }
        ("getLocale", "()Ljava/util/Locale;") => Ok(Some(
            vm.heap.get_field(recv, "rd_locale").unwrap_or(Value::Null),
        )),
        // `getContents()` do próprio convidado nunca chega aqui: o método é
        // concreto no DEX. Só a variante abstrata da plataforma seria host.
        _ => Ok(None),
    }
}
