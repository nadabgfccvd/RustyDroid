//! Apêndice D (S8) — estado host de DrawerLayout, SwipeRefreshLayout e
//! CardView (`androidx_widgets.rs`). Cobre a função de módulo e o caminho
//! real de dispatch (`Engine::invoke_instance` → camada de shim host),
//! com sucesso, erro (NPE, método fora da lista) e a fronteira de
//! subclasse (MaterialCardView).

use rd_vm::androidx_widgets::{self as aw, call_instance};
use rd_vm::err::VmExit;
use rd_vm::value::Value;
use rd_vm::{Engine, VmConfig};

const VIEW: &str = "Landroid/view/View;";
const VIEW_SIG: &str = "(Landroid/view/View;)V";
const VIEW_Z: &str = "(Landroid/view/View;)Z";

fn engine() -> Engine {
    Engine::new(Vec::new(), VmConfig::default())
}

fn obj(e: &mut Engine, class: &str) -> u32 {
    e.heap
        .alloc_instance(class.to_string(), Vec::new())
        .unwrap()
}

fn int_of(v: Option<Value>) -> i32 {
    match v {
        Some(Value::Int(n)) => n,
        other => panic!("esperava Int, veio {other:?}"),
    }
}

fn float_of(v: Option<Value>) -> f32 {
    match v {
        Some(Value::Float(f)) => f,
        other => panic!("esperava Float, veio {other:?}"),
    }
}

fn expect_npe(err: VmExit) {
    match err {
        VmExit::Exception(t) => assert_eq!(t.class, "Ljava/lang/NullPointerException;"),
        other => panic!("esperava NPE, veio {other:?}"),
    }
}

// ── DrawerLayout ─────────────────────────────────────────────────────────

#[test]
fn drawer_open_close_state_is_per_drawer_view() {
    let mut e = engine();
    let layout = obj(&mut e, aw::DRAWER_LAYOUT);
    let drawer = obj(&mut e, VIEW);
    let other = obj(&mut e, VIEW);

    assert_eq!(
        int_of(
            call_instance(
                &mut e,
                aw::DRAWER_LAYOUT,
                "isDrawerOpen",
                VIEW_Z,
                layout,
                &[Value::Obj(drawer)]
            )
            .unwrap()
        ),
        0,
        "gaveta começa fechada"
    );
    call_instance(
        &mut e,
        aw::DRAWER_LAYOUT,
        "openDrawer",
        VIEW_SIG,
        layout,
        &[Value::Obj(drawer)],
    )
    .unwrap();
    assert!(aw::drawer_is_open(&e, drawer));
    assert_eq!(
        int_of(
            call_instance(
                &mut e,
                aw::DRAWER_LAYOUT,
                "isDrawerVisible",
                VIEW_Z,
                layout,
                &[Value::Obj(drawer)]
            )
            .unwrap()
        ),
        1
    );
    // Outra View não é afetada.
    assert!(!aw::drawer_is_open(&e, other));

    call_instance(
        &mut e,
        aw::DRAWER_LAYOUT,
        "closeDrawer",
        VIEW_SIG,
        layout,
        &[Value::Obj(drawer)],
    )
    .unwrap();
    assert!(!aw::drawer_is_open(&e, drawer));
    assert_eq!(
        int_of(
            call_instance(
                &mut e,
                aw::DRAWER_LAYOUT,
                "isDrawerOpen",
                VIEW_Z,
                layout,
                &[Value::Obj(drawer)]
            )
            .unwrap()
        ),
        0
    );
}

#[test]
fn drawer_null_view_throws_npe() {
    let mut e = engine();
    let layout = obj(&mut e, aw::DRAWER_LAYOUT);
    let err = call_instance(
        &mut e,
        aw::DRAWER_LAYOUT,
        "openDrawer",
        VIEW_SIG,
        layout,
        &[Value::Null],
    )
    .unwrap_err();
    expect_npe(err);
}

