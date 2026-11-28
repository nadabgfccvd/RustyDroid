//! Interpreter over `rd-stub-tables` packed static descriptor tables for
//! `android_stub_surfaces` (Issue #227).

use crate::android_stub_surfaces::{
    alloc_empty_obj, jstr_owned, jstr_str, npe, obj_array, p57_clone_date,
    p57_clone_string_array_or_empty, p57_euicc_send_unavailable, p57_mtp_builder_info,
    p57_mtp_checked_int, p57_mtp_long_to_uint32, put, throw_str, value_string, StubResult,
};
use crate::android_sysservices::{bool_value, farg, get, iarg, larg, opt_str};
use crate::engine::Engine;
use crate::framework::HostObj;
use crate::heap::{ElemKind, ObjRef};
use crate::value::Value;
pub(crate) use rd_stub_tables::{in_class_set, lookup_class_id};
use rd_stub_tables::{
    ALL_CLASS_METHODS, ALL_COPIES, ALL_FMT_SEGS, ALL_METHODS, ALL_NAME_METHODS, ALL_STEPS,
    ALL_STR_IDS, BUILD_SPECS, EQ_SPECS, EX_CLASS_POOL, INT_CONST_POOL, LONG_POOL,
    STATIC_CONST_CLASSES, STATIC_CONST_FIELDS,
};
use std::fmt::Write as _;

#[inline(never)]
pub(crate) fn eval_static_const(
    vm: &mut Engine,
    owner: &str,
    field: &str,
) -> StubResult<Option<Value>> {
    let ob = owner.as_bytes();
    let Ok(cidx) = STATIC_CONST_CLASSES.binary_search_by(|c| c.owner.bytes().cmp(ob)) else {
        return Ok(None);
    };
    let cls = STATIC_CONST_CLASSES[cidx];
    let fb = field.as_bytes();
    let fields = &STATIC_CONST_FIELDS[cls.start as usize..(cls.start + cls.len) as usize];
    let Ok(fidx) = fields.binary_search_by(|f| f.field.bytes().cmp(fb)) else {
        return Ok(None);
    };
    let entry = fields[fidx];
    let v = match entry.kind {
        0 => Value::Int(entry.payload),
        1 => Value::Long(LONG_POOL[entry.payload as usize]),
        2 => Value::Float(f32::from_bits(entry.payload as u32)),
        3 => jstr_str(vm, StrId(entry.payload as u32).as_str())?,
        _ => return Ok(None),
    };
    Ok(Some(v))
}

#[cold]
#[inline(never)]
fn alloc_empty_bundle(vm: &mut Engine) -> StubResult<Value> {
    let r = alloc_empty_obj(vm, "Landroid/os/Bundle;")?;
    vm.fw.objects.insert(r, HostObj::Bundle { map: Vec::new() });
    Ok(Value::Obj(r))
}

