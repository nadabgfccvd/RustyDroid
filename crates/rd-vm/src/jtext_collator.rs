//! `java.text.Collator` and `CollationKey`.
//!
//! F-Droid sorts app names, categories and repository indexes with
//! `Collator.getInstance(Locale)`, and the sort has to be *locale aware*:
//! `"Ábaco"` must group with `"abaco"`, not after `"Z"`. The host keeps the
//! AOSP/ICU root collation order at the three comparison levels:
//!
//! 1. **primary** — base letter (accents and case removed): `á == a`
//! 2. **secondary** — accent presence: `a < á`
//! 3. **tertiary** — case: `a < A` (ICU root orders lowercase first)
//!
//! Strings that tie at all three levels compare equal — Java's `Collator` is
//! deliberately not a total order at reduced strength. `IDENTICAL` adds the
//! code-point comparison as a fourth level.
//!
//! `Collator.setStrength` selects how many levels participate
//! (`PRIMARY`/`SECONDARY`/`TERTIARY`/`IDENTICAL`), which is what
//! `Collator.getInstance()` consumers that only want case-insensitive grouping
//! rely on.
//!
//! `CollationKey` is precomputed: `toByteArray` returns the three level keys as
//! big-endian `int`s, and `compareTo` replays the same comparison on them.
//!
//! **Deviation:** the accent table covers Latin-1 Supplement and Latin
//! Extended-A (the repertoire of app names in the wild). Characters outside it
//! keep their own code point as primary weight, which sorts them after the
//! Latin range — the ICU root ordering for unlisted scripts is not modelled.

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::heap::{ElemKind, ObjRef};
use crate::value::Value;

const COLLATOR: &str = "Ljava/text/Collator;";
const COLLATION_KEY: &str = "Ljava/text/CollationKey;";
const LOCALE: &str = "Ljava/util/Locale;";

/// `Collator.PRIMARY` / `SECONDARY` / `TERTIARY` / `IDENTICAL`.
const PRIMARY: i32 = 0;
const SECONDARY: i32 = 1;
const TERTIARY: i32 = 2;
const IDENTICAL: i32 = 3;

fn exc(class: &str, msg: impl Into<String>) -> VmExit {
    VmExit::Exception(Throwable::new(class, msg))
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
fn alloc(vm: &mut Engine, class: &str, fields: Vec<(&str, Value)>) -> Result<ObjRef, VmExit> {
    let fields: Vec<(String, Value)> = fields
        .into_iter()
        .map(|(n, v)| (n.to_string(), v))
        .collect();
    vm.heap
        .alloc_instance(class.to_string(), fields)
        .map_err(crate::err::oom_throwable)
}
fn get_int(vm: &Engine, r: ObjRef, f: &str, d: i32) -> i32 {
    vm.heap
        .get_field(r, f)
        .ok()
        .and_then(|v| v.as_int().ok())
        .unwrap_or(d)
}
fn put(vm: &mut Engine, r: ObjRef, f: &str, v: Value) -> Result<(), VmExit> {
    crate::err::put_field_result(vm.heap.put_field(r, f, v))
}

/// Base letter for a Latin character with diacritics.
fn base_letter(c: char) -> char {
    let u = c as u32;
    // Latin-1 Supplement (U+00C0..U+00FF) and Latin Extended-A (U+0100..U+017F).
    match u {
        0x00C0..=0x00C5 => 'a',
        0x00C7 => 'c',
        0x00C8..=0x00CB => 'e',
        0x00CC..=0x00CF => 'i',
        0x00D1 => 'n',
        0x00D2..=0x00D6 | 0x00D8 => 'o',
        0x00D9..=0x00DC => 'u',
        0x00DD => 'y',
        0x00E0..=0x00E5 => 'a',
        0x00E7 => 'c',
        0x00E8..=0x00EB => 'e',
        0x00EC..=0x00EF => 'i',
        0x00F1 => 'n',
        0x00F2..=0x00F6 | 0x00F8 => 'o',
        0x00F9..=0x00FC => 'u',
        0x00FD | 0x00FF => 'y',
        0x0100..=0x0105 => 'a',
        0x0106..=0x010D => 'c',
        0x010E..=0x0111 => 'd',
        0x0112..=0x011B => 'e',
        0x011C..=0x0123 => 'g',
        0x0124..=0x0127 => 'h',
        0x0128..=0x0131 => 'i',
        0x0134..=0x0135 => 'j',
        0x0136..=0x0138 => 'k',
        0x0139..=0x013E => 'l',
        0x0141..=0x0142 => 'l',
        0x0143..=0x0148 => 'n',
        0x014C..=0x0151 => 'o',
        0x0154..=0x0159 => 'r',
        0x015A..=0x0161 => 's',
        0x0162..=0x0167 => 't',
        0x0168..=0x0173 => 'u',
        0x0174..=0x0175 => 'w',
        0x0176..=0x0178 => 'y',
        0x0179..=0x017E => 'z',
        _ => c,
    }
}

/// Whether the character carries a diacritic (secondary weight > 0).
fn has_diacritic(c: char) -> bool {
    base_letter(c) != c
}

/// One collation element: `(primary, secondary, tertiary)`.
fn element(c: char) -> (u32, u32, u32) {
    let base = base_letter(c);
    let primary = base.to_lowercase().next().unwrap_or(base) as u32;
    let secondary = u32::from(has_diacritic(c));
    let tertiary = u32::from(base.is_uppercase());
    (primary, secondary, tertiary)
}

fn levels(s: &str) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let mut p = Vec::with_capacity(s.len());
    let mut sec = Vec::with_capacity(s.len());
    let mut ter = Vec::with_capacity(s.len());
    for c in s.chars() {
        let (a, b, c3) = element(c);
        p.push(a);
        sec.push(b);
        ter.push(c3);
    }
    (p, sec, ter)
}

