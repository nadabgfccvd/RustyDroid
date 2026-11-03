//! `java.nio.charset` codecs and `java.nio.CharBuffer`.
//!
//! `jtext_format` models `Charset` itself (`forName`, `defaultCharset`,
//! `isSupported`, `name`, `equals`, …) but a charset is only useful through its
//! *coders*. `kotlinx.serialization`'s `CharsetReader` (used by ktor for every
//! JSON body) builds a `CharsetDecoder`, configures it with
//! `CodingErrorAction.REPLACE`, and then streams an `InputStream` through
//! `decode(ByteBuffer, CharBuffer, boolean)` — a stateful, incremental decoder
//! that must survive multi-byte sequences split across 8 KiB reads.
//!
//! **Decoder** (`CharsetDecoder`): `onMalformedInput`/`onUnmappableCharacter`,
//! `decode(ByteBuffer, CharBuffer, Z)` → `CoderResult`,
//! `decode(ByteBuffer)`, `flush`, `reset`, `charset`, `replacement`,
//! `malformedInputAction`/`unmappableCharacterAction`,
//! `averageCharsPerByte`/`maxCharsPerByte`. Bytes of an incomplete trailing
//! sequence are kept in the decoder (`rd_pending`) until the next call, exactly
//! like the JDK; only `endOfInput = true` turns them into replacement chars (or
//! a `MalformedInputException` under `REPORT`).
//!
//! **Encoder** (`CharsetEncoder`): `encode(CharBuffer, ByteBuffer, Z)`,
//! `encode(CharBuffer)`, `flush`, `reset`, `charset`, `replacement`,
//! `maxBytesPerChar`, with a lone surrogate handled per
//! `onMalformedInput`/`onUnmappableCharacter`.
//!
//! **`CoderResult`**: the four singletons' semantics (`isUnderflow`,
//! `isOverflow`, `isError`, `isMalformed`, `isUnmappable`, `length`,
//! `throwException`, `toString`) plus `malformedForLength`/
//! `unmappableForLength` and the `UNDERFLOW`/`OVERFLOW` statics.
//!
//! **`CodingErrorAction`**: the `REPORT`/`REPLACE`/`IGNORE` statics and their
//! `toString`.
//!
//! **`CharBuffer`**: `wrap([C)`, `wrap([CII)`, `wrap(CharSequence)`, the
//! `Buffer` position/limit/capacity state machine (`flip`, `rewind`, `clear`,
//! `mark`/`reset`, `slice`, `duplicate`), `get`/`put` (single, array, indexed,
//! `String`), `array`/`arrayOffset`, `length`/`charAt`/`subSequence`,
//! `toString`/`equals`/`hashCode`, `chars`. A buffer created by `wrap(array,
//! offset, length)` is a *window* into that array: `arrayOffset()` is the
//! window base and `position()` starts at zero, so `slice()` used by
//! `CharsetReader.doRead` addresses the caller's `char[]` directly.
//!
//! Charsets implemented: UTF-8, UTF-16 (with BOM), UTF-16BE, UTF-16LE,
//! ISO-8859-1, US-ASCII — the same set `Charset.forName` accepts.

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::heap::{ElemKind, ObjRef};
use crate::value::Value;

const CHARSET: &str = "Ljava/nio/charset/Charset;";
const DECODER: &str = "Ljava/nio/charset/CharsetDecoder;";
const ENCODER: &str = "Ljava/nio/charset/CharsetEncoder;";
const CODER_RESULT: &str = "Ljava/nio/charset/CoderResult;";
const CODING_ERROR_ACTION: &str = "Ljava/nio/charset/CodingErrorAction;";
const CHAR_BUFFER: &str = "Ljava/nio/CharBuffer;";
const BYTE_BUFFER: &str = "Ljava/nio/ByteBuffer;";

const UNDERFLOW: i32 = 0;
const OVERFLOW: i32 = 1;
const MALFORMED: i32 = 2;
const UNMAPPABLE: i32 = 3;

const ACTION_REPORT: i32 = 0;
const ACTION_REPLACE: i32 = 1;
const ACTION_IGNORE: i32 = 2;

fn err(class: &str, msg: impl Into<String>) -> VmExit {
    VmExit::Exception(Throwable::new(class, msg))
}
fn illegal(msg: impl Into<String>) -> VmExit {
    err("Ljava/lang/IllegalArgumentException;", msg)
}
fn npe(msg: impl Into<String>) -> VmExit {
    err("Ljava/lang/NullPointerException;", msg)
}
fn unsupported(msg: impl Into<String>) -> VmExit {
    err("Ljava/lang/UnsupportedOperationException;", msg)
}

fn int_arg(args: &[Value], i: usize, default: i32) -> i32 {
    args.get(i).and_then(|v| v.as_int().ok()).unwrap_or(default)
}
fn obj_arg(args: &[Value], i: usize) -> Option<ObjRef> {
    match args.get(i) {
        Some(Value::Obj(r)) => Some(*r),
        _ => None,
    }
}
fn jstr(vm: &mut Engine, s: impl Into<String>) -> Result<Value, VmExit> {
    Ok(Value::Obj(crate::intrinsics::alloc_string(vm, s.into())?))
}
fn alloc(vm: &mut Engine, class: &str, fields: Vec<(String, Value)>) -> Result<ObjRef, VmExit> {
    vm.heap
        .alloc_instance(class.to_string(), fields)
        .map_err(crate::err::oom_throwable)
}
fn get_int(vm: &Engine, r: ObjRef, f: &str, default: i32) -> i32 {
    vm.heap
        .get_field(r, f)
        .ok()
        .and_then(|v| v.as_int().ok())
        .unwrap_or(default)
}
fn put(vm: &mut Engine, r: ObjRef, f: &str, v: Value) -> Result<(), VmExit> {
    crate::err::put_field_result(vm.heap.put_field(r, f, v))
}
fn field_obj(vm: &Engine, r: ObjRef, f: &str) -> Option<ObjRef> {
    match vm.heap.get_field(r, f) {
        Ok(Value::Obj(o)) => Some(o),
        _ => None,
    }
}
fn field_str(vm: &Engine, r: ObjRef, f: &str) -> Option<String> {
    let o = field_obj(vm, r, f)?;
    vm.heap.as_str(o).ok().map(str::to_string)
}

/// UTF-16 code units of a host array plus its window state.
#[derive(Clone, Copy)]
struct Buf {
    arr: Option<ObjRef>,
    base: i32,
    pos: i32,
    limit: i32,
    cap: i32,
}

impl Buf {
    fn remaining(self) -> i32 {
        (self.limit - self.pos).max(0)
    }
    fn at(self, idx: i32) -> i32 {
        self.base.saturating_add(idx)
    }
}

fn byte_buf(vm: &Engine, r: ObjRef) -> Buf {
    Buf {
        arr: field_obj(vm, r, "rd_buf"),
        base: get_int(vm, r, "rd_base", 0),
        pos: get_int(vm, r, "rd_pos", 0),
        limit: get_int(vm, r, "rd_limit", 0),
        cap: get_int(vm, r, "rd_cap", 0),
    }
}
fn char_buf(vm: &Engine, r: ObjRef) -> Buf {
    Buf {
        arr: field_obj(vm, r, "rd_arr"),
        base: get_int(vm, r, "rd_base", 0),
        pos: get_int(vm, r, "rd_pos", 0),
        limit: get_int(vm, r, "rd_limit", 0),
        cap: get_int(vm, r, "rd_cap", 0),
    }
}

/// Raw bytes behind a `ByteBuffer` (its `[B`), as `u8`.
fn byte_array_all(vm: &Engine, r: Option<ObjRef>) -> Result<Vec<u8>, VmExit> {
    let Some(r) = r else { return Ok(Vec::new()) };
    let (elems, _) = vm
        .heap
        .as_array_elems(r)
        .map_err(|e| VmExit::from(crate::err::vm_error("VM_TYPE_ERROR", e)))?;
    Ok(elems
        .iter()
        .map(|v| v.as_int().unwrap_or(0) as u8)
        .collect())
}

fn char_array_all(vm: &Engine, r: ObjRef) -> Result<Vec<u16>, VmExit> {
    let (elems, _) = vm
        .heap
        .as_array_elems(r)
        .map_err(|e| VmExit::from(crate::err::vm_error("VM_TYPE_ERROR", e)))?;
    Ok(elems
        .iter()
        .map(|v| v.as_int().unwrap_or(0) as u16)
        .collect())
}

