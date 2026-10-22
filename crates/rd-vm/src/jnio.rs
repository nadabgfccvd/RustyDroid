//! `java.nio.ByteBuffer` / `Buffer` / `ByteOrder`: heap-backed model.
//!
//! State lives in heap fields (`bb_arr` = the backing `[B`, `bb_off`,
//! `bb_pos`, `bb_lim`, `bb_cap`, `bb_mark`, `bb_big`, `bb_direct`,
//! `bb_ro`). `duplicate`/`slice` share the same backing array object, as
//! AOSP shares storage. Direct buffers are heap-backed too (documented
//! deviation: JNI code cannot consume them, but no JNI runs here);
//! `array()` still rejects direct/read-only misuse per AOSP. Bounds and
//! overflow raise the real `java.nio` exceptions.

use crate::engine::Engine;
use crate::err::VmExit;
use crate::heap::{ElemKind, ObjRef};
use crate::value::Value;

pub(crate) const BYTE_BUFFER: &str = "Ljava/nio/ByteBuffer;";
const BUFFER: &str = "Ljava/nio/Buffer;";
pub(crate) const BYTE_ORDER: &str = "Ljava/nio/ByteOrder;";

fn exc(c: &str, m: impl Into<String>) -> VmExit {
    VmExit::Exception(crate::err::Throwable::new(c, m.into()))
}

fn put(vm: &mut Engine, r: ObjRef, f: &str, v: Value) -> Result<(), VmExit> {
    crate::err::put_field_result(vm.heap.put_field(r, f, v))
}

fn geti(vm: &Engine, r: ObjRef, f: &str, d: i32) -> i32 {
    match vm.heap.get_field(r, f) {
        Ok(Value::Int(i)) => i,
        _ => d,
    }
}

#[allow(clippy::too_many_arguments)] // buffer state mirrors java.nio.Buffer fields
fn new_buffer(
    vm: &mut Engine,
    arr: Value,
    off: i32,
    pos: i32,
    lim: i32,
    cap: i32,
    direct: bool,
    ro: bool,
) -> Result<ObjRef, VmExit> {
    let b = vm
        .heap
        .alloc_instance(BYTE_BUFFER.to_string(), Vec::new())
        .map_err(crate::err::oom_throwable)?;
    put(vm, b, "bb_arr", arr)?;
    put(vm, b, "bb_off", Value::Int(off))?;
    put(vm, b, "bb_pos", Value::Int(pos))?;
    put(vm, b, "bb_lim", Value::Int(lim))?;
    put(vm, b, "bb_cap", Value::Int(cap))?;
    put(vm, b, "bb_mark", Value::Int(-1))?;
    put(vm, b, "bb_big", Value::Int(1))?;
    put(vm, b, "bb_direct", Value::Int(direct as i32))?;
    put(vm, b, "bb_ro", Value::Int(ro as i32))?;
    Ok(b)
}

/// Canonical `ByteOrder` singletons (identity-compared by guests).
pub(crate) fn byte_order_obj(vm: &mut Engine, big: bool) -> Result<ObjRef, VmExit> {
    let name = if big { "BIG_ENDIAN" } else { "LITTLE_ENDIAN" };
    let key = (BYTE_ORDER.to_string(), name.to_string());
    if let Some(Value::Obj(o)) = vm.statics.get(&key) {
        return Ok(*o);
    }
    let o = vm
        .heap
        .alloc_instance(BYTE_ORDER.to_string(), Vec::new())
        .map_err(crate::err::oom_throwable)?;
    put(vm, o, "bb_big_endian", Value::Int(big as i32))?;
    vm.statics.insert(key, Value::Obj(o));
    Ok(o)
}

fn arr_of(vm: &Engine, buf: ObjRef) -> Result<ObjRef, VmExit> {
    match vm.heap.get_field(buf, "bb_arr") {
        Ok(Value::Obj(a)) => Ok(a),
        _ => Err(exc(
            "Ljava/lang/IllegalStateException;",
            "buffer sem backing array",
        )),
    }
}

fn byte_at(vm: &Engine, buf: ObjRef, index: i32) -> Result<u8, VmExit> {
    let arr = arr_of(vm, buf)?;
    let (elems, _) = vm
        .heap
        .as_array_elems(arr)
        .map_err(|m| exc("Ljava/lang/IndexOutOfBoundsException;", &m))?;
    elems
        .get(index as usize)
        .and_then(|v| v.as_int().ok())
        .map(|i| i as u8)
        .ok_or_else(|| {
            exc(
                "Ljava/lang/IndexOutOfBoundsException;",
                "leitura fora do buffer",
            )
        })
}