/// `Collator.compare`, honouring the strength.
fn compare_with_strength(a: &str, b: &str, strength: i32) -> i32 {
    if will_strength_ios(a, b, strength) {
        return 0;
    }
    let (pa, sa, ta) = levels(a);
    let (pb, sb, tb) = levels(b);
    match pa.cmp(&pb) {
        std::cmp::Ordering::Less => return -1,
        std::cmp::Ordering::Greater => return 1,
        std::cmp::Ordering::Equal => {}
    }
    if strength >= SECONDARY {
        match sa.cmp(&sb) {
            std::cmp::Ordering::Less => return -1,
            std::cmp::Ordering::Greater => return 1,
            std::cmp::Ordering::Equal => {}
        }
    }
    if strength >= TERTIARY {
        match ta.cmp(&tb) {
            std::cmp::Ordering::Less => return -1,
            std::cmp::Ordering::Greater => return 1,
            std::cmp::Ordering::Equal => {}
        }
    }
    // `IDENTICAL` adds the code-point comparison; every lower strength stops
    // here and reports equality — `Collator.compare` is *not* a total order
    // below IDENTICAL (two distinct strings can compare equal).
    if strength >= IDENTICAL {
        return match a.cmp(b) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Equal => 0,
        };
    }
    0
}

/// Primary-level equality shortcut (avoids building the level vectors twice
/// for the common "same string" case in `equals` comparisons).
fn will_strength_ios(a: &str, b: &str, _strength: i32) -> bool {
    a == b
}

fn key_bytes(s: &str) -> Vec<u8> {
    let (p, sec, ter) = levels(s);
    let mut out = Vec::with_capacity(12 + (p.len() + sec.len() + ter.len()) * 2);
    for v in &p {
        out.extend_from_slice(&(*v as u16).to_be_bytes());
    }
    out.push(0);
    for v in &sec {
        out.extend_from_slice(&(*v as u16).to_be_bytes());
    }
    out.push(0);
    for v in &ter {
        out.extend_from_slice(&(*v as u16).to_be_bytes());
    }
    out
}

fn new_collator(vm: &mut Engine) -> Result<ObjRef, VmExit> {
    alloc(
        vm,
        COLLATOR,
        vec![
            ("rd_strength", Value::Int(TERTIARY)),
            ("rd_decomposition", Value::Int(0)),
        ],
    )
}

