//! Guest-visible `android.util.StateSet`: DEX `sget-object` materializes
//! the shared WILD_CARD/NOTHING arrays and `invoke-static` reaches the
//! AOSP state-spec matching implementation.

mod common;

use common::*;
use rd_vm::heap::ElemKind;
use rd_vm::value::Value;
use rd_vm::Engine;

const STATE_SET: &str = "Landroid/util/StateSet;";
const PROBE: &str = "LStateSetProbe;";
const PRESSED: i32 = 0x0101_00a7;
const FOCUSED: i32 = 0x0101_009c;
const ENABLED: i32 = 0x0101_009e;

fn build_probe() -> DexBuilder {
    let mut b = DexBuilder::new();
    let probe = b.class(PROBE, "Ljava/lang/Object;");
    let state_set_type = b.type_idx(STATE_SET);
    let wild = b.field_idx(STATE_SET, "[I", "WILD_CARD");
    let nothing = b.field_idx(STATE_SET, "[I", "NOTHING");
    let matches_proto = b.proto_idx("Z", vec!["[I".to_string(), "[I".to_string()]);
    let matches = b.method_idx(STATE_SET, matches_proto, "stateSetMatches");
    let init_proto = b.proto_idx("V", vec![]);
    let init = b.method_idx(STATE_SET, init_proto, "<init>");

    b.direct(
        probe,
        "wildCard",
        "[I",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(1, 0, 0, {
            let mut code = op21c(0x62, 0, wild); // sget-object v0, WILD_CARD
            code.extend(op11x(0x11, 0)); // return-object v0
            code
        })),
    );
    b.direct(
        probe,
        "nothing",
        "[I",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(1, 0, 0, {
            let mut code = op21c(0x62, 0, nothing); // sget-object v0, NOTHING
            code.extend(op11x(0x11, 0)); // return-object v0
            code
        })),
    );
    b.direct(
        probe,
        "matches",
        "Z",
        vec!["[I", "[I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(4, 2, 2, {
            // regs=4, ins=2: v2=spec, v3=set.
            let mut code = op35c(0x71, 2, matches, [2, 3, 0, 0, 0]);
            code.extend(op11x(0x0a, 0)); // move-result v0
            code.extend(op11x(0x0f, 0)); // return v0
            code
        })),
    );
    b.direct(
        probe,
        "make",
        STATE_SET,
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(1, 0, 1, {
            let mut code = op21c(0x22, 0, state_set_type); // new-instance v0, StateSet
            code.extend(op35c(0x70, 1, init, [0, 0, 0, 0, 0])); // <init> {v0}
            code.extend(op11x(0x11, 0)); // return-object v0
            code
        })),
    );
    b
}

fn int_array(engine: &mut Engine, values: &[i32]) -> Value {
    let array = engine
        .heap
        .alloc_array_zeroed(ElemKind::Int, values.len(), "[I")
        .unwrap();
    *engine.heap.array_elems_mut(array).unwrap() = values.iter().map(|v| Value::Int(*v)).collect();
    Value::Obj(array)
}

fn ints(engine: &Engine, value: &Value) -> Vec<i32> {
    let Value::Obj(array) = value else {
        panic!("expected int array, got {value:?}");
    };
    engine
        .heap
        .as_array_elems(*array)
        .unwrap()
        .0
        .iter()
        .map(|v| v.as_int().unwrap())
        .collect()
}

#[test]
fn guest_state_set_sget_and_static_matching() {
    let mut engine = engine_of(&build_probe());

    let wild_first = engine
        .invoke_static(PROBE, "wildCard", "()[I", &[])
        .unwrap();
    let wild_second = engine
        .invoke_static(PROBE, "wildCard", "()[I", &[])
        .unwrap();
    assert_eq!(wild_first, wild_second, "WILD_CARD is one shared array");
    assert_eq!(ints(&engine, &wild_first), Vec::<i32>::new());
    let nothing = engine.invoke_static(PROBE, "nothing", "()[I", &[]).unwrap();
    assert_eq!(ints(&engine, &nothing), vec![0]);

    let spec = int_array(&mut engine, &[PRESSED, -FOCUSED]);
    let matching = int_array(&mut engine, &[PRESSED, ENABLED]);
    assert_eq!(
        engine
            .invoke_static(PROBE, "matches", "([I[I)Z", &[spec.clone(), matching])
            .unwrap(),
        Value::Int(1)
    );
    let prohibited = int_array(&mut engine, &[PRESSED, FOCUSED]);
    assert_eq!(
        engine
            .invoke_static(PROBE, "matches", "([I[I)Z", &[spec, prohibited])
            .unwrap(),
        Value::Int(0)
    );

    let Value::Obj(state_set) = engine
        .invoke_static(PROBE, "make", "()Landroid/util/StateSet;", &[])
        .unwrap()
    else {
        panic!("StateSet.<init> should produce an object");
    };
    assert_eq!(engine.heap.class_of(state_set).unwrap(), STATE_SET);
}