#[test]
fn drawer_gravity_overloads_fall_through_to_dex() {
    // Sobrecargas (I) não são modeladas: o host devolve None e o DEX decide.
    let mut e = engine();
    let layout = obj(&mut e, aw::DRAWER_LAYOUT);
    let r = call_instance(
        &mut e,
        aw::DRAWER_LAYOUT,
        "openDrawer",
        "(I)V",
        layout,
        &[Value::Int(3)],
    )
    .unwrap();
    assert!(
        r.is_none(),
        "openDrawer(int) não deve ser sequestrado pelo host"
    );
}

// ── SwipeRefreshLayout ──────────────────────────────────────────────────

#[test]
fn swipe_refresh_state_and_listener_registration() {
    let mut e = engine();
    let layout = obj(&mut e, aw::SWIPE_REFRESH);
    let listener = obj(&mut e, "Lcom/example/Listener;");

    assert_eq!(
        int_of(
            call_instance(
                &mut e,
                aw::SWIPE_REFRESH,
                "isRefreshing",
                "()Z",
                layout,
                &[]
            )
            .unwrap()
        ),
        0
    );
    call_instance(
        &mut e,
        aw::SWIPE_REFRESH,
        "setRefreshing",
        "(Z)V",
        layout,
        &[Value::Int(1)],
    )
    .unwrap();
    assert_eq!(
        int_of(
            call_instance(
                &mut e,
                aw::SWIPE_REFRESH,
                "isRefreshing",
                "()Z",
                layout,
                &[]
            )
            .unwrap()
        ),
        1
    );
    call_instance(
        &mut e,
        aw::SWIPE_REFRESH,
        "setRefreshing",
        "(Z)V",
        layout,
        &[Value::Int(0)],
    )
    .unwrap();
    assert_eq!(
        int_of(
            call_instance(
                &mut e,
                aw::SWIPE_REFRESH,
                "isRefreshing",
                "()Z",
                layout,
                &[]
            )
            .unwrap()
        ),
        0
    );

    call_instance(
        &mut e,
        aw::SWIPE_REFRESH,
        "setOnRefreshListener",
        "(Landroidx/swiperefreshlayout/widget/SwipeRefreshLayout$OnRefreshListener;)V",
        layout,
        &[Value::Obj(listener)],
    )
    .unwrap();
    assert_eq!(aw::swipe_refresh_listener(&e, layout), Some(listener));
    // null remove o listener (AOSP: setOnRefreshListener(null)).
    call_instance(
        &mut e,
        aw::SWIPE_REFRESH,
        "setOnRefreshListener",
        "(Landroidx/swiperefreshlayout/widget/SwipeRefreshLayout$OnRefreshListener;)V",
        layout,
        &[Value::Null],
    )
    .unwrap();
    assert_eq!(aw::swipe_refresh_listener(&e, layout), None);
}

// ── CardView ────────────────────────────────────────────────────────────

#[test]
fn card_view_defaults_follow_aosp() {
    let mut e = engine();
    let card = obj(&mut e, aw::CARD_VIEW);
    let get = |e: &mut Engine, name: &str, sig: &str| {
        call_instance(e, aw::CARD_VIEW, name, sig, card, &[]).unwrap()
    };
    assert_eq!(float_of(get(&mut e, "getRadius", "()F")), 0.0);
    assert_eq!(float_of(get(&mut e, "getCardElevation", "()F")), 0.0);
    assert_eq!(float_of(get(&mut e, "getMaxCardElevation", "()F")), 0.0);
    assert_eq!(int_of(get(&mut e, "getContentPaddingLeft", "()I")), 0);
    assert_eq!(int_of(get(&mut e, "getUseCompatPadding", "()Z")), 0);
    // AOSP: preventCornerOverlap é true por padrão.
    assert_eq!(int_of(get(&mut e, "getPreventCornerOverlap", "()Z")), 1);
}

