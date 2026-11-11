//! Compiled XML resources: `Resources.getXml`/`getLayout`/`getAnimation`
//! return an `XmlResourceParser` over the APK's binary XML (AXML). It is
//! both an `XmlPullParser` and an `AttributeSet`.
//!
//! `Resources.obtainAttributes(set, attrs)` and
//! `Theme/Context.obtainStyledAttributes(set, attrs, ..)` on such a set
//! return a TypedArray whose indexes are filled from the current start tag's
//! attributes, matched by attribute resource id.
//!
//! Event model: START_DOCUMENT, then START_TAG/TEXT/END_TAG in document
//! order, then END_DOCUMENT. Parser and TypedArray side state lives in a
//! per-thread table keyed by object.
//!
//! **Deviations:**
//! - Theme attribute references (`?attr/x`) are reported as absent: themes
//!   are not resolved.
//! - Default styles (`defStyleAttr`/`defStyleRes`) are ignored.
//! - AXML decoding folds `TYPE_INT_COLOR_*` into plain ints, so
//!   `TypedArray.getValue` reports an int as a color type when the attribute
//!   name contains "color" or "tint".

use std::collections::HashMap;

use rd_apk::axml::{AttrValue, XmlAttribute, XmlElement};

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::heap::ObjRef;
use crate::value::Value;

pub(crate) const PARSER: &str = "Landroid/content/res/XmlResourceParser;";
const RESOURCES: &str = "Landroid/content/res/Resources;";
const ANDROID_NS: &str = "http://schemas.android.com/apk/res/android";

#[derive(Clone, Debug)]
pub(crate) enum Ev {
    Start {
        name: String,
        attrs: Vec<XmlAttribute>,
        depth: i32,
    },
    Text(String, i32),
    End {
        name: String,
        depth: i32,
    },
}

#[derive(Debug)]
pub(crate) struct ParserState {
    file: String,
    events: Vec<Ev>,
    /// -1 = START_DOCUMENT, events.len() = END_DOCUMENT.
    pos: isize,
}

/// Estimativa dos bytes de host de um parser: caminho, eventos, nomes, textos e atributos.
fn parser_host_cost(file: &str, events: &[Ev]) -> usize {
    events.iter().fold(
        file.len() + std::mem::size_of::<ParserState>(),
        |acc, ev| {
            let payload = match ev {
                Ev::Start { name, attrs, .. } => {
                    name.len() + attrs.len() * std::mem::size_of::<XmlAttribute>()
                }
                Ev::Text(s, _) => s.len(),
                Ev::End { name, .. } => name.len(),
            };
            acc + std::mem::size_of::<Ev>() + payload
        },
    )
}

fn parser_host_release(held: &mut usize, cost: usize) {
    *held = held.saturating_sub(cost);
}

/// Estado de XmlPullParser/arrays por engine (ADR 024, passo 2).
#[derive(Debug, Default)]
pub(crate) struct Tables {
    pub(crate) parsers: HashMap<ObjRef, ParserState>,
    pub(crate) arrays: HashMap<ObjRef, Vec<Option<XmlAttribute>>>,
    /// Bytes de host (estimativa) retidos por `parsers`. Ficam fora de
    /// `heap.used()`, então são cobrados contra `heap_budget` na abertura (#370).
    pub(crate) host_bytes: usize,
}

/// GC: drop state of collected objects (slots are reused).
pub(crate) fn gc_purge(t: &mut Tables, alive: &dyn Fn(ObjRef) -> bool) {
    let held = &mut t.host_bytes;
    t.parsers.retain(|k, v| {
        if alive(*k) {
            return true;
        }
        parser_host_release(held, parser_host_cost(&v.file, &v.events));
        false
    });
    t.arrays.retain(|k, _| alive(*k));
}

