//! Apêndice D (S8) — estado host de três widgets AndroidX sem modelo de
//! estado até aqui: `DrawerLayout`, `SwipeRefreshLayout` e `CardView`.
//!
//! Estratégia `host-shim` (ADR do Apêndice D): o host intercepta só os
//! métodos listados abaixo, com estado real consultável pelo harness e
//! pelo agente. Qualquer outro método (inclusive as sobrecargas de
//! `DrawerLayout` por gravidade) devolve `None` e segue para o DEX da
//! biblioteca, se ela estiver no APK.
//!
//! Desvios documentados (não silenciosos):
//! - `DrawerLayout.openDrawer(View)` aceita qualquer `View` não nula como
//!   gaveta; a validação de `layout_gravity` do AOSP (que lança
//!   `IllegalArgumentException` para não-gavetas) não é modelada, porque
//!   o host não acompanha os filhos do layout.
//! - Animação de abertura/fechamento não é modelada: o estado muda de
//!   imediato.
//! - `SwipeRefreshLayout` guarda o `OnRefreshListener`, mas não dispara
//!   `onRefresh()` por gesto (não há modelo de toque). O harness pode
//!   consultar o listener com `swipe_refresh_listener` e chamá-lo.
//! - `CardView` guarda raio, elevação, padding e flags, com os defaults do
//!   AOSP. Sombra, cor de fundo e desenho não são modelados.

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::heap::ObjRef;
use crate::value::Value;

pub const DRAWER_LAYOUT: &str = "Landroidx/drawerlayout/widget/DrawerLayout;";
pub const SWIPE_REFRESH: &str = "Landroidx/swiperefreshlayout/widget/SwipeRefreshLayout;";
pub const CARD_VIEW: &str = "Landroidx/cardview/widget/CardView;";

const DRAWER_OPEN: &str = "rd_drawer_open";
const REFRESHING: &str = "rd_refreshing";
const REFRESH_LISTENER: &str = "rd_refresh_listener";
const CARD_RADIUS: &str = "rd_card_radius";
const CARD_ELEVATION: &str = "rd_card_elevation";
const CARD_MAX_ELEVATION: &str = "rd_card_max_elevation";
const CARD_PAD_LEFT: &str = "rd_card_pad_left";
const CARD_PAD_TOP: &str = "rd_card_pad_top";
const CARD_PAD_RIGHT: &str = "rd_card_pad_right";
const CARD_PAD_BOTTOM: &str = "rd_card_pad_bottom";
const CARD_COMPAT_PADDING: &str = "rd_card_compat_padding";
const CARD_PREVENT_OVERLAP: &str = "rd_card_prevent_overlap";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    Drawer,
    Swipe,
    Card,
}

fn npe(message: &str) -> VmExit {
    VmExit::Exception(Throwable::new("Ljava/lang/NullPointerException;", message))
}

fn family_of(vm: &Engine, mref_class: &str, recv_class: &str) -> Option<Family> {
    let candidates = [
        (DRAWER_LAYOUT, Family::Drawer, "Landroidx/drawerlayout/"),
        (
            SWIPE_REFRESH,
            Family::Swipe,
            "Landroidx/swiperefreshlayout/",
        ),
        (CARD_VIEW, Family::Card, "Landroidx/cardview/"),
        // MaterialCardView estende CardView no classpath builtin.
        (
            CARD_VIEW,
            Family::Card,
            "Lcom/google/android/material/card/",
        ),
    ];
    for (root, fam, prefix) in candidates {
        let related = |c: &str| c.starts_with(prefix) && (c == root || vm.cp.is_subtype(c, root));
        if related(mref_class) || related(recv_class) {
            return Some(fam);
        }
    }
    None
}

fn int_arg(args: &[Value], i: usize) -> i32 {
    match args.get(i) {
        Some(Value::Int(v)) => *v,
        _ => 0,
    }
}

fn float_arg(args: &[Value], i: usize) -> f32 {
    match args.get(i) {
        Some(Value::Float(v)) => *v,
        _ => 0.0,
    }
}

fn get_int(vm: &Engine, r: ObjRef, field: &str, default: i32) -> i32 {
    match vm.heap.get_field(r, field) {
        Ok(Value::Int(v)) => v,
        _ => default,
    }
}

fn get_float(vm: &Engine, r: ObjRef, field: &str, default: f32) -> f32 {
    match vm.heap.get_field(r, field) {
        Ok(Value::Float(v)) => v,
        _ => default,
    }
}

fn put(vm: &mut Engine, r: ObjRef, field: &str, v: Value) -> Result<(), VmExit> {
    crate::err::put_field_result(vm.heap.put_field(r, field, v))
}

/// Dispatch de instância para os três widgets. `None` = não é deste módulo.
pub fn call_instance(
    vm: &mut Engine,
    mref_class: &str,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let recv_class = vm.heap.class_of(recv).unwrap_or("").to_string();
    let Some(fam) = family_of(vm, mref_class, &recv_class) else {
        return Ok(None);
    };
    match fam {
        Family::Drawer => drawer(vm, name, sig, args),
        Family::Swipe => swipe(vm, name, sig, recv, args),
        Family::Card => card(vm, name, sig, recv, args),
    }
}