fn set_byte(vm: &mut Engine, buf: ObjRef, index: i32, v: u8) -> Result<(), VmExit> {
    if geti(vm, buf, "bb_ro", 0) != 0 {
        return Err(exc(
            "Ljava/nio/ReadOnlyBufferException;",
            "put em buffer read-only",
        ));
    }
    let arr = arr_of(vm, buf)?;
    let elems = vm
        .heap
        .array_elems_mut(arr)
        .map_err(|m| exc("Ljava/lang/IndexOutOfBoundsException;", &m))?;
    let slot = elems.get_mut(index as usize).ok_or_else(|| {
        exc(
            "Ljava/lang/IndexOutOfBoundsException;",
            "escrita fora do buffer",
        )
    })?;
    *slot = Value::Int(v as i8 as i32);
    Ok(())
}

fn check_range(vm: &Engine, buf: ObjRef, index: i32, size: i32) -> Result<(), VmExit> {
    let lim = geti(vm, buf, "bb_lim", 0);
    if index < 0 || index > lim - size {
        return Err(exc(
            "Ljava/lang/IndexOutOfBoundsException;",
            "índice fora dos limites do buffer",
        ));
    }
    Ok(())
}

fn read_bytes(vm: &Engine, buf: ObjRef, index: i32, n: usize) -> Result<Vec<u8>, VmExit> {
    check_range(vm, buf, index, n as i32)?;
    let off = geti(vm, buf, "bb_off", 0);
    (0..n)
        .map(|k| byte_at(vm, buf, off + index + k as i32))
        .collect()
}

fn write_bytes(vm: &mut Engine, buf: ObjRef, index: i32, data: &[u8]) -> Result<(), VmExit> {
    check_range(vm, buf, index, data.len() as i32)?;
    let off = geti(vm, buf, "bb_off", 0);
    for (k, b) in data.iter().enumerate() {
        set_byte(vm, buf, off + index + k as i32, *b)?;
    }
    Ok(())
}

fn encode(vm: &Engine, buf: ObjRef, v: u64, n: usize) -> [u8; 8] {
    let big = geti(vm, buf, "bb_big", 1) != 0;
    let mut out = [0u8; 8];
    for (i, slot) in out.iter_mut().take(n).enumerate() {
        let shift = if big { (n - 1 - i) * 8 } else { i * 8 };
        *slot = (v >> shift) as u8;
    }
    out
}

fn decode(vm: &Engine, buf: ObjRef, bytes: &[u8]) -> u64 {
    let big = geti(vm, buf, "bb_big", 1) != 0;
    let mut v: u64 = 0;
    for (i, b) in bytes.iter().enumerate() {
        let shift = if big {
            (bytes.len() - 1 - i) * 8
        } else {
            i * 8
        };
        v |= (*b as u64) << shift;
    }
    v
}

fn int_arg(args: &[Value], i: usize) -> i32 {
    args.get(i).and_then(|v| v.as_int().ok()).unwrap_or(0)
}

pub(crate) fn static_intrinsic(
    vm: &mut Engine,
    class: &str,
    name: &str,
    sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if class == BYTE_ORDER {
        return Ok(match name {
            "nativeOrder" => Some(Value::Obj(byte_order_obj(vm, true)?)),
            _ => None,
        });
    }
    if class != BYTE_BUFFER {
        return Ok(None);
    }
    Ok(Some(match (name, sig) {
        ("wrap", "([B)Ljava/nio/ByteBuffer;") | ("wrap", "([BII)Ljava/nio/ByteBuffer;") => {
            let Some(Value::Obj(arr)) = args.first().cloned() else {
                return Err(exc("Ljava/lang/NullPointerException;", "wrap(null)"));
            };
            let len = vm
                .heap
                .as_array_elems(arr)
                .map(|(e, _)| e.len())
                .unwrap_or(0) as i32;
            let (off, count) = if sig.contains("II") {
                (int_arg(args, 1), int_arg(args, 2))
            } else {
                (0, len)
            };
            if off < 0 || count < 0 || off + count > len {
                return Err(exc(
                    "Ljava/lang/IndexOutOfBoundsException;",
                    "wrap fora dos limites",
                ));
            }
            Value::Obj(new_buffer(
                vm,
                Value::Obj(arr),
                0,
                off,
                off + count,
                len,
                false,
                false,
            )?)
        }
        ("allocate", "(I)Ljava/nio/ByteBuffer;")
        | ("allocateDirect", "(I)Ljava/nio/ByteBuffer;") => {
            let cap = int_arg(args, 0);
            if cap < 0 {
                return Err(exc(
                    "Ljava/lang/IllegalArgumentException;",
                    "capacidade negativa",
                ));
            }
            let arr = vm
                .heap
                .alloc_array_zeroed(ElemKind::Int, cap as usize, "[B")
                .map_err(crate::err::oom_throwable)?;
            Value::Obj(new_buffer(
                vm,
                Value::Obj(arr),
                0,
                0,
                cap,
                cap,
                name == "allocateDirect",
                false,
            )?)
        }
        _ => return Ok(None),
    }))
}

