//! Apêndice D (S8) — estado host de DrawerLayout, SwipeRefreshLayout e
//! CardView (`androidx_widgets.rs`). Cobre a função de módulo e o caminho
//! real de dispatch (`Engine::invoke_instance` → camada de shim host),
//! com sucesso, erro (NPE, método fora da lista) e a fronteira de
//! subclasse (MaterialCardView).

mod common;

use common::{engine_of, op11n, op11x, DexBuilder, ACC_PUBLIC};
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

// ── ViewPager2 (S9) ──────────────────────────────────────────────────────

const ADAPTER_SIG: &str = "(Landroidx/recyclerview/widget/RecyclerView$Adapter;)V";
const ADAPTER_GET: &str = "()Landroidx/recyclerview/widget/RecyclerView$Adapter;";
const ADAPTER_BASE: &str = "Landroidx/recyclerview/widget/RecyclerView$Adapter;";

/// Adapter do app com `getItemCount()` no DEX, retornando `n`.
fn count_adapter_class(b: &mut DexBuilder, class: &str, n: i8) {
    let cls = b.class(class, ADAPTER_BASE);
    b.r#virtual(
        cls,
        "getItemCount",
        "I",
        vec![],
        ACC_PUBLIC,
        Some(b.code(2, 1, 0, {
            let mut u = op11n(0x12, 0, n); // const/4 v0, #n
            u.extend(op11x(0x0f, 0)); // return v0
            u
        })),
    );
}

fn pager_engine() -> Engine {
    let mut b = DexBuilder::new();
    count_adapter_class(&mut b, "LVpThree;", 3);
    count_adapter_class(&mut b, "LVpEmpty;", 0);
    engine_of(&b)
}

fn set_item(e: &mut Engine, pager: u32, item: i32) {
    e.invoke_instance(pager, "setCurrentItem", "(I)V", &[Value::Int(item)])
        .expect("setCurrentItem(I)");
}

fn current(e: &mut Engine, pager: u32) -> i32 {
    int_of(Some(
        e.invoke_instance(pager, "getCurrentItem", "()I", &[])
            .expect("getCurrentItem"),
    ))
}

fn set_adapter(e: &mut Engine, pager: u32, adapter: Option<u32>) {
    let arg = adapter.map(Value::Obj).unwrap_or(Value::Null);
    e.invoke_instance(pager, "setAdapter", ADAPTER_SIG, &[arg])
        .expect("setAdapter");
}

#[test]
fn pager_adapter_roundtrip_and_setadapter_resets_page() {
    let mut e = pager_engine();
    let pager = obj(&mut e, aw::VIEW_PAGER2);
    let a = obj(&mut e, "LVpThree;");
    assert_eq!(current(&mut e, pager), 0, "página padrão é 0");

    set_adapter(&mut e, pager, Some(a));
    let got = e
        .invoke_instance(pager, "getAdapter", ADAPTER_GET, &[])
        .expect("getAdapter");
    assert!(
        matches!(got, Value::Obj(r) if r == a),
        "getAdapter devolve o adapter"
    );

    set_item(&mut e, pager, 2);
    assert_eq!(current(&mut e, pager), 2);
    // setAdapter zera a página, como no AOSP.
    set_adapter(&mut e, pager, Some(a));
    assert_eq!(current(&mut e, pager), 0, "setAdapter zera a página");

    set_adapter(&mut e, pager, None);
    let got = e
        .invoke_instance(pager, "getAdapter", ADAPTER_GET, &[])
        .expect("getAdapter nulo");
    assert!(
        matches!(got, Value::Null),
        "setAdapter(null) remove o adapter"
    );
}

#[test]
fn pager_set_current_item_clamps_to_adapter_range() {
    let mut e = pager_engine();
    let pager = obj(&mut e, aw::VIEW_PAGER2);
    let a = obj(&mut e, "LVpThree;");
    set_adapter(&mut e, pager, Some(a));

    set_item(&mut e, pager, 1);
    assert_eq!(current(&mut e, pager), 1, "dentro do intervalo é aceito");
    set_item(&mut e, pager, 7);
    assert_eq!(current(&mut e, pager), 2, "acima de count-1 vira count-1");
    set_item(&mut e, pager, -4);
    assert_eq!(current(&mut e, pager), 0, "negativo vira 0");
    set_item(&mut e, pager, 2);
    assert_eq!(current(&mut e, pager), 2, "limite superior exato é aceito");
}

#[test]
fn pager_set_current_item_with_smooth_flag_follows_same_clamp() {
    // O flag `smoothScroll` é aceito; a página muda de imediato nos dois casos.
    let mut e = pager_engine();
    let pager = obj(&mut e, aw::VIEW_PAGER2);
    let a = obj(&mut e, "LVpThree;");
    set_adapter(&mut e, pager, Some(a));

    e.invoke_instance(
        pager,
        "setCurrentItem",
        "(IZ)V",
        &[Value::Int(9), Value::Int(1)],
    )
    .expect("setCurrentItem(IZ) suave");
    assert_eq!(current(&mut e, pager), 2);
    e.invoke_instance(
        pager,
        "setCurrentItem",
        "(IZ)V",
        &[Value::Int(0), Value::Int(0)],
    )
    .expect("setCurrentItem(IZ) sem animação");
    assert_eq!(current(&mut e, pager), 0);
}

#[test]
fn pager_set_current_item_ignored_without_adapter_or_when_empty() {
    let mut e = pager_engine();
    let pager = obj(&mut e, aw::VIEW_PAGER2);
    set_item(&mut e, pager, 2);
    assert_eq!(current(&mut e, pager), 0, "sem adapter é ignorado");

    let empty = obj(&mut e, "LVpEmpty;");
    set_adapter(&mut e, pager, Some(empty));
    set_item(&mut e, pager, 1);
    assert_eq!(current(&mut e, pager), 0, "adapter vazio é ignorado");

    set_adapter(&mut e, pager, None);
    set_item(&mut e, pager, 1);
    assert_eq!(current(&mut e, pager), 0, "adapter nulo é ignorado");
}

#[test]
fn pager_adapter_without_dex_count_is_ignored() {
    // Adapter só do host (sem `getItemCount` no DEX): contagem 0, sem panic.
    let mut e = pager_engine();
    let pager = obj(&mut e, aw::VIEW_PAGER2);
    let host_adapter = obj(&mut e, ADAPTER_BASE);
    set_adapter(&mut e, pager, Some(host_adapter));
    set_item(&mut e, pager, 1);
    assert_eq!(current(&mut e, pager), 0);
}

#[test]
fn pager_state_is_per_instance_and_unrelated_receiver_untouched() {
    let mut e = pager_engine();
    let p1 = obj(&mut e, aw::VIEW_PAGER2);
    let p2 = obj(&mut e, aw::VIEW_PAGER2);
    let a = obj(&mut e, "LVpThree;");
    set_adapter(&mut e, p1, Some(a));
    set_adapter(&mut e, p2, Some(a));
    set_item(&mut e, p1, 2);
    assert_eq!(current(&mut e, p1), 2);
    assert_eq!(current(&mut e, p2), 0, "outra instância não muda");
    assert_eq!(aw::pager_current_item(&e, p1), 2);
    assert_eq!(aw::pager_adapter_of(&e, p2), Some(a));
}
