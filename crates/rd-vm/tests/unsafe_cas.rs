//! `sun.misc.Unsafe` operands: the rate-limiting bug of the NewPipe cold start.
//!
//! The intrinsics used to read their operands one index too high (the receiver
//! is **not** part of `args`, so `obj = args[0]`, `offset = args[1]`,
//! `expect = args[2]`, `update = args[3]`). The observable damage was:
//!
//! * `putInt(obj, offset, v)` wrote a default value instead of `v`;
//! * `compareAndSetInt(obj, offset, expect, update)` compared against `update`
//!   and answered `false`, so every CAS retry loop in a lock-free structure
//!   (okhttp's `ConcurrentHashMap.initTable()`) spun until the fuel budget ran
//!   out — inside `Application.onCreate`.
//!
//! These tests drive the real dispatch path: a synthetic DEX class calls the
//! `Lsun/misc/Unsafe;` methods through `invoke-virtual/range`, exactly as
//! desugared `j$` code does.

use rd_testsupport::vm::*;
use rd_vm::value::Value;
use rd_vm::{Engine, ObjRef};

const UNSAFE: &str = "Lsun/misc/Unsafe;";
const TARGET: &str = "LUnsafeTarget;";
const HOLDER: &str = "LUnsafeHolder;";
const OFFSET: i64 = 8;

/// `if-test` formato 21t: `off` é relativo à própria instrução.
fn op21t(op: u8, a: u8, off: i16) -> Vec<u16> {
    vec![u16::from(op) | (u16::from(a) << 8), off as u16]
}

/// `invoke-virtual/range` (op 0x74): three code units.
fn op3rc(op: u8, count: u8, idx: u16, first: u16) -> Vec<u16> {
    vec![u16::from(op) | (u16::from(count) << 8), idx, first]
}