fn new_key(vm: &mut Engine, source: &str) -> Result<ObjRef, VmExit> {
    let bytes = key_bytes(source);
    let arr = vm
        .heap
        .alloc_array_zeroed(ElemKind::Int, bytes.len(), "[B")
        .map_err(crate::err::oom_throwable)?;
    {
        let elems = vm.heap.array_elems_mut(arr)?;
        for (slot, b) in elems.iter_mut().zip(&bytes) {
            *slot = Value::Int(i32::from(*b));
        }
    }
    let source_val = jstr(vm, source)?;
    alloc(
        vm,
        COLLATION_KEY,
        vec![
            ("rd_source", source_val),
            ("rd_key", Value::Obj(arr)),
            ("rd_text", Value::Null),
        ],
    )
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
    if class != COLLATOR {
        return Ok(None);
    }
    Ok(Some(match name {
        "getInstance" => {
            // The locale only selects the rule set; the root rules above apply
            // to every locale the VM reports.
            match args.first() {
                None | Some(Value::Null) => Value::Obj(new_collator(vm)?),
                Some(Value::Obj(_)) => Value::Obj(new_collator(vm)?),
                Some(_) => return Err(exc("Ljava/lang/IllegalArgumentException;", "locale")),
            }
        }
        "getAvailableLocales" => {
            let r = vm
                .heap
                .alloc_array(ElemKind::Obj, 0, LOCALE)
                .map_err(crate::err::oom_throwable)?;
            Value::Obj(r)
        }
        _ => return Ok(None),
    }))
}

fn collator_instance(
    vm: &mut Engine,
    name: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let strength = get_int(vm, recv, "rd_strength", TERTIARY);
    Ok(Some(match name {
        "<init>" => Value::Null,
        "compare" => {
            let Some(a) = sval(vm, args.first()) else {
                return Err(exc(
                    "Ljava/lang/NullPointerException;",
                    "compare: source is null",
                ));
            };
            let Some(b) = sval(vm, args.get(1)) else {
                return Err(exc(
                    "Ljava/lang/NullPointerException;",
                    "compare: target is null",
                ));
            };
            Value::Int(compare_with_strength(&a, &b, strength))
        }
        "equals" => {
            // `Collator.equals(Object)` compares rules and strength, not the
            // collator's identity: two instances with the same strength are
            // interchangeable.
            Value::Int(i32::from(match args.first() {
                Some(Value::Obj(o)) => {
                    vm.heap.class_of(*o).ok() == Some(COLLATOR)
                        && get_int(vm, *o, "rd_strength", TERTIARY) == strength
                }
                _ => false,
            }))
        }
        "hashCode" => Value::Int(31 * strength + 1),
        "setStrength" | "setDecomposition" => {
            let field = if name == "setStrength" {
                "rd_strength"
            } else {
                "rd_decomposition"
            };
            let v = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
            if name == "setStrength" && !(PRIMARY..=IDENTICAL).contains(&v) {
                return Err(exc(
                    "Ljava/lang/IllegalArgumentException;",
                    format!("wrong strength value: {v}"),
                ));
            }
            put(vm, recv, field, Value::Int(v))?;
            Value::Null
        }
        "getStrength" => Value::Int(strength),
        "getDecomposition" => Value::Int(get_int(vm, recv, "rd_decomposition", 0)),
        "getCollationKey" => {
            let Some(s) = sval(vm, args.first()) else {
                return Err(exc(
                    "Ljava/lang/NullPointerException;",
                    "getCollationKey: source is null",
                ));
            };
            Value::Obj(new_key(vm, &s)?)
        }
        "clone" => Value::Obj(new_collator(vm)?),
        "toString" => jstr(
            vm,
            match strength {
                PRIMARY => "Collator: PRIMARY",
                SECONDARY => "Collator: SECONDARY",
                IDENTICAL => "Collator: IDENTICAL",
                _ => "Collator: TERTIARY",
            },
        )?,
        _ => return Ok(None),
    }))
}