/// Achata a árvore em eventos na mesma ordem de antes (Start, Text, filhos, End),
/// com pilha explícita: uma árvore profunda não estoura a stack do host (#370 item 4).
fn flatten(root: &XmlElement, depth: i32, out: &mut Vec<Ev>) {
    enum Step<'a> {
        Open(&'a XmlElement, i32),
        Close(&'a str, i32),
    }
    let mut stack = vec![Step::Open(root, depth)];
    while let Some(step) = stack.pop() {
        match step {
            Step::Open(e, d) => {
                out.push(Ev::Start {
                    name: e.name.clone(),
                    attrs: e.attrs.clone(),
                    depth: d,
                });
                if let Some(t) = &e.text {
                    if !t.trim().is_empty() {
                        out.push(Ev::Text(t.clone(), d));
                    }
                }
                stack.push(Step::Close(&e.name, d));
                for c in e.children.iter().rev() {
                    stack.push(Step::Open(c, d + 1));
                }
            }
            Step::Close(name, d) => out.push(Ev::End {
                name: name.to_string(),
                depth: d,
            }),
        }
    }
}

fn exc(class: &str, msg: impl Into<String>) -> VmExit {
    Throwable::new(class, msg.into()).into()
}

fn not_found(id: i32) -> VmExit {
    exc(
        "Landroid/content/res/Resources$NotFoundException;",
        format!("Resource ID #0x{:x}", id as u32),
    )
}

/// Opens the compiled XML behind resource `id` as a parser object.
/// Bytes of the APK file a file-based resource id resolves to
/// (res/raw/*, res/xml/*, …), following resource references.
fn resource_file_bytes(vm: &mut Engine, id: i32) -> Result<Vec<u8>, VmExit> {
    let mut rid = id as u32;
    let mut path = None;
    for _ in 0..8 {
        match vm.resolve_resource_value(rid).map_err(|_| not_found(id))? {
            rd_apk::arsc::ResolvedValue::Str(s) => {
                path = Some(s);
                break;
            }
            rd_apk::arsc::ResolvedValue::Reference(r) => rid = r,
            _ => break,
        }
    }
    let path = path
        .filter(|p| p.starts_with("res/"))
        .ok_or_else(|| not_found(id))?;
    let apk = vm.fw.resources.as_ref().ok_or_else(|| not_found(id))?;
    let entry = apk.zip.find(&path).ok_or_else(|| not_found(id))?;
    apk.zip.read_entry(entry).map_err(|_| not_found(id))
}

fn open_xml(vm: &mut Engine, id: i32) -> Result<Value, VmExit> {
    let mut rid = id as u32;
    let mut path = None;
    for _ in 0..8 {
        match vm.resolve_resource_value(rid).map_err(|_| not_found(id))? {
            rd_apk::arsc::ResolvedValue::Str(s) => {
                path = Some(s);
                break;
            }
            rd_apk::arsc::ResolvedValue::Reference(r) => rid = r,
            _ => break,
        }
    }
    let path = path
        .filter(|p| p.ends_with(".xml"))
        .ok_or_else(|| not_found(id))?;
    let bytes = {
        let apk = vm.fw.resources.as_ref().ok_or_else(|| not_found(id))?;
        let entry = apk.zip.find(&path).ok_or_else(|| not_found(id))?;
        apk.zip.read_entry(entry).map_err(|_| not_found(id))?
    };
    let doc = rd_apk::axml::parse(&bytes).map_err(|e| {
        exc(
            "Landroid/content/res/Resources$NotFoundException;",
            format!("{path}: {}", e.cause),
        )
    })?;
    let mut events = Vec::new();
    flatten(&doc.root, 1, &mut events);
    // #370: a árvore vive no host com pouco custo no heap guest; cobra-se o
    // total retido contra o orçamento antes de guardar o parser.
    let cost = parser_host_cost(&path, &events);
    let held = vm.fw.xml.host_bytes;
    if vm.heap.used().saturating_add(held).saturating_add(cost) > vm.heap.budget() {
        return Err(VmExit::Exception(Throwable::new(
            "Ljava/lang/OutOfMemoryError;",
            format!(
                "XML parser for {path} needs {cost} host bytes; heap_budget is {}",
                vm.heap.budget()
            ),
        )));
    }
    let r = vm
        .heap
        .alloc_instance(PARSER.to_string(), Vec::new())
        .map_err(crate::err::oom_throwable)?;
    let old = {
        vm.fw.xml.parsers.insert(
            r,
            ParserState {
                file: path,
                events,
                pos: -1,
            },
        )
    };
    if let Some(old) = old {
        parser_host_release(
            &mut vm.fw.xml.host_bytes,
            parser_host_cost(&old.file, &old.events),
        );
    }
    vm.fw.xml.host_bytes = vm.fw.xml.host_bytes.saturating_add(cost);
    Ok(Value::Obj(r))
}

fn is_parser(vm: &Engine, r: ObjRef) -> bool {
    vm.fw.xml.parsers.contains_key(&r)
}

/// Attributes of the parser's current start tag.
fn current_attrs(vm: &Engine, r: ObjRef) -> Option<Vec<XmlAttribute>> {
    {
        let st = vm.fw.xml.parsers.get(&r)?;
        match st.events.get(usize::try_from(st.pos).ok()?) {
            Some(Ev::Start { attrs, .. }) => Some(attrs.clone()),
            _ => None,
        }
    }
}

/// Fills TypedArray `ta` from AttributeSet `set` for the `attrs` id array.
/// No-op unless `set` is a parser created here.
pub(crate) fn fill_typed_array(
    vm: &mut Engine,
    ta: ObjRef,
    set: Option<&Value>,
    attrs: Option<&Value>,
) {
    let (Some(Value::Obj(s)), Some(Value::Obj(a))) = (set, attrs) else {
        return;
    };
    let Some(cur) = current_attrs(vm, *s) else {
        return;
    };
    let Ok((ids, _)) = vm.heap.as_array_elems(*a) else {
        return;
    };
    let vals: Vec<Option<XmlAttribute>> = ids
        .iter()
        .map(|id| {
            let id = match id {
                Value::Int(i) => *i as u32,
                _ => return None,
            };
            cur.iter()
                .find(|x| {
                    x.res_id == Some(id)
                        && !matches!(x.value, AttrValue::Attribute(_) | AttrValue::Null)
                })
                .cloned()
        })
        .collect();
    ({
        let arrays = &mut vm.fw.xml.arrays;
        if let Some(existing) = arrays.get(&ta) {
            let mut merged = existing.clone();
            merged.resize(vals.len(), None);
            for (i, value) in vals.into_iter().enumerate() {
                if value.is_some() {
                    merged[i] = value;
                }
            }
            arrays.insert(ta, merged);
        } else {
            arrays.insert(ta, vals);
        }
    });
}

/// Fill a TypedArray from a resolved style bag. Missing attributes remain
/// `None`, allowing the normal TypedArray default-value path to apply.
pub(crate) fn fill_theme_typed_array(
    vm: &mut Engine,
    ta: ObjRef,
    attrs: Option<&Value>,
    style_id: u32,
) {
    let Some(Value::Obj(a)) = attrs else { return };
    let Ok((ids, _)) = vm.heap.as_array_elems(*a) else {
        return;
    };
    let Some(arsc) = vm.fw.resources.as_ref().and_then(|apk| apk.arsc.as_ref()) else {
        return;
    };
    let values =
        ids.iter()
            .map(|id| {
                let Value::Int(attr_id) = id else {
                    return None;
                };
                let attr_id = *attr_id as u32;
                let value = arsc.resolve_style_attribute(style_id, attr_id)?;
                let value = match value {
                    rd_apk::arsc::ResolvedValue::Str(s) => AttrValue::String(s),
                    rd_apk::arsc::ResolvedValue::Boolean(b) => AttrValue::Bool(b),
                    rd_apk::arsc::ResolvedValue::Integer(i)
                    | rd_apk::arsc::ResolvedValue::Color(i) => AttrValue::Int(i),
                    rd_apk::arsc::ResolvedValue::Float(f) => AttrValue::Float(f),
                    rd_apk::arsc::ResolvedValue::Dimension { value, unit } => {
                        AttrValue::Dimension(value, unit)
                    }
                    rd_apk::arsc::ResolvedValue::Reference(r) => AttrValue::Reference(r),
                    rd_apk::arsc::ResolvedValue::Attribute(a) => AttrValue::Attribute(a),
                };
                Some(XmlAttribute {
                    ns: None,
                    name: arsc.resolve_name(attr_id).unwrap_or_default(),
                    raw: None,
                    value,
                    res_id: Some(attr_id),
                })
            })
            .collect();
    vm.fw.xml.arrays.insert(ta, values);
}

fn str_val(vm: &mut Engine, s: String) -> Result<Value, VmExit> {
    Ok(Value::Obj(crate::intrinsics::alloc_string(vm, s)?))
}

fn str_arg(vm: &Engine, v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::Obj(r)) => vm.heap.as_str(*r).ok().map(str::to_string),
        _ => None,
    }
}