/// `LUnsafeTarget;` = a guest class with one instance `int x` (initialised to 7)
/// and `LUnsafeHolder;` = the static entry points that call `Unsafe`.
fn fixture() -> DexBuilder {
    let mut b = DexBuilder::new();

    let target = b.class(TARGET, "Ljava/lang/Object;");
    b.instance_field(target, "x", "I");
    let x_f = b.field_idx(TARGET, "I", "x");
    // <init>()V { this.x = 7; }  (regs=2, ins=1 → p0 = v1)
    let mut ctor = op11n(0x12, 0, 7);
    ctor.extend(op22c(0x59, 0, 1, x_f)); // iput v0, v1, x:I
    ctor.extend(op10x(0x0e)); // return-void
    b.direct(
        target,
        "<init>",
        "V",
        vec![],
        ACC_PUBLIC,
        Some(b.code(2, 1, 0, ctor)),
    );

    let holder = b.class(HOLDER, "Ljava/lang/Object;");
    let p_cas = b.proto_idx(
        "Z",
        vec![
            "Ljava/lang/Object;".into(),
            "J".into(),
            "I".into(),
            "I".into(),
        ],
    );
    let p_put = b.proto_idx(
        "V",
        vec!["Ljava/lang/Object;".into(), "J".into(), "I".into()],
    );
    let m_cas = b.method_idx(UNSAFE, p_cas, "compareAndSetInt");
    let m_put_int = b.method_idx(UNSAFE, p_put, "putInt");

    // static int casInt(Unsafe u, LUnsafeTarget t)
    //   return u.compareAndSetInt(t, 8L, 7, 42) ? t.x : -1;
    // regs=13 ins=2 → p0=v11 (u), p1=v12 (t); the call uses v5..v10 contiguous.
    let mut cas = op12x(0x07, 5, 11); // move-object v5, p0   (receiver)
    cas.extend(op12x(0x07, 6, 12)); // move-object v6, p1   (object)
    cas.extend(op21s(0x16, 7, OFFSET as i16)); // const-wide/16 v7, 8
    cas.extend(op11n(0x12, 9, 7)); // const/4 v9, 7        (expected)
    cas.extend(op21s(0x13, 10, 42)); // const/16 v10, 42     (update)
    cas.extend(op3rc(0x74, 6, m_cas, 5)); // invoke-virtual/range {v5..v10}
    cas.extend(op11x(0x0a, 4)); // move-result v4
    cas.extend(op22c(0x52, 3, 6, x_f)); // iget v3, v6, x:I
    cas.extend(op21t(0x38, 4, 3)); // if-eqz v4, :failed
    cas.extend(op11x(0x0f, 3)); // return v3
    cas.extend(op11n(0x12, 3, -1)); // :failed const/4 v3, -1
    cas.extend(op11x(0x0f, 3)); // return v3
    b.direct(
        holder,
        "casInt",
        "I",
        vec![UNSAFE.into(), TARGET.into()],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(13, 2, 0, cas)),
    );

    // static void putInt3(Unsafe u, LUnsafeTarget t, int value)
    //   u.putInt(t, 8L, value);
    // regs=14 ins=3 → p0=v11, p1=v12, p2=v13; call uses v6..v9.
    let mut put = op12x(0x07, 6, 11);
    put.extend(op12x(0x07, 7, 12));
    put.extend(op21s(0x16, 8, OFFSET as i16)); // const-wide/16 v8, 8
    put.extend(op12x(0x01, 9, 13)); // move v9, p2 (value)
    put.extend(op3rc(0x74, 5, m_put_int, 6)); // invoke-virtual/range {v6..v9}
    put.extend(op10x(0x0e));
    b.direct(
        holder,
        "putInt3",
        "V",
        vec![UNSAFE.into(), TARGET.into(), "I".into()],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(14, 3, 0, put)),
    );

    // static int casArray(Unsafe u, int[] table)
    //   return u.compareAndSetInt(table, 16 + 2*8, 0, 5) ? 1 : -1;
    let mut arr = op12x(0x07, 5, 11); // receiver
    arr.extend(op12x(0x07, 6, 12)); // object (the array)
    arr.extend(op21s(0x16, 7, 32)); // const-wide/16 v7, 32 == 16 + 2*8
    arr.extend(op11n(0x12, 9, 0)); // expected 0
    arr.extend(op11n(0x12, 10, 5)); // update 5 (const/4 é lit4: -8..7)
    arr.extend(op3rc(0x74, 6, m_cas, 5));
    arr.extend(op11x(0x0a, 4)); // move-result v4
    arr.extend(op21t(0x38, 4, 2)); // if-eqz v4, :failed
    arr.extend(op11n(0x12, 2, 1)); // const/4 v2, 1
    arr.extend(op11x(0x0f, 2)); // return v2
    arr.extend(op11n(0x12, 2, -1)); // :failed
    arr.extend(op11x(0x0f, 2));
    b.direct(
        holder,
        "casArray",
        "I",
        vec![UNSAFE.into(), "[I".into()],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(13, 2, 0, arr)),
    );

    b
}

fn boot(b: &DexBuilder) -> Engine {
    let mut engine = engine_of(b);
    // O offset sintético que `objectFieldOffset` registraria para `x:I`.
    engine.fw.unsafe_offsets.insert(
        OFFSET,
        (TARGET.to_string(), "x".to_string(), "I".to_string()),
    );
    engine
}

fn unsafe_instance(engine: &mut Engine) -> ObjRef {
    engine
        .heap
        .alloc_instance(UNSAFE.to_string(), Vec::new())
        .expect("aloca instância de Unsafe (despacho é por classe do método)")
}

fn target_instance(engine: &mut Engine) -> ObjRef {
    engine
        .new_dex_instance_pub(TARGET)
        .expect("instancia LUnsafeTarget;")
}

#[test]
fn compare_and_set_int_compares_expected_and_stores_update() {
    let b = fixture();
    let mut engine = boot(&b);
    let u = unsafe_instance(&mut engine);
    let target = target_instance(&mut engine);

    let result = engine
        .invoke_static(
            HOLDER,
            "casInt",
            "(Lsun/misc/Unsafe;LUnsafeTarget;)I",
            &[Value::Obj(u), Value::Obj(target)],
        )
        .expect("compareAndSetInt não lança");

    assert_eq!(
        result,
        Value::Int(42),
        "CAS(7 → 42) precisa vencer e o campo precisa passar a valer 42"
    );
    assert_eq!(
        engine.heap.get_field(target, "x").ok(),
        Some(Value::Int(42)),
        "o update do CAS precisa chegar ao campo (era gravado um valor fantasma)"
    );
}