fn key_instance(
    vm: &mut Engine,
    name: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let bytes = match vm.heap.get_field(recv, "rd_key") {
        Ok(Value::Obj(a)) => {
            let (elems, _) = vm.heap.as_array_elems(a)?;
            elems
                .iter()
                .map(|v| v.as_int().unwrap_or(0) as u8)
                .collect::<Vec<u8>>()
        }
        _ => Vec::new(),
    };
    Ok(Some(match name {
        "compareTo" => {
            let other = match args.first() {
                Some(Value::Obj(o)) => match vm.heap.get_field(*o, "rd_key") {
                    Ok(Value::Obj(a)) => {
                        let (elems, _) = vm.heap.as_array_elems(a)?;
                        elems
                            .iter()
                            .map(|v| v.as_int().unwrap_or(0) as u8)
                            .collect::<Vec<u8>>()
                    }
                    _ => Vec::new(),
                },
                _ => return Err(exc("Ljava/lang/ClassCastException;", "not a CollationKey")),
            };
            Value::Int(match bytes.cmp(&other) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Greater => 1,
                std::cmp::Ordering::Equal => 0,
            })
        }
        "toByteArray" => {
            let arr = vm
                .heap
                .alloc_array_zeroed(ElemKind::Int, bytes.len(), "[B")
                .map_err(crate::err::oom_throwable)?;
            {
                let elems = vm.heap.array_elems_mut(arr)?;
                for (slot, b) in elems.iter_mut().zip(&bytes) {
                    *slot = Value::Int(i32::from(*b));
                }
            }
            Value::Obj(arr)
        }
        "getSourceString" => vm.heap.get_field(recv, "rd_source").unwrap_or(Value::Null),
        "hashCode" => {
            let mut h: i32 = 0;
            for b in bytes.iter().take(4) {
                h = h.wrapping_mul(31).wrapping_add(i32::from(*b));
            }
            Value::Int(h)
        }
        "equals" => Value::Int(i32::from(match args.first() {
            Some(Value::Obj(o)) => match vm.heap.get_field(*o, "rd_key") {
                Ok(Value::Obj(a)) => {
                    let (elems, _) = vm.heap.as_array_elems(a)?;
                    let other: Vec<u8> = elems
                        .iter()
                        .map(|v| v.as_int().unwrap_or(0) as u8)
                        .collect();
                    other == bytes
                }
                _ => false,
            },
            _ => false,
        })),
        "toString" => jstr(
            vm,
            sval(
                vm,
                Some(&vm.heap.get_field(recv, "rd_source").unwrap_or(Value::Null)),
            )
            .unwrap_or_default(),
        )?,
        _ => return Ok(None),
    }))
}