#[test]
fn card_view_setters_round_trip() {
    let mut e = engine();
    let card = obj(&mut e, aw::CARD_VIEW);
    call_instance(
        &mut e,
        aw::CARD_VIEW,
        "setRadius",
        "(F)V",
        card,
        &[Value::Float(8.5)],
    )
    .unwrap();
    call_instance(
        &mut e,
        aw::CARD_VIEW,
        "setCardElevation",
        "(F)V",
        card,
        &[Value::Float(4.0)],
    )
    .unwrap();
    call_instance(
        &mut e,
        aw::CARD_VIEW,
        "setMaxCardElevation",
        "(F)V",
        card,
        &[Value::Float(12.0)],
    )
    .unwrap();
    call_instance(
        &mut e,
        aw::CARD_VIEW,
        "setContentPadding",
        "(IIII)V",
        card,
        &[Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)],
    )
    .unwrap();
    call_instance(
        &mut e,
        aw::CARD_VIEW,
        "setUseCompatPadding",
        "(Z)V",
        card,
        &[Value::Int(1)],
    )
    .unwrap();
    call_instance(
        &mut e,
        aw::CARD_VIEW,
        "setPreventCornerOverlap",
        "(Z)V",
        card,
        &[Value::Int(0)],
    )
    .unwrap();

    let get = |e: &mut Engine, name: &str, sig: &str| {
        call_instance(e, aw::CARD_VIEW, name, sig, card, &[])
            .unwrap()
            .unwrap()
    };
    assert_eq!(float_of(Some(get(&mut e, "getRadius", "()F"))), 8.5);
    assert_eq!(float_of(Some(get(&mut e, "getCardElevation", "()F"))), 4.0);
    assert_eq!(
        float_of(Some(get(&mut e, "getMaxCardElevation", "()F"))),
        12.0
    );
    assert_eq!(int_of(Some(get(&mut e, "getContentPaddingLeft", "()I"))), 1);
    assert_eq!(int_of(Some(get(&mut e, "getContentPaddingTop", "()I"))), 2);
    assert_eq!(
        int_of(Some(get(&mut e, "getContentPaddingRight", "()I"))),
        3
    );
    assert_eq!(
        int_of(Some(get(&mut e, "getContentPaddingBottom", "()I"))),
        4
    );
    assert_eq!(int_of(Some(get(&mut e, "getUseCompatPadding", "()Z"))), 1);
    assert_eq!(
        int_of(Some(get(&mut e, "getPreventCornerOverlap", "()Z"))),
        0
    );
}

#[test]
fn material_card_subclass_shares_card_view_state() {
    // MaterialCardView estende CardView no classpath builtin: o estado vale.
    let mut e = engine();
    let mc = obj(
        &mut e,
        "Lcom/google/android/material/card/MaterialCardView;",
    );
    call_instance(
        &mut e,
        aw::CARD_VIEW,
        "setRadius",
        "(F)V",
        mc,
        &[Value::Float(6.0)],
    )
    .unwrap();
    let r = call_instance(&mut e, aw::CARD_VIEW, "getRadius", "()F", mc, &[]).unwrap();
    assert_eq!(float_of(r), 6.0);
}

#[test]
fn unrelated_receiver_is_not_intercepted() {
    let mut e = engine();
    let plain = obj(&mut e, "Ljava/lang/Object;");
    let r = call_instance(
        &mut e,
        "Ljava/lang/Object;",
        "setRadius",
        "(F)V",
        plain,
        &[Value::Float(1.0)],
    )
    .unwrap();
    assert!(r.is_none());
}

// ── Dispatch real (Engine::invoke_instance → camada de shim host) ───────

#[test]
fn engine_dispatch_reaches_widget_state_end_to_end() {
    let mut e = engine();
    let layout = obj(&mut e, aw::SWIPE_REFRESH);
    e.invoke_instance(layout, "setRefreshing", "(Z)V", &[Value::Int(1)])
        .expect("setRefreshing via dispatch");
    let r = e
        .invoke_instance(layout, "isRefreshing", "()Z", &[])
        .expect("isRefreshing via dispatch");
    assert_eq!(
        int_of(Some(r)),
        1,
        "estado precisa vir do shim host, não de um default"
    );

    let card = obj(&mut e, aw::CARD_VIEW);
    e.invoke_instance(card, "setRadius", "(F)V", &[Value::Float(3.0)])
        .expect("setRadius via dispatch");
    let r = e
        .invoke_instance(card, "getRadius", "()F", &[])
        .expect("getRadius");
    assert_eq!(float_of(Some(r)), 3.0);
}
