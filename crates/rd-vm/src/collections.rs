//! `java.util.Collections` statics that were missing from the host surface.
//!
//! Covered: `newSetFromMap`, `max`/`min` (± Comparator), `reverse`,
//! `singletonMap`, `emptyIterator`, `addAll(Collection, T...)`,
//! `list(Enumeration)`, and the `IdentityHashMap` hierarchy.
//!
//! These operate on host collection state (`HostObj::List` / `Map` / `Iter`).
//! Element comparison uses the guest `Comparator.compare` or
//! `Comparable.compareTo` (see `arrays::compare`).
//!
//! `newSetFromMap(m)` returns a fresh host `LinkedHashSet` instead of a view
//! backed by `m`. The universal idiom passes an empty, freshly created map,
//! where the two behave the same: a documented deviation.

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::framework::HostObj;
use crate::value::Value;

fn host_elems(vm: &Engine, v: Option<&Value>) -> Option<Vec<Value>> {
    match v {
        Some(Value::Obj(r)) => match vm.fw.objects.get(r) {
            Some(HostObj::List { elems }) | Some(HostObj::Iter { elems, .. }) => {
                Some(elems.clone())
            }
            Some(HostObj::Map { entries }) => {
                Some(entries.iter().map(|(k, _)| k.clone()).collect())
            }
            _ => None,
        },
        _ => None,
    }
}

/// Snapshot of a `java.util.List` seen through the interface: host-modelled
/// lists are copied from `HostObj::List`; any other `List` (a guest class
/// implementing the interface) is read with `size()`/`get(i)`, which is
/// exactly the random-access contract `Collections.binarySearch` assumes.
fn list_snapshot(vm: &mut Engine, list: crate::heap::ObjRef) -> Result<Option<Vec<Value>>, VmExit> {
    if let Some(HostObj::List { elems }) = vm.fw.objects.get(&list) {
        return Ok(Some(elems.clone()));
    }
    let size = vm.invoke_instance(list, "size", "()I", &[])?;
    let Ok(size) = size.as_int() else {
        return Ok(None);
    };
    let mut out = Vec::with_capacity(size.max(0) as usize);
    for i in 0..size {
        out.push(vm.invoke_instance(list, "get", "(I)Ljava/lang/Object;", &[Value::Int(i)])?);
    }
    Ok(Some(out))
}

pub(crate) fn new_list(vm: &mut Engine, class: &str, elems: Vec<Value>) -> Result<Value, VmExit> {
    let r = vm.heap.alloc_instance(class.to_string(), Vec::new())?;
    vm.fw.objects.insert(r, HostObj::List { elems });
    Ok(Value::Obj(r))
}

/// Materialize JDK `Collections.EMPTY_*` fields on first `sget-object`.
/// AndroidX FragmentStore uses EMPTY_LIST as the no-fragments result; a null
/// fallback is not equivalent because callers immediately iterate it.
pub(crate) fn host_static(
    vm: &mut Engine,
    class: &str,
    name: &str,
) -> Result<Option<Value>, VmExit> {
    if class != "Ljava/util/Collections;" {
        return Ok(None);
    }
    let (class, state) = match name {
        "EMPTY_LIST" => (
            "Ljava/util/Collections$SingletonList;",
            HostObj::List { elems: Vec::new() },
        ),
        "EMPTY_SET" | "EMPTY_SORTED_SET" => (
            "Ljava/util/Collections$UnmodifiableSet;",
            HostObj::List { elems: Vec::new() },
        ),
        "EMPTY_MAP" | "EMPTY_SORTED_MAP" => (
            "Ljava/util/Collections$UnmodifiableMap;",
            HostObj::Map {
                entries: Vec::new(),
            },
        ),
        _ => return Ok(None),
    };
    let r = vm.heap.alloc_instance(class.to_string(), Vec::new())?;
    vm.fw.objects.insert(r, state);
    Ok(Some(Value::Obj(r)))
}