fn new_char_array(vm: &mut Engine, units: &[u16]) -> Result<ObjRef, VmExit> {
    let r = vm
        .heap
        .alloc_array_zeroed(ElemKind::Int, units.len(), "[C")
        .map_err(crate::err::oom_throwable)?;
    let elems = vm.heap.array_elems_mut(r)?;
    for (slot, u) in elems.iter_mut().zip(units) {
        *slot = Value::Int(i32::from(*u));
    }
    Ok(r)
}

fn decoder_actions(vm: &Engine, r: ObjRef) -> (i32, i32, String) {
    (
        get_int(vm, r, "rd_malformed", ACTION_REPORT),
        get_int(vm, r, "rd_unmappable", ACTION_REPORT),
        field_str(vm, r, "rd_name").unwrap_or_else(|| "UTF-8".into()),
    )
}

// ---------------------------------------------------------------------------
// CoderResult / CodingErrorAction
// ---------------------------------------------------------------------------

fn coder_result(vm: &mut Engine, kind: i32, len: i32) -> Result<Value, VmExit> {
    let r = alloc(
        vm,
        CODER_RESULT,
        vec![
            ("rd_kind".to_string(), Value::Int(kind)),
            ("rd_len".to_string(), Value::Int(len)),
        ],
    )?;
    Ok(Value::Obj(r))
}

fn coding_error_action(vm: &mut Engine, action: i32) -> Result<Value, VmExit> {
    let r = alloc(
        vm,
        CODING_ERROR_ACTION,
        vec![("rd_action".to_string(), Value::Int(action))],
    )?;
    Ok(Value::Obj(r))
}

/// Constants that the host provides for `sget-object` of a platform class.
pub(crate) fn host_static(
    vm: &mut Engine,
    owner: &str,
    field: &str,
) -> Result<Option<Value>, VmExit> {
    match owner {
        CODING_ERROR_ACTION => Ok(Some(coding_error_action(
            vm,
            match field {
                "REPORT" => ACTION_REPORT,
                "REPLACE" => ACTION_REPLACE,
                "IGNORE" => ACTION_IGNORE,
                _ => return Ok(None),
            },
        )?)),
        CODER_RESULT => Ok(Some(match field {
            "UNDERFLOW" => coder_result(vm, UNDERFLOW, 0)?,
            "OVERFLOW" => coder_result(vm, OVERFLOW, 0)?,
            _ => return Ok(None),
        })),
        _ => Ok(None),
    }
}

fn coder_result_instance(
    vm: &mut Engine,
    name: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let kind = get_int(vm, recv, "rd_kind", UNDERFLOW);
    let len = get_int(vm, recv, "rd_len", 0);
    Ok(Some(match name {
        "isUnderflow" => Value::Int(i32::from(kind == UNDERFLOW)),
        "isOverflow" => Value::Int(i32::from(kind == OVERFLOW)),
        "isError" => Value::Int(i32::from(kind == MALFORMED || kind == UNMAPPABLE)),
        "isMalformed" => Value::Int(i32::from(kind == MALFORMED)),
        "isUnmappable" => Value::Int(i32::from(kind == UNMAPPABLE)),
        "length" => {
            if kind == UNDERFLOW || kind == OVERFLOW {
                return Err(unsupported("CoderResult.length on underflow/overflow"));
            }
            Value::Int(len)
        }
        "throwException" => {
            match kind {
                MALFORMED => {
                    return Err(err(
                        "Ljava/nio/charset/MalformedInputException;",
                        format!("Input length = {len}"),
                    ))
                }
                UNMAPPABLE => {
                    return Err(err(
                        "Ljava/nio/charset/UnmappableCharacterException;",
                        format!("Input length = {len}"),
                    ))
                }
                // Verified Java behaviour: underflow/overflow are not throwable.
                _ => Value::Null,
            }
        }
        "toString" => {
            let s = match kind {
                UNDERFLOW => "UNDERFLOW".to_string(),
                OVERFLOW => "OVERFLOW".to_string(),
                MALFORMED => format!("MALFORMED[{len}]"),
                _ => format!("UNMAPPABLE[{len}]"),
            };
            jstr(vm, s)?
        }
        "hashCode" => Value::Int(kind * 31 + len),
        "equals" => Value::Int(i32::from(matches!(args.first(), Some(Value::Obj(o))
                if get_int(vm, *o, "rd_kind", UNDERFLOW) == kind
                    && get_int(vm, *o, "rd_len", 0) == len))),
        _ => return Ok(None),
    }))
}

// ---------------------------------------------------------------------------
// Incremental decoders
// ---------------------------------------------------------------------------

/// One decoded code point plus the input offset just past it.
#[derive(Clone, Copy, Debug)]
struct Word {
    hi: u16,
    lo: Option<u16>,
    end: usize,
}

impl Word {
    fn units(self) -> usize {
        1 + usize::from(self.lo.is_some())
    }
}

#[derive(Debug, PartialEq)]
enum Tail {
    /// Every byte was decoded.
    Clean,
    /// The input ends in the middle of a valid sequence.
    Incomplete,
    /// An invalid sequence of `n` bytes starts at the reported offset.
    Malformed(usize),
}

/// Charset family, resolved from `Charset.name()`.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Family {
    Utf8,
    Utf16Be,
    Utf16Le,
    Utf16, // BOM decides
    Latin1,
    Ascii,
}

fn family(name: &str) -> Family {
    match name.to_ascii_uppercase().replace('_', "-").as_str() {
        "UTF-16BE" => Family::Utf16Be,
        "UTF-16LE" => Family::Utf16Le,
        "UTF-16" => Family::Utf16,
        "ISO-8859-1" => Family::Latin1,
        "US-ASCII" => Family::Ascii,
        _ => Family::Utf8,
    }
}

struct Decoded {
    words: Vec<Word>,
    consumed: usize,
    tail: Tail,
}