fn density() -> f32 {
    crate::android_res::device().dpi as f32 / 160.0
}

/// Follows resource references to a final value.
fn deref(vm: &Engine, v: &AttrValue) -> Option<rd_apk::arsc::ResolvedValue> {
    use rd_apk::arsc::ResolvedValue as R;
    let mut cur = match v {
        AttrValue::Reference(id) => R::Reference(*id),
        AttrValue::String(s) => return Some(R::Str(s.clone())),
        AttrValue::Int(i) => return Some(R::Integer(*i)),
        AttrValue::Bool(b) => return Some(R::Boolean(*b)),
        AttrValue::Float(f) => return Some(R::Float(*f)),
        AttrValue::Dimension(f, u) => {
            return Some(R::Dimension {
                value: *f,
                unit: *u,
            })
        }
        _ => return None,
    };
    for _ in 0..8 {
        match cur {
            R::Reference(id) => cur = vm.resolve_resource_value(id).ok()?,
            other => return Some(other),
        }
    }
    None
}

fn to_px(value: f32, unit: rd_apk::axml::ComplexUnit) -> f32 {
    let name = format!("{unit:?}").to_ascii_lowercase();
    let d = density();
    match name.as_str() {
        "dip" | "dp" | "sp" => value * d,
        "pt" => value * 280.0 / 72.0,
        "in" => value * 280.0,
        "mm" => value * 280.0 / 25.4,
        _ => value,
    }
}