fn drawer(vm: &mut Engine, name: &str, sig: &str, args: &[Value]) -> Result<Option<Value>, VmExit> {
    // Só a sobrecarga `View` é modelada; as de gravidade `(I)` seguem para o DEX.
    if sig != "(Landroid/view/View;)V" && sig != "(Landroid/view/View;)Z" {
        return Ok(None);
    }
    let Some(Value::Obj(drawer_view)) = args.first().cloned() else {
        if matches!(args.first(), Some(Value::Null) | None) {
            return Err(npe("drawerView == null"));
        }
        return Ok(None);
    };
    match name {
        "openDrawer" => {
            put(vm, drawer_view, DRAWER_OPEN, Value::Int(1))?;
            Ok(Some(Value::Null))
        }
        "closeDrawer" => {
            put(vm, drawer_view, DRAWER_OPEN, Value::Int(0))?;
            Ok(Some(Value::Null))
        }
        "isDrawerOpen" | "isDrawerVisible" => Ok(Some(Value::Int(
            (get_int(vm, drawer_view, DRAWER_OPEN, 0) != 0) as i32,
        ))),
        _ => Ok(None),
    }
}

fn swipe(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match (name, sig) {
        ("setRefreshing", "(Z)V") => {
            put(
                vm,
                recv,
                REFRESHING,
                Value::Int((int_arg(args, 0) != 0) as i32),
            )?;
            Ok(Some(Value::Null))
        }
        ("isRefreshing", "()Z") => Ok(Some(Value::Int(get_int(vm, recv, REFRESHING, 0)))),
        ("setOnRefreshListener", _) if sig.ends_with(")V") => {
            let listener = match args.first() {
                Some(Value::Obj(r)) => Value::Obj(*r),
                _ => Value::Null,
            };
            put(vm, recv, REFRESH_LISTENER, listener)?;
            Ok(Some(Value::Null))
        }
        _ => Ok(None),
    }
}

fn card(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let out = match (name, sig) {
        ("setRadius", "(F)V") => {
            put(vm, recv, CARD_RADIUS, Value::Float(float_arg(args, 0)))?;
            Value::Null
        }
        ("getRadius", "()F") => Value::Float(get_float(vm, recv, CARD_RADIUS, 0.0)),
        ("setCardElevation", "(F)V") => {
            put(vm, recv, CARD_ELEVATION, Value::Float(float_arg(args, 0)))?;
            Value::Null
        }
        ("getCardElevation", "()F") => Value::Float(get_float(vm, recv, CARD_ELEVATION, 0.0)),
        ("setMaxCardElevation", "(F)V") => {
            put(
                vm,
                recv,
                CARD_MAX_ELEVATION,
                Value::Float(float_arg(args, 0)),
            )?;
            Value::Null
        }
        ("getMaxCardElevation", "()F") => {
            Value::Float(get_float(vm, recv, CARD_MAX_ELEVATION, 0.0))
        }
        ("setContentPadding", "(IIII)V") => {
            put(vm, recv, CARD_PAD_LEFT, Value::Int(int_arg(args, 0)))?;
            put(vm, recv, CARD_PAD_TOP, Value::Int(int_arg(args, 1)))?;
            put(vm, recv, CARD_PAD_RIGHT, Value::Int(int_arg(args, 2)))?;
            put(vm, recv, CARD_PAD_BOTTOM, Value::Int(int_arg(args, 3)))?;
            Value::Null
        }
        ("getContentPaddingLeft", "()I") => Value::Int(get_int(vm, recv, CARD_PAD_LEFT, 0)),
        ("getContentPaddingTop", "()I") => Value::Int(get_int(vm, recv, CARD_PAD_TOP, 0)),
        ("getContentPaddingRight", "()I") => Value::Int(get_int(vm, recv, CARD_PAD_RIGHT, 0)),
        ("getContentPaddingBottom", "()I") => Value::Int(get_int(vm, recv, CARD_PAD_BOTTOM, 0)),
        ("setUseCompatPadding", "(Z)V") => {
            put(
                vm,
                recv,
                CARD_COMPAT_PADDING,
                Value::Int((int_arg(args, 0) != 0) as i32),
            )?;
            Value::Null
        }
        ("getUseCompatPadding", "()Z") => Value::Int(get_int(vm, recv, CARD_COMPAT_PADDING, 0)),
        ("setPreventCornerOverlap", "(Z)V") => {
            put(
                vm,
                recv,
                CARD_PREVENT_OVERLAP,
                Value::Int((int_arg(args, 0) != 0) as i32),
            )?;
            Value::Null
        }
        // AOSP: preventCornerOverlap é true por padrão.
        ("getPreventCornerOverlap", "()Z") => {
            Value::Int(get_int(vm, recv, CARD_PREVENT_OVERLAP, 1))
        }
        _ => return Ok(None),
    };
    Ok(Some(out))
}

/// Acessor para o harness: a gaveta está aberta (estado `openDrawer`/`closeDrawer`)?
pub fn drawer_is_open(vm: &Engine, drawer_view: ObjRef) -> bool {
    get_int(vm, drawer_view, DRAWER_OPEN, 0) != 0
}

/// Acessor para o harness: o `OnRefreshListener` registrado, se houver.
pub fn swipe_refresh_listener(vm: &Engine, layout: ObjRef) -> Option<ObjRef> {
    match vm.heap.get_field(layout, REFRESH_LISTENER) {
        Ok(Value::Obj(r)) => Some(r),
        _ => None,
    }
}