pub(crate) fn instance_intrinsic(
    vm: &mut Engine,
    recv_class: &str,
    mref_class: &str,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let is_buffer = [BYTE_BUFFER, BUFFER].contains(&mref_class)
        || [BYTE_BUFFER, BUFFER].contains(&recv_class)
        || vm.cp.is_subtype(recv_class, BUFFER);
    if !is_buffer {
        return Ok(None);
    }
    let pos = geti(vm, recv, "bb_pos", 0);
    let lim = geti(vm, recv, "bb_lim", 0);
    let obj = Value::Obj(recv);
    Ok(Some(match (name, sig) {
        ("position", "()I") => Value::Int(pos),
        ("position", "(I)Ljava/nio/Buffer;") => {
            let p = int_arg(args, 0);
            if p < 0 || p > lim {
                return Err(exc(
                    "Ljava/lang/IllegalArgumentException;",
                    "position fora de [0, limit]",
                ));
            }
            if geti(vm, recv, "bb_mark", -1) > p {
                put(vm, recv, "bb_mark", Value::Int(-1))?;
            }
            put(vm, recv, "bb_pos", Value::Int(p))?;
            obj
        }
        ("limit", "()I") => Value::Int(lim),
        ("limit", "(I)Ljava/nio/Buffer;") => {
            let l = int_arg(args, 0);
            let cap = geti(vm, recv, "bb_cap", 0);
            if l < 0 || l > cap {
                return Err(exc(
                    "Ljava/lang/IllegalArgumentException;",
                    "limit fora de [0, capacity]",
                ));
            }
            put(vm, recv, "bb_lim", Value::Int(l))?;
            if pos > l {
                put(vm, recv, "bb_pos", Value::Int(l))?;
            }
            if geti(vm, recv, "bb_mark", -1) > l {
                put(vm, recv, "bb_mark", Value::Int(-1))?;
            }
            obj
        }
        ("capacity", "()I") => Value::Int(geti(vm, recv, "bb_cap", 0)),
        ("remaining", "()I") => Value::Int(lim - pos),
        ("hasRemaining", "()Z") => Value::Int((pos < lim) as i32),
        ("flip", "()Ljava/nio/Buffer;") => {
            put(vm, recv, "bb_lim", Value::Int(pos))?;
            put(vm, recv, "bb_pos", Value::Int(0))?;
            put(vm, recv, "bb_mark", Value::Int(-1))?;
            obj
        }
        ("clear", "()Ljava/nio/Buffer;") => {
            put(vm, recv, "bb_pos", Value::Int(0))?;
            put(vm, recv, "bb_lim", Value::Int(geti(vm, recv, "bb_cap", 0)))?;
            put(vm, recv, "bb_mark", Value::Int(-1))?;
            obj
        }
        ("rewind", "()Ljava/nio/Buffer;") => {
            put(vm, recv, "bb_pos", Value::Int(0))?;
            put(vm, recv, "bb_mark", Value::Int(-1))?;
            obj
        }
        ("mark", "()Ljava/nio/Buffer;") => {
            put(vm, recv, "bb_mark", Value::Int(pos))?;
            obj
        }
        ("reset", "()Ljava/nio/Buffer;") => {
            let m = geti(vm, recv, "bb_mark", -1);
            if m < 0 {
                return Err(exc("Ljava/nio/InvalidMarkException;", "reset sem mark"));
            }
            put(vm, recv, "bb_pos", Value::Int(m))?;
            obj
        }
        ("isReadOnly", "()Z") => Value::Int(geti(vm, recv, "bb_ro", 0)),
        ("isDirect", "()Z") => Value::Int(geti(vm, recv, "bb_direct", 0)),
        ("hasArray", "()Z") => Value::Int((geti(vm, recv, "bb_direct", 0) == 0) as i32),
        ("array", "()Ljava/lang/Object;") | ("array", "()[B") => {
            if geti(vm, recv, "bb_direct", 0) != 0 {
                return Err(exc(
                    "Ljava/lang/UnsupportedOperationException;",
                    "buffer direto não tem array",
                ));
            }
            if geti(vm, recv, "bb_ro", 0) != 0 {
                return Err(exc(
                    "Ljava/nio/ReadOnlyBufferException;",
                    "array() em read-only",
                ));
            }
            vm.heap.get_field(recv, "bb_arr").unwrap_or(Value::Null)
        }
        ("arrayOffset", "()I") => Value::Int(geti(vm, recv, "bb_off", 0)),
        ("order", "()Ljava/nio/ByteOrder;") => {
            Value::Obj(byte_order_obj(vm, geti(vm, recv, "bb_big", 1) != 0)?)
        }
        ("order", "(Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;") => {
            let big = match args.first() {
                Some(Value::Obj(o)) => geti(vm, *o, "bb_big_endian", 1) != 0,
                _ => return Err(exc("Ljava/lang/NullPointerException;", "order(null)")),
            };
            put(vm, recv, "bb_big", Value::Int(big as i32))?;
            obj
        }
        ("duplicate", "()Ljava/nio/ByteBuffer;") => Value::Obj(new_buffer(
            vm,
            vm.heap.get_field(recv, "bb_arr").unwrap_or(Value::Null),
            geti(vm, recv, "bb_off", 0),
            pos,
            lim,
            geti(vm, recv, "bb_cap", 0),
            geti(vm, recv, "bb_direct", 0) != 0,
            geti(vm, recv, "bb_ro", 0) != 0,
        )?),
        ("slice", "()Ljava/nio/ByteBuffer;") => Value::Obj(new_buffer(
            vm,
            vm.heap.get_field(recv, "bb_arr").unwrap_or(Value::Null),
            geti(vm, recv, "bb_off", 0) + pos,
            0,
            lim - pos,
            lim - pos,
            geti(vm, recv, "bb_direct", 0) != 0,
            geti(vm, recv, "bb_ro", 0) != 0,
        )?),
        ("asReadOnlyBuffer", "()Ljava/nio/ByteBuffer;") => Value::Obj(new_buffer(
            vm,
            vm.heap.get_field(recv, "bb_arr").unwrap_or(Value::Null),
            geti(vm, recv, "bb_off", 0),
            pos,
            lim,
            geti(vm, recv, "bb_cap", 0),
            geti(vm, recv, "bb_direct", 0) != 0,
            true,
        )?),
        ("compact", "()Ljava/nio/ByteBuffer;") => {
            let data = read_bytes(vm, recv, pos, (lim - pos).max(0) as usize)?;
            write_bytes(vm, recv, 0, &data)?;
            put(vm, recv, "bb_pos", Value::Int(data.len() as i32))?;
            put(vm, recv, "bb_lim", Value::Int(geti(vm, recv, "bb_cap", 0)))?;
            put(vm, recv, "bb_mark", Value::Int(-1))?;
            obj
        }
        ("get", "()B") => {
            if pos >= lim {
                return Err(exc(
                    "Ljava/nio/BufferUnderflowException;",
                    "get além do limit",
                ));
            }
            let b = read_bytes(vm, recv, pos, 1)?[0];
            put(vm, recv, "bb_pos", Value::Int(pos + 1))?;
            Value::Int(b as i8 as i32)
        }
        ("get", "(I)B") => Value::Int(read_bytes(vm, recv, int_arg(args, 0), 1)?[0] as i8 as i32),
        ("put", "(B)Ljava/nio/ByteBuffer;") => {
            if pos >= lim {
                return Err(exc(
                    "Ljava/nio/BufferOverflowException;",
                    "put além do limit",
                ));
            }
            write_bytes(vm, recv, pos, &[int_arg(args, 0) as u8])?;
            put(vm, recv, "bb_pos", Value::Int(pos + 1))?;
            obj
        }
        ("put", "(IB)Ljava/nio/ByteBuffer;") => {
            write_bytes(vm, recv, int_arg(args, 0), &[int_arg(args, 1) as u8])?;
            obj
        }
        ("getInt", "()I") => {
            let b = read_bytes(vm, recv, pos, 4)?;
            put(vm, recv, "bb_pos", Value::Int(pos + 4))?;
            Value::Int(decode(vm, recv, &b) as u32 as i32)
        }
        ("getInt", "(I)I") => {
            Value::Int(decode(vm, recv, &read_bytes(vm, recv, int_arg(args, 0), 4)?) as u32 as i32)
        }
        ("putInt", "(I)Ljava/nio/ByteBuffer;") => {
            let e = encode(vm, recv, int_arg(args, 0) as u32 as u64, 4);
            write_bytes(vm, recv, pos, &e[..4])?;
            put(vm, recv, "bb_pos", Value::Int(pos + 4))?;
            obj
        }
        ("putInt", "(II)Ljava/nio/ByteBuffer;") => {
            let e = encode(vm, recv, int_arg(args, 1) as u32 as u64, 4);
            write_bytes(vm, recv, int_arg(args, 0), &e[..4])?;
            obj
        }
        ("getLong", "()J") => {
            let b = read_bytes(vm, recv, pos, 8)?;
            put(vm, recv, "bb_pos", Value::Int(pos + 8))?;
            Value::Long(decode(vm, recv, &b) as i64)
        }
        ("getLong", "(I)J") => {
            Value::Long(decode(vm, recv, &read_bytes(vm, recv, int_arg(args, 0), 8)?) as i64)
        }
        ("putLong", "(J)Ljava/nio/ByteBuffer;") => {
            let v = args.first().and_then(|v| v.as_long().ok()).unwrap_or(0);
            let e = encode(vm, recv, v as u64, 8);
            write_bytes(vm, recv, pos, &e[..8])?;
            put(vm, recv, "bb_pos", Value::Int(pos + 8))?;
            obj
        }
        ("putLong", "(IJ)Ljava/nio/ByteBuffer;") => {
            let v = args.get(1).and_then(|v| v.as_long().ok()).unwrap_or(0);
            let e = encode(vm, recv, v as u64, 8);
            write_bytes(vm, recv, int_arg(args, 0), &e[..8])?;
            obj
        }
        ("getShort", "()S") => {
            let b = read_bytes(vm, recv, pos, 2)?;
            put(vm, recv, "bb_pos", Value::Int(pos + 2))?;
            Value::Int(decode(vm, recv, &b) as u16 as i16 as i32)
        }
        ("getShort", "(I)S") => {
            Value::Int(
                decode(vm, recv, &read_bytes(vm, recv, int_arg(args, 0), 2)?) as u16 as i16 as i32,
            )
        }
        ("putShort", "(S)Ljava/nio/ByteBuffer;") => {
            let e = encode(vm, recv, int_arg(args, 0) as u16 as u64, 2);
            write_bytes(vm, recv, pos, &e[..2])?;
            put(vm, recv, "bb_pos", Value::Int(pos + 2))?;
            obj
        }
        ("putShort", "(IS)Ljava/nio/ByteBuffer;") => {
            let e = encode(vm, recv, int_arg(args, 1) as u16 as u64, 2);
            write_bytes(vm, recv, int_arg(args, 0), &e[..2])?;
            obj
        }
        ("getFloat", "()F") => {
            let b = read_bytes(vm, recv, pos, 4)?;
            put(vm, recv, "bb_pos", Value::Int(pos + 4))?;
            Value::Float(f32::from_bits(decode(vm, recv, &b) as u32))
        }
        ("getFloat", "(I)F") => Value::Float(f32::from_bits(decode(
            vm,
            recv,
            &read_bytes(vm, recv, int_arg(args, 0), 4)?,
        ) as u32)),
        ("putFloat", "(F)Ljava/nio/ByteBuffer;") => {
            let f = args.first().and_then(|v| v.as_float().ok()).unwrap_or(0.0);
            let e = encode(vm, recv, f.to_bits() as u64, 4);
            write_bytes(vm, recv, pos, &e[..4])?;
            put(vm, recv, "bb_pos", Value::Int(pos + 4))?;
            obj
        }
        ("putFloat", "(IF)Ljava/nio/ByteBuffer;") => {
            let f = args.get(1).and_then(|v| v.as_float().ok()).unwrap_or(0.0);
            let e = encode(vm, recv, f.to_bits() as u64, 4);
            write_bytes(vm, recv, int_arg(args, 0), &e[..4])?;
            obj
        }
        ("getDouble", "()D") => {
            let b = read_bytes(vm, recv, pos, 8)?;
            put(vm, recv, "bb_pos", Value::Int(pos + 8))?;
            Value::Double(f64::from_bits(decode(vm, recv, &b)))
        }
        ("getDouble", "(I)D") => Value::Double(f64::from_bits(decode(
            vm,
            recv,
            &read_bytes(vm, recv, int_arg(args, 0), 8)?,
        ))),
        ("putDouble", "(D)Ljava/nio/ByteBuffer;") => {
            let f = args.first().and_then(|v| v.as_double().ok()).unwrap_or(0.0);
            let e = encode(vm, recv, f.to_bits(), 8);
            write_bytes(vm, recv, pos, &e[..8])?;
            put(vm, recv, "bb_pos", Value::Int(pos + 8))?;
            obj
        }
        ("putDouble", "(ID)Ljava/nio/ByteBuffer;") => {
            let f = args.get(1).and_then(|v| v.as_double().ok()).unwrap_or(0.0);
            let e = encode(vm, recv, f.to_bits(), 8);
            write_bytes(vm, recv, int_arg(args, 0), &e[..8])?;
            obj
        }
        ("getChar", "()C") => {
            let b = read_bytes(vm, recv, pos, 2)?;
            put(vm, recv, "bb_pos", Value::Int(pos + 2))?;
            Value::Int(decode(vm, recv, &b) as i32)
        }
        ("putChar", "(C)Ljava/nio/ByteBuffer;") => {
            let e = encode(vm, recv, int_arg(args, 0) as u64, 2);
            write_bytes(vm, recv, pos, &e[..2])?;
            put(vm, recv, "bb_pos", Value::Int(pos + 2))?;
            obj
        }
        ("get", "([B)Ljava/nio/ByteBuffer;") | ("get", "([BII)Ljava/nio/ByteBuffer;") => {
            let Some(Value::Obj(dst)) = args.first().cloned() else {
                return Err(exc("Ljava/lang/NullPointerException;", "get(null)"));
            };
            let (doff, n) = if sig.contains("II") {
                (int_arg(args, 1), int_arg(args, 2))
            } else {
                (
                    0,
                    vm.heap
                        .as_array_elems(dst)
                        .map(|(e, _)| e.len())
                        .unwrap_or(0) as i32,
                )
            };
            let data = read_bytes(vm, recv, pos, n.max(0) as usize)?;
            let elems = vm
                .heap
                .array_elems_mut(dst)
                .map_err(|m| exc("Ljava/lang/IndexOutOfBoundsException;", &m))?;
            for (k, b) in data.iter().enumerate() {
                if let Some(slot) = elems.get_mut((doff + k as i32) as usize) {
                    *slot = Value::Int(*b as i8 as i32);
                }
            }
            put(vm, recv, "bb_pos", Value::Int(pos + n))?;
            obj
        }
        ("put", "([B)Ljava/nio/ByteBuffer;")
        | ("put", "([BII)Ljava/nio/ByteBuffer;")
        | ("put", "(Ljava/nio/ByteBuffer;)Ljava/nio/ByteBuffer;") => {
            let data: Vec<u8> = if sig.starts_with("(Ljava") {
                let Some(Value::Obj(src)) = args.first().cloned() else {
                    return Err(exc("Ljava/lang/NullPointerException;", "put(null)"));
                };
                let spos = geti(vm, src, "bb_pos", 0);
                let slim = geti(vm, src, "bb_lim", 0);
                let d = read_bytes(vm, src, spos, (slim - spos).max(0) as usize)?;
                put(vm, src, "bb_pos", Value::Int(slim))?;
                d
            } else {
                let Some(Value::Obj(src)) = args.first().cloned() else {
                    return Err(exc("Ljava/lang/NullPointerException;", "put(null)"));
                };
                let (soff, n) = if sig.contains("II") {
                    (int_arg(args, 1), int_arg(args, 2))
                } else {
                    (
                        0,
                        vm.heap
                            .as_array_elems(src)
                            .map(|(e, _)| e.len())
                            .unwrap_or(0) as i32,
                    )
                };
                let (elems, _) = vm
                    .heap
                    .as_array_elems(src)
                    .map_err(|m| exc("Ljava/lang/IndexOutOfBoundsException;", &m))?;
                elems
                    .iter()
                    .skip(soff.max(0) as usize)
                    .take(n.max(0) as usize)
                    .map(|v| v.as_int().map(|i| i as u8).unwrap_or(0))
                    .collect()
            };
            write_bytes(vm, recv, pos, &data)?;
            put(vm, recv, "bb_pos", Value::Int(pos + data.len() as i32))?;
            obj
        }
        _ => return Ok(None),
    }))
}