pub(crate) fn instance_intrinsic(
    vm: &mut Engine,
    recv_class: &str,
    name: &str,
    _sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match recv_class {
        COLLATOR => collator_instance(vm, name, recv, args),
        COLLATION_KEY => key_instance(vm, name, recv, args),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::VmConfig;

    fn collator() -> (Engine, ObjRef) {
        let mut vm = Engine::new(Vec::new(), VmConfig::default());
        let c = match static_intrinsic(
            &mut vm,
            COLLATOR,
            "getInstance",
            "()Ljava/text/Collator;",
            &[],
        )
        .unwrap()
        {
            Some(Value::Obj(o)) => o,
            other => panic!("getInstance returned {other:?}"),
        };
        (vm, c)
    }

    fn collation_key(vm: &mut Engine, c: ObjRef, s: &str) -> ObjRef {
        let arg = jstr(vm, s).unwrap();
        match instance_intrinsic(
            vm,
            COLLATOR,
            "getCollationKey",
            "(Ljava/lang/String;)Ljava/text/CollationKey;",
            c,
            &[arg],
        )
        .unwrap()
        {
            Some(Value::Obj(k)) => k,
            other => panic!("getCollationKey returned {other:?}"),
        }
    }

    fn cmp(vm: &mut Engine, c: ObjRef, a: &str, b: &str) -> i32 {
        let av = jstr(vm, a).unwrap();
        let bv = jstr(vm, b).unwrap();
        match instance_intrinsic(
            vm,
            COLLATOR,
            "compare",
            "(Ljava/lang/Object;Ljava/lang/Object;)I",
            c,
            &[av, bv],
        )
        .unwrap()
        {
            Some(Value::Int(i)) => i,
            other => panic!("compare returned {other:?}"),
        }
    }

    #[test]
    fn accents_sort_with_their_base_letter() {
        let (mut vm, c) = collator();
        assert!(cmp(&mut vm, c, "Ábaco", "Zebra") < 0, "Á groups under A");
        assert!(cmp(&mut vm, c, "abaco", "Ábaco") < 0, "plain a before á");
        assert!(cmp(&mut vm, c, "Ábaco", "abaco") > 0);
        // Case is the last level: same letters, lowercase first.
        assert!(cmp(&mut vm, c, "apple", "Apple") < 0);
        assert!(cmp(&mut vm, c, "Apple", "apricot") < 0);
    }

    #[test]
    fn strength_controls_the_levels_used() {
        let (mut vm, c) = collator();
        // PRIMARY: letters only — case and accents do not matter.
        let v = instance_intrinsic(
            &mut vm,
            COLLATOR,
            "setStrength",
            "(I)V",
            c,
            &[Value::Int(PRIMARY)],
        )
        .unwrap();
        assert!(v.is_some());
        assert_eq!(cmp(&mut vm, c, "abaco", "Ábaco"), 0);
        // SECONDARY: accents count, case does not.
        instance_intrinsic(
            &mut vm,
            COLLATOR,
            "setStrength",
            "(I)V",
            c,
            &[Value::Int(SECONDARY)],
        )
        .unwrap();
        assert!(cmp(&mut vm, c, "abaco", "Ábaco") < 0);
        assert_eq!(cmp(&mut vm, c, "abaco", "Abaco"), 0);
    }

    #[test]
    fn strengths_are_not_interchangeable_and_reject_out_of_range() {
        let (mut vm, c) = collator();
        let other = match static_intrinsic(
            &mut vm,
            COLLATOR,
            "getInstance",
            "()Ljava/text/Collator;",
            &[],
        )
        .unwrap()
        {
            Some(Value::Obj(o)) => o,
            other => panic!("getInstance returned {other:?}"),
        };
        let same = match instance_intrinsic(
            &mut vm,
            COLLATOR,
            "equals",
            "(Ljava/lang/Object;)Z",
            c,
            &[Value::Obj(other)],
        )
        .unwrap()
        {
            Some(Value::Int(i)) => i,
            other => panic!("equals returned {other:?}"),
        };
        assert_eq!(same, 1, "same strength ⇒ equal collators");
        instance_intrinsic(
            &mut vm,
            COLLATOR,
            "setStrength",
            "(I)V",
            other,
            &[Value::Int(PRIMARY)],
        )
        .unwrap();
        let diff = match instance_intrinsic(
            &mut vm,
            COLLATOR,
            "equals",
            "(Ljava/lang/Object;)Z",
            c,
            &[Value::Obj(other)],
        )
        .unwrap()
        {
            Some(Value::Int(i)) => i,
            other => panic!("equals returned {other:?}"),
        };
        assert_eq!(diff, 0);
        assert!(instance_intrinsic(
            &mut vm,
            COLLATOR,
            "setStrength",
            "(I)V",
            c,
            &[Value::Int(9)]
        )
        .is_err());
    }

    #[test]
    fn collation_keys_compare_like_the_collator() {
        let (mut vm, c) = collator();
        let ka = collation_key(&mut vm, c, "ábaco");
        let kb = collation_key(&mut vm, c, "zebra");
        let r = instance_intrinsic(
            &mut vm,
            COLLATION_KEY,
            "compareTo",
            "(Ljava/lang/Object;)I",
            ka,
            &[Value::Obj(kb)],
        )
        .unwrap();
        assert!(matches!(r, Some(Value::Int(i)) if i < 0));
        let src = instance_intrinsic(
            &mut vm,
            COLLATION_KEY,
            "getSourceString",
            "()Ljava/lang/String;",
            ka,
            &[],
        )
        .unwrap();
        assert_eq!(sval(&vm, src.as_ref()), Some("ábaco".to_string()));
    }
}
