//! `org.json` — `JSONObject`, `JSONArray`, `JSONTokener` and `JSONException`.
//!
//! Android ships org.json as part of the platform, so apps call it from DEX and
//! expect no dependency of their own. F-Droid parses its repository index,
//! mirrors and preferences with it, so the whole read side has to behave like
//! the platform class.
//!
//! Each host `JSONObject`/`JSONArray` owns a parsed document
//! (`serde_json::Value`) in [`Tables`]; nested objects handed to the app are
//! materialised as their own documents, so a mutation through
//! `getJSONObject("x")` affects that child only — org.json would write through
//! to the parent. Reads (what every F-Droid path does) are exact.
//!
//! Covered: `JSONObject(String)`, `JSONObject()`, `get*`/`opt*` for
//! String/int/long/double/boolean/JSONObject/JSONArray, `has`, `isNull`,
//! `keys`/`names`, `length`, `get`, `put` (all overloads), `remove`, `toString`,
//! `toString(int)`, `JSONObject.NULL`; `JSONArray(String)`, `JSONArray()`,
//! `length`, `get`/`opt`/`getString`… by index, `put` (all overloads),
//! `toString`, `join`; `JSONTokener` (`nextValue`, `nextString`, `next(char)`,
//! `more`, `back`, `toString`). Parsing is org.json-lenient where it matters for
//! stored preferences: single quotes, unquoted keys, trailing commas and
//! `;`/`=` separators are accepted by the preprocessing step below.

use std::collections::HashMap;

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::heap::ObjRef;
use crate::value::Value;

const JSON_OBJECT: &str = "Lorg/json/JSONObject;";
const JSON_ARRAY: &str = "Lorg/json/JSONArray;";
const JSON_TOKENER: &str = "Lorg/json/JSONTokener;";
const JSON_NULL: &str = "Lorg/json/JSONObject$Null;";
const JSON_EXCEPTION: &str = "Lorg/json/JSONException;";
const ITERATOR: &str = "Ljava/util/Iterator;";

/// Parsed documents, keyed by the host object that owns them (ADR 024 tables).
#[derive(Debug, Default)]
pub(crate) struct Tables {
    pub(crate) docs: HashMap<ObjRef, serde_json::Value>,
    /// `JSONTokener` text, keyed by the tokener object.
    pub(crate) tokens: HashMap<ObjRef, (String, usize)>,
}

/// GC: keep only entries whose host object is still alive.
pub(crate) fn gc_purge(t: &mut Tables, alive: &dyn Fn(ObjRef) -> bool) {
    t.docs.retain(|k, _| alive(*k));
    t.tokens.retain(|k, _| alive(*k));
}

fn exc(msg: impl Into<String>) -> VmExit {
    VmExit::Exception(Throwable::new(JSON_EXCEPTION, msg))
}
fn jstr(vm: &mut Engine, s: impl Into<String>) -> Result<Value, VmExit> {
    Ok(Value::Obj(crate::intrinsics::alloc_string(vm, s.into())?))
}
fn sval(vm: &Engine, v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::Obj(r)) => vm.heap.as_str(*r).ok().map(str::to_string),
        _ => None,
    }
}
fn int_arg(args: &[Value], i: usize) -> i32 {
    args.get(i).and_then(|v| v.as_int().ok()).unwrap_or(0)
}
/// org.json accepts a few things strict JSON rejects; normalise them before
/// handing the text to `serde_json`.
fn lenient_json(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_string: Option<char> = None;
    while let Some(c) = chars.next() {
        match in_string {
            Some(q) => {
                if c == '\\' {
                    out.push(c);
                    if let Some(n) = chars.next() {
                        out.push(n);
                    }
                    continue;
                }
                if c == q {
                    out.push(if q == '\'' { '"' } else { c });
                    in_string = None;
                } else if c == '"' && q == '\'' {
                    out.push_str("\\\"");
                } else {
                    out.push(c);
                }
            }
            None => match c {
                '"' | '\'' => {
                    out.push('"');
                    in_string = Some(c);
                }
                '=' => out.push(':'),
                '/' => {
                    // `//` comment or `/* */` block.
                    match chars.peek() {
                        Some('/') => {
                            for n in chars.by_ref() {
                                if n == '\n' {
                                    break;
                                }
                            }
                        }
                        Some('*') => {
                            chars.next();
                            let mut prev = '\0';
                            for n in chars.by_ref() {
                                if prev == '*' && n == '/' {
                                    break;
                                }
                                prev = n;
                            }
                        }
                        _ => out.push('/'),
                    }
                }
                _ => out.push(c),
            },
        }
    }
    // Trailing commas before `}` or `]`.
    let mut cleaned = String::with_capacity(out.len());
    let bytes: Vec<char> = out.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == ',' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_whitespace() {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == '}' || bytes[j] == ']') {
                i += 1;
                continue;
            }
        }
        cleaned.push(c);
        i += 1;
    }
    cleaned
}