fn as_f32(vm: &Engine, v: &AttrValue) -> Option<f32> {
    use rd_apk::arsc::ResolvedValue as R;
    Some(match deref(vm, v)? {
        R::Float(f) => f,
        R::Integer(i) | R::Color(i) => i as f32,
        R::Dimension { value, unit } => to_px(value, unit),
        R::Str(s) => s.parse().ok()?,
        R::Boolean(b) => b as i32 as f32,
        R::Reference(_) | R::Attribute(_) => return None,
    })
}

fn as_i32(vm: &Engine, v: &AttrValue) -> Option<i32> {
    use rd_apk::arsc::ResolvedValue as R;
    Some(match deref(vm, v)? {
        R::Integer(i) | R::Color(i) => i,
        R::Boolean(b) => b as i32,
        R::Float(f) => f as i32,
        R::Dimension { value, unit } => to_px(value, unit) as i32,
        R::Str(s) => {
            let t = s.trim();
            if let Some(h) = t.strip_prefix('#') {
                // #RGB / #ARGB expand each nibble (#f00 == #ff0000), #RRGGBB / #AARRGGBB as-is.
                // Android aceita só #RGB, #ARGB, #RRGGBB e #AARRGGBB, todos hex
                // (#370 item 3). `from_str_radix` aceitaria um `+` inicial.
                if !matches!(h.len(), 3 | 4 | 6 | 8) || !h.chars().all(|c| c.is_ascii_hexdigit()) {
                    return None;
                }
                let expanded: String = match h.len() {
                    3 | 4 => h.chars().flat_map(|c| [c, c]).collect(),
                    _ => h.to_string(),
                };
                let n = u32::from_str_radix(&expanded, 16).ok()?;
                (if expanded.len() <= 6 {
                    n | 0xFF00_0000
                } else {
                    n
                }) as i32
            } else {
                t.parse().ok()?
            }
        }
        R::Reference(_) | R::Attribute(_) => return None,
    })
}

/// TypedArray reads backed by a filled index table. `None` = not ours.
fn typed_array_get(
    vm: &mut Engine,
    recv: ObjRef,
    name: &str,
    sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let Some(vals) = vm.fw.xml.arrays.get(&recv).cloned() else {
        return Ok(None);
    };
    let idx = match args.first() {
        Some(Value::Int(i)) => *i as usize,
        _ => usize::MAX,
    };
    let attr = vals.get(idx).cloned().flatten();
    let def = args.last().cloned().unwrap_or(Value::Null);
    let v = match name {
        "length" => Value::Int(vals.len() as i32),
        "getIndexCount" => Value::Int(vals.iter().filter(|v| v.is_some()).count() as i32),
        "getIndex" => {
            let n = vals
                .iter()
                .enumerate()
                .filter(|(_, v)| v.is_some())
                .nth(idx)
                .map(|(i, _)| i as i32);
            Value::Int(n.unwrap_or(0))
        }
        "recycle" | "close" => {
            vm.fw.xml.arrays.remove(&recv);
            Value::Null
        }
        _ => {
            let Some(a) = attr else { return Ok(None) }; // absent → legacy default path
            match name {
                "hasValue" | "hasValueOrEmpty" => Value::Int(1),
                "getFloat" => as_f32(vm, &a.value).map(Value::Float).unwrap_or(def),
                "getDimension" => as_f32(vm, &a.value).map(Value::Float).unwrap_or(def),
                "getDimensionPixelSize" => as_f32(vm, &a.value)
                    .map(|f| {
                        Value::Int(if f > 0.0 {
                            (f + 0.5).max(1.0) as i32
                        } else {
                            f.round() as i32
                        })
                    })
                    .unwrap_or(def),
                "getDimensionPixelOffset" | "getLayoutDimension" => as_f32(vm, &a.value)
                    .map(|f| Value::Int(f as i32))
                    .unwrap_or(def),
                "getInt" | "getInteger" | "getColor" => {
                    as_i32(vm, &a.value).map(Value::Int).unwrap_or(def)
                }
                "getBoolean" => as_i32(vm, &a.value)
                    .map(|i| Value::Int((i != 0) as i32))
                    .unwrap_or(def),
                "getResourceId" | "getSourceResourceId" => match a.value {
                    AttrValue::Reference(id) => Value::Int(id as i32),
                    _ => def,
                },
                "getString" | "getText" | "getNonResourceString" => match deref(vm, &a.value) {
                    Some(rd_apk::arsc::ResolvedValue::Str(s)) => str_val(vm, s)?,
                    Some(_) if name != "getNonResourceString" => str_val(vm, a.text())?,
                    _ => Value::Null,
                },
                "getType" => Value::Int(type_of(&a)),
                "getValue" if sig.contains("TypedValue") => {
                    let Some(Value::Obj(tv)) = args.get(1).cloned() else {
                        return Ok(Some(Value::Int(0)));
                    };
                    let ty = type_of(&a);
                    let data = match &a.value {
                        AttrValue::Reference(id) => *id as i32,
                        AttrValue::Float(f) => f.to_bits() as i32,
                        AttrValue::Bool(b) => *b as i32,
                        other => as_i32(vm, other).unwrap_or(0),
                    };
                    let _ = vm.heap.put_field(tv, "type", Value::Int(ty));
                    let _ = vm.heap.put_field(tv, "data", Value::Int(data));
                    let _ = vm.heap.put_field(
                        tv,
                        "resourceId",
                        Value::Int(if let AttrValue::Reference(id) = a.value {
                            id as i32
                        } else {
                            0
                        }),
                    );
                    if let AttrValue::String(s) = &a.value {
                        let s = str_val(vm, s.clone())?;
                        let _ = vm.heap.put_field(tv, "string", s);
                    }
                    Value::Int(1)
                }
                // Resolve an ARRAY resource (string-array/integer-array). This
                // is what a real preference screen needs: `entries`/
                // `entryValues` are array references, and returning null made
                // `MultiSelectListPreference.<init>` do `array-length` on null
                // (NewPipe cold start).
                // `getTextArray(int)` takes no default argument, so an
                // attribute that is not an array answers null (the JDK
                // returns null too) — never the index that `def` would hold.
                "getTextArray" => match array_values(vm, &a.value)? {
                    Some(v) => v,
                    None => Value::Null,
                },
                // Complex colors / drawables need inflation that is not modelled.
                "getColorStateList" | "getDrawable" | "getFont" => Value::Null,
                _ => return Ok(None),
            }
        }
    };
    Ok(Some(v))
}