/// Decode `bytes` (an input window) into UTF-16 code units.
///
/// `replace` selects the `CodingErrorAction.REPLACE` behaviour for malformed
/// input; when false the caller turns [`Tail::Malformed`] into a
/// `CoderResult` and the offending bytes stay unconsumed.
fn decode_words(
    fam: Family,
    bytes: &[u8],
    flush: bool,
    replace: bool,
    big_endian: bool,
) -> Decoded {
    let mut words = Vec::new();
    let mut i = 0usize;
    let mut tail = Tail::Clean;
    match fam {
        Family::Latin1 => {
            for b in bytes {
                words.push(Word {
                    hi: u16::from(*b),
                    lo: None,
                    end: i + 1,
                });
                i += 1;
            }
        }
        Family::Ascii => {
            while i < bytes.len() {
                let b = bytes[i];
                if b < 0x80 {
                    words.push(Word {
                        hi: u16::from(b),
                        lo: None,
                        end: i + 1,
                    });
                    i += 1;
                } else if replace {
                    words.push(Word {
                        hi: 0xFFFD,
                        lo: None,
                        end: i + 1,
                    });
                    i += 1;
                } else {
                    tail = Tail::Malformed(1);
                    break;
                }
            }
        }
        Family::Utf8 => {
            while i < bytes.len() {
                let b = bytes[i];
                let need = if b < 0x80 {
                    1
                } else if b & 0xE0 == 0xC0 {
                    2
                } else if b & 0xF0 == 0xE0 {
                    3
                } else if b & 0xF8 == 0xF0 {
                    4
                } else {
                    // Continuation byte or 0xF8.. without a lead.
                    if replace {
                        words.push(Word {
                            hi: 0xFFFD,
                            lo: None,
                            end: i + 1,
                        });
                        i += 1;
                        continue;
                    }
                    tail = Tail::Malformed(1);
                    break;
                };
                if need == 1 {
                    words.push(Word {
                        hi: u16::from(b),
                        lo: None,
                        end: i + 1,
                    });
                    i += 1;
                    continue;
                }
                if i + need > bytes.len() {
                    if !flush {
                        tail = Tail::Incomplete;
                        break;
                    }
                    if replace {
                        words.push(Word {
                            hi: 0xFFFD,
                            lo: None,
                            end: bytes.len(),
                        });
                        i = bytes.len();
                        continue;
                    }
                    tail = Tail::Malformed(bytes.len() - i);
                    break;
                }
                let ok = bytes[i + 1..i + need].iter().all(|c| c & 0xC0 == 0x80);
                let mut cp = if need == 2 {
                    u32::from(b & 0x1F)
                } else if need == 3 {
                    u32::from(b & 0x0F)
                } else {
                    u32::from(b & 0x07)
                };
                for c in &bytes[i + 1..i + need] {
                    cp = (cp << 6) | u32::from(c & 0x3F);
                }
                let overlong =
                    matches!((need, cp), (2, 0..=0x7F) | (3, 0..=0x7FF) | (4, 0..=0xFFFF));
                let valid = ok && !overlong && !(0xD800..=0xDFFF).contains(&cp) && cp <= 0x10FFFF;
                if !valid {
                    if replace {
                        words.push(Word {
                            hi: 0xFFFD,
                            lo: None,
                            end: i + need,
                        });
                        i += need;
                        continue;
                    }
                    tail = Tail::Malformed(if ok { need } else { 1 });
                    break;
                }
                let end = i + need;
                i = end;
                if cp >= 0x10000 {
                    let v = cp - 0x10000;
                    words.push(Word {
                        hi: 0xD800 + ((v >> 10) as u16),
                        lo: Some(0xDC00 + ((v & 0x3FF) as u16)),
                        end,
                    });
                } else {
                    words.push(Word {
                        hi: cp as u16,
                        lo: None,
                        end,
                    });
                }
            }
        }
        Family::Utf16Be | Family::Utf16Le | Family::Utf16 => {
            let le = match fam {
                Family::Utf16Be => false,
                Family::Utf16Le => true,
                _ => !big_endian,
            };
            while i + 2 <= bytes.len() {
                let unit = if le {
                    u16::from_le_bytes([bytes[i], bytes[i + 1]])
                } else {
                    u16::from_be_bytes([bytes[i], bytes[i + 1]])
                };
                if (0xD800..=0xDBFF).contains(&unit) {
                    if i + 4 > bytes.len() {
                        if !flush {
                            tail = Tail::Incomplete;
                            break;
                        }
                        if replace {
                            words.push(Word {
                                hi: 0xFFFD,
                                lo: None,
                                end: bytes.len(),
                            });
                            i = bytes.len();
                            continue;
                        }
                        tail = Tail::Malformed(bytes.len() - i);
                        break;
                    }
                    let low = if le {
                        u16::from_le_bytes([bytes[i + 2], bytes[i + 3]])
                    } else {
                        u16::from_be_bytes([bytes[i + 2], bytes[i + 3]])
                    };
                    if !(0xDC00..=0xDFFF).contains(&low) {
                        if replace {
                            words.push(Word {
                                hi: 0xFFFD,
                                lo: None,
                                end: i + 2,
                            });
                            i += 2;
                            continue;
                        }
                        tail = Tail::Malformed(2);
                        break;
                    }
                    words.push(Word {
                        hi: unit,
                        lo: Some(low),
                        end: i + 4,
                    });
                    i += 4;
                } else if (0xDC00..=0xDFFF).contains(&unit) {
                    if replace {
                        words.push(Word {
                            hi: 0xFFFD,
                            lo: None,
                            end: i + 2,
                        });
                        i += 2;
                        continue;
                    }
                    tail = Tail::Malformed(2);
                    break;
                } else {
                    words.push(Word {
                        hi: unit,
                        lo: None,
                        end: i + 2,
                    });
                    i += 2;
                }
            }
            if tail == Tail::Clean && i < bytes.len() {
                if flush {
                    if replace {
                        words.push(Word {
                            hi: 0xFFFD,
                            lo: None,
                            end: bytes.len(),
                        });
                        i = bytes.len();
                    } else {
                        tail = Tail::Malformed(bytes.len() - i);
                    }
                } else {
                    tail = Tail::Incomplete;
                }
            }
        }
    }
    Decoded {
        words,
        consumed: i,
        tail,
    }
}

/// `CharBuffer` write of `words`, honouring the destination's remaining space.
/// Returns `(units written, bytes consumed, overflow)`.
fn write_words(
    vm: &mut Engine,
    dst: ObjRef,
    words: &[Word],
    mut dst_pos: i32,
    dst_limit: i32,
    base: i32,
) -> Result<(usize, usize, bool), VmExit> {
    let Some(arr) = field_obj(vm, dst, "rd_arr") else {
        return Ok((0, 0, !words.is_empty()));
    };
    let mut written = 0usize;
    let mut consumed = 0usize;
    let mut overflow = false;
    for w in words {
        let need = w.units() as i32;
        if dst_pos + need > dst_limit {
            overflow = true;
            break;
        }
        let hi_at = (base + dst_pos) as usize;
        let elems = vm.heap.array_elems_mut(arr)?;
        if let Some(slot) = elems.get_mut(hi_at) {
            *slot = Value::Int(i32::from(w.hi));
        }
        if let Some(lo) = w.lo {
            if let Some(slot) = elems.get_mut(hi_at + 1) {
                *slot = Value::Int(i32::from(lo));
            }
        }
        dst_pos += need;
        written += w.units();
        consumed = w.end;
    }
    put(vm, dst, "rd_pos", Value::Int(dst_pos))?;
    Ok((written, consumed, overflow))
}

// ---------------------------------------------------------------------------
// CharsetDecoder
// ---------------------------------------------------------------------------

fn new_decoder(vm: &mut Engine, charset: ObjRef) -> Result<Value, VmExit> {
    let name = field_str(vm, charset, "rd_name").unwrap_or_else(|| "UTF-8".into());
    let mut fields = vec![
        ("rd_name".to_string(), Value::Null),
        ("rd_malformed".to_string(), Value::Int(ACTION_REPORT)),
        ("rd_unmappable".to_string(), Value::Int(ACTION_REPORT)),
        ("rd_pending".to_string(), Value::Null),
        ("rd_bom_done".to_string(), Value::Int(0)),
        ("rd_be".to_string(), Value::Int(1)),
    ];
    fields[0].1 = jstr(vm, name)?;
    let r = alloc(vm, DECODER, fields)?;
    Ok(Value::Obj(r))
}

fn new_encoder(vm: &mut Engine, charset: ObjRef) -> Result<Value, VmExit> {
    let name = field_str(vm, charset, "rd_name").unwrap_or_else(|| "UTF-8".into());
    let mut fields = vec![
        ("rd_name".to_string(), Value::Null),
        ("rd_malformed".to_string(), Value::Int(ACTION_REPORT)),
        ("rd_unmappable".to_string(), Value::Int(ACTION_REPORT)),
        ("rd_high".to_string(), Value::Int(-1)),
    ];
    fields[0].1 = jstr(vm, name)?;
    let r = alloc(vm, ENCODER, fields)?;
    Ok(Value::Obj(r))
}

fn decoder_charset(vm: &mut Engine, recv: ObjRef) -> Result<Value, VmExit> {
    let name = field_str(vm, recv, "rd_name").unwrap_or_else(|| "UTF-8".into());
    let n = jstr(vm, name)?;
    let r = alloc(vm, CHARSET, vec![("rd_name".to_string(), n)])?;
    Ok(Value::Obj(r))
}

fn decoder_pending(vm: &Engine, recv: ObjRef) -> Result<Vec<u8>, VmExit> {
    byte_array_all(vm, field_obj(vm, recv, "rd_pending"))
}

fn set_decoder_pending(vm: &mut Engine, recv: ObjRef, bytes: &[u8]) -> Result<(), VmExit> {
    if bytes.is_empty() {
        return put(vm, recv, "rd_pending", Value::Null);
    }
    let arr = vm
        .heap
        .alloc_array_zeroed(ElemKind::Int, bytes.len(), "[B")
        .map_err(crate::err::oom_throwable)?;
    {
        let elems = vm.heap.array_elems_mut(arr)?;
        for (slot, b) in elems.iter_mut().zip(bytes) {
            *slot = Value::Int(i32::from(*b));
        }
    }
    put(vm, recv, "rd_pending", Value::Obj(arr))
}