fn parse_doc(text: &str) -> Result<serde_json::Value, VmExit> {
    serde_json::from_str::<serde_json::Value>(text)
        .or_else(|_| serde_json::from_str::<serde_json::Value>(&lenient_json(text)))
        .map_err(|e| exc(format!("unexpected JSON input: {e}")))
}

fn doc_of(vm: &Engine, r: ObjRef) -> serde_json::Value {
    vm.fw
        .org_json
        .docs
        .get(&r)
        .cloned()
        .unwrap_or(serde_json::Value::Null)
}

fn set_doc(vm: &mut Engine, r: ObjRef, v: serde_json::Value) {
    vm.fw.org_json.docs.insert(r, v);
}

fn new_object(vm: &mut Engine, v: serde_json::Value) -> Result<Value, VmExit> {
    let r = vm
        .heap
        .alloc_instance(JSON_OBJECT.to_string(), Vec::new())
        .map_err(crate::err::oom_throwable)?;
    set_doc(vm, r, v);
    Ok(Value::Obj(r))
}

fn new_array(vm: &mut Engine, v: serde_json::Value) -> Result<Value, VmExit> {
    let r = vm
        .heap
        .alloc_instance(JSON_ARRAY.to_string(), Vec::new())
        .map_err(crate::err::oom_throwable)?;
    set_doc(vm, r, v);
    Ok(Value::Obj(r))
}