/// Materializes an ARRAY resource referenced by `value` as a
/// `CharSequence[]` (the runtime type of `TypedArray.getTextArray`).
///
/// `Ok(None)` = the attribute is absent or does not point at an array, so the
/// caller keeps the legacy default (null) instead of inventing an empty array.
/// A reference to something that is not an array is also `None`: Leak 1 forbids
/// pretending that a style bag is a list of entries.
fn array_values(vm: &mut Engine, value: &AttrValue) -> Result<Option<Value>, VmExit> {
    use rd_apk::arsc::ResolvedValue as R;
    let AttrValue::Reference(id) = value else {
        return Ok(None);
    };
    let dev = vm.resource_device_config();
    let Some(items) = vm
        .fw
        .resources
        .as_ref()
        .and_then(|a| a.arsc.as_ref())
        .and_then(|arsc| arsc.resolve_array_for(*id, &dev))
    else {
        return Ok(None);
    };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let v = match item.data_type {
            0x03 => match item.string {
                Some(t) => str_val(vm, t)?,
                None => Value::Null,
            },
            // INT_DEC/INT_HEX/INT_BOOLEAN/INT_COLOR_*: raw i32, like TypedValue.data
            0x10 | 0x11 | 0x12 | 0x1c..=0x1f => Value::Int(item.data as i32),
            0x04 => Value::Float(f32::from_bits(item.data)),
            // A reference inside the array is dereferenced with the same
            // resource policy the rest of the surface uses.
            0x01 => match vm.resolve_resource_value(item.data) {
                Ok(R::Str(t)) => str_val(vm, t)?,
                Ok(R::Integer(i)) => Value::Int(i),
                Ok(R::Boolean(b)) => Value::Int(b as i32),
                Ok(R::Float(f)) => Value::Float(f),
                _ => Value::Null,
            },
            _ => Value::Null,
        };
        out.push(v);
    }
    Ok(Some(new_char_sequence_array(vm, out)?))
}

fn new_char_sequence_array(vm: &mut Engine, vals: Vec<Value>) -> Result<Value, VmExit> {
    let a = vm.heap.alloc_array(
        crate::heap::ElemKind::Obj,
        vals.len(),
        "Ljava/lang/CharSequence;",
    )?;
    *vm.heap
        .array_elems_mut(a)
        .map_err(|e| VmExit::from(crate::err::vm_error("INVALID_FORMAT", e)))? = vals;
    Ok(Value::Obj(a))
}

fn type_of(a: &XmlAttribute) -> i32 {
    match &a.value {
        AttrValue::Null => 0,
        AttrValue::Reference(_) => 1,
        AttrValue::Attribute(_) => 2,
        AttrValue::String(_) => 3,
        AttrValue::Float(_) => 4,
        AttrValue::Dimension(..) => 5,
        AttrValue::Fraction(..) => 6,
        AttrValue::Bool(_) => 0x12,
        AttrValue::Int(_) => {
            let n = a.name.to_ascii_lowercase();
            if n.contains("color") || n.contains("tint") {
                0x1c
            } else {
                0x10
            }
        }
        AttrValue::Raw { dtype, .. } => *dtype as i32,
    }
}

