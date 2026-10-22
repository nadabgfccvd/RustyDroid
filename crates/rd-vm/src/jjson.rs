//! `org.json` (`JSONObject` / `JSONArray`): platform library classes not
//! present in app DEXes, modelled with real parse/serialize semantics.
//!
//! State: a host `ArrayList` in a heap field — `json_map` holds flattened
//! [key, value, key, value, ...] pairs (insertion order, as org.json's
//! LinkedHashMap), `json_items` holds array elements. Values are stored
//! as the guest Values given to `put` (boxed objects or primitives);
//! `JSONObject.NULL` is a canonical singleton. Serialization follows
//! org.json's compact form. Numbers parse to Long when integral, else
//! Double (org.json behaviour).

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::heap::ObjRef;
use crate::value::Value;

pub(crate) const JSON_OBJECT: &str = "Lorg/json/JSONObject;";
pub(crate) const JSON_ARRAY: &str = "Lorg/json/JSONArray;";
const JSON_EXCEPTION: &str = "Lorg/json/JSONException;";

fn exc(m: impl Into<String>) -> VmExit {
    VmExit::Exception(Throwable::new(JSON_EXCEPTION, m.into()))
}

fn put(vm: &mut Engine, r: ObjRef, f: &str, v: Value) -> Result<(), VmExit> {
    crate::err::put_field_result(vm.heap.put_field(r, f, v))
}

fn list_obj(vm: &mut Engine, r: ObjRef, field: &str) -> Result<ObjRef, VmExit> {
    if let Ok(Value::Obj(l)) = vm.heap.get_field(r, field) {
        return Ok(l);
    }
    let l = vm
        .heap
        .alloc_instance("Ljava/util/ArrayList;".to_string(), Vec::new())
        .map_err(crate::err::oom_throwable)?;
    vm.fw
        .objects
        .insert(l, crate::framework::HostObj::List { elems: Vec::new() });
    put(vm, r, field, Value::Obj(l))?;
    Ok(l)
}

fn elems<'a>(vm: &'a mut Engine, r: ObjRef, field: &str) -> Result<&'a mut Vec<Value>, VmExit> {
    let l = list_obj(vm, r, field)?;
    match vm.fw.objects.get_mut(&l) {
        Some(crate::framework::HostObj::List { elems }) => Ok(elems),
        _ => Err(crate::err::vm_error("VM_TYPE_ERROR", "json sem lista host").into()),
    }
}