/// JSON → guest value (`Value::Obj` for nested documents).
fn to_value(vm: &mut Engine, j: &serde_json::Value) -> Result<Value, VmExit> {
    Ok(match j {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Int(i32::from(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                if let Ok(small) = i32::try_from(i) {
                    Value::Int(small)
                } else if let Some(l) = n.as_i64() {
                    Value::Long(l)
                } else {
                    Value::Double(n.as_f64().unwrap_or(0.0))
                }
            } else {
                Value::Double(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => jstr(vm, s.clone())?,
        serde_json::Value::Object(_) => new_object(vm, j.clone())?,
        serde_json::Value::Array(_) => new_array(vm, j.clone())?,
    })
}

fn number_from(_vm: &Engine, v: &Value) -> Option<serde_json::Number> {
    match v {
        Value::Int(i) => Some(serde_json::Number::from(*i)),
        Value::Long(l) => Some(serde_json::Number::from(*l)),
        Value::Float(f) => serde_json::Number::from_f64(f64::from(*f)),
        Value::Double(d) => serde_json::Number::from_f64(*d),
        // Strings are not numbers; neither are host documents.
        Value::Obj(_) => None,
        _ => None,
    }
}

/// guest value → JSON. `as_bool` distinguishes `put(name, int)` (number) from
/// `put(name, boolean)` (JSON boolean) — org.json's overloads differ there.
fn from_value(vm: &Engine, v: &Value, as_bool: bool) -> serde_json::Value {
    match v {
        Value::Null => serde_json::Value::Null,
        Value::Obj(r) => {
            if let Ok(s) = vm.heap.as_str(*r) {
                return serde_json::Value::String(s.to_string());
            }
            if vm.heap.class_of(*r).ok() == Some(JSON_NULL) {
                return serde_json::Value::Null;
            }
            if let Some(d) = vm.fw.org_json.docs.get(r) {
                return d.clone();
            }
            serde_json::Value::String(String::new())
        }
        Value::Int(i) => {
            if as_bool {
                serde_json::Value::Bool(*i != 0)
            } else if let Some(n) = number_from(vm, v) {
                serde_json::Value::Number(n)
            } else {
                serde_json::Value::Null
            }
        }
        Value::Long(_) | Value::Float(_) | Value::Double(_) => match number_from(vm, v) {
            Some(n) => serde_json::Value::Number(n),
            None => serde_json::Value::Null,
        },
        // Internal interpreter encodings never reach a JSON document.
        _ => serde_json::Value::Null,
    }
}

fn as_string(j: &serde_json::Value) -> String {
    match j {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn as_int(j: &serde_json::Value) -> Result<i32, VmExit> {
    match j {
        serde_json::Value::Number(n) => n
            .as_i64()
            .map(|v| v as i32)
            .or_else(|| n.as_f64().map(|f| f as i32))
            .ok_or_else(|| exc("Not a number")),
        serde_json::Value::String(s) => s
            .trim()
            .parse::<f64>()
            .map(|f| f as i32)
            .map_err(|_| exc(format!("For input string: \"{s}\""))),
        serde_json::Value::Bool(b) => Ok(i32::from(*b)),
        _ => Err(exc("Not a number")),
    }
}

fn as_long(j: &serde_json::Value) -> Result<i64, VmExit> {
    match j {
        serde_json::Value::Number(n) => n
            .as_i64()
            .or_else(|| n.as_f64().map(|f| f as i64))
            .ok_or_else(|| exc("Not a number")),
        serde_json::Value::String(s) => s
            .trim()
            .parse::<f64>()
            .map(|f| f as i64)
            .map_err(|_| exc(format!("For input string: \"{s}\""))),
        serde_json::Value::Bool(b) => Ok(i64::from(*b)),
        _ => Err(exc("Not a number")),
    }
}

fn as_double(j: &serde_json::Value) -> Result<f64, VmExit> {
    match j {
        serde_json::Value::Number(n) => n.as_f64().ok_or_else(|| exc("Not a number")),
        serde_json::Value::String(s) => s
            .trim()
            .parse::<f64>()
            .map_err(|_| exc(format!("For input string: \"{s}\""))),
        serde_json::Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        _ => Err(exc("Not a number")),
    }
}

fn as_boolean(j: &serde_json::Value) -> Result<bool, VmExit> {
    match j {
        serde_json::Value::Bool(b) => Ok(*b),
        serde_json::Value::String(s) => match s.trim().to_ascii_lowercase().as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(exc(format!("\"{s}\" is not a boolean."))),
        },
        serde_json::Value::Number(n) => Ok(n.as_f64().map(|f| f != 0.0).unwrap_or(false)),
        serde_json::Value::Null => Err(exc("JSONObject[\"\"] is not a boolean.")),
        _ => Err(exc("not a boolean")),
    }
}

// ---------------------------------------------------------------------------
// JSONObject
// ---------------------------------------------------------------------------

fn object_static(vm: &mut Engine, name: &str, args: &[Value]) -> Result<Option<Value>, VmExit> {
    Ok(Some(match name {
        "numberToString" => {
            let s = match args.first() {
                Some(Value::Double(d)) => format!("{d}"),
                Some(Value::Float(f)) => format!("{f}"),
                Some(Value::Long(l)) => l.to_string(),
                Some(Value::Int(i)) => i.to_string(),
                _ => "0".to_string(),
            };
            jstr(vm, s)?
        }
        "quote" => {
            let s = sval(vm, args.first()).unwrap_or_default();
            jstr(vm, serde_json::Value::String(s).to_string())?
        }
        "wrap" => {
            let inner = args.first().cloned().unwrap_or(Value::Null);
            new_object(vm, from_value(vm, &inner, false))?
        }
        _ => return Ok(None),
    }))
}

/// `JSONObject.NULL`: a sentinel whose `toString` is `"null"`.
pub(crate) fn host_static(
    vm: &mut Engine,
    owner: &str,
    field: &str,
) -> Result<Option<Value>, VmExit> {
    if owner != JSON_OBJECT || field != "NULL" {
        return Ok(None);
    }
    let r = vm
        .heap
        .alloc_instance(JSON_NULL.to_string(), Vec::new())
        .map_err(crate::err::oom_throwable)?;
    Ok(Some(Value::Obj(r)))
}

fn object_instance(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    // ---- constructors -----------------------------------------------------
    if name == "<init>" {
        let doc = match args.first() {
            Some(Value::Obj(_)) => {
                let text = sval(vm, args.first()).unwrap_or_default();
                parse_doc(&text)?
            }
            _ => serde_json::Value::Object(serde_json::Map::new()),
        };
        set_doc(vm, recv, doc);
        return Ok(Some(Value::Null));
    }

    let doc = doc_of(vm, recv);
    let is_object = doc.is_object();
    let mut obj = if is_object {
        doc.as_object().cloned().unwrap_or_default()
    } else {
        serde_json::Map::new()
    };

    // Index-based methods only exist on JSONArray, but `get(int)` on an object
    // is a programming error in org.json too: not found here.
    Ok(Some(match name {
        "getString" | "optString" => {
            let key = sval(vm, args.first()).unwrap_or_default();
            match obj.get(&key) {
                Some(v) if name == "getString" || !v.is_null() => jstr(vm, as_string(v))?,
                _ if name == "optString" && args.len() >= 2 => args[1].clone(),
                _ if name == "optString" => jstr(vm, "")?,
                _ => return Err(exc(format!("No value for {key}"))),
            }
        }
        "getInt" | "optInt" => {
            let key = sval(vm, args.first()).unwrap_or_default();
            match obj.get(&key) {
                Some(v) if !v.is_null() => Value::Int(as_int(v)?),
                _ if name == "optInt" => Value::Int(int_arg(args, 1)),
                _ => return Err(exc(format!("No value for {key}"))),
            }
        }
        "getLong" | "optLong" => {
            let key = sval(vm, args.first()).unwrap_or_default();
            match obj.get(&key) {
                Some(v) if !v.is_null() => Value::Long(as_long(v)?),
                _ if name == "optLong" => Value::Long(
                    args.get(1)
                        .and_then(|v| match v {
                            Value::Long(l) => Some(*l),
                            Value::Int(i) => Some(i64::from(*i)),
                            _ => None,
                        })
                        .unwrap_or(0),
                ),
                _ => return Err(exc(format!("No value for {key}"))),
            }
        }
        "getDouble" | "optDouble" => {
            let key = sval(vm, args.first()).unwrap_or_default();
            match obj.get(&key) {
                Some(v) if !v.is_null() => Value::Double(as_double(v)?),
                _ if name == "optDouble" => Value::Double(match args.get(1) {
                    Some(Value::Double(d)) => *d,
                    Some(Value::Int(i)) => f64::from(*i),
                    _ => f64::NAN,
                }),
                _ => return Err(exc(format!("No value for {key}"))),
            }
        }
        "getBoolean" | "optBoolean" => {
            let key = sval(vm, args.first()).unwrap_or_default();
            match obj.get(&key) {
                Some(v) if !v.is_null() => Value::Int(i32::from(as_boolean(v)?)),
                _ if name == "optBoolean" => Value::Int(int_arg(args, 1)),
                _ => return Err(exc(format!("No value for {key}"))),
            }
        }
        "get" => {
            let key = sval(vm, args.first()).unwrap_or_default();
            match obj.get(&key) {
                Some(v) => to_value(vm, v)?,
                None => return Err(exc(format!("No value for {key}"))),
            }
        }
        "opt" => {
            let key = sval(vm, args.first()).unwrap_or_default();
            match obj.get(&key) {
                Some(v) => to_value(vm, v)?,
                None => Value::Null,
            }
        }
        "getJSONObject" | "optJSONObject" => {
            let key = sval(vm, args.first()).unwrap_or_default();
            match obj.get(&key) {
                Some(serde_json::Value::Object(o)) => {
                    new_object(vm, serde_json::Value::Object(o.clone()))?
                }
                Some(_) => return Err(exc(format!("Value {key} is not a JSONObject."))),
                None if name == "optJSONObject" => Value::Null,
                None => return Err(exc(format!("No value for {key}"))),
            }
        }
        "getJSONArray" | "optJSONArray" => {
            let key = sval(vm, args.first()).unwrap_or_default();
            match obj.get(&key) {
                Some(serde_json::Value::Array(a)) => {
                    new_array(vm, serde_json::Value::Array(a.clone()))?
                }
                Some(_) => return Err(exc(format!("Value {key} is not a JSONArray."))),
                None if name == "optJSONArray" => Value::Null,
                None => return Err(exc(format!("No value for {key}"))),
            }
        }
        "has" => Value::Int(i32::from(
            obj.contains_key(&sval(vm, args.first()).unwrap_or_default()),
        )),
        "isNull" => {
            let key = sval(vm, args.first()).unwrap_or_default();
            Value::Int(i32::from(matches!(
                obj.get(&key),
                Some(serde_json::Value::Null) | None
            )))
        }
        "length" => Value::Int(obj.len() as i32),
        "keys" | "names" => {
            let names: Vec<String> = obj.keys().cloned().collect();
            if name == "names" {
                let mut elems = Vec::with_capacity(names.len());
                for n in &names {
                    elems.push(jstr(vm, n.clone())?);
                }
                let arr = vm
                    .heap
                    .alloc_array(
                        crate::heap::ElemKind::Obj,
                        elems.len(),
                        "Ljava/lang/String;",
                    )
                    .map_err(crate::err::oom_throwable)?;
                *vm.heap.array_elems_mut(arr)? = elems;
                Value::Obj(arr)
            } else {
                let mut elems = Vec::with_capacity(names.len());
                for n in &names {
                    elems.push(jstr(vm, n.clone())?);
                }
                let it = vm
                    .heap
                    .alloc_instance(ITERATOR.to_string(), Vec::new())
                    .map_err(crate::err::oom_throwable)?;
                vm.fw
                    .objects
                    .insert(it, crate::framework::HostObj::Iter { elems, pos: 0 });
                Value::Obj(it)
            }
        }
        "put" => {
            let key = sval(vm, args.first()).unwrap_or_else(|| {
                if args.is_empty() {
                    String::new()
                } else {
                    as_string(&from_value(vm, &args[0], false))
                }
            });
            if key.is_empty() {
                return Err(exc("Names must be non-null and non-empty strings."));
            }
            let as_bool = sig.starts_with("(Ljava/lang/String;Z)");
            let j = match args.get(1) {
                Some(v) => from_value(vm, v, as_bool),
                None => serde_json::Value::Null,
            };
            obj.insert(key, j);
            set_doc(vm, recv, serde_json::Value::Object(obj));
            Value::Obj(recv)
        }
        "accumulate" => {
            let key = sval(vm, args.first()).unwrap_or_default();
            let j = from_value(vm, args.get(1).unwrap_or(&Value::Null), false);
            let entry = obj
                .entry(key)
                .or_insert_with(|| serde_json::Value::Array(Vec::new()));
            match entry {
                serde_json::Value::Array(a) => a.push(j),
                other => {
                    let prev = other.clone();
                    *other = serde_json::Value::Array(vec![prev, j]);
                }
            }
            set_doc(vm, recv, serde_json::Value::Object(obj));
            Value::Obj(recv)
        }
        "remove" => {
            let key = sval(vm, args.first()).unwrap_or_default();
            let removed = obj.remove(&key);
            set_doc(vm, recv, serde_json::Value::Object(obj));
            match removed {
                Some(v) => to_value(vm, &v)?,
                None => Value::Null,
            }
        }
        "toString" => {
            let text = if args.is_empty() {
                serde_json::Value::Object(obj).to_string()
            } else {
                serde_json::to_string_pretty(&serde_json::Value::Object(obj))
                    .unwrap_or_else(|_| "{}".to_string())
            };
            jstr(vm, text)?
        }
        "equals" => {
            let other = args.first().and_then(|v| match v {
                Value::Obj(o) => vm.fw.org_json.docs.get(o).cloned(),
                _ => None,
            });
            let self_doc = serde_json::Value::Object(obj);
            Value::Int(i32::from(other.as_ref() == Some(&self_doc)))
        }
        "hashCode" => {
            let self_doc = serde_json::Value::Object(obj);
            Value::Int(self_doc.to_string().len() as i32)
        }
        "keySet" => {
            let names: Vec<String> = obj.keys().cloned().collect();
            let r = vm
                .heap
                .alloc_instance("Ljava/util/LinkedHashSet;".to_string(), Vec::new())
                .map_err(crate::err::oom_throwable)?;
            let mut elems = Vec::with_capacity(names.len());
            for n in names {
                elems.push(jstr(vm, n)?);
            }
            vm.fw
                .objects
                .insert(r, crate::framework::HostObj::List { elems });
            Value::Obj(r)
        }
        "toMap" | "toList" => return Ok(None),
        _ => return Ok(None),
    }))
}

fn null_instance(
    vm: &mut Engine,
    name: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let _ = recv;
    Ok(Some(match name {
        "toString" => jstr(vm, "null")?,
        "equals" => Value::Int(i32::from(match args.first() {
            None | Some(Value::Null) => true,
            Some(Value::Obj(o)) => vm.heap.class_of(*o).ok() == Some(JSON_NULL),
            _ => false,
        })),
        "hashCode" => Value::Int(0),
        _ => return Ok(None),
    }))
}

// ---------------------------------------------------------------------------
// JSONArray
// ---------------------------------------------------------------------------

fn array_instance(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if name == "<init>" {
        let doc = match (args.first(), args.get(1)) {
            (Some(Value::Obj(_)), Some(Value::Obj(_)))
                if sig.starts_with("(Ljava/util/Collection;)") =>
            {
                serde_json::Value::Array(Vec::new())
            }
            (Some(Value::Obj(_)), _) => {
                let text = sval(vm, args.first()).unwrap_or_default();
                match parse_doc(&text)? {
                    v @ serde_json::Value::Array(_) => v,
                    _ => serde_json::Value::Array(Vec::new()),
                }
            }
            _ => serde_json::Value::Array(Vec::new()),
        };
        set_doc(vm, recv, doc);
        return Ok(Some(Value::Null));
    }

    let doc = doc_of(vm, recv);
    let mut arr = match doc {
        serde_json::Value::Array(a) => a,
        _ => Vec::new(),
    };
    let idx = int_arg(args, 0);
    let check = |i: i32, len: usize| -> Result<usize, VmExit> {
        if i < 0 || i as usize >= len {
            Err(exc(format!("Index {i} out of range [0..{len})")))
        } else {
            Ok(i as usize)
        }
    };

    Ok(Some(match name {
        "length" => Value::Int(arr.len() as i32),
        "getString" | "optString" => {
            if args
                .first()
                .map(|v| matches!(v, Value::Obj(_)))
                .unwrap_or(false)
            {
                let i = check(idx, arr.len())?;
                jstr(vm, as_string(&arr[i]))?
            } else if name == "optString" {
                arr.get(idx.max(0) as usize)
                    .map(as_string)
                    .map(|s| jstr(vm, s))
                    .unwrap_or_else(|| jstr(vm, ""))?
            } else {
                let i = check(idx, arr.len())?;
                jstr(vm, as_string(&arr[i]))?
            }
        }
        "getInt" | "optInt" => {
            if name == "optInt" {
                arr.get(idx.max(0) as usize)
                    .and_then(|v| as_int(v).ok())
                    .map(Value::Int)
                    .unwrap_or(Value::Int(int_arg(args, 1)))
            } else {
                Value::Int(as_int(&arr[check(idx, arr.len())?])?)
            }
        }
        "getLong" | "optLong" => {
            if name == "optLong" {
                Value::Long(
                    arr.get(idx.max(0) as usize)
                        .and_then(|v| as_long(v).ok())
                        .unwrap_or(0),
                )
            } else {
                Value::Long(as_long(&arr[check(idx, arr.len())?])?)
            }
        }
        "getDouble" | "optDouble" => {
            if name == "optDouble" {
                arr.get(idx.max(0) as usize)
                    .and_then(|v| as_double(v).ok())
                    .map(Value::Double)
                    .unwrap_or(Value::Double(f64::NAN))
            } else {
                Value::Double(as_double(&arr[check(idx, arr.len())?])?)
            }
        }
        "getBoolean" | "optBoolean" => {
            if name == "optBoolean" {
                arr.get(idx.max(0) as usize)
                    .and_then(|v| as_boolean(v).ok())
                    .map(|b| Value::Int(i32::from(b)))
                    .unwrap_or(Value::Int(int_arg(args, 1)))
            } else {
                Value::Int(i32::from(as_boolean(&arr[check(idx, arr.len())?])?))
            }
        }
        "get" => {
            let i = check(idx, arr.len())?;
            to_value(vm, &arr[i])?
        }
        "opt" => match arr.get(idx.max(0) as usize) {
            Some(v) => to_value(vm, v)?,
            None => Value::Null,
        },
        "getJSONObject" | "optJSONObject" => {
            if name == "optJSONObject" {
                match arr.get(idx.max(0) as usize) {
                    Some(serde_json::Value::Object(o)) => {
                        new_object(vm, serde_json::Value::Object(o.clone()))?
                    }
                    _ => Value::Null,
                }
            } else {
                match &arr[check(idx, arr.len())?] {
                    serde_json::Value::Object(o) => {
                        new_object(vm, serde_json::Value::Object(o.clone()))?
                    }
                    _ => return Err(exc(format!("Value at {idx} is not a JSONObject."))),
                }
            }
        }
        "getJSONArray" | "optJSONArray" => {
            if name == "optJSONArray" {
                match arr.get(idx.max(0) as usize) {
                    Some(serde_json::Value::Array(a)) => {
                        new_array(vm, serde_json::Value::Array(a.clone()))?
                    }
                    _ => Value::Null,
                }
            } else {
                match &arr[check(idx, arr.len())?] {
                    serde_json::Value::Array(a) => {
                        new_array(vm, serde_json::Value::Array(a.clone()))?
                    }
                    _ => return Err(exc(format!("Value at {idx} is not a JSONArray."))),
                }
            }
        }
        "put" => {
            // `put(int index, Object value)` vs `put(Object value)`.
            let as_bool = sig.starts_with("(Z)");
            let (at, v) = if sig.starts_with("(ILjava/lang/Object;)")
                || sig.starts_with("(I") && args.len() == 2
            {
                (
                    Some(int_arg(args, 0)),
                    args.get(1).cloned().unwrap_or(Value::Null),
                )
            } else {
                (None, args.first().cloned().unwrap_or(Value::Null))
            };
            let j = from_value(vm, &v, as_bool);
            match at {
                Some(i) if i >= 0 && (i as usize) < arr.len() => arr[i as usize] = j,
                _ => arr.push(j),
            }
            set_doc(vm, recv, serde_json::Value::Array(arr));
            Value::Obj(recv)
        }
        "isNull" => Value::Int(i32::from(matches!(
            arr.get(idx.max(0) as usize),
            None | Some(serde_json::Value::Null)
        ))),
        "remove" => {
            if args.is_empty() {
                set_doc(vm, recv, serde_json::Value::Array(Vec::new()));
                Value::Null
            } else if idx >= 0 && (idx as usize) < arr.len() {
                let v = arr.remove(idx as usize);
                set_doc(vm, recv, serde_json::Value::Array(arr));
                to_value(vm, &v)?
            } else {
                Value::Null
            }
        }
        "toString" => {
            let text = if args.is_empty() {
                serde_json::Value::Array(arr).to_string()
            } else {
                serde_json::to_string_pretty(&serde_json::Value::Array(arr))
                    .unwrap_or_else(|_| "[]".to_string())
            };
            jstr(vm, text)?
        }
        "join" => {
            let sep = sval(vm, args.first()).unwrap_or_else(|| ",".to_string());
            let parts: Vec<String> = arr
                .iter()
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect();
            jstr(vm, parts.join(&sep))?
        }
        "equals" => {
            let other = args.first().and_then(|v| match v {
                Value::Obj(o) => vm.fw.org_json.docs.get(o).cloned(),
                _ => None,
            });
            let self_doc = serde_json::Value::Array(arr);
            Value::Int(i32::from(other.as_ref() == Some(&self_doc)))
        }
        "hashCode" => Value::Int(arr.len() as i32),
        "iterator" | "toList" => return Ok(None),
        _ => return Ok(None),
    }))
}

// ---------------------------------------------------------------------------
// JSONTokener
// ---------------------------------------------------------------------------

fn tokener_instance(
    vm: &mut Engine,
    name: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if name == "<init>" {
        let text = sval(vm, args.first()).unwrap_or_default();
        vm.fw.org_json.tokens.insert(recv, (text, 0));
        return Ok(Some(Value::Null));
    }
    let Some((text, mut pos)) = vm.fw.org_json.tokens.get(&recv).cloned() else {
        return Ok(None);
    };
    let chars: Vec<char> = text.chars().collect();
    let skip_ws = |p: &mut usize| {
        while *p < chars.len() && chars[*p].is_whitespace() {
            *p += 1;
        }
    };
    Ok(Some(match name {
        "more" => {
            skip_ws(&mut pos);
            vm.fw.org_json.tokens.insert(recv, (text.clone(), pos));
            Value::Int(i32::from(pos < chars.len()))
        }
        "next" => {
            skip_ws(&mut pos);
            let c = chars.get(pos).copied().unwrap_or('\0');
            pos += 1;
            vm.fw.org_json.tokens.insert(recv, (text.clone(), pos));
            Value::Int(u32::from(c) as i32)
        }
        "back" => {
            pos = pos.saturating_sub(1);
            vm.fw.org_json.tokens.insert(recv, (text.clone(), pos));
            Value::Null
        }
        "nextString" => {
            skip_ws(&mut pos);
            let quote = chars.get(pos).copied().unwrap_or('"');
            pos += 1;
            let mut out = String::new();
            while pos < chars.len() {
                let c = chars[pos];
                pos += 1;
                if c == quote {
                    break;
                }
                if c == '\\' && pos < chars.len() {
                    let e = chars[pos];
                    pos += 1;
                    out.push(match e {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        'b' => '\u{8}',
                        'f' => '\u{c}',
                        other => other,
                    });
                    continue;
                }
                out.push(c);
            }
            vm.fw.org_json.tokens.insert(recv, (text.clone(), pos));
            jstr(vm, out)?
        }
        "nextValue" => {
            skip_ws(&mut pos);
            let rest: String = chars[pos..].iter().collect();
            let mut stream =
                serde_json::Deserializer::from_str(&rest).into_iter::<serde_json::Value>();
            let val = stream.next().and_then(|r| r.ok());
            match val {
                Some(v) => {
                    let used = stream.byte_offset();
                    // Byte offset → char offset.
                    let consumed = rest
                        .char_indices()
                        .take_while(|(i, _)| *i < used)
                        .count()
                        .max(1);
                    vm.fw
                        .org_json
                        .tokens
                        .insert(recv, (text.clone(), pos + consumed));
                    to_value(vm, &v)?
                }
                None => Value::Null,
            }
        }
        "toString" => jstr(vm, text.clone())?,
        _ => return Ok(None),
    }))
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub(crate) fn static_intrinsic(
    vm: &mut Engine,
    class: &str,
    name: &str,
    _sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match class {
        JSON_OBJECT => object_static(vm, name, args),
        _ => Ok(None),
    }
}

pub(crate) fn instance_intrinsic(
    vm: &mut Engine,
    recv_class: &str,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match recv_class {
        JSON_OBJECT => object_instance(vm, name, sig, recv, args),
        JSON_ARRAY => array_instance(vm, name, sig, recv, args),
        JSON_TOKENER => tokener_instance(vm, name, recv, args),
        JSON_NULL => null_instance(vm, name, recv, args),
        _ => Ok(None),
    }
}