/// `CharsetDecoder.decode(ByteBuffer, CharBuffer, boolean)`.
///
/// The decoder keeps a state that survives calls: bytes of an incomplete
/// trailing sequence move into `rd_pending` and are *consumed from the source*
/// (Java's `ByteBuffer` position advances over them), so the next call
/// continues where this one stopped instead of re-reading them.
fn decode_stream(vm: &mut Engine, recv: ObjRef, args: &[Value]) -> Result<Value, VmExit> {
    let Some(src) = obj_arg(args, 0) else {
        return Err(npe("CharsetDecoder.decode null source"));
    };
    let Some(dst) = obj_arg(args, 1) else {
        return Err(npe("CharsetDecoder.decode null target"));
    };
    let end_of_input = int_arg(args, 2, 0) != 0;
    let (action_mal, _action_unm, name) = decoder_actions(vm, recv);

    let sb = byte_buf(vm, src);
    let db = char_buf(vm, dst);
    let all = byte_array_all(vm, sb.arr)?;
    let pos = sb.pos.clamp(0, all.len() as i32) as usize;
    let limit = sb.limit.clamp(pos as i32, all.len() as i32) as usize;
    let window = &all[pos..limit];

    let pending = decoder_pending(vm, recv)?;
    let pending_len = pending.len();
    let mut input = pending;
    input.extend_from_slice(window);

    let fam = family(&name);
    // The BOM decides endianness once per stream for plain `UTF-16`.
    let mut bom_skipped = 0usize;
    if fam == Family::Utf16 && get_int(vm, recv, "rd_bom_done", 0) == 0 && input.len() >= 2 {
        if input[..2] == [0xFE, 0xFF] || input[..2] == [0xFF, 0xFE] {
            let be = input[..2] == [0xFE, 0xFF];
            put(vm, recv, "rd_be", Value::Int(i32::from(be)))?;
            bom_skipped = 2;
        }
        put(vm, recv, "rd_bom_done", Value::Int(1))?;
    }
    if bom_skipped > 0 {
        input.drain(..2);
    }
    // BOM bytes only ever arrive in the source window, never in `pending`.
    let pending_len = pending_len.saturating_sub(bom_skipped);
    let big_endian = get_int(vm, recv, "rd_be", 1) != 0;

    let decoded = decode_words(
        fam,
        &input,
        end_of_input,
        action_mal == ACTION_REPLACE,
        big_endian,
    );

    let (_written, written_bytes, overflow) =
        write_words(vm, dst, &decoded.words, db.pos, db.limit, db.base)?;

    let mut malformed_len = None;
    let (taken, new_pending) = match decoded.tail {
        _ if overflow => (written_bytes, Vec::new()),
        Tail::Clean => (input.len(), Vec::new()),
        Tail::Incomplete => {
            let from = decoded.consumed.min(input.len());
            (input.len(), input[from..].to_vec())
        }
        Tail::Malformed(n) => {
            if action_mal == ACTION_REPORT {
                malformed_len = Some(n.max(1) as i32);
                (decoded.consumed, Vec::new())
            } else {
                (input.len(), Vec::new())
            }
        }
    };
    set_decoder_pending(vm, recv, &new_pending)?;
    let src_advance = taken.saturating_sub(pending_len).min(window.len());
    put(vm, src, "rd_pos", Value::Int((pos + src_advance) as i32))?;

    if let Some(n) = malformed_len {
        return coder_result(vm, MALFORMED, n);
    }
    Ok(if overflow {
        coder_result(vm, OVERFLOW, 0)?
    } else {
        coder_result(vm, UNDERFLOW, 0)?
    })
}

/// `CharsetDecoder.decode(ByteBuffer)`: the whole buffer in one shot.
fn decode_oneshot(vm: &mut Engine, recv: ObjRef, args: &[Value]) -> Result<Value, VmExit> {
    let Some(src) = obj_arg(args, 0) else {
        return Err(npe("CharsetDecoder.decode null source"));
    };
    let sb = byte_buf(vm, src);
    let all = byte_array_all(vm, sb.arr)?;
    let pos = sb.pos.clamp(0, all.len() as i32) as usize;
    let limit = sb.limit.clamp(pos as i32, all.len() as i32) as usize;
    let pending = decoder_pending(vm, recv)?;
    let mut input = pending;
    input.extend_from_slice(&all[pos..limit]);
    let (_mal, _unm, name) = decoder_actions(vm, recv);
    let decoded = decode_words(family(&name), &input, true, false, true);
    if let Tail::Malformed(n) = decoded.tail {
        return coder_result(vm, MALFORMED, n.max(1) as i32);
    }
    let mut units = Vec::with_capacity(decoded.words.len());
    for w in &decoded.words {
        units.push(w.hi);
        if let Some(lo) = w.lo {
            units.push(lo);
        }
    }
    let arr = new_char_array(vm, &units)?;
    let cap = units.len() as i32;
    let buf = alloc(
        vm,
        CHAR_BUFFER,
        vec![
            ("rd_arr".to_string(), Value::Obj(arr)),
            ("rd_pos".to_string(), Value::Int(0)),
            ("rd_limit".to_string(), Value::Int(cap)),
            ("rd_cap".to_string(), Value::Int(cap)),
            ("rd_base".to_string(), Value::Int(0)),
        ],
    )?;
    put(vm, src, "rd_pos", Value::Int(limit as i32))?;
    set_decoder_pending(vm, recv, &[])?;
    Ok(Value::Obj(buf))
}

// ---------------------------------------------------------------------------
// CharsetEncoder
// ---------------------------------------------------------------------------

fn encode_units(name: &str, units: &[u16], replace: bool) -> (Vec<u8>, Option<(i32, usize)>) {
    let fam = family(name);
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut bad: Option<(i32, usize)> = None;
    while i < units.len() {
        let u = units[i];
        match fam {
            Family::Latin1 => {
                if u < 0x100 {
                    out.push(u as u8);
                } else if replace {
                    out.push(b'?');
                } else {
                    bad = Some((UNMAPPABLE, 1));
                    break;
                }
                i += 1;
            }
            Family::Ascii => {
                if u < 0x80 {
                    out.push(u as u8);
                } else if replace {
                    out.push(b'?');
                } else {
                    bad = Some((UNMAPPABLE, 1));
                    break;
                }
                i += 1;
            }
            Family::Utf8 => {
                let cp = if (0xD800..=0xDBFF).contains(&u) {
                    match units.get(i + 1) {
                        Some(lo) if (0xDC00..=0xDFFF).contains(lo) => {
                            0x10000 + ((u32::from(u) - 0xD800) << 10) + (u32::from(*lo) - 0xDC00)
                        }
                        _ => {
                            if replace {
                                out.extend_from_slice("\u{FFFD}".as_bytes());
                                i += 1;
                                continue;
                            }
                            bad = Some((MALFORMED, 1));
                            break;
                        }
                    }
                } else if (0xDC00..=0xDFFF).contains(&u) {
                    if replace {
                        out.extend_from_slice("\u{FFFD}".as_bytes());
                        i += 1;
                        continue;
                    }
                    bad = Some((MALFORMED, 1));
                    break;
                } else {
                    u32::from(u)
                };
                let adv = if cp >= 0x10000 { 2 } else { 1 };
                let ch = char::from_u32(cp).unwrap_or('\u{FFFD}');
                let mut buf = [0u8; 4];
                out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                i += adv;
            }
            Family::Utf16Be | Family::Utf16Le | Family::Utf16 => {
                let le = fam == Family::Utf16Le;
                if (0xD800..=0xDBFF).contains(&u) {
                    match units.get(i + 1) {
                        Some(lo) if (0xDC00..=0xDFFF).contains(lo) => {
                            for unit in [u, *lo] {
                                out.extend_from_slice(&if le {
                                    unit.to_le_bytes()
                                } else {
                                    unit.to_be_bytes()
                                });
                            }
                            i += 2;
                            continue;
                        }
                        _ => {
                            if replace {
                                out.extend_from_slice(&if le {
                                    0xFFFDu16.to_le_bytes()
                                } else {
                                    0xFFFDu16.to_be_bytes()
                                });
                                i += 1;
                                continue;
                            }
                            bad = Some((MALFORMED, 1));
                            break;
                        }
                    }
                }
                if (0xDC00..=0xDFFF).contains(&u) {
                    if replace {
                        out.extend_from_slice(&if le {
                            0xFFFDu16.to_le_bytes()
                        } else {
                            0xFFFDu16.to_be_bytes()
                        });
                        i += 1;
                        continue;
                    }
                    bad = Some((MALFORMED, 1));
                    break;
                }
                out.extend_from_slice(&if le { u.to_le_bytes() } else { u.to_be_bytes() });
                i += 1;
            }
        }
    }
    (out, bad)
}