/// `Collections.reverseOrder()` / `reverseOrder(Comparator)` marker classes.
/// The ordering lives here, not in a fake DEX class: `compare` swaps the
/// operands (natural order) or negates the wrapped comparator's answer.
pub(crate) const REVERSE_COMPARATOR: &str = "Ljava/util/Collections$ReverseComparator;";
pub(crate) const REVERSE_COMPARATOR2: &str = "Ljava/util/Collections$ReverseComparator2;";

/// The marker instance for `reverseOrder()` lives in `Engine::statics`, so
/// `Collections.reverseOrder() == Collections.reverseOrder()` (JDK identity).
fn reverse_comparator(
    vm: &mut Engine,
    class: &str,
    wrapped: Option<Value>,
) -> Result<Value, VmExit> {
    let cacheable = wrapped.is_none();
    let key = (class.to_string(), "INSTANCE".to_string());
    if cacheable {
        if let Some(v) = vm.statics.get(&key) {
            return Ok(v.clone());
        }
    }
    let r = vm.heap.alloc_instance(class.to_string(), Vec::new())?;
    if let Some(w) = wrapped {
        let _ = vm.heap.put_field(r, "rd_cmp", w);
    }
    let v = Value::Obj(r);
    if cacheable {
        vm.statics.insert(key, v.clone());
    }
    Ok(v)
}

/// Instance surface of the two reverse comparators.
pub(crate) fn instance_intrinsic(
    vm: &mut Engine,
    recv_class: &str,
    name: &str,
    sig: &str,
    recv: crate::heap::ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if recv_class != REVERSE_COMPARATOR && recv_class != REVERSE_COMPARATOR2 {
        return Ok(None);
    }
    match (name, sig) {
        ("compare", "(Ljava/lang/Object;Ljava/lang/Object;)I") => {
            let a = args.first().cloned().unwrap_or(Value::Null);
            let b = args.get(1).cloned().unwrap_or(Value::Null);
            let wrapped = match vm.heap.get_field(recv, "rd_cmp") {
                Ok(v @ Value::Obj(_)) => Some(v),
                _ => None,
            };
            // Reversed order: compare(b, a), through the wrapped comparator
            // when there is one (Collections.reverseOrder(cmp)).
            let c = crate::arrays::compare(vm, wrapped.as_ref(), &b, &a)?;
            Ok(Some(Value::Int(c)))
        }
        ("equals", "(Ljava/lang/Object;)Z") => Ok(Some(Value::Int(0))),
        ("hashCode", "()I") => Ok(Some(Value::Int(1))),
        _ => Ok(None),
    }
}