#[test]
fn compare_and_set_int_rejects_a_stale_expectation() {
    let b = fixture();
    let mut engine = boot(&b);
    let u = unsafe_instance(&mut engine);
    let target = target_instance(&mut engine);
    // Depois do primeiro CAS o campo vale 42; repetir com expectativa 7 falha.
    engine
        .invoke_static(
            HOLDER,
            "casInt",
            "(Lsun/misc/Unsafe;LUnsafeTarget;)I",
            &[Value::Obj(u), Value::Obj(target)],
        )
        .expect("primeiro CAS");

    let result = engine
        .invoke_static(
            HOLDER,
            "casInt",
            "(Lsun/misc/Unsafe;LUnsafeTarget;)I",
            &[Value::Obj(u), Value::Obj(target)],
        )
        .expect("segundo CAS");

    assert_eq!(result, Value::Int(-1), "CAS com expectativa obsoleta falha");
    assert_eq!(
        engine.heap.get_field(target, "x").ok(),
        Some(Value::Int(42))
    );
}

#[test]
fn put_int_stores_the_third_argument() {
    let b = fixture();
    let mut engine = boot(&b);
    let u = unsafe_instance(&mut engine);
    let target = target_instance(&mut engine);

    engine
        .invoke_static(
            HOLDER,
            "putInt3",
            "(Lsun/misc/Unsafe;LUnsafeTarget;I)V",
            &[Value::Obj(u), Value::Obj(target), Value::Int(1234)],
        )
        .expect("putInt não lança");

    assert_eq!(
        engine.heap.get_field(target, "x").ok(),
        Some(Value::Int(1234)),
        "putInt precisa gravar o valor recebido, não um default"
    );
}

#[test]
fn offsets_past_the_array_base_address_elements() {
    let b = fixture();
    let mut engine = boot(&b);
    let u = unsafe_instance(&mut engine);
    let array = engine
        .heap
        .alloc_array_zeroed(rd_vm::heap::ElemKind::Int, 4, "[I")
        .expect("aloca int[4]");

    let result = engine
        .invoke_static(
            HOLDER,
            "casArray",
            "(Lsun/misc/Unsafe;[I)I",
            &[Value::Obj(u), Value::Obj(array)],
        )
        .expect("CAS em elemento de array não lança");

    assert_eq!(
        result,
        Value::Int(1),
        "CAS no slot 2 do array precisa vencer"
    );
    let elems = engine.heap.as_array_elems(array).expect("array válido").0;
    assert_eq!(elems[2], Value::Int(5), "o elemento 2 recebe o update");
    assert_eq!(elems[0], Value::Int(0), "os outros elementos não mudam");
}

#[test]
fn an_offset_outside_both_spaces_is_a_typed_error() {
    let b = fixture();
    let mut engine = boot(&b);
    let u = unsafe_instance(&mut engine);
    let target = target_instance(&mut engine);

    let err = engine
        .invoke_static(
            HOLDER,
            "casInt",
            "(Lsun/misc/Unsafe;LUnsafeTarget;)I",
            &[Value::Obj(u), Value::Obj(target)],
        )
        .and_then(|_| {
            engine.invoke_static(
                HOLDER,
                "casInt",
                "(Lsun/misc/Unsafe;LUnsafeTarget;)I",
                &[Value::Obj(u), Value::Obj(target)],
            )
        });
    // Sanidade: o caminho normal funciona duas vezes (nada de erro aqui).
    assert!(matches!(err, Ok(Value::Int(-1))), "sanidade: {err:?}");

    engine.fw.unsafe_offsets.clear();
    let failure = engine.invoke_static(
        HOLDER,
        "casInt",
        "(Lsun/misc/Unsafe;LUnsafeTarget;)I",
        &[Value::Obj(u), Value::Obj(target)],
    );
    let message = format!("{failure:?}");
    assert!(
        message.contains("unknown synthetic field offset"),
        "offset desconhecido precisa virar erro tipado, veio: {message}"
    );
}