fn find_attr(vm: &Engine, recv: ObjRef, args: &[Value], indexed: bool) -> Option<XmlAttribute> {
    let attrs = current_attrs(vm, recv)?;
    if indexed {
        let i = match args.first() {
            Some(Value::Int(i)) => *i as usize,
            _ => return None,
        };
        return attrs.get(i).cloned();
    }
    let ns = str_arg(vm, args.first());
    let name = str_arg(vm, args.get(1))?;
    attrs.into_iter().find(|a| {
        a.name == name
            && match ns.as_deref() {
                None => true,
                Some(ANDROID_NS) => a.is_android(),
                Some(other) => a.ns.as_deref() == Some(other),
            }
    })
}

pub fn instance_intrinsic(
    vm: &mut Engine,
    recv_class: &str,
    mref_class: &str,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if recv_class == RESOURCES || mref_class == RESOURCES {
        return Ok(match (name, sig) {
            (
                "getXml" | "getLayout" | "getAnimation",
                "(I)Landroid/content/res/XmlResourceParser;",
            ) => {
                let id = match args.first() {
                    Some(Value::Int(i)) => *i,
                    _ => 0,
                };
                Some(open_xml(vm, id)?)
            }
            ("openRawResource", "(I)Ljava/io/InputStream;")
            | ("openRawResource", "(ILandroid/util/TypedValue;)Ljava/io/InputStream;") => {
                let id = match args.first() {
                    Some(Value::Int(i)) => *i,
                    _ => 0,
                };
                let bytes = resource_file_bytes(vm, id)?;
                Some(crate::jio_chars::new_byte_input_stream(vm, bytes)?)
            }
            _ => None,
        });
    }
    if recv_class == "Landroid/content/res/TypedArray;" {
        return typed_array_get(vm, recv, name, sig, args);
    }
    if recv_class != PARSER || !is_parser(vm, recv) {
        return Ok(None);
    }
    // Pull-parser state machine.
    let state = {
        let st = &vm.fw.xml.parsers[&recv];
        let ev = usize::try_from(st.pos)
            .ok()
            .and_then(|i| st.events.get(i).cloned());
        (st.pos, st.events.len() as isize, ev, st.file.clone())
    };
    let (pos, len, ev, file) = state;
    let event_type = |pos: isize, ev: &Option<Ev>| -> i32 {
        if pos < 0 {
            0
        } else if pos >= len {
            1
        } else {
            match ev {
                Some(Ev::Start { .. }) => 2,
                Some(Ev::End { .. }) => 3,
                _ => 4,
            }
        }
    };
    let opt_str = |vm: &mut Engine, s: Option<String>| -> Result<Value, VmExit> {
        match s {
            Some(s) => str_val(vm, s),
            None => Ok(Value::Null),
        }
    };
    let v = match name {
        "next" | "nextToken" | "nextTag" => {
            // XmlPullParser: once END_DOCUMENT is reached, next() keeps returning
            // END_DOCUMENT; nextTag() must throw instead.
            if pos >= len {
                if name == "nextTag" {
                    return Err(exc(
                        "Lorg/xmlpull/v1/XmlPullParserException;",
                        "END_DOCUMENT reached",
                    ));
                }
                Value::Int(1)
            } else {
                let mut np = pos + 1;
                if name == "nextTag" {
                    // Only whitespace text may be skipped by nextTag().
                    while np < len
                        && matches!(
                            &vm.fw.xml.parsers[&recv].events.get(np as usize).cloned(),
                            Some(Ev::Text(ref t, _)) if t.trim().is_empty()
                        )
                    {
                        np += 1;
                    }
                }
                if let Some(s) = vm.fw.xml.parsers.get_mut(&recv) {
                    s.pos = np;
                }
                let nev = vm.fw.xml.parsers[&recv].events.get(np as usize).cloned();
                let t = event_type(np, &nev);
                if name == "nextTag" && t != 2 && t != 3 {
                    return Err(exc(
                        "Lorg/xmlpull/v1/XmlPullParserException;",
                        "expected START_TAG or END_TAG",
                    ));
                }
                Value::Int(t)
            }
        }
        "getEventType" => Value::Int(event_type(pos, &ev)),
        "getName" => {
            let n = match &ev {
                Some(Ev::Start { name, .. }) | Some(Ev::End { name, .. }) => Some(name.clone()),
                _ => None,
            };
            opt_str(vm, n)?
        }
        "getText" => {
            let t = match &ev {
                Some(Ev::Text(t, _)) => Some(t.clone()),
                _ => None,
            };
            opt_str(vm, t)?
        }
        "isEmptyElementWhitespace" | "isWhitespace" => Value::Int(0),
        "getDepth" => Value::Int(match &ev {
            Some(Ev::Start { depth, .. })
            | Some(Ev::End { depth, .. })
            | Some(Ev::Text(_, depth)) => *depth,
            _ => 0,
        }),
        "getNamespace" => {
            if sig == "()Ljava/lang/String;" {
                str_val(vm, String::new())?
            } else {
                Value::Null
            }
        }
        "getPositionDescription" => {
            str_val(vm, format!("Binary XML file {file} event #{}", pos + 1))?
        }
        "getLineNumber" => Value::Int((pos + 1) as i32),
        "getColumnNumber" => Value::Int(-1),
        "close" => {
            // Idempotent: keep the parser registered and drop its events, so
            // later close()/getEventType() calls still answer (END_DOCUMENT).

            if let Some(st) = vm.fw.xml.parsers.get_mut(&recv) {
                st.events.clear();
                st.pos = 0;
            };
            Value::Null
        }
        // XmlPullParser.require(type, namespace, name): throws on any mismatch (#370).
        "require" => {
            let want_type = match args.first() {
                Some(Value::Int(i)) => *i,
                _ => -1,
            };
            let want_ns = str_arg(vm, args.get(1));
            let want_name = str_arg(vm, args.get(2));
            let cur_name = match &ev {
                Some(Ev::Start { name, .. }) | Some(Ev::End { name, .. }) => Some(name.clone()),
                _ => None,
            };
            let cur_type = event_type(pos, &ev);
            // The parser has no namespaces: only null or "" matches.
            let ns_ok = match want_ns.as_deref() {
                None => true,
                Some(n) => n.is_empty(),
            };
            let name_ok = want_name.is_none() || want_name == cur_name;
            if cur_type != want_type || !ns_ok || !name_ok {
                return Err(exc(
                    "Lorg/xmlpull/v1/XmlPullParserException;",
                    format!("expected type {want_type} name {want_name:?}"),
                ));
            }
            Value::Null
        }
        // XmlPullParser.nextText(): the element must hold only text; leaves the parser on END_TAG (#370).
        "nextText" => {
            if event_type(pos, &ev) != 2 {
                return Err(exc(
                    "Lorg/xmlpull/v1/XmlPullParserException;",
                    "parser must be on START_TAG to read next text",
                ));
            }
            let st_events = vm.fw.xml.parsers[&recv].events.clone();
            let mut np = (pos + 1) as usize;
            let mut text = String::new();
            if let Some(Ev::Text(t, _)) = st_events.get(np) {
                text = t.clone();
                np += 1;
            }
            match st_events.get(np) {
                Some(Ev::End { .. }) => {
                    if let Some(s) = vm.fw.xml.parsers.get_mut(&recv) {
                        s.pos = np as isize;
                    }
                    str_val(vm, text)?
                }
                _ => {
                    return Err(exc(
                        "Lorg/xmlpull/v1/XmlPullParserException;",
                        "type START_TAG is not text-only; END_TAG expected",
                    ))
                }
            }
        }
        "getAttributeCount" => Value::Int(
            current_attrs(vm, recv)
                .map(|a| a.len() as i32)
                .unwrap_or(-1),
        ),
        "getAttributeName" => {
            let a = find_attr(vm, recv, args, true).map(|a| a.name);
            opt_str(vm, a)?
        }
        "getAttributeNamespace" => {
            let a = find_attr(vm, recv, args, true).map(|a| {
                if a.is_android() {
                    ANDROID_NS.to_string()
                } else {
                    a.ns.unwrap_or_default()
                }
            });
            opt_str(vm, a)?
        }
        "getAttributeNameResource" => Value::Int(
            find_attr(vm, recv, args, true)
                .and_then(|a| a.res_id)
                .unwrap_or(0) as i32,
        ),
        "getAttributeValue" => {
            let indexed = sig == "(I)Ljava/lang/String;";
            let a = find_attr(vm, recv, args, indexed).map(|a| match a.value {
                AttrValue::Reference(id) => format!("@{}", id as i32),
                AttrValue::Attribute(id) => format!("?{}", id as i32),
                _ => a.text(),
            });
            opt_str(vm, a)?
        }
        "getAttributeResourceValue"
        | "getAttributeIntValue"
        | "getAttributeUnsignedIntValue"
        | "getAttributeBooleanValue"
        | "getAttributeFloatValue" => {
            let indexed = sig.starts_with("(I");
            let def = args.last().cloned().unwrap_or(Value::Int(0));
            match find_attr(vm, recv, args, indexed) {
                None => def,
                Some(a) => match name {
                    "getAttributeResourceValue" => match a.value {
                        AttrValue::Reference(id) => Value::Int(id as i32),
                        _ => def,
                    },
                    "getAttributeFloatValue" => {
                        as_f32(vm, &a.value).map(Value::Float).unwrap_or(def)
                    }
                    "getAttributeBooleanValue" => {
                        a.as_bool().map(|b| Value::Int(b as i32)).unwrap_or(def)
                    }
                    _ => a.as_i32().map(Value::Int).unwrap_or(def),
                },
            }
        }
        "getIdAttribute" | "getClassAttribute" => {
            let n = if name == "getIdAttribute" {
                "id"
            } else {
                "class"
            };
            let a = current_attrs(vm, recv)
                .and_then(|v| v.into_iter().find(|a| a.name == n))
                .map(|a| a.text());
            opt_str(vm, a)?
        }
        "getIdAttributeResourceValue" => {
            let d = args.first().cloned().unwrap_or(Value::Int(0));
            match current_attrs(vm, recv)
                .and_then(|v| v.into_iter().find(|a| a.name == "id"))
                .map(|a| a.value)
            {
                Some(AttrValue::Reference(id)) => Value::Int(id as i32),
                _ => d,
            }
        }
        "getStyleAttribute" => Value::Int(0),
        "getFeature" => Value::Int(0),
        "setFeature" | "setInput" | "defineEntityReplacementText" => Value::Null,
        _ => return Ok(None),
    };
    Ok(Some(v))
}