fn encode_stream(vm: &mut Engine, recv: ObjRef, args: &[Value]) -> Result<Value, VmExit> {
    let Some(src) = obj_arg(args, 0) else {
        return Err(npe("CharsetEncoder.encode null source"));
    };
    let Some(dst) = obj_arg(args, 1) else {
        return Err(npe("CharsetEncoder.encode null target"));
    };
    let (action_mal, action_unm, name) = decoder_actions(vm, recv);
    let sb = char_buf(vm, src);
    let units_all = char_array_all(vm, sb.arr.ok_or_else(|| npe("encode source has no array"))?)?;
    let pos = sb.pos.clamp(0, units_all.len() as i32) as usize;
    let limit = sb.limit.clamp(pos as i32, units_all.len() as i32) as usize;
    let replace = action_mal == ACTION_REPLACE || action_unm == ACTION_REPLACE;
    let (bytes, bad) = encode_units(&name, &units_all[pos..limit], replace);
    let db = byte_buf(vm, dst);
    let capacity = db.remaining().max(0) as usize;
    let n = bytes.len().min(capacity);
    if let Some(arr) = db.arr {
        let elems = vm.heap.array_elems_mut(arr)?;
        for (k, b) in bytes[..n].iter().enumerate() {
            let at = db.at(db.pos) as usize + k;
            if let Some(slot) = elems.get_mut(at) {
                *slot = Value::Int(i32::from(*b));
            }
        }
    }
    put(vm, dst, "rd_pos", Value::Int(db.pos + n as i32))?;
    // Chars consumed: everything up to the stopping point.
    let consumed_units = match bad {
        Some(_) => (limit - pos).saturating_sub(1),
        None if n < bytes.len() => 0,
        None => limit - pos,
    };
    put(vm, src, "rd_pos", Value::Int((pos + consumed_units) as i32))?;
    if let Some((kind, len)) = bad {
        if !replace {
            return coder_result(vm, kind, len as i32);
        }
    }
    coder_result(vm, if n < bytes.len() { OVERFLOW } else { UNDERFLOW }, 0)
}

fn encode_oneshot(vm: &mut Engine, recv: ObjRef, args: &[Value]) -> Result<Value, VmExit> {
    let Some(src) = obj_arg(args, 0) else {
        return Err(npe("CharsetEncoder.encode null source"));
    };
    let sb = char_buf(vm, src);
    let arr = sb.arr.ok_or_else(|| npe("encode source has no array"))?;
    let units = char_array_all(vm, arr)?;
    let pos = sb.pos.clamp(0, units.len() as i32) as usize;
    let limit = sb.limit.clamp(pos as i32, units.len() as i32) as usize;
    let (_mal, _unm, name) = decoder_actions(vm, recv);
    let (bytes, _bad) = encode_units(&name, &units[pos..limit], true);
    let out = vm
        .heap
        .alloc_array_zeroed(ElemKind::Int, bytes.len(), "[B")
        .map_err(crate::err::oom_throwable)?;
    {
        let elems = vm.heap.array_elems_mut(out)?;
        for (slot, b) in elems.iter_mut().zip(&bytes) {
            *slot = Value::Int(i32::from(*b));
        }
    }
    let cap = bytes.len() as i32;
    let bb = alloc(
        vm,
        BYTE_BUFFER,
        vec![
            ("rd_buf".to_string(), Value::Obj(out)),
            ("rd_pos".to_string(), Value::Int(0)),
            ("rd_limit".to_string(), Value::Int(cap)),
            ("rd_cap".to_string(), Value::Int(cap)),
        ],
    )?;
    put(vm, src, "rd_pos", Value::Int(limit as i32))?;
    Ok(Value::Obj(bb))
}

fn encoder_instance(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let cs_name = field_str(vm, recv, "rd_name").unwrap_or_else(|| "UTF-8".into());
    Ok(Some(match name {
        "<init>" => Value::Null,
        "encode" => {
            if sig.starts_with("(Ljava/nio/CharBuffer;)") {
                encode_oneshot(vm, recv, args)?
            } else {
                encode_stream(vm, recv, args)?
            }
        }
        "flush" => coder_result(vm, UNDERFLOW, 0)?,
        "reset" => Value::Obj(recv),
        "charset" => decoder_charset(vm, recv)?,
        "replacement" => {
            let bytes: &[u8] = match family(&cs_name) {
                Family::Utf16Be | Family::Utf16 | Family::Utf16Le => &[0x00, 0x3F],
                _ => b"?",
            };
            let arr = vm
                .heap
                .alloc_array_zeroed(ElemKind::Int, bytes.len(), "[B")
                .map_err(crate::err::oom_throwable)?;
            {
                let elems = vm.heap.array_elems_mut(arr)?;
                for (slot, b) in elems.iter_mut().zip(bytes) {
                    *slot = Value::Int(i32::from(*b));
                }
            }
            let cap = bytes.len() as i32;
            Value::Obj(alloc(
                vm,
                BYTE_BUFFER,
                vec![
                    ("rd_buf".to_string(), Value::Obj(arr)),
                    ("rd_pos".to_string(), Value::Int(0)),
                    ("rd_limit".to_string(), Value::Int(cap)),
                    ("rd_cap".to_string(), Value::Int(cap)),
                ],
            )?)
        }
        "maxBytesPerChar" | "averageBytesPerChar" => match family(&cs_name) {
            Family::Utf16 | Family::Utf16Be | Family::Utf16Le => Value::Float(4.0),
            Family::Utf8 => Value::Float(4.0),
            _ => Value::Float(1.0),
        },
        "canEncode" => Value::Int(1),
        "isLegalReplacement" => Value::Int(1),
        "toString" => jstr(vm, "CharsetEncoder")?,
        _ => return Ok(None),
    }))
}

// ---------------------------------------------------------------------------
// CharBuffer
// ---------------------------------------------------------------------------

fn char_buffer_fields(
    vm: &mut Engine,
    arr: ObjRef,
    base: i32,
    pos: i32,
    limit: i32,
    cap: i32,
) -> Result<Value, VmExit> {
    let r = alloc(
        vm,
        CHAR_BUFFER,
        vec![
            ("rd_arr".to_string(), Value::Obj(arr)),
            ("rd_base".to_string(), Value::Int(base)),
            ("rd_pos".to_string(), Value::Int(pos)),
            ("rd_limit".to_string(), Value::Int(limit)),
            ("rd_cap".to_string(), Value::Int(cap)),
        ],
    )?;
    Ok(Value::Obj(r))
}

fn char_buffer_static(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if name != "wrap" {
        return Ok(None);
    }
    let Some(arr) = obj_arg(args, 0) else {
        return Err(npe("CharBuffer.wrap null array"));
    };
    // `wrap(CharSequence)` — a String-backed buffer.
    if let Ok(s) = vm.heap.as_str(arr) {
        let units: Vec<u16> = s.encode_utf16().collect();
        let cs: String = s.to_string();
        // A String is immutable: back the buffer with a private copy.
        let a = new_char_array(vm, &units)?;
        let cap = units.len() as i32;
        let _ = cs;
        return Ok(Some(char_buffer_fields(vm, a, 0, 0, cap, cap)?));
    }
    let units = char_array_all(vm, arr)?;
    let len = units.len() as i32;
    let (off, count) = if args.len() >= 3 && sig.starts_with("([CII)") {
        let off = int_arg(args, 1, 0);
        let count = int_arg(args, 2, 0);
        if off < 0 || count < 0 || off > len || count > len - off {
            return Err(err(
                "Ljava/lang/IndexOutOfBoundsException;",
                format!("offset {off} length {count} capacity {len}"),
            ));
        }
        (off, count)
    } else {
        (0, len)
    };
    Ok(Some(char_buffer_fields(vm, arr, off, 0, count, len - off)?))
}

