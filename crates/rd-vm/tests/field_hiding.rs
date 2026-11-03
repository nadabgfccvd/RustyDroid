//! Java field hiding (JLS 8.3) on the VM heap.
//!
//! The heap used to key instance fields by NAME only. A subclass that declares
//! a field with the same name as an inherited one therefore shared a single
//! slot with it, so a write through the subclass field overwrote the superclass
//! field. The launch of a real APK (NewPipe 0.27) died on exactly that:
//! `SwitchPreference.mListener` (a `SwitchPreference$Listener`) and
//! `Preference.mListener` (a `PreferenceGroupAdapter`) collapsed into one slot,
//! and `Preference.notifyChanged()` then read the boolean listener where it
//! expected the adapter and dereferenced its null `mVisiblePreferences`.
//!
//! Contract proven here, at bytecode level: both fields exist at the same time,
//! each one addressed through its own declaring class.

mod common;

use common::*;
use rd_vm::value::Value;

const BASE: &str = "Lhid/Base;";
const SUB: &str = "Lhid/Sub;";
const CASO: &str = "LCaso;";
const CONSTRUCTOR: u32 = 0x1_0000;

fn build() -> DexBuilder {
    let mut b = DexBuilder::new();
    let obj_init = {
        let proto = b.proto_idx("V", vec![]);
        b.method_idx("Ljava/lang/Object;", proto, "<init>")
    };

    // ── Base: field v, baseV(), setBase(I) ───────────────────────────────────
    let base = b.class(BASE, "Ljava/lang/Object;");
    b.instance_field(base, "v", "I");
    let base_v = b.field_idx(BASE, "I", "v");
    {
        let proto = b.proto_idx("I", vec![]);
        b.r#virtual(
            base,
            "baseV",
            "I",
            vec![],
            ACC_PUBLIC,
            Some(b.code(2, 1, 0, {
                let mut code = op22c(0x52, 0, 1, base_v); // iget v0, v1(p0), Base.v
                code.extend(op11x(0x0f, 0)); // return v0
                code
            })),
        );
        let _ = proto;
    }
    {
        let proto = b.proto_idx("V", vec!["I".into()]);
        b.r#virtual(
            base,
            "setBase",
            "V",
            vec!["I"],
            ACC_PUBLIC,
            Some(b.code(3, 2, 0, {
                let mut code = op22c(0x59, 2, 1, base_v); // iput v2(p1) -> Base.v
                code.extend(op10x(0x0e)); // return-void
                code
            })),
        );
        let _ = proto;
    }

    // ── Sub: hides v, set(I, I) writes BOTH fields, subV() ───────────────────
    let sub = b.class(SUB, BASE);
    b.instance_field(sub, "v", "I");
    let sub_v = b.field_idx(SUB, "I", "v");
    {
        let proto = b.proto_idx("V", vec![]);
        b.direct(
            sub,
            "<init>",
            "V",
            vec![],
            ACC_PUBLIC | CONSTRUCTOR,
            Some(b.code(1, 1, 1, {
                let mut code = op35c(0x70, 1, obj_init, [0, 0, 0, 0, 0]);
                code.extend(op10x(0x0e));
                code
            })),
        );
        let _ = proto;
    }
    {
        // set(int subValue, int baseValue): this=v1, subValue=v2, baseValue=v3
        let proto = b.proto_idx("V", vec!["I".into(), "I".into()]);
        b.r#virtual(
            sub,
            "set",
            "V",
            vec!["I", "I"],
            ACC_PUBLIC,
            Some(b.code(4, 3, 0, {
                let mut code = op22c(0x59, 2, 1, sub_v); // Sub.v  = v2
                code.extend(op22c(0x59, 3, 1, base_v)); // Base.v = v3
                code.extend(op10x(0x0e));
                code
            })),
        );
        let _ = proto;
    }
    {
        let proto = b.proto_idx("I", vec![]);
        b.r#virtual(
            sub,
            "subV",
            "I",
            vec![],
            ACC_PUBLIC,
            Some(b.code(2, 1, 0, {
                let mut code = op22c(0x52, 0, 1, sub_v); // iget v0, v1, Sub.v
                code.extend(op11x(0x0f, 0));
                code
            })),
        );
        let _ = proto;
    }

    // ── Caso: harness that builds a Sub and reads one field back ─────────────
    let caso = b.class(CASO, "Ljava/lang/Object;");
    let sub_type = b.type_idx(SUB);
    let sub_init = {
        let p = b.proto_idx("V", vec![]);
        b.method_idx(SUB, p, "<init>")
    };
    let set = {
        let p = b.proto_idx("V", vec!["I".into(), "I".into()]);
        b.method_idx(SUB, p, "set")
    };
    let read_base = {
        let p = b.proto_idx("I", vec![]);
        b.method_idx(BASE, p, "baseV")
    };
    let read_sub = {
        let p = b.proto_idx("I", vec![]);
        b.method_idx(SUB, p, "subV")
    };
    // Reads ARE selected with the field ref's declaring class, so each helper
    // is the same harness with a different last call.
    for (name, read) in [("runBase", read_base), ("runSub", read_sub)] {
        b.direct(
            caso,
            name,
            "I",
            vec![],
            ACC_PUBLIC | ACC_STATIC,
            Some(b.code(4, 0, 3, {
                let mut code = op21c(0x22, 0, sub_type); // new-instance v0, Sub
                code.extend(op35c(0x70, 1, sub_init, [0, 0, 0, 0, 0]));
                code.extend(op21s(0x13, 1, 10)); // const/16 v1, 10 (Sub.v)
                code.extend(op11n(0x12, 2, 2)); // const/4  v2, 2  (Base.v)
                code.extend(op35c(0x6e, 3, set, [0, 1, 2, 0, 0]));
                code.extend(op35c(0x6e, 1, read, [0, 0, 0, 0, 0]));
                code.extend(op11x(0x0a, 0)); // move-result v0
                code.extend(op11x(0x0f, 0)); // return v0
                code
            })),
        );
    }
    b
}

#[test]
fn hidden_field_keeps_its_own_slot_in_subclass_and_superclass() {
    let b = build();
    let mut eng = engine_of(&b);
    // Both methods run the same `set(10, 2)`; only the *declaring class* of the
    // field read differs.
    let base = invoke(&mut eng, "runBase", "()I", &[]).expect("runBase executes");
    let sub = invoke(&mut eng, "runSub", "()I", &[]).expect("runSub executes");
    assert_eq!(
        (base, sub),
        (Value::Int(2), Value::Int(10)),
        "Base.v and Sub.v must be independent slots"
    );
}

#[test]
fn write_through_superclass_ref_does_not_clobber_subclass_field() {
    let b = build();
    let mut eng = engine_of(&b);
    let sub = invoke(&mut eng, "runSub", "()I", &[]).expect("runSub executes");
    assert_eq!(sub, Value::Int(10), "Base.v write clobbered Sub.v");
}