pub(crate) fn static_intrinsic(
    vm: &mut Engine,
    class: &str,
    name: &str,
    sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if class != "Ljava/util/Collections;" {
        return Ok(None);
    }
    match (name, sig) {
        ("newSetFromMap", "(Ljava/util/Map;)Ljava/util/Set;") => {
            let init = host_elems(vm, args.first()).unwrap_or_default();
            Ok(Some(new_list(vm, "Ljava/util/LinkedHashSet;", init)?))
        }
        ("max", _) | ("min", _) => {
            let Some(items) = host_elems(vm, args.first()) else {
                return Ok(None);
            };
            if items.is_empty() {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/util/NoSuchElementException;",
                    String::new(),
                )));
            }
            let cmp = args.get(1).filter(|v| !matches!(v, Value::Null)).cloned();
            let mut best = items[0].clone();
            for it in &items[1..] {
                let c = crate::arrays::compare(vm, cmp.as_ref(), it, &best)?;
                if (name == "max" && c > 0) || (name == "min" && c < 0) {
                    best = it.clone();
                }
            }
            Ok(Some(best))
        }
        // Stable merge sort (the JDK's TimSort is stable too); a null
        // comparator means natural ordering.
        ("sort", "(Ljava/util/List;)V") | ("sort", "(Ljava/util/List;Ljava/util/Comparator;)V") => {
            let Some(Value::Obj(r)) = args.first() else {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/lang/NullPointerException;",
                    "Collections.sort(null)",
                )));
            };
            let Some(HostObj::List { elems }) = vm.fw.objects.get(r) else {
                return Ok(None);
            };
            let items = elems.clone();
            let cmp = args.get(1).filter(|v| !matches!(v, Value::Null)).cloned();
            let sorted = crate::arrays::merge_sort(vm, items, cmp.as_ref())?;
            if let Some(HostObj::List { elems }) = vm.fw.objects.get_mut(r) {
                *elems = sorted;
            }
            Ok(Some(Value::Null))
        }
        // `Collections.binarySearch(List, key)` / `(List, key, Comparator)` —
        // gap real do launch do NewPipe (`Collections.binarySearch` sobre a
        // lista de chaves de preferência). Contrato do JDK: a lista precisa
        // estar ordenada; o retorno é o índice do elemento achado ou
        // `-(ponto de inserção) - 1` quando não existe. A comparação é
        // `midVal` contra `key` (o mesmo sentido do `Arrays.binarySearch` em
        // `arrays.rs`), usando `Comparable.compareTo` do guest ou o
        // `Comparator.compare` quando o overload com comparador é chamado.
        // A leitura passa pela interface `List` (`size`/`get`): serve tanto
        // para a lista de host (ArrayList/`List` imutável) quanto para uma
        // `List` implementada em DEX.
        ("binarySearch", "(Ljava/util/List;Ljava/lang/Object;)I")
        | ("binarySearch", "(Ljava/util/List;Ljava/lang/Object;Ljava/util/Comparator;)I") => {
            let Some(Value::Obj(list)) = args.first().cloned() else {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/lang/NullPointerException;",
                    "Collections.binarySearch com lista null",
                )));
            };
            let key = args.get(1).cloned().unwrap_or(Value::Null);
            let cmp = args.get(2).filter(|v| !matches!(v, Value::Null)).cloned();
            let Some(items) = list_snapshot(vm, list)? else {
                return Ok(None);
            };
            let (mut lo, mut hi) = (0i32, items.len() as i32 - 1);
            while lo <= hi {
                let mid = ((lo + hi) as u32 >> 1) as i32;
                let c = crate::arrays::compare(vm, cmp.as_ref(), &items[mid as usize], &key)?;
                if c < 0 {
                    lo = mid + 1;
                } else if c > 0 {
                    hi = mid - 1;
                } else {
                    return Ok(Some(Value::Int(mid)));
                }
            }
            Ok(Some(Value::Int(-(lo + 1))))
        }
        ("reverse", "(Ljava/util/List;)V") => {
            if let Some(Value::Obj(r)) = args.first() {
                if let Some(HostObj::List { elems }) = vm.fw.objects.get_mut(r) {
                    elems.reverse();
                    return Ok(Some(Value::Null));
                }
            }
            Ok(None)
        }
        ("singletonMap", "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/util/Map;") => {
            let r = vm
                .heap
                .alloc_instance("Ljava/util/HashMap;".to_string(), Vec::new())?;
            let e = vec![(args[0].clone(), args[1].clone())];
            vm.fw.objects.insert(r, HostObj::Map { entries: e });
            Ok(Some(Value::Obj(r)))
        }
        ("emptyIterator", "()Ljava/util/Iterator;") => {
            let r = vm
                .heap
                .alloc_instance("Ljava/util/Iterator;".to_string(), Vec::new())?;
            vm.fw.objects.insert(
                r,
                HostObj::Iter {
                    elems: Vec::new(),
                    pos: 0,
                },
            );
            Ok(Some(Value::Obj(r)))
        }
        ("list", "(Ljava/util/Enumeration;)Ljava/util/ArrayList;") => {
            let Some(items) = host_elems(vm, args.first()) else {
                return Ok(None);
            };
            Ok(Some(new_list(vm, "Ljava/util/ArrayList;", items)?))
        }
        ("addAll", "(Ljava/util/Collection;[Ljava/lang/Object;)Z") => {
            let (Some(Value::Obj(c)), Some(Value::Obj(a))) = (args.first(), args.get(1)) else {
                return Ok(None);
            };
            let add: Vec<Value> = vm.heap.as_array_elems(*a)?.0.to_vec();
            let is_set = vm
                .heap
                .class_of(*c)
                .map(|k| k.contains("Set"))
                .unwrap_or(false);
            let heap_snapshot: Vec<Value>;
            let mut changed = false;
            if let Some(HostObj::List { elems }) = vm.fw.objects.get(c) {
                heap_snapshot = elems.clone();
            } else {
                return Ok(None);
            }
            let mut out = heap_snapshot;
            for v in add {
                if is_set
                    && out
                        .iter()
                        .any(|x| crate::intrinsics::values_equal(&vm.heap, x, &v).unwrap_or(false))
                {
                    continue;
                }
                out.push(v);
                changed = true;
            }
            vm.fw.objects.insert(*c, HostObj::List { elems: out });
            Ok(Some(Value::Int(changed as i32)))
        }
        // `Collections.reverseOrder()` — sort descending without a lambda or a
        // hand-written Comparator class: the marker answers `compare` with the
        // operands swapped.
        ("reverseOrder", "()Ljava/util/Comparator;") => {
            Ok(Some(reverse_comparator(vm, REVERSE_COMPARATOR, None)?))
        }
        ("reverseOrder", "(Ljava/util/Comparator;)Ljava/util/Comparator;") => {
            let inner = args.first().cloned().unwrap_or(Value::Null);
            Ok(Some(reverse_comparator(
                vm,
                REVERSE_COMPARATOR2,
                Some(inner),
            )?))
        }
        // `nCopies(n, o)` — immutable list of n copies (JDK: CopiesList). The
        // immutable marker class gives the JDK's UnsupportedOperationException
        // on mutation.
        ("nCopies", "(ILjava/lang/Object;)Ljava/util/List;") => {
            let n = match args.first() {
                Some(Value::Int(n)) => *n,
                _ => 0,
            };
            if n < 0 {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/lang/IllegalArgumentException;",
                    format!("List length is negative: {n}"),
                )));
            }
            let elem = args.get(1).cloned().unwrap_or(Value::Null);
            let elems = vec![elem; n as usize];
            Ok(Some(new_list(
                vm,
                crate::intrinsics::SINGLETON_LIST,
                elems,
            )?))
        }
        // `frequency(c, o)` — number of elements equal to o.
        ("frequency", "(Ljava/util/Collection;Ljava/lang/Object;)I") => {
            let Some(items) = host_elems(vm, args.first()) else {
                return Ok(None);
            };
            let needle = args.get(1).cloned().unwrap_or(Value::Null);
            let n = items
                .iter()
                .filter(|v| crate::intrinsics::values_equal(&vm.heap, v, &needle).unwrap_or(false))
                .count();
            Ok(Some(Value::Int(n as i32)))
        }
        // `disjoint(a, b)` — true when the collections share no element.
        ("disjoint", "(Ljava/util/Collection;Ljava/util/Collection;)Z") => {
            let (Some(a), Some(b)) = (host_elems(vm, args.first()), host_elems(vm, args.get(1)))
            else {
                return Ok(None);
            };
            let common = a.iter().any(|x| {
                b.iter()
                    .any(|y| crate::intrinsics::values_equal(&vm.heap, x, y).unwrap_or(false))
            });
            Ok(Some(Value::Int(!common as i32)))
        }
        // In-place List operations. They need the host list state to write
        // back; a guest List implementation keeps falling through to its DEX
        // class (honest gap instead of a silent no-op).
        ("fill", "(Ljava/util/List;Ljava/lang/Object;)V") => {
            let Some(Value::Obj(list)) = args.first().cloned() else {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/lang/NullPointerException;",
                    "Collections.fill(null, obj)",
                )));
            };
            let obj = args.get(1).cloned().unwrap_or(Value::Null);
            let Some(HostObj::List { elems }) = vm.fw.objects.get_mut(&list) else {
                return Ok(None);
            };
            for e in elems.iter_mut() {
                *e = obj.clone();
            }
            Ok(Some(Value::Null))
        }
        ("swap", "(Ljava/util/List;II)V") => {
            let Some(Value::Obj(list)) = args.first().cloned() else {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/lang/NullPointerException;",
                    "Collections.swap(null, i, j)",
                )));
            };
            let i = args.get(1).and_then(|v| v.as_int().ok());
            let j = args.get(2).and_then(|v| v.as_int().ok());
            let (Some(i), Some(j)) = (i, j) else {
                return Ok(None);
            };
            let Some(HostObj::List { elems }) = vm.fw.objects.get_mut(&list) else {
                return Ok(None);
            };
            let len = elems.len() as i32;
            if i < 0 || j < 0 || i >= len || j >= len {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/lang/IndexOutOfBoundsException;",
                    format!("Index: {i}, Size: {len}"),
                )));
            }
            elems.swap(i as usize, j as usize);
            Ok(Some(Value::Null))
        }
        ("rotate", "(Ljava/util/List;I)V") => {
            let Some(Value::Obj(list)) = args.first().cloned() else {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/lang/NullPointerException;",
                    "Collections.rotate(null, d)",
                )));
            };
            let Some(dist) = args.get(1).and_then(|v| v.as_int().ok()) else {
                return Ok(None);
            };
            let Some(HostObj::List { elems }) = vm.fw.objects.get_mut(&list) else {
                return Ok(None);
            };
            let len = elems.len() as i64;
            if len > 0 {
                let shift = (dist as i64).rem_euclid(len) as usize;
                elems.rotate_right(shift);
            }
            Ok(Some(Value::Null))
        }
        ("copy", "(Ljava/util/List;Ljava/util/List;)V") => {
            let Some(Value::Obj(dest)) = args.first().cloned() else {
                return Ok(None);
            };
            let Some(src) = host_elems(vm, args.get(1)) else {
                return Ok(None);
            };
            let Some(HostObj::List { elems }) = vm.fw.objects.get(&dest) else {
                return Ok(None);
            };
            if src.len() > elems.len() {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/lang/IndexOutOfBoundsException;",
                    format!(
                        "Source does not fit in dest: {} > {}",
                        src.len(),
                        elems.len()
                    ),
                )));
            }
            if let Some(HostObj::List { elems }) = vm.fw.objects.get_mut(&dest) {
                elems[..src.len()].clone_from_slice(&src);
            }
            Ok(Some(Value::Null))
        }
        ("replaceAll", "(Ljava/util/List;Ljava/lang/Object;Ljava/lang/Object;)Z") => {
            let Some(Value::Obj(list)) = args.first().cloned() else {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/lang/NullPointerException;",
                    "Collections.replaceAll(null, old, new)",
                )));
            };
            let old = args.get(1).cloned().unwrap_or(Value::Null);
            let new = args.get(2).cloned().unwrap_or(Value::Null);
            let Some(HostObj::List { elems }) = vm.fw.objects.get(&list) else {
                return Ok(None);
            };
            let hits: Vec<usize> = elems
                .iter()
                .enumerate()
                .filter(|(_, v)| {
                    crate::intrinsics::values_equal(&vm.heap, v, &old).unwrap_or(false)
                })
                .map(|(i, _)| i)
                .collect();
            if let Some(HostObj::List { elems }) = vm.fw.objects.get_mut(&list) {
                for i in &hits {
                    elems[*i] = new.clone();
                }
            }
            Ok(Some(Value::Int(!hits.is_empty() as i32)))
        }
        _ => Ok(None),
    }
}