fn elems_ref(vm: &Engine, r: ObjRef, field: &str) -> Vec<Value> {
    match vm.heap.get_field(r, field) {
        Ok(Value::Obj(l)) => match vm.fw.objects.get(&l) {
            Some(crate::framework::HostObj::List { elems }) => elems.clone(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

pub(crate) fn null_value(vm: &mut Engine) -> Result<Value, VmExit> {
    let key = (JSON_OBJECT.to_string(), "NULL".to_string());
    if let Some(v) = vm.statics.get(&key).cloned() {
        return Ok(v);
    }
    let o = vm
        .heap
        .alloc_instance(JSON_OBJECT.to_string(), Vec::new())
        .map_err(crate::err::oom_throwable)?;
    put(vm, o, "json_null", Value::Int(1))?;
    vm.statics.insert(key, Value::Obj(o));
    Ok(Value::Obj(o))
}

fn is_null(vm: &Engine, v: &Value) -> bool {
    match v {
        Value::Null | Value::Int(0) => true,
        Value::Obj(o) => matches!(vm.heap.get_field(*o, "json_null"), Ok(Value::Int(1))),
        _ => false,
    }
}

fn str_arg(vm: &Engine, args: &[Value], i: usize) -> String {
    match args.get(i) {
        Some(Value::Obj(o)) => vm.heap.as_str(*o).unwrap_or_default().to_string(),
        _ => String::new(),
    }
}

// ------------------------------------------------------------ parser

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn ws(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }
    fn value(&mut self, vm: &mut Engine) -> Result<Value, VmExit> {
        self.ws();
        match self.peek() {
            Some('{') => {
                self.pos += 1;
                let obj = vm
                    .heap
                    .alloc_instance(JSON_OBJECT.to_string(), Vec::new())
                    .map_err(crate::err::oom_throwable)?;
                loop {
                    self.ws();
                    if self.peek() == Some('}') {
                        self.pos += 1;
                        return Ok(Value::Obj(obj));
                    }
                    let k = match self.value(vm)? {
                        Value::Obj(o) => vm.heap.as_str(o).unwrap_or_default().to_string(),
                        _ => return Err(exc("chave JSON inválida")),
                    };
                    self.ws();
                    if self.peek() != Some(':') {
                        return Err(exc("esperado ':' em objeto JSON"));
                    }
                    self.pos += 1;
                    let v = self.value(vm)?;
                    json_put(vm, obj, &k, v)?;
                    self.ws();
                    match self.peek() {
                        Some(',') => self.pos += 1,
                        Some('}') => {
                            self.pos += 1;
                            return Ok(Value::Obj(obj));
                        }
                        _ => return Err(exc("esperado ',' ou '}' em objeto JSON")),
                    }
                }
            }
            Some('[') => {
                self.pos += 1;
                let arr = vm
                    .heap
                    .alloc_instance(JSON_ARRAY.to_string(), Vec::new())
                    .map_err(crate::err::oom_throwable)?;
                loop {
                    self.ws();
                    if self.peek() == Some(']') {
                        self.pos += 1;
                        return Ok(Value::Obj(arr));
                    }
                    let v = self.value(vm)?;
                    elems(vm, arr, "json_items")?.push(v);
                    self.ws();
                    match self.peek() {
                        Some(',') => self.pos += 1,
                        Some(']') => {
                            self.pos += 1;
                            return Ok(Value::Obj(arr));
                        }
                        _ => return Err(exc("esperado ',' ou ']' em array JSON")),
                    }
                }
            }
            Some('"') => {
                let s = self.string()?;
                Ok(Value::Obj(crate::intrinsics::alloc_string(vm, s)?))
            }
            Some('t') if self.rest("true") => Ok(Value::Obj(
                vm.heap
                    .alloc_instance(
                        "Ljava/lang/Boolean;".to_string(),
                        vec![("value".to_string(), Value::Int(1))],
                    )
                    .map_err(crate::err::oom_throwable)?,
            )),
            Some('f') if self.rest("false") => Ok(Value::Obj(
                vm.heap
                    .alloc_instance(
                        "Ljava/lang/Boolean;".to_string(),
                        vec![("value".to_string(), Value::Int(0))],
                    )
                    .map_err(crate::err::oom_throwable)?,
            )),
            Some('n') if self.rest("null") => null_value(vm),
            Some(_) => self.number(),
            None => Err(exc("fim inesperado do JSON")),
        }
    }
    fn rest(&mut self, lit: &str) -> bool {
        let lit: Vec<char> = lit.chars().collect();
        if self.chars[self.pos..].starts_with(&lit) {
            self.pos += lit.len();
            true
        } else {
            false
        }
    }
    fn string(&mut self) -> Result<String, VmExit> {
        // assumes peek == '"'
        self.pos += 1;
        let mut out = String::new();
        loop {
            let Some(c) = self.peek() else {
                return Err(exc("string JSON não terminada"));
            };
            self.pos += 1;
            match c {
                '"' => return Ok(out),
                '\\' => {
                    let e = self.peek().ok_or_else(|| exc("escape inválido"))?;
                    self.pos += 1;
                    match e {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        '/' => out.push('/'),
                        'b' => out.push('\u{0008}'),
                        'f' => out.push('\u{000C}'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => {
                            let hex: String = self.chars[self.pos..self.pos + 4].iter().collect();
                            self.pos += 4;
                            let cp = u32::from_str_radix(&hex, 16)
                                .map_err(|_| exc("escape \\u inválido"))?;
                            out.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                        }
                        _ => return Err(exc("escape JSON inválido")),
                    }
                }
                _ => out.push(c),
            }
        }
    }
    fn number(&mut self) -> Result<Value, VmExit> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E') {
                self.pos += 1;
            } else {
                break;
            }
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        if let Ok(i) = text.parse::<i64>() {
            return Ok(Value::Long(i));
        }
        text.parse::<f64>()
            .map(Value::Double)
            .map_err(|_| exc(format!("número JSON inválido: {text}")))
    }
}

fn parse_into(vm: &mut Engine, recv: ObjRef, text: &str) -> Result<(), VmExit> {
    let mut p = Parser {
        chars: text.chars().collect(),
        pos: 0,
    };
    let v = p.value(vm)?;
    match v {
        Value::Obj(src) => {
            // copy parsed state into the receiver (ctor target)
            let field = if vm
                .heap
                .class_of(recv)
                .map(|c| c == JSON_ARRAY)
                .unwrap_or(false)
            {
                "json_items"
            } else {
                "json_map"
            };
            let items = elems_ref(vm, src, field);
            *elems(vm, recv, field)? = items;
            Ok(())
        }
        _ => Err(exc("raiz JSON deve ser objeto ou array")),
    }
}

fn json_put(vm: &mut Engine, obj: ObjRef, key: &str, v: Value) -> Result<(), VmExit> {
    let null = is_null(vm, &v);
    let mut list = elems_ref(vm, obj, "json_map");
    let keys: Vec<String> = list
        .iter()
        .step_by(2)
        .map(|k| match k {
            Value::Obj(o) => vm.heap.as_str(*o).unwrap_or_default().to_string(),
            _ => String::new(),
        })
        .collect();
    if let Some(pos) = keys.iter().position(|k| k == key) {
        if null {
            list.drain(pos * 2..pos * 2 + 2);
        } else {
            list[pos * 2 + 1] = v;
        }
    } else if !null {
        let key_v = Value::Obj(crate::intrinsics::alloc_string(vm, key.to_string())?);
        list.push(key_v);
        list.push(v);
    }
    *elems(vm, obj, "json_map")? = list;
    Ok(())
}

fn json_get(vm: &Engine, obj: ObjRef, key: &str) -> Option<Value> {
    let list = elems_ref(vm, obj, "json_map");
    let mut i = 0;
    while i + 1 < list.len() {
        let existing = match &list[i] {
            Value::Obj(o) => vm.heap.as_str(*o).unwrap_or_default().to_string(),
            _ => String::new(),
        };
        if existing == key {
            return Some(list[i + 1].clone());
        }
        i += 2;
    }
    None
}

// ---------------------------------------------------------- serializer

fn quote_into(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn num_str(vm: &Engine, v: &Value) -> Option<String> {
    match v {
        Value::Int(i) => Some(i.to_string()),
        Value::Long(l) => Some(l.to_string()),
        Value::Float(f) => Some(format!("{f}")),
        Value::Double(d) => Some(format!("{d}")),
        Value::Obj(o) => {
            // boxed numbers/booleans/CharSequence unwrap via `value`
            match vm.heap.get_field(*o, "value") {
                Ok(inner) => num_str(vm, &inner),
                _ => None,
            }
        }
        _ => None,
    }
}

fn write_value(vm: &mut Engine, v: &Value, out: &mut String) -> Result<(), VmExit> {
    if is_null(vm, v) {
        out.push_str("null");
        return Ok(());
    }
    if let Value::Obj(o) = v {
        if vm
            .heap
            .class_of(*o)
            .map(|c| c == "Ljava/lang/Boolean;")
            .unwrap_or(false)
        {
            let on = matches!(vm.heap.get_field(*o, "value"), Ok(Value::Int(1)));
            out.push_str(if on { "true" } else { "false" });
            return Ok(());
        }
    }
    if let Some(n) = num_str(vm, v) {
        out.push_str(&n);
        return Ok(());
    }
    match v {
        Value::Obj(o) => {
            let class = vm.heap.class_of(*o).unwrap_or_default().to_string();
            if class == JSON_OBJECT {
                write_object(vm, *o, out)
            } else if class == JSON_ARRAY {
                write_array(vm, *o, out)
            } else if let Ok(s) = vm.heap.as_str(*o) {
                quote_into(&s, out);
                Ok(())
            } else {
                let text = match vm.invoke_instance(*o, "toString", "()Ljava/lang/String;", &[]) {
                    Ok(Value::Obj(ts)) => vm.heap.as_str(ts).unwrap_or_default().to_string(),
                    _ => String::new(),
                };
                quote_into(&text, out);
                Ok(())
            }
        }
        _ => {
            out.push_str("null");
            Ok(())
        }
    }
}

fn write_object(vm: &mut Engine, obj: ObjRef, out: &mut String) -> Result<(), VmExit> {
    out.push('{');
    let list = elems_ref(vm, obj, "json_map");
    let mut first = true;
    let mut i = 0;
    while i + 1 < list.len() {
        let key = match &list[i] {
            Value::Obj(o) => vm.heap.as_str(*o).unwrap_or_default().to_string(),
            _ => String::new(),
        };
        if !first {
            out.push(',');
        }
        first = false;
        quote_into(&key, out);
        out.push(':');
        write_value(vm, &list[i + 1], out)?;
        i += 2;
    }
    out.push('}');
    Ok(())
}

fn write_array(vm: &mut Engine, arr: ObjRef, out: &mut String) -> Result<(), VmExit> {
    out.push('[');
    let list = elems_ref(vm, arr, "json_items");
    for (i, v) in list.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_value(vm, v, out)?;
    }
    out.push(']');
    Ok(())
}

fn value_to_string(vm: &mut Engine, v: &Value) -> Result<String, VmExit> {
    if is_null(vm, v) {
        return Ok("null".to_string());
    }
    if let Value::Obj(o) = v {
        if let Ok(sv) = vm.heap.as_str(*o) {
            return Ok(sv.to_string());
        }
        let class = vm.heap.class_of(*o).unwrap_or_default().to_string();
        if class == JSON_OBJECT || class == JSON_ARRAY {
            let mut out = String::new();
            write_value(vm, v, &mut out)?;
            return Ok(out);
        }
        if vm
            .heap
            .class_of(*o)
            .map(|c| c == "Ljava/lang/Boolean;")
            .unwrap_or(false)
        {
            let on = matches!(vm.heap.get_field(*o, "value"), Ok(Value::Int(1)));
            return Ok(if on { "true" } else { "false" }.to_string());
        }
        if let Ok(Value::Obj(s)) = vm.invoke_instance(*o, "toString", "()Ljava/lang/String;", &[]) {
            return Ok(vm.heap.as_str(s).unwrap_or_default().to_string());
        }
    }
    if let Some(n) = num_str(vm, v) {
        return Ok(n);
    }
    Ok(String::new())
}

// --------------------------------------------------------------- VM

pub(crate) fn static_intrinsic(
    vm: &mut Engine,
    class: &str,
    name: &str,
    sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let _ = sig;
    if class != JSON_OBJECT && class != JSON_ARRAY {
        return Ok(None);
    }
    Ok(Some(match name {
        "quote" => {
            let mut out = String::new();
            quote_into(&str_arg(vm, args, 0), &mut out);
            Value::Obj(crate::intrinsics::alloc_string(vm, out)?)
        }
        "wrap" => {
            let v = args.first().cloned().unwrap_or(Value::Null);
            wrap_value(vm, &v)?
        }
        _ => return Ok(None),
    }))
}

fn wrap_value(vm: &mut Engine, v: &Value) -> Result<Value, VmExit> {
    if let Value::Obj(o) = v {
        let class = vm.heap.class_of(*o).unwrap_or_default().to_string();
        if class == JSON_OBJECT || class == JSON_ARRAY {
            return Ok(v.clone());
        }
        if vm.fw.objects.contains_key(o) {
            if let Some(crate::framework::HostObj::Map { entries }) = vm.fw.objects.get(o).cloned()
            {
                let obj = vm
                    .heap
                    .alloc_instance(JSON_OBJECT.to_string(), Vec::new())
                    .map_err(crate::err::oom_throwable)?;
                for (k, val) in entries {
                    let key = match &k {
                        Value::Obj(s) => vm.heap.as_str(*s).unwrap_or_default().to_string(),
                        other => value_to_string(vm, other)?,
                    };
                    let wrapped = wrap_value(vm, &val)?;
                    json_put(vm, obj, &key, wrapped)?;
                }
                return Ok(Value::Obj(obj));
            }
            if let Some(crate::framework::HostObj::List { elems: items }) =
                vm.fw.objects.get(o).cloned()
            {
                let arr = vm
                    .heap
                    .alloc_instance(JSON_ARRAY.to_string(), Vec::new())
                    .map_err(crate::err::oom_throwable)?;
                for e in items.iter() {
                    let wrapped = wrap_value(vm, e)?;
                    elems(vm, arr, "json_items")?.push(wrapped);
                }
                return Ok(Value::Obj(arr));
            }
        }
    }
    Ok(v.clone())
}

pub(crate) fn instance_intrinsic(
    vm: &mut Engine,
    recv_class: &str,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if recv_class == JSON_OBJECT {
        return json_object(vm, name, sig, recv, args);
    }
    if recv_class == JSON_ARRAY {
        return json_array(vm, name, sig, recv, args);
    }
    Ok(None)
}

fn json_object(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let key = str_arg(vm, args, 0);
    Ok(Some(match name {
        "<init>" => {
            if sig.contains("String") {
                let text = str_arg(vm, args, 0);
                parse_into(vm, recv, &text)?;
            } else if sig.contains("Map") {
                if let Some(Value::Obj(m)) = args.first().cloned() {
                    let wrapped = wrap_value(vm, &Value::Obj(m))?;
                    if let Value::Obj(src) = wrapped {
                        let items = elems_ref(vm, src, "json_map");
                        *elems(vm, recv, "json_map")? = items;
                    }
                }
            } else {
                let _ = elems(vm, recv, "json_map")?;
            }
            Value::Null
        }
        "put" => {
            let raw = args.get(1).cloned().unwrap_or(Value::Null);
            // put(name, boolean) arrives as a raw Int and would serialize
            // as a number; box it (org.json stores Boolean objects).
            let v = if sig == "(Ljava/lang/String;Z)Lorg/json/JSONObject;" {
                Value::Obj(vm.heap.alloc_instance(
                    "Ljava/lang/Boolean;".to_string(),
                    vec![("value".to_string(), raw.clone())],
                )?)
            } else {
                raw
            };
            json_put(vm, recv, &key, v)?;
            Value::Obj(recv)
        }
        "get" => match json_get(vm, recv, &key) {
            Some(v) => v,
            None => return Err(exc(format!("JSONObject[\"{key}\"] not found."))),
        },
        "opt" => json_get(vm, recv, &key).unwrap_or(Value::Null),
        "has" => Value::Int(json_get(vm, recv, &key).is_some() as i32),
        "isNull" => {
            let v = json_get(vm, recv, &key);
            Value::Int(v.map(|v| is_null(vm, &v)).unwrap_or(true) as i32)
        }
        "length" => Value::Int((elems_ref(vm, recv, "json_map").len() / 2) as i32),
        "remove" => {
            let prev = json_get(vm, recv, &key).unwrap_or(Value::Null);
            let mut list = elems_ref(vm, recv, "json_map");
            let keys: Vec<String> = list
                .iter()
                .step_by(2)
                .map(|k| match k {
                    Value::Obj(o) => vm.heap.as_str(*o).unwrap_or_default().to_string(),
                    _ => String::new(),
                })
                .collect();
            if let Some(pos) = keys.iter().position(|k| k == &key) {
                list.drain(pos * 2..pos * 2 + 2);
            }
            *elems(vm, recv, "json_map")? = list;
            prev
        }
        "getString" => match json_get(vm, recv, &key) {
            Some(v) => {
                let text = value_to_string(vm, &v)?;
                Value::Obj(crate::intrinsics::alloc_string(vm, text)?)
            }
            None => return Err(exc(format!("JSONObject[\"{key}\"] not found."))),
        },
        "optString" => {
            let def = if args.len() > 1 {
                str_arg(vm, args, 1)
            } else {
                String::new()
            };
            match json_get(vm, recv, &key) {
                Some(v) if !is_null(vm, &v) => {
                    let text = value_to_string(vm, &v)?;
                    Value::Obj(crate::intrinsics::alloc_string(vm, text)?)
                }
                _ => Value::Obj(crate::intrinsics::alloc_string(vm, def)?),
            }
        }
        "getJSONObject" => match json_get(vm, recv, &key) {
            Some(v @ Value::Obj(_)) => v,
            _ => return Err(exc(format!("JSONObject[\"{key}\"] is not a JSONObject."))),
        },
        "optJSONObject" => match json_get(vm, recv, &key) {
            Some(v @ Value::Obj(_)) => v,
            _ => Value::Null,
        },
        "getJSONArray" => match json_get(vm, recv, &key) {
            Some(v @ Value::Obj(_)) => v,
            _ => return Err(exc(format!("JSONObject[\"{key}\"] is not a JSONArray."))),
        },
        "optJSONArray" => match json_get(vm, recv, &key) {
            Some(v @ Value::Obj(_)) => v,
            _ => Value::Null,
        },
        "getInt" | "getLong" | "getDouble" | "getBoolean" => {
            let v = json_get(vm, recv, &key)
                .ok_or_else(|| exc(format!("JSONObject[\"{key}\"] not found.")))?;
            coerce(vm, &v, name)?
        }
        "optInt" | "optLong" | "optDouble" | "optBoolean" => {
            let def = args.get(1).cloned();
            match json_get(vm, recv, &key) {
                Some(v) if !is_null(vm, &v) => {
                    coerce(vm, &v, name).unwrap_or_else(|_| def_result(vm, name, def))
                }
                _ => def_result(vm, name, def),
            }
        }
        "keys" => {
            let list = vm
                .heap
                .alloc_instance("Ljava/util/ArrayList;".to_string(), Vec::new())
                .map_err(crate::err::oom_throwable)?;
            let pairs = elems_ref(vm, recv, "json_map");
            let mut keys = Vec::new();
            let mut i = 0;
            while i < pairs.len() {
                keys.push(pairs[i].clone());
                i += 2;
            }
            vm.fw
                .objects
                .insert(list, crate::framework::HostObj::List { elems: keys });
            vm.invoke_instance(list, "iterator", "()Ljava/util/Iterator;", &[])?
        }
        "names" => {
            let arr = vm
                .heap
                .alloc_instance(JSON_ARRAY.to_string(), Vec::new())
                .map_err(crate::err::oom_throwable)?;
            let pairs = elems_ref(vm, recv, "json_map");
            let mut i = 0;
            let mut out = Vec::new();
            while i < pairs.len() {
                out.push(pairs[i].clone());
                i += 2;
            }
            *elems(vm, arr, "json_items")? = out;
            Value::Obj(arr)
        }
        "toString" => {
            let mut out = String::new();
            write_object(vm, recv, &mut out)?;
            Value::Obj(crate::intrinsics::alloc_string(vm, out)?)
        }
        _ => return Ok(None),
    }))
}

fn def_result(vm: &mut Engine, name: &str, def: Option<Value>) -> Value {
    let _ = vm;
    def.unwrap_or(match name {
        "optLong" => Value::Long(0),
        "optDouble" => Value::Double(0.0),
        _ => Value::Int(0),
    })
}

fn coerce(vm: &Engine, v: &Value, name: &str) -> Result<Value, VmExit> {
    let n = num_str(vm, v);
    Ok(match name {
        "getInt" | "optInt" => Value::Int(
            n.and_then(|s| s.parse::<f64>().ok())
                .map(|f| f as i32)
                .unwrap_or(0),
        ),
        "getLong" | "optLong" => Value::Long(
            n.and_then(|s| s.parse::<f64>().ok())
                .map(|f| f as i64)
                .unwrap_or(0),
        ),
        "getDouble" | "optDouble" => {
            Value::Double(n.and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0))
        }
        _ => Value::Int(match v {
            Value::Int(i) => (*i != 0) as i32,
            Value::Obj(o) => matches!(vm.heap.get_field(*o, "value"), Ok(Value::Int(1))) as i32,
            _ => 0,
        }),
    })
}

fn json_array(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
    Ok(Some(match name {
        "<init>" => {
            if sig.contains("String") {
                let text = str_arg(vm, args, 0);
                parse_into(vm, recv, &text)?;
            } else if sig.contains("Collection") {
                if let Some(Value::Obj(c)) = args.first().cloned() {
                    let wrapped = wrap_value(vm, &Value::Obj(c))?;
                    if let Value::Obj(src) = wrapped {
                        let items = elems_ref(vm, src, "json_items");
                        *elems(vm, recv, "json_items")? = items;
                    }
                }
            } else {
                let _ = elems(vm, recv, "json_items")?;
            }
            Value::Null
        }
        "put" => {
            if args.len() >= 2 && sig.starts_with("(I") {
                let v = args.get(1).cloned().unwrap_or(Value::Null);
                let nullv = null_value(vm)?;
                let list = elems(vm, recv, "json_items")?;
                while list.len() <= idx {
                    list.push(nullv.clone());
                }
                list[idx] = v;
            } else {
                let v = args.first().cloned().unwrap_or(Value::Null);
                elems(vm, recv, "json_items")?.push(v);
            }
            Value::Obj(recv)
        }
        "get" => elems_ref(vm, recv, "json_items")
            .get(idx)
            .cloned()
            .ok_or_else(|| exc(format!("JSONArray[{idx}] not found.")))?,
        "opt" => elems_ref(vm, recv, "json_items")
            .get(idx)
            .cloned()
            .unwrap_or(Value::Null),
        "length" => Value::Int(elems_ref(vm, recv, "json_items").len() as i32),
        "isNull" => {
            let v = elems_ref(vm, recv, "json_items").get(idx).cloned();
            Value::Int(v.map(|v| is_null(vm, &v)).unwrap_or(true) as i32)
        }
        "getString" => {
            let v = elems_ref(vm, recv, "json_items")
                .get(idx)
                .cloned()
                .ok_or_else(|| exc(format!("JSONArray[{idx}] not found.")))?;
            {
                let text = value_to_string(vm, &v)?;
                Value::Obj(crate::intrinsics::alloc_string(vm, text)?)
            }
        }
        "optString" => {
            let def = if args.len() > 1 {
                str_arg(vm, args, 1)
            } else {
                String::new()
            };
            match elems_ref(vm, recv, "json_items").get(idx).cloned() {
                Some(v) if !is_null(vm, &v) => {
                    let text = value_to_string(vm, &v)?;
                    Value::Obj(crate::intrinsics::alloc_string(vm, text)?)
                }
                _ => Value::Obj(crate::intrinsics::alloc_string(vm, def)?),
            }
        }
        "getJSONObject" => elems_ref(vm, recv, "json_items")
            .get(idx)
            .cloned()
            .ok_or_else(|| exc(format!("JSONArray[{idx}] is not a JSONObject.")))?,
        "optJSONObject" => elems_ref(vm, recv, "json_items")
            .get(idx)
            .cloned()
            .unwrap_or(Value::Null),
        "getJSONArray" => elems_ref(vm, recv, "json_items")
            .get(idx)
            .cloned()
            .ok_or_else(|| exc(format!("JSONArray[{idx}] is not a JSONArray.")))?,
        "optJSONArray" => elems_ref(vm, recv, "json_items")
            .get(idx)
            .cloned()
            .unwrap_or(Value::Null),
        "getInt" | "getLong" | "getDouble" | "getBoolean" => {
            let v = elems_ref(vm, recv, "json_items")
                .get(idx)
                .cloned()
                .ok_or_else(|| exc(format!("JSONArray[{idx}] not found.")))?;
            coerce(vm, &v, name)?
        }
        "optInt" | "optLong" | "optDouble" | "optBoolean" => {
            let def = args.get(1).cloned();
            match elems_ref(vm, recv, "json_items").get(idx).cloned() {
                Some(v) if !is_null(vm, &v) => {
                    coerce(vm, &v, name).unwrap_or_else(|_| def_result(vm, name, def))
                }
                _ => def_result(vm, name, def),
            }
        }
        "remove" => {
            let list = elems(vm, recv, "json_items")?;
            if idx < list.len() {
                list.remove(idx)
            } else {
                Value::Null
            }
        }
        "toString" => {
            let mut out = String::new();
            write_array(vm, recv, &mut out)?;
            Value::Obj(crate::intrinsics::alloc_string(vm, out)?)
        }
        _ => return Ok(None),
    }))
}