/// `android.util.Xml.asAttributeSet(parser)`: a compiled-resource parser is
/// its own AttributeSet, as `XmlBlock.Parser` is in AOSP.
pub fn static_intrinsic(
    vm: &mut Engine,
    class: &str,
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let _ = vm;
    if class != "Landroid/util/Xml;" || name != "asAttributeSet" {
        return Ok(None);
    }
    Ok(Some(args.first().cloned().unwrap_or(Value::Null)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn color(s: &str) -> Option<i32> {
        let vm = Engine::new(Vec::new(), crate::VmConfig::default());
        as_i32(&vm, &AttrValue::String(s.to_string()))
    }

    /// #370: `#RGB` / `#ARGB` expand each nibble; `#RRGGBB` / `#AARRGGBB` are taken as written.
    #[test]
    fn hex_color_shorthand_expands_each_nibble() {
        assert_eq!(color("#f00"), Some(0xFFFF_0000u32 as i32));
        assert_eq!(color("#abc"), Some(0xFFAA_BBCCu32 as i32));
        assert_eq!(color("#8f00"), Some(0x88FF_0000u32 as i32));
        assert_eq!(color("#ff0000"), Some(0xFFFF_0000u32 as i32));
        assert_eq!(color("#80ff0000"), Some(0x80FF_0000u32 as i32));
    }
}

#[cfg(test)]
mod color_tests {
    use super::*;

    fn color(text: &str) -> Option<i32> {
        let vm = Engine::new(Vec::new(), crate::VmConfig::default());
        as_i32(&vm, &AttrValue::String(text.to_string()))
    }

    #[test]
    fn issue_370_shorthand_colors_expand_each_nibble() {
        assert_eq!(color("#fff"), Some(0xFFFF_FFFFu32 as i32));
        assert_eq!(color("#f00"), Some(0xFFFF_0000u32 as i32));
        assert_eq!(color("#f08c"), Some(0xFF00_88CCu32 as i32));
        assert_eq!(color("#ff0000"), Some(0xFFFF_0000u32 as i32));
        assert_eq!(color("#80ff0000"), Some(0x80FF_0000u32 as i32));
    }

    #[test]
    fn issue_370_invalid_color_lengths_and_signs_are_rejected() {
        for bad in ["#1", "#12", "#12345", "#1234567", "#+12", "#+12345", "#zzz"] {
            assert_eq!(color(bad), None, "{bad} must not parse as a color");
        }
    }
}

#[cfg(test)]
mod flatten_tests {
    use super::*;
    use rd_apk::axml::XmlElement;

    fn el(name: &str, children: Vec<XmlElement>) -> XmlElement {
        XmlElement {
            name: name.to_string(),
            attrs: Vec::new(),
            children,
            text: None,
        }
    }

    #[test]
    fn issue_370_flatten_keeps_document_order_and_depths() {
        let mut root = el(
            "a",
            vec![el("b", vec![el("c", Vec::new())]), el("d", Vec::new())],
        );
        root.text = Some("hi".to_string());
        let mut out = Vec::new();
        flatten(&root, 1, &mut out);
        let names: Vec<String> = out
            .iter()
            .map(|ev| match ev {
                Ev::Start { name, depth, .. } => format!("<{name}@{depth}"),
                Ev::End { name, depth } => format!("</{name}@{depth}"),
                Ev::Text(t, depth) => format!("#{t}@{depth}"),
            })
            .collect();
        assert_eq!(
            names,
            vec!["<a@1", "#hi@1", "<b@2", "<c@3", "</c@3", "</b@2", "<d@2", "</d@2", "</a@1"]
        );
    }

    #[test]
    fn issue_370_deep_tree_flattens_without_stack_overflow() {
        const DEPTH: usize = 200_000;
        let mut node = el("leaf", Vec::new());
        for _ in 0..DEPTH {
            node = el("n", vec![node]);
        }
        let mut out = Vec::new();
        flatten(&node, 1, &mut out);
        assert_eq!(out.len(), 2 * (DEPTH + 1));
        assert!(matches!(out.last(), Some(Ev::End { depth: 1, .. })));
    }
}