fn char_buffer_instance(
    vm: &mut Engine,
    name: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let b = char_buf(vm, recv);
    let remaining = b.remaining();
    Ok(Some(match name {
        "position" => {
            if args.is_empty() {
                Value::Int(b.pos)
            } else {
                let np = int_arg(args, 0, 0);
                if np < 0 || np > b.limit {
                    return Err(illegal(format!("newPosition({np}) > limit({})", b.limit)));
                }
                put(vm, recv, "rd_pos", Value::Int(np))?;
                Value::Obj(recv)
            }
        }
        "limit" => {
            if args.is_empty() {
                Value::Int(b.limit)
            } else {
                let nl = int_arg(args, 0, 0);
                if nl < 0 || nl > b.cap {
                    return Err(illegal(format!("newLimit({nl}) > capacity({})", b.cap)));
                }
                put(vm, recv, "rd_limit", Value::Int(nl))?;
                if b.pos > nl {
                    put(vm, recv, "rd_pos", Value::Int(nl))?;
                }
                Value::Obj(recv)
            }
        }
        "capacity" => Value::Int(b.cap),
        "remaining" => Value::Int(remaining),
        "hasRemaining" => Value::Int(i32::from(remaining > 0)),
        "flip" => {
            put(vm, recv, "rd_limit", Value::Int(b.pos.max(0)))?;
            put(vm, recv, "rd_pos", Value::Int(0))?;
            Value::Obj(recv)
        }
        "rewind" => {
            put(vm, recv, "rd_pos", Value::Int(0))?;
            Value::Obj(recv)
        }
        "clear" => {
            put(vm, recv, "rd_pos", Value::Int(0))?;
            put(vm, recv, "rd_limit", Value::Int(b.cap))?;
            Value::Obj(recv)
        }
        "mark" => {
            put(vm, recv, "rd_mark", Value::Int(b.pos))?;
            Value::Obj(recv)
        }
        "reset" => {
            let m = get_int(vm, recv, "rd_mark", -1);
            if m < 0 {
                return Err(err(
                    "Ljava/nio/InvalidMarkException;",
                    "mark has not been set",
                ));
            }
            put(vm, recv, "rd_pos", Value::Int(m))?;
            Value::Obj(recv)
        }
        "slice" | "duplicate" => {
            let arr = b.arr.ok_or_else(|| npe("CharBuffer has no array"))?;
            let (base, pos, limit, cap) = if name == "slice" {
                (b.base + b.pos.max(0), 0, remaining, remaining)
            } else {
                (b.base, b.pos, b.limit, b.cap)
            };
            char_buffer_fields(vm, arr, base, pos, limit, cap)?
        }
        "array" => match b.arr {
            Some(a) => Value::Obj(a),
            None => Value::Null,
        },
        "hasArray" => Value::Int(i32::from(b.arr.is_some())),
        "arrayOffset" => Value::Int(b.base),
        "isDirect" | "isReadOnly" => Value::Int(0),
        "length" => Value::Int(b.cap),
        "charAt" => {
            let idx = int_arg(args, 0, 0);
            if idx < 0 || idx >= b.cap {
                return Err(err(
                    "Ljava/lang/IndexOutOfBoundsException;",
                    format!("index {idx}"),
                ));
            }
            let arr = b.arr.ok_or_else(|| npe("CharBuffer has no array"))?;
            let at = b.at(idx) as usize;
            let (elems, _) = vm.heap.as_array_elems(arr)?;
            let u = elems.get(at).and_then(|v| v.as_int().ok()).unwrap_or(0);
            Value::Int(u & 0xFFFF)
        }
        "get" => {
            if args.is_empty() {
                if remaining <= 0 {
                    return Err(err("Ljava/nio/BufferUnderflowException;", "underflow"));
                }
                let arr = b.arr.ok_or_else(|| npe("CharBuffer has no array"))?;
                let at = b.at(b.pos) as usize;
                let (elems, _) = vm.heap.as_array_elems(arr)?;
                let u = elems.get(at).and_then(|v| v.as_int().ok()).unwrap_or(0);
                put(vm, recv, "rd_pos", Value::Int(b.pos + 1))?;
                Value::Int(u & 0xFFFF)
            } else if let Some(dst) = obj_arg(args, 0) {
                let dst_len = vm.heap.as_array_elems(dst).map(|(e, _)| e.len())?;
                let (off, len) = if args.len() >= 3 {
                    (int_arg(args, 1, 0), int_arg(args, 2, 0))
                } else {
                    (0, dst_len as i32)
                };
                if off < 0
                    || len < 0
                    || off as usize > dst_len
                    || len as usize > dst_len - off as usize
                {
                    return Err(err(
                        "Ljava/lang/IndexOutOfBoundsException;",
                        "CharBuffer.get bounds",
                    ));
                }
                if len > remaining {
                    return Err(err("Ljava/nio/BufferUnderflowException;", "underflow"));
                }
                let arr = b.arr.ok_or_else(|| npe("CharBuffer has no array"))?;
                let mut copy = Vec::with_capacity(len as usize);
                {
                    let (src, _) = vm.heap.as_array_elems(arr)?;
                    for k in 0..len as usize {
                        let at = b.at(b.pos) as usize + k;
                        copy.push(src.get(at).and_then(|v| v.as_int().ok()).unwrap_or(0));
                    }
                }
                let d = vm.heap.array_elems_mut(dst)?;
                for (slot, u) in d[off as usize..off as usize + len as usize]
                    .iter_mut()
                    .zip(copy)
                {
                    *slot = Value::Int(u);
                }
                put(vm, recv, "rd_pos", Value::Int(b.pos + len))?;
                Value::Obj(recv)
            } else {
                let idx = int_arg(args, 0, 0);
                if idx < 0 || idx >= b.limit {
                    return Err(err(
                        "Ljava/lang/IndexOutOfBoundsException;",
                        format!("index {idx}"),
                    ));
                }
                let arr = b.arr.ok_or_else(|| npe("CharBuffer has no array"))?;
                let at = b.at(idx) as usize;
                let (elems, _) = vm.heap.as_array_elems(arr)?;
                let u = elems.get(at).and_then(|v| v.as_int().ok()).unwrap_or(0);
                Value::Int(u & 0xFFFF)
            }
        }
        "put" => {
            let arr = b.arr.ok_or_else(|| npe("CharBuffer has no array"))?;
            if let Some(s) = args.first().and_then(|v| match v {
                Value::Obj(o) => vm.heap.as_str(*o).ok().map(str::to_string),
                _ => None,
            }) {
                let units: Vec<u16> = s.encode_utf16().collect();
                if units.len() as i32 > remaining {
                    return Err(err("Ljava/nio/BufferOverflowException;", "overflow"));
                }
                let elems = vm.heap.array_elems_mut(arr)?;
                for (k, u) in units.iter().enumerate() {
                    let at = b.at(b.pos) as usize + k;
                    if let Some(slot) = elems.get_mut(at) {
                        *slot = Value::Int(i32::from(*u));
                    }
                }
                put(vm, recv, "rd_pos", Value::Int(b.pos + units.len() as i32))?;
                return Ok(Some(Value::Obj(recv)));
            }
            if let Some(src) = obj_arg(args, 0) {
                let src_units = char_array_all(vm, src)?;
                let (off, len) = if args.len() >= 3 {
                    (int_arg(args, 1, 0), int_arg(args, 2, 0))
                } else {
                    (0, src_units.len() as i32)
                };
                if off < 0
                    || len < 0
                    || off as usize > src_units.len()
                    || len as usize > src_units.len() - off as usize
                {
                    return Err(err(
                        "Ljava/lang/IndexOutOfBoundsException;",
                        "CharBuffer.put bounds",
                    ));
                }
                if len > remaining {
                    return Err(err("Ljava/nio/BufferOverflowException;", "overflow"));
                }
                let elems = vm.heap.array_elems_mut(arr)?;
                for k in 0..len as usize {
                    let at = b.at(b.pos) as usize + k;
                    if let Some(slot) = elems.get_mut(at) {
                        *slot = Value::Int(i32::from(src_units[off as usize + k]));
                    }
                }
                put(vm, recv, "rd_pos", Value::Int(b.pos + len))?;
                Value::Obj(recv)
            } else {
                let u = int_arg(args, 0, 0) as u16;
                if remaining <= 0 {
                    return Err(err("Ljava/nio/BufferOverflowException;", "overflow"));
                }
                let elems = vm.heap.array_elems_mut(arr)?;
                let at = b.at(b.pos) as usize;
                if let Some(slot) = elems.get_mut(at) {
                    *slot = Value::Int(i32::from(u));
                }
                put(vm, recv, "rd_pos", Value::Int(b.pos + 1))?;
                Value::Obj(recv)
            }
        }
        "toString" | "subSequence" => {
            let arr = b.arr.ok_or_else(|| npe("CharBuffer has no array"))?;
            let (start, count) = if name == "subSequence" {
                let s = int_arg(args, 0, 0);
                let e = int_arg(args, 1, 0);
                if s < 0 || e < s || e > b.cap {
                    return Err(err(
                        "Ljava/lang/IndexOutOfBoundsException;",
                        "subSequence range",
                    ));
                }
                (s, e - s)
            } else {
                (b.pos.max(0), remaining)
            };
            let (elems, _) = vm.heap.as_array_elems(arr)?;
            let mut units = Vec::with_capacity(count.max(0) as usize);
            for k in 0..count.max(0) as usize {
                let at = b.at(start) as usize + k;
                units.push(elems.get(at).and_then(|v| v.as_int().ok()).unwrap_or(0) as u16);
            }
            jstr(vm, String::from_utf16_lossy(&units))?
        }
        "equals" => {
            let Some(other) = obj_arg(args, 0) else {
                return Ok(Some(Value::Int(0)));
            };
            let ob = char_buf(vm, other);
            if ob.remaining() != remaining {
                return Ok(Some(Value::Int(0)));
            }
            let (Some(a1), Some(a2)) = (b.arr, ob.arr) else {
                return Ok(Some(Value::Int(0)));
            };
            let (e1, _) = vm.heap.as_array_elems(a1)?;
            let (e2, _) = vm.heap.as_array_elems(a2)?;
            let mut same = true;
            for k in 0..remaining.max(0) as usize {
                let x = e1
                    .get(b.at(b.pos) as usize + k)
                    .and_then(|v| v.as_int().ok());
                let y = e2
                    .get(ob.at(ob.pos) as usize + k)
                    .and_then(|v| v.as_int().ok());
                if x != y {
                    same = false;
                    break;
                }
            }
            Value::Int(i32::from(same))
        }
        "hashCode" => Value::Int(b.pos * 31 + b.limit * 17 + b.cap),
        "compareTo" => Value::Int(0),
        "chars" => return Ok(None),
        _ => return Ok(None),
    }))
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

pub(crate) fn static_intrinsic(
    vm: &mut Engine,
    class: &str,
    name: &str,
    sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match class {
        CHAR_BUFFER => char_buffer_static(vm, name, sig, args),
        CODER_RESULT => Ok(Some(match name {
            "malformedForLength" => coder_result(vm, MALFORMED, int_arg(args, 0, 1).max(1))?,
            "unmappableForLength" => coder_result(vm, UNMAPPABLE, int_arg(args, 0, 1).max(1))?,
            _ => return Ok(None),
        })),
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
        CHARSET => Ok(match name {
            "newDecoder" => Some(new_decoder(vm, recv)?),
            "newEncoder" => Some(new_encoder(vm, recv)?),
            _ => None,
        }),
        DECODER => decoder_instance(vm, name, sig, recv, args),
        ENCODER => encoder_instance(vm, name, sig, recv, args),
        CODER_RESULT => coder_result_instance(vm, name, recv, args),
        CODING_ERROR_ACTION => Ok(Some(match name {
            "toString" => jstr(
                vm,
                match get_int(vm, recv, "rd_action", ACTION_REPORT) {
                    ACTION_REPLACE => "REPLACE",
                    ACTION_IGNORE => "IGNORE",
                    _ => "REPORT",
                },
            )?,
            "hashCode" => Value::Int(get_int(vm, recv, "rd_action", 0)),
            "equals" => Value::Int(matches!(args.first(), Some(Value::Obj(o))
                    if get_int(vm, *o, "rd_action", -1) == get_int(vm, recv, "rd_action", -2))
                as i32),
            _ => return Ok(None),
        })),
        CHAR_BUFFER => char_buffer_instance(vm, name, recv, args),
        BYTE_BUFFER if name == "compact" => byte_buffer_compact(vm, recv),
        _ => Ok(None),
    }
}

fn decoder_instance(
    vm: &mut Engine,
    name: &str,
    _sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    Ok(Some(match name {
        "<init>" => Value::Null,
        "decode" => {
            // `decode(ByteBuffer, CharBuffer, boolean)` streams; the one-argument
            // `decode(ByteBuffer)` decodes the whole buffer at once.
            if args.len() >= 3 {
                decode_stream(vm, recv, args)?
            } else {
                decode_oneshot(vm, recv, args)?
            }
        }
        "onMalformedInput" | "onUnmappableCharacter" | "replaceWith" | "onOverflow" => {
            if let Some(action) = obj_arg(args, 0) {
                let a = get_int(vm, action, "rd_action", ACTION_REPORT);
                let field = if name == "onMalformedInput" {
                    "rd_malformed"
                } else {
                    "rd_unmappable"
                };
                put(vm, recv, field, Value::Int(a))?;
            }
            Value::Obj(recv)
        }
        "malformedInputAction" | "unmappableCharacterAction" => {
            let (mal, unm, _) = decoder_actions(vm, recv);
            coding_error_action(
                vm,
                if name == "malformedInputAction" {
                    mal
                } else {
                    unm
                },
            )?
        }
        "reset" => {
            set_decoder_pending(vm, recv, &[])?;
            put(vm, recv, "rd_bom_done", Value::Int(0))?;
            put(vm, recv, "rd_be", Value::Int(1))?;
            Value::Obj(recv)
        }
        "flush" => coder_result(vm, UNDERFLOW, 0)?,
        "charset" => decoder_charset(vm, recv)?,
        "averageCharsPerByte" | "maxCharsPerByte" => Value::Float(1.0),
        "replacement" => jstr(vm, "\u{FFFD}")?,
        "isAutoDetecting" => Value::Int(0),
        "isCharsetDetected" => Value::Int(1),
        "detectedCharset" => decoder_charset(vm, recv)?,
        "toString" => jstr(vm, "CharsetDecoder")?,
        _ => return Ok(None),
    }))
}

/// `ByteBuffer.compact()`: move the remaining bytes to the front, then prepare
/// the buffer for a fresh `put`/`read` (`position = remaining`, `limit =
/// capacity`). `CharsetReader.fillByteBuffer` relies on this to keep a partial
/// code point between reads.
fn byte_buffer_compact(vm: &mut Engine, recv: ObjRef) -> Result<Option<Value>, VmExit> {
    let b = byte_buf(vm, recv);
    let Some(arr) = b.arr else {
        return Ok(Some(Value::Obj(recv)));
    };
    let rem = b.remaining().max(0) as usize;
    if rem > 0 {
        let mut keep = Vec::with_capacity(rem);
        {
            let (elems, _) = vm.heap.as_array_elems(arr)?;
            for k in 0..rem {
                let at = b.at(b.pos) as usize + k;
                keep.push(elems.get(at).and_then(|v| v.as_int().ok()).unwrap_or(0));
            }
        }
        let elems = vm.heap.array_elems_mut(arr)?;
        for (k, v) in keep.into_iter().enumerate() {
            if let Some(slot) = elems.get_mut(k) {
                *slot = Value::Int(v);
            }
        }
    }
    put(vm, recv, "rd_pos", Value::Int(rem as i32))?;
    put(vm, recv, "rd_limit", Value::Int(b.cap))?;
    Ok(Some(Value::Obj(recv)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::VmConfig;

    fn byte_buf_with(vm: &mut Engine, bytes: &[u8]) -> ObjRef {
        let arr = vm
            .heap
            .alloc_array_zeroed(ElemKind::Int, bytes.len(), "[B")
            .unwrap();
        {
            let elems = vm.heap.array_elems_mut(arr).unwrap();
            for (slot, b) in elems.iter_mut().zip(bytes) {
                *slot = Value::Int(i32::from(*b));
            }
        }
        let len = bytes.len() as i32;
        alloc(
            vm,
            BYTE_BUFFER,
            vec![
                ("rd_buf".to_string(), Value::Obj(arr)),
                ("rd_pos".to_string(), Value::Int(0)),
                ("rd_limit".to_string(), Value::Int(len)),
                ("rd_cap".to_string(), Value::Int(len)),
            ],
        )
        .unwrap()
    }

    fn char_buf_of(vm: &mut Engine, cap: usize) -> (ObjRef, ObjRef) {
        let arr = vm
            .heap
            .alloc_array_zeroed(ElemKind::Int, cap, "[C")
            .unwrap();
        let cap = cap as i32;
        let buf = alloc(
            vm,
            CHAR_BUFFER,
            vec![
                ("rd_arr".to_string(), Value::Obj(arr)),
                ("rd_pos".to_string(), Value::Int(0)),
                ("rd_limit".to_string(), Value::Int(cap)),
                ("rd_cap".to_string(), Value::Int(cap)),
                ("rd_base".to_string(), Value::Int(0)),
            ],
        )
        .unwrap();
        (buf, arr)
    }

    fn units_of(vm: &Engine, arr: ObjRef) -> Vec<u16> {
        char_array_all(vm, arr).unwrap()
    }

    fn decode_call(vm: &mut Engine, dec: ObjRef, src: ObjRef, dst: ObjRef, eof: bool) -> ObjRef {
        match instance_intrinsic(
            vm,
            DECODER,
            "decode",
            "(Ljava/nio/ByteBuffer;Ljava/nio/CharBuffer;Z)Ljava/nio/charset/CoderResult;",
            dec,
            &[Value::Obj(src), Value::Obj(dst), Value::Int(i32::from(eof))],
        )
        .unwrap()
        {
            Some(Value::Obj(r)) => r,
            other => panic!("decode returned {other:?}"),
        }
    }

    fn result_kind(vm: &Engine, r: ObjRef) -> i32 {
        get_int(vm, r, "rd_kind", -1)
    }

    fn charset_for_name(vm: &mut Engine, name: &str) -> ObjRef {
        let n = jstr(vm, name).unwrap();
        match crate::jtext_format::static_intrinsic(vm, CHARSET, "forName", &[n]).unwrap() {
            Some(Value::Obj(c)) => c,
            other => panic!("forName returned {other:?}"),
        }
    }

    fn utf8_decoder(vm: &mut Engine) -> ObjRef {
        let charset = charset_for_name(vm, "UTF-8");
        let dec = match instance_intrinsic(
            vm,
            CHARSET,
            "newDecoder",
            "()Ljava/nio/charset/CharsetDecoder;",
            charset,
            &[],
        )
        .unwrap()
        {
            Some(Value::Obj(d)) => d,
            other => panic!("newDecoder returned {other:?}"),
        };
        // CodingErrorAction.REPLACE, exactly like CharsetReader.<init>.
        let replace = match host_static(vm, CODING_ERROR_ACTION, "REPLACE").unwrap() {
            Some(Value::Obj(a)) => a,
            other => panic!("REPLACE returned {other:?}"),
        };
        for m in ["onMalformedInput", "onUnmappableCharacter"] {
            instance_intrinsic(
                vm,
                DECODER,
                m,
                "(Ljava/nio/charset/CodingErrorAction;)Ljava/nio/charset/CharsetDecoder;",
                dec,
                &[Value::Obj(replace)],
            )
            .unwrap();
        }
        dec
    }

    /// `CharsetReader` streams an 8 KiB `ByteBuffer`; a 2-byte UTF-8 sequence
    /// split across two reads must survive in the decoder state.
    #[test]
    fn decoder_keeps_split_multibyte_sequence_between_calls() {
        let mut vm = Engine::new(Vec::new(), VmConfig::default());
        let dec = utf8_decoder(&mut vm);
        let (dst, dst_arr) = char_buf_of(&mut vm, 8);

        // "é" = C3 A9, split after the lead byte.
        let first = byte_buf_with(&mut vm, &[0xC3]);
        let r = decode_call(&mut vm, dec, first, dst, false);
        assert_eq!(result_kind(&vm, r), UNDERFLOW);
        assert_eq!(get_int(&vm, dst, "rd_pos", -1), 0, "no char yet");

        let second = byte_buf_with(&mut vm, &[0xA9, b'x']);
        let r = decode_call(&mut vm, dec, second, dst, false);
        assert_eq!(result_kind(&vm, r), UNDERFLOW);
        assert_eq!(units_of(&vm, dst_arr)[..2], [0x00E9, u16::from(b'x')]);
        assert_eq!(get_int(&vm, dst, "rd_pos", -1), 2);
    }

    /// Malformed input under `CodingErrorAction.REPLACE` becomes U+FFFD and is
    /// consumed (the default `REPORT` would raise `MalformedInputException`).
    #[test]
    fn malformed_utf8_uses_the_configured_action() {
        let mut vm = Engine::new(Vec::new(), VmConfig::default());
        let dec = utf8_decoder(&mut vm);
        let (dst, dst_arr) = char_buf_of(&mut vm, 8);
        // 0xFF is never a valid UTF-8 lead byte.
        let src = byte_buf_with(&mut vm, &[b'a', 0xFF, b'b']);
        let r = decode_call(&mut vm, dec, src, dst, true);
        assert_eq!(result_kind(&vm, r), UNDERFLOW);
        assert_eq!(
            units_of(&vm, dst_arr)[..3],
            [u16::from(b'a'), 0xFFFD, u16::from(b'b')]
        );
        assert_eq!(get_int(&vm, src, "rd_pos", -1), 3, "bad byte consumed");

        // A fresh decoder keeps REPORT: the same input reports a length.
        let report = charset_for_name(&mut vm, "UTF-8");
        let dec2 = match instance_intrinsic(
            &mut vm,
            CHARSET,
            "newDecoder",
            "()Ljava/nio/charset/CharsetDecoder;",
            report,
            &[],
        )
        .unwrap()
        {
            Some(Value::Obj(d)) => d,
            other => panic!("newDecoder returned {other:?}"),
        };
        let (dst2, _) = char_buf_of(&mut vm, 8);
        let src2 = byte_buf_with(&mut vm, &[b'a', 0xFF]);
        let r = decode_call(&mut vm, dec2, src2, dst2, true);
        assert_eq!(result_kind(&vm, r), MALFORMED);
        assert_eq!(get_int(&vm, r, "rd_len", 0), 1);
        assert_eq!(
            get_int(&vm, src2, "rd_pos", -1),
            1,
            "the offending byte stays unconsumed"
        );
    }

    /// `CharBuffer.wrap(char[], off, len)` is a window: `slice()` addresses the
    /// caller's array, and the produced count is the buffer's position — the
    /// contract `CharsetReader.doRead` relies on.
    #[test]
    fn char_buffer_wrap_and_slice_write_into_the_caller_array() {
        let mut vm = Engine::new(Vec::new(), VmConfig::default());
        let (_, arr) = char_buf_of(&mut vm, 6);
        let buf = match static_intrinsic(
            &mut vm,
            CHAR_BUFFER,
            "wrap",
            "([CII)Ljava/nio/CharBuffer;",
            &[Value::Obj(arr), Value::Int(2), Value::Int(4)],
        )
        .unwrap()
        {
            Some(Value::Obj(b)) => b,
            other => panic!("wrap returned {other:?}"),
        };
        assert_eq!(get_int(&vm, buf, "rd_pos", -1), 0, "window starts at 0");
        assert_eq!(get_int(&vm, buf, "rd_limit", -1), 4);
        assert_eq!(get_int(&vm, buf, "rd_base", -1), 2, "arrayOffset");
        let slice = match instance_intrinsic(
            &mut vm,
            CHAR_BUFFER,
            "slice",
            "()Ljava/nio/CharBuffer;",
            buf,
            &[],
        )
        .unwrap()
        {
            Some(Value::Obj(b)) => b,
            other => panic!("slice returned {other:?}"),
        };
        let elems = vm.heap.array_elems_mut(arr).unwrap();
        for (i, slot) in elems.iter_mut().enumerate() {
            *slot = Value::Int(0x1000 + i as i32);
        }
        let got = units_of(&vm, arr);
        assert_eq!(&got[2..6], &[0x1002, 0x1003, 0x1004, 0x1005]);
        assert_eq!(get_int(&vm, slice, "rd_base", -1), 2);
        assert_eq!(get_int(&vm, slice, "rd_limit", -1), 4);
    }

    #[test]
    fn encoder_round_trips_supplementary_characters() {
        let mut vm = Engine::new(Vec::new(), VmConfig::default());
        let charset = charset_for_name(&mut vm, "UTF-8");
        let enc = match instance_intrinsic(
            &mut vm,
            CHARSET,
            "newEncoder",
            "()Ljava/nio/charset/CharsetEncoder;",
            charset,
            &[],
        )
        .unwrap()
        {
            Some(Value::Obj(e)) => e,
            other => panic!("newEncoder returned {other:?}"),
        };
        // U+1F600 (😀) = D83D DE00.
        let (src, src_arr) = char_buf_of(&mut vm, 2);
        {
            let elems = vm.heap.array_elems_mut(src_arr).unwrap();
            elems[0] = Value::Int(0xD83D);
            elems[1] = Value::Int(0xDE00);
        }
        let bb = match instance_intrinsic(
            &mut vm,
            ENCODER,
            "encode",
            "(Ljava/nio/CharBuffer;)Ljava/nio/ByteBuffer;",
            enc,
            &[Value::Obj(src)],
        )
        .unwrap()
        {
            Some(Value::Obj(b)) => b,
            other => panic!("encode returned {other:?}"),
        };
        let bytes = byte_array_all(&vm, field_obj(&vm, bb, "rd_buf")).unwrap();
        assert_eq!(bytes, vec![0xF0, 0x9F, 0x98, 0x80]);
        assert_eq!(get_int(&vm, src, "rd_pos", -1), 2, "both units consumed");
    }

    #[test]
    fn compact_keeps_the_unread_bytes_at_the_front() {
        let mut vm = Engine::new(Vec::new(), VmConfig::default());
        let src = byte_buf_with(&mut vm, &[1, 2, 3, 4]);
        put(&mut vm, src, "rd_pos", Value::Int(2)).unwrap();
        instance_intrinsic(
            &mut vm,
            BYTE_BUFFER,
            "compact",
            "()Ljava/nio/ByteBuffer;",
            src,
            &[],
        )
        .unwrap();
        assert_eq!(get_int(&vm, src, "rd_pos", -1), 2);
        assert_eq!(get_int(&vm, src, "rd_limit", -1), 4);
        let bytes = byte_array_all(&vm, field_obj(&vm, src, "rd_buf")).unwrap();
        assert_eq!(bytes, vec![3, 4, 3, 4], "remaining moved to the front");
    }
}