#[cold]
#[inline(never)]
fn run_steps(
    vm: &mut Engine,
    recv: ObjRef,
    args: &[Value],
    start: usize,
    len: usize,
) -> StubResult<()> {
    for step in &ALL_STEPS[start..start + len] {
        let idx = step.arg as usize;
        let fname = step.s1.as_str();
        match step.kind {
            1 => {
                if !matches!(args.get(idx), Some(Value::Obj(_))) {
                    return Err(npe(fname));
                }
            }
            2 => {
                if opt_str(vm, args.get(idx)).is_none() {
                    return Err(npe(fname));
                }
            }
            3 => {
                if opt_str(vm, args.get(idx))
                    .as_deref()
                    .map(str::is_empty)
                    .unwrap_or(true)
                {
                    return Err(throw_str("Ljava/lang/IllegalArgumentException;", fname));
                }
            }
            4 => {
                if iarg(args, idx) < step.param {
                    return Err(throw_str("Ljava/lang/IllegalArgumentException;", fname));
                }
            }
            5 => {
                if larg(args, idx) < LONG_POOL[step.param as usize] {
                    return Err(throw_str("Ljava/lang/IllegalArgumentException;", fname));
                }
            }
            6 => {
                if get(vm, recv, fname).as_int().unwrap_or(0) != 0 {
                    return Err(throw_str(
                        "Ljava/lang/IllegalStateException;",
                        StrId(step.param as u32).as_str(),
                    ));
                }
            }
            7 => {
                let v = args.get(idx).cloned().unwrap_or(Value::Null);
                put(vm, recv, fname, v)?;
            }
            8 => put(vm, recv, fname, Value::Int(iarg(args, idx)))?,
            9 => put(vm, recv, fname, bool_value(iarg(args, idx) != 0))?,
            10 => put(vm, recv, fname, Value::Long(larg(args, idx)))?,
            11 => put(vm, recv, fname, Value::Float(farg(args, idx)))?,
            12 => put(vm, recv, fname, Value::Int(step.param))?,
            13 => put(vm, recv, fname, Value::Long(LONG_POOL[step.param as usize]))?,
            14 => put(
                vm,
                recv,
                fname,
                Value::Float(f32::from_bits(step.param as u32)),
            )?,
            15 => put(vm, recv, fname, Value::Null)?,
            16 => {
                let s = jstr_str(vm, StrId(step.param as u32).as_str())?;
                put(vm, recv, fname, s)?;
            }
            17 => {
                let b = alloc_empty_bundle(vm)?;
                put(vm, recv, fname, b)?;
            }
            18 => {
                let b = match args.get(idx) {
                    Some(v @ Value::Obj(_)) => v.clone(),
                    _ => alloc_empty_bundle(vm)?,
                };
                put(vm, recv, fname, b)?;
            }
            19 => {
                let raw = args.get(idx).cloned().unwrap_or(Value::Null);
                let cloned = p57_clone_date(vm, &raw)?;
                put(vm, recv, fname, cloned)?;
            }
            20 => {
                let raw = args.get(idx).cloned().unwrap_or(Value::Null);
                let cloned = p57_clone_string_array_or_empty(vm, &raw)?;
                put(vm, recv, fname, cloned)?;
            }
            21 => {
                let info = p57_mtp_builder_info(vm, recv)?;
                put(vm, info, fname, Value::Int(iarg(args, idx)))?;
            }
            22 => {
                let u = p57_mtp_long_to_uint32(larg(args, idx))?;
                let info = p57_mtp_builder_info(vm, recv)?;
                put(vm, info, fname, Value::Int(u as i32))?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[cold]
#[inline(never)]
fn exec_op(
    vm: &mut Engine,
    op_kind: u8,
    op_aux: u16,
    op_arg: i32,
    recv: Option<ObjRef>,
    args: &[Value],
) -> StubResult<Value> {
    let r = recv.unwrap_or(0);
    let sid = StrId(op_arg as u32);
    let v = match op_kind {
        0 => Value::Null,
        1 => Value::Int(op_arg),
        2 => Value::Long(LONG_POOL[op_arg as usize]),
        3 => Value::Float(f32::from_bits(op_arg as u32)),
        4 => jstr_str(vm, sid.as_str())?,
        5 => get(vm, r, sid.as_str()),
        6 => Value::Int(get(vm, r, sid.as_str()).as_int().unwrap_or(0)),
        7 => match get(vm, r, sid.as_str()) {
            Value::Int(v) => Value::Int(v),
            _ => Value::Int(INT_CONST_POOL[op_aux as usize]),
        },
        8 => bool_value(get(vm, r, sid.as_str()).as_int().unwrap_or(0) != 0),
        9 => Value::Long(get(vm, r, sid.as_str()).as_long().unwrap_or(0)),
        10 => Value::Float(get(vm, r, sid.as_str()).as_float().unwrap_or(0.0)),
        11 => bool_value(
            get(vm, r, sid.as_str()).as_int().unwrap_or(0) == INT_CONST_POOL[op_aux as usize],
        ),
        12 => {
            let len = match get(vm, r, sid.as_str()) {
                Value::Obj(arr_r) => vm
                    .heap
                    .as_array_elems(arr_r)
                    .map(|(e, _)| e.len())
                    .unwrap_or(0),
                _ => 0,
            };
            Value::Int(len as i32)
        }
        13 => crate::arrays::new_array(vm, "[I", ElemKind::Int, Vec::new())?,
        14 => crate::arrays::new_array(vm, "[F", ElemKind::Float, Vec::new())?,
        15 => obj_array(vm, sid.as_str(), Vec::new())?,
        16 => alloc_empty_bundle(vm)?,
        17 => {
            let list_r = alloc_empty_obj(vm, "Ljava/util/ArrayList;")?;
            vm.fw
                .objects
                .insert(list_r, HostObj::List { elems: Vec::new() });
            Value::Obj(list_r)
        }
        18 => {
            run_steps(vm, r, args, op_arg as usize, op_aux as usize)?;
            Value::Null
        }
        19 => {
            run_steps(vm, r, args, op_arg as usize, op_aux as usize)?;
            Value::Obj(r)
        }
        20 => {
            let out = alloc_empty_obj(vm, sid.as_str())?;
            let (c_start, c_len) = BUILD_SPECS[op_aux as usize];
            for &(src_s, dst_s) in &ALL_COPIES[c_start as usize..(c_start + c_len) as usize] {
                let fv = get(vm, r, src_s.as_str());
                put(vm, out, dst_s.as_str(), fv)?;
            }
            Value::Obj(out)
        }
        21 => {
            let exc = EX_CLASS_POOL[op_aux as usize].as_str();
            return Err(throw_str(exc, sid.as_str()));
        }
        22 => {
            let mut h: i32 = 1;
            let s_start = op_arg as usize;
            let s_len = op_aux as usize;
            for &fsid in &ALL_STR_IDS[s_start..s_start + s_len] {
                let fv = get(vm, r, fsid.as_str()).as_int().unwrap_or(0);
                h = h.wrapping_mul(31).wrapping_add(fv);
            }
            Value::Int(h)
        }
        23 => {
            let eq = match args.first() {
                Some(Value::Obj(other))
                    if vm.heap.class_of(*other).unwrap_or("") == sid.as_str() =>
                {
                    let (e_start, e_len) = EQ_SPECS[op_aux as usize];
                    ALL_STR_IDS[e_start as usize..(e_start + e_len) as usize]
                        .iter()
                        .all(|fsid| {
                            let fnm = fsid.as_str();
                            get(vm, r, fnm).as_int().unwrap_or(0)
                                == get(vm, *other, fnm).as_int().unwrap_or(0)
                        })
                }
                _ => false,
            };
            bool_value(eq)
        }
        24 => {
            let fv = get(vm, r, sid.as_str());
            p57_clone_date(vm, &fv)?
        }
        25 => {
            let fv = get(vm, r, sid.as_str());
            p57_clone_string_array_or_empty(vm, &fv)?
        }
        26 => p57_mtp_checked_int(vm, r, sid.as_str())?,
        27 => Value::Long((get(vm, r, sid.as_str()).as_int().unwrap_or(0) as u32) as i64),
        28 => bool_value(matches!(get(vm, r, sid.as_str()), Value::Obj(_))),
        29 => match get(vm, r, sid.as_str()) {
            v @ Value::Obj(_) => v,
            _ => {
                let list_r = alloc_empty_obj(vm, "Ljava/util/ArrayList;")?;
                vm.fw
                    .objects
                    .insert(list_r, HostObj::List { elems: Vec::new() });
                Value::Obj(list_r)
            }
        },
        31 => {
            p57_euicc_send_unavailable(vm, args.get(op_aux as usize))?;
            Value::Null
        }
        32 => {
            let mut out = String::new();
            let f_start = op_arg as usize;
            let f_len = op_aux as usize;
            for seg in &ALL_FMT_SEGS[f_start..f_start + f_len] {
                match seg.kind {
                    0 => out.push_str(seg.s.as_str()),
                    1 => {
                        let _ = write!(out, "{}", get(vm, r, seg.s.as_str()).as_int().unwrap_or(0));
                    }
                    2 => {
                        let _ =
                            write!(out, "{}", get(vm, r, seg.s.as_str()).as_long().unwrap_or(0));
                    }
                    3 => {
                        let _ = write!(
                            out,
                            "{}",
                            get(vm, r, seg.s.as_str()).as_float().unwrap_or(0.0)
                        );
                    }
                    4 => out.push_str(if get(vm, r, seg.s.as_str()).as_int().unwrap_or(0) != 0 {
                        "true"
                    } else {
                        "false"
                    }),
                    5 => {
                        let fv = get(vm, r, seg.s.as_str());
                        match opt_str(vm, Some(&fv)) {
                            Some(s) => out.push_str(&s),
                            None => out.push_str("null"),
                        }
                    }
                    6 => {
                        let fv = get(vm, r, seg.s.as_str());
                        let s = value_string(vm, &fv)?;
                        out.push_str(&s);
                    }
                    _ => {}
                }
            }
            jstr_owned(vm, out)?
        }
        _ => Value::Null,
    };
    Ok(v)
}

#[cold]
#[inline(never)]
pub(crate) fn eval_method_range(
    vm: &mut Engine,
    start: usize,
    end: usize,
    name: &str,
    sig: &str,
    recv: Option<ObjRef>,
    args: &[Value],
) -> StubResult<Option<Value>> {
    let key = (name.as_bytes(), sig.as_bytes());
    let slice = &ALL_METHODS[start..end];
    let Ok(idx) = slice.binary_search_by(|m| (m.name.bytes(), m.sig.bytes()).cmp(&key)) else {
        return Ok(None);
    };
    let m = slice[idx];
    exec_op(vm, m.op_kind, m.op_aux, m.op_arg, recv, args).map(Some)
}

#[cold]
#[inline(never)]
pub(crate) fn eval_name_range(
    vm: &mut Engine,
    start: usize,
    end: usize,
    name: &str,
    recv: Option<ObjRef>,
    args: &[Value],
) -> StubResult<Option<Value>> {
    let nb = name.as_bytes();
    let slice = &ALL_NAME_METHODS[start..end];
    let Ok(idx) = slice.binary_search_by(|m| m.name.bytes().cmp(nb)) else {
        return Ok(None);
    };
    let m = slice[idx];
    exec_op(vm, m.op_kind, m.op_aux, m.op_arg, recv, args).map(Some)
}

#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)] // vm + range + cls + name + sig + recv + args
pub(crate) fn eval_class_method_range(
    vm: &mut Engine,
    start: usize,
    end: usize,
    class: &str,
    name: &str,
    sig: &str,
    recv: Option<ObjRef>,
    args: &[Value],
) -> StubResult<Option<Value>> {
    let key = (class.as_bytes(), name.as_bytes(), sig.as_bytes());
    let slice = &ALL_CLASS_METHODS[start..end];
    let Ok(idx) =
        slice.binary_search_by(|m| (m.cls.bytes(), m.name.bytes(), m.sig.bytes()).cmp(&key))
    else {
        return Ok(None);
    };
    let m = slice[idx];
    exec_op(vm, m.op_kind, m.op_aux, m.op_arg, recv, args).map(Some)
}

