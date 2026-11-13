//! `android.view.Menu` / `MenuItem` / `MenuInflater` (headless).
//!
//! `MenuInflater.inflate(menuRes, menu)` parses `res/menu/*.xml` with the
//! AXML parser and fills the host `Menu` (`HostObj::Menu`) with `MenuItem`
//! objects: `android:id`, `android:title` (literal or string resource),
//! `android:visible`, `android:enabled`, `android:checkable`, `android:checked`.
//! `<group>` children are flattened into the menu. An `<item>` with a `<menu>`
//! child becomes a `SubMenu` (`Menu.addSubMenu`, `MenuItem.getSubMenu`), nested
//! up to `MAX_MENU_DEPTH` levels. Menu items do not render: they are state the
//! app can query and change (`findItem`, `setVisible`, `setTitle`, …), which
//! keeps the app's own logic running without a visible action bar. A submenu's
//! header (`setHeaderTitle`/`setHeaderIcon`/`setHeaderView`) is stored by
//! nothing and rendered by nothing: the calls are accepted and chain.

use crate::engine::Engine;
use crate::err::{put_field_result, VmExit};
use crate::framework::HostObj;
use crate::heap::ObjRef;
use crate::value::Value;
use rd_apk::axml::{AttrValue, XmlElement};

pub(crate) const MENU_INFLATER: &str = "Landroid/view/MenuInflater;";
pub(crate) const MENU: &str = "Landroid/view/Menu;";
pub(crate) const MENU_ITEM: &str = "Landroid/view/MenuItem;";
pub(crate) const SUB_MENU: &str = "Landroid/view/SubMenu;";

/// Nesting cap for `<menu>` inside `<item>` (hostile XML must not recurse forever).
const MAX_MENU_DEPTH: usize = 8;

/// Instance methods of the menu surface. `Ok(None)` = not ours.
pub(crate) fn instance(
    vm: &mut Engine,
    recv_class: &str,
    mref_class: &str,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if mref_class == MENU_INFLATER || recv_class == MENU_INFLATER {
        return inflater(vm, name, sig, recv, args);
    }
    // Host receivers only: a DEX Menu/MenuItem (AppCompat's MenuBuilder) owns
    // its own methods and must not be answered by this host model.
    if recv_class == MENU {
        return menu(vm, name, sig, recv, args);
    }
    if recv_class == MENU_ITEM {
        return item(vm, name, sig, recv, args);
    }
    if recv_class == SUB_MENU {
        return sub_menu(vm, name, sig, recv, args);
    }
    Ok(None)
}

fn inflater(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match (name, sig) {
        ("<init>", "(Landroid/content/Context;)V") => {
            if let Some(Value::Obj(ctx)) = args.first() {
                put_field_result(vm.heap.put_field(recv, "rd_context", Value::Obj(*ctx)))?;
            }
            Ok(Some(Value::Null))
        }
        ("inflate", "(ILandroid/view/Menu;)V") => {
            let resid = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0) as u32;
            let Some(Value::Obj(menu)) = args.get(1).cloned() else {
                return Ok(Some(Value::Null));
            };
            let specs = inflate_menu_resource(vm, resid)?;
            apply_specs(vm, menu, &specs)?;
            Ok(Some(Value::Null))
        }
        _ => Ok(None),
    }
}

/// Adds `specs` to `menu` through the menu's own `add`/`addSubMenu` and item
/// setters: the target may be a host `Menu` or a DEX `Menu` (AppCompat's
/// `MenuBuilder`), so the same calls work for both.
fn apply_specs(vm: &mut Engine, menu: ObjRef, specs: &[ItemSpec]) -> Result<(), VmExit> {
    for spec in specs {
        let title = match &spec.title {
            Some(t) => Value::Obj(crate::intrinsics::alloc_string(vm, t.clone())?),
            None => Value::Null,
        };
        let args = [
            Value::Int(spec.group),
            Value::Int(spec.id),
            Value::Int(spec.order),
            title,
        ];
        let it = match &spec.submenu {
            Some(children) => {
                let sub = vm.invoke_instance(
                    menu,
                    "addSubMenu",
                    "(IIILjava/lang/CharSequence;)Landroid/view/SubMenu;",
                    &args,
                )?;
                let Value::Obj(sub) = sub else {
                    continue;
                };
                apply_specs(vm, sub, children)?;
                vm.invoke_instance(sub, "getItem", "()Landroid/view/MenuItem;", &[])?
            }
            None => vm.invoke_instance(
                menu,
                "add",
                "(IIILjava/lang/CharSequence;)Landroid/view/MenuItem;",
                &args,
            )?,
        };
        let Value::Obj(it) = it else {
            continue;
        };
        if !spec.visible {
            vm.invoke_instance(
                it,
                "setVisible",
                "(Z)Landroid/view/MenuItem;",
                &[Value::Int(0)],
            )?;
        }
        if !spec.enabled {
            vm.invoke_instance(
                it,
                "setEnabled",
                "(Z)Landroid/view/MenuItem;",
                &[Value::Int(0)],
            )?;
        }
        if spec.checkable {
            vm.invoke_instance(
                it,
                "setCheckable",
                "(Z)Landroid/view/MenuItem;",
                &[Value::Int(1)],
            )?;
        }
        if spec.checked {
            vm.invoke_instance(
                it,
                "setChecked",
                "(Z)Landroid/view/MenuItem;",
                &[Value::Int(1)],
            )?;
        }
    }
    Ok(())
}

/// Parsed `<item>` of a menu resource.
#[derive(Debug, Clone, PartialEq)]
struct ItemSpec {
    id: i32,
    group: i32,
    order: i32,
    title: Option<String>,
    visible: bool,
    enabled: bool,
    checkable: bool,
    checked: bool,
    /// `<menu>` filho de `<item>`: itens do submenu, em ordem de declaração.
    submenu: Option<Vec<ItemSpec>>,
}

fn attr<'a>(el: &'a XmlElement, name: &str) -> Option<&'a AttrValue> {
    el.attrs
        .iter()
        .find(|a| a.is_android() && a.name == name)
        .map(|a| &a.value)
}

fn bool_attr(el: &XmlElement, name: &str, default: bool) -> bool {
    match attr(el, name) {
        Some(AttrValue::Bool(b)) => *b,
        Some(AttrValue::Int(i)) => *i != 0,
        Some(AttrValue::String(s)) => matches!(s.as_str(), "true" | "1"),
        _ => default,
    }
}

fn int_attr(vm: &Engine, el: &XmlElement, name: &str) -> i32 {
    match attr(el, name) {
        Some(AttrValue::Reference(r)) => *r as i32,
        Some(AttrValue::Int(i)) => *i,
        _ => {
            let _ = vm;
            0
        }
    }
}

fn title_attr(vm: &Engine, el: &XmlElement) -> Option<String> {
    match attr(el, "title")? {
        AttrValue::String(s) => Some(s.clone()),
        AttrValue::Reference(r) => {
            let dev = vm.resource_device_config();
            vm.fw
                .resources
                .as_ref()
                .and_then(|a| a.arsc.as_ref())
                .and_then(|a| a.resolve_string_for(*r, &dev))
        }
        _ => None,
    }
}

/// Flattens `<menu>` → `<item>`/`<group>` into specs, declaration order. An
/// `<item>` whose `<menu>` child exists (within the depth cap) carries that
/// child's items as its submenu.
fn collect(vm: &Engine, el: &XmlElement, group: i32, out: &mut Vec<ItemSpec>, depth: usize) {
    for child in &el.children {
        match child.name.as_str() {
            "item" => {
                let submenu = child
                    .children
                    .iter()
                    .find(|c| c.name == "menu")
                    .filter(|_| depth < MAX_MENU_DEPTH)
                    .map(|menu| {
                        let mut sub = Vec::new();
                        collect(vm, menu, 0, &mut sub, depth + 1);
                        sub
                    });
                out.push(ItemSpec {
                    id: int_attr(vm, child, "id"),
                    group,
                    order: int_attr(vm, child, "orderInCategory"),
                    title: title_attr(vm, child),
                    visible: bool_attr(child, "visible", true),
                    enabled: bool_attr(child, "enabled", true),
                    checkable: bool_attr(child, "checkable", false),
                    checked: bool_attr(child, "checked", false),
                    submenu,
                })
            }
            "group" => {
                let gid = int_attr(vm, child, "id");
                collect(vm, child, gid, out, depth);
            }
            _ => {}
        }
    }
}

fn inflate_menu_resource(vm: &mut Engine, resid: u32) -> Result<Vec<ItemSpec>, VmExit> {
    let apk = vm.resources()?;
    let Some(arsc) = apk.arsc.as_ref() else {
        return Ok(Vec::new());
    };
    let Some(name) = arsc.resolve_name(resid) else {
        return Ok(Vec::new());
    };
    let Some(key) = name.strip_prefix("@menu/") else {
        return Ok(Vec::new());
    };
    // The ARSC entry holds the real ZIP path (compiled APKs may rename
    // `res/menu/x.xml`); the canonical path is the fallback.
    let dev = vm.resource_device_config();
    let mut entry = format!("res/menu/{key}.xml");
    if let Some(rd_apk::arsc::ResolvedValue::Str(path)) = arsc.resolve_value_for(resid, &dev) {
        if path.starts_with("res/")
            && path.ends_with(".xml")
            && apk.container().find(&path).is_some()
        {
            entry = path;
        }
    }
    let Ok(bytes) = apk.container().read(&entry) else {
        return Ok(Vec::new());
    };
    let Ok(doc) = rd_apk::axml::parse(&bytes) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    collect(vm, &doc.root, 0, &mut out, 0);
    Ok(out)
}

fn make_item(vm: &mut Engine, spec: &ItemSpec) -> Result<ObjRef, VmExit> {
    let item = vm.heap.alloc_instance(MENU_ITEM.to_string(), Vec::new())?;
    put_field_result(vm.heap.put_field(item, "rd_id", Value::Int(spec.id)))?;
    put_field_result(vm.heap.put_field(item, "rd_group", Value::Int(spec.group)))?;
    put_field_result(vm.heap.put_field(item, "rd_order", Value::Int(spec.order)))?;
    put_field_result(
        vm.heap
            .put_field(item, "rd_visible", Value::Int(spec.visible as i32)),
    )?;
    put_field_result(
        vm.heap
            .put_field(item, "rd_enabled", Value::Int(spec.enabled as i32)),
    )?;
    put_field_result(
        vm.heap
            .put_field(item, "rd_checkable", Value::Int(spec.checkable as i32)),
    )?;
    put_field_result(
        vm.heap
            .put_field(item, "rd_checked", Value::Int(spec.checked as i32)),
    )?;
    if let Some(title) = &spec.title {
        let s = crate::intrinsics::alloc_string(vm, title.clone())?;
        put_field_result(vm.heap.put_field(item, "rd_title", Value::Obj(s)))?;
    }
    Ok(item)
}

fn res_string(vm: &Engine, resid: u32) -> Option<String> {
    let dev = vm.resource_device_config();
    vm.fw
        .resources
        .as_ref()
        .and_then(|a| a.arsc.as_ref())
        .and_then(|a| a.resolve_string_for(resid, &dev))
}

fn add_item(vm: &mut Engine, menu: ObjRef, spec: ItemSpec) -> Result<Value, VmExit> {
    let it = make_item(vm, &spec)?;
    if let Some(HostObj::Menu { items }) = vm.fw.objects.get_mut(&menu) {
        items.push(it);
    }
    Ok(Value::Obj(it))
}

/// Spec of an `addSubMenu` overload (same argument shapes as `add`).
fn sub_spec(vm: &Engine, sig: &str, args: &[Value]) -> Option<ItemSpec> {
    let arg_int = |i: usize| args.get(i).and_then(|v| v.as_int().ok()).unwrap_or(0);
    let text = |i: usize| match args.get(i) {
        Some(Value::Obj(s)) => vm.heap.as_str(*s).ok().map(|t| t.to_string()),
        _ => None,
    };
    let plain = |id, group, order, title| ItemSpec {
        id,
        group,
        order,
        title,
        visible: true,
        enabled: true,
        checkable: false,
        checked: false,
        submenu: None,
    };
    match sig {
        "(Ljava/lang/CharSequence;)Landroid/view/SubMenu;" => Some(plain(0, 0, 0, text(0))),
        "(I)Landroid/view/SubMenu;" => Some(plain(0, 0, 0, res_string(vm, arg_int(0) as u32))),
        "(IIII)Landroid/view/SubMenu;" => Some(plain(
            arg_int(1),
            arg_int(0),
            arg_int(2),
            res_string(vm, arg_int(3) as u32),
        )),
        "(IIILjava/lang/CharSequence;)Landroid/view/SubMenu;" => {
            Some(plain(arg_int(1), arg_int(0), arg_int(2), text(3)))
        }
        _ => None,
    }
}

/// Adds the item that opens a submenu and returns its `SubMenu`, linked both
/// ways (`MenuItem.getSubMenu` ↔ `SubMenu.getItem`).
fn add_sub_item(vm: &mut Engine, menu: ObjRef, spec: ItemSpec) -> Result<Value, VmExit> {
    let it = make_item(vm, &spec)?;
    if let Some(HostObj::Menu { items }) = vm.fw.objects.get_mut(&menu) {
        items.push(it);
    }
    let sub = vm.heap.alloc_instance(SUB_MENU.to_string(), Vec::new())?;
    vm.fw
        .objects
        .insert(sub, HostObj::Menu { items: Vec::new() });
    put_field_result(vm.heap.put_field(sub, "rd_item", Value::Obj(it)))?;
    put_field_result(vm.heap.put_field(it, "rd_submenu", Value::Obj(sub)))?;
    Ok(Value::Obj(sub))
}

/// `SubMenu`: its own getters/header setters; everything else is `Menu`.
fn sub_menu(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let v = match (name, sig) {
        ("getItem", "()Landroid/view/MenuItem;") => {
            vm.heap.get_field(recv, "rd_item").unwrap_or(Value::Null)
        }
        ("clearHeader", "()V") => Value::Null,
        ("setHeaderTitle" | "setHeaderIcon" | "setHeaderView" | "setIcon", s)
            if s.ends_with("Landroid/view/SubMenu;") =>
        {
            Value::Obj(recv)
        }
        _ => return menu(vm, name, sig, recv, args),
    };
    Ok(Some(v))
}

fn items_of(vm: &Engine, menu: ObjRef) -> Vec<ObjRef> {
    match vm.fw.objects.get(&menu) {
        Some(HostObj::Menu { items }) => items.clone(),
        _ => Vec::new(),
    }
}

fn menu(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let arg_int = |i: usize| args.get(i).and_then(|v| v.as_int().ok()).unwrap_or(0);
    let find = |vm: &Engine, id: i32| -> Option<ObjRef> {
        items_of(vm, recv)
            .into_iter()
            .find(|it| matches!(vm.heap.get_field(*it, "rd_id"), Ok(Value::Int(v)) if v == id))
    };
    let v = match (name, sig) {
        ("<init>", "()V") => {
            vm.fw
                .objects
                .insert(recv, HostObj::Menu { items: Vec::new() });
            Value::Null
        }
        ("size", "()I") => Value::Int(items_of(vm, recv).len() as i32),
        ("getItem", "(I)Landroid/view/MenuItem;") => {
            let i = arg_int(0);
            let it = items_of(vm, recv).get(i.max(0) as usize).copied();
            it.map_or(Value::Null, Value::Obj)
        }
        ("findItem", "(I)Landroid/view/MenuItem;") => {
            find(vm, arg_int(0)).map_or(Value::Null, Value::Obj)
        }
        ("hasVisibleItems", "()Z") => Value::Int(
            items_of(vm, recv)
                .into_iter()
                .any(|it| matches!(vm.heap.get_field(it, "rd_visible"), Ok(Value::Int(1))))
                as i32,
        ),
        ("clear", "()V") => {
            if let Some(HostObj::Menu { items }) = vm.fw.objects.get_mut(&recv) {
                items.clear();
            }
            Value::Null
        }
        ("removeItem", "(I)V") => {
            if let Some(it) = find(vm, arg_int(0)) {
                if let Some(HostObj::Menu { items }) = vm.fw.objects.get_mut(&recv) {
                    items.retain(|x| *x != it);
                }
            }
            Value::Null
        }
        ("addSubMenu", _) => match sub_spec(vm, sig, args) {
            Some(spec) => add_sub_item(vm, recv, spec)?,
            None => return Ok(None),
        },
        // AOSP overloads: add(CharSequence), add(int titleRes),
        // add(int group, int id, int order, CharSequence|int titleRes).
        ("add", "(Ljava/lang/CharSequence;)Landroid/view/MenuItem;") => {
            let title = match args.first() {
                Some(Value::Obj(s)) => vm.heap.as_str(*s).ok().map(|t| t.to_string()),
                _ => None,
            };
            add_item(
                vm,
                recv,
                ItemSpec {
                    id: 0,
                    group: 0,
                    order: 0,
                    title,
                    visible: true,
                    enabled: true,
                    checkable: false,
                    checked: false,
                    submenu: None,
                },
            )?
        }
        ("add", "(I)Landroid/view/MenuItem;") => {
            let title = res_string(vm, arg_int(0) as u32);
            add_item(
                vm,
                recv,
                ItemSpec {
                    id: 0,
                    group: 0,
                    order: 0,
                    title,
                    visible: true,
                    enabled: true,
                    checkable: false,
                    checked: false,
                    submenu: None,
                },
            )?
        }
        ("add", "(IIII)Landroid/view/MenuItem;") => {
            let title = res_string(vm, arg_int(3) as u32);
            add_item(
                vm,
                recv,
                ItemSpec {
                    id: arg_int(1),
                    group: arg_int(0),
                    order: arg_int(2),
                    title,
                    visible: true,
                    enabled: true,
                    checkable: false,
                    checked: false,
                    submenu: None,
                },
            )?
        }
        ("add", "(IIILjava/lang/CharSequence;)Landroid/view/MenuItem;") => {
            let title = match args.get(3) {
                Some(Value::Obj(s)) => vm.heap.as_str(*s).ok().map(|t| t.to_string()),
                _ => None,
            };
            add_item(
                vm,
                recv,
                ItemSpec {
                    id: arg_int(1),
                    group: arg_int(0),
                    order: arg_int(2),
                    title,
                    visible: true,
                    enabled: true,
                    checkable: false,
                    checked: false,
                    submenu: None,
                },
            )?
        }
        _ => return Ok(None),
    };
    Ok(Some(v))
}

fn item(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let flag =
        |vm: &Engine, f: &str| matches!(vm.heap.get_field(recv, f), Ok(Value::Int(v)) if v != 0);
    let arg_bool = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0) != 0;
    let v = match (name, sig) {
        ("getItemId", "()I") => vm.heap.get_field(recv, "rd_id").unwrap_or(Value::Int(0)),
        ("getGroupId", "()I") => vm.heap.get_field(recv, "rd_group").unwrap_or(Value::Int(0)),
        ("getOrder", "()I") => vm.heap.get_field(recv, "rd_order").unwrap_or(Value::Int(0)),
        ("getTitle", "()Ljava/lang/CharSequence;") => {
            vm.heap.get_field(recv, "rd_title").unwrap_or(Value::Null)
        }
        ("setTitle", "(Ljava/lang/CharSequence;)Landroid/view/MenuItem;") => {
            let t = args.first().cloned().unwrap_or(Value::Null);
            put_field_result(vm.heap.put_field(recv, "rd_title", t))?;
            Value::Obj(recv)
        }
        ("setTitle", "(I)Landroid/view/MenuItem;") => Value::Obj(recv),
        ("setVisible", "(Z)Landroid/view/MenuItem;") => {
            put_field_result(
                vm.heap
                    .put_field(recv, "rd_visible", Value::Int(arg_bool as i32)),
            )?;
            Value::Obj(recv)
        }
        ("isVisible", "()Z") => Value::Int(flag(vm, "rd_visible") as i32),
        ("setEnabled", "(Z)Landroid/view/MenuItem;") => {
            put_field_result(
                vm.heap
                    .put_field(recv, "rd_enabled", Value::Int(arg_bool as i32)),
            )?;
            Value::Obj(recv)
        }
        ("isEnabled", "()Z") => Value::Int(flag(vm, "rd_enabled") as i32),
        ("setChecked", "(Z)Landroid/view/MenuItem;") => {
            put_field_result(
                vm.heap
                    .put_field(recv, "rd_checked", Value::Int(arg_bool as i32)),
            )?;
            Value::Obj(recv)
        }
        ("isChecked", "()Z") => Value::Int(flag(vm, "rd_checked") as i32),
        ("setCheckable", "(Z)Landroid/view/MenuItem;") => {
            put_field_result(
                vm.heap
                    .put_field(recv, "rd_checkable", Value::Int(arg_bool as i32)),
            )?;
            Value::Obj(recv)
        }
        ("isCheckable", "()Z") => Value::Int(flag(vm, "rd_checkable") as i32),
        ("getSubMenu", "()Landroid/view/SubMenu;") => {
            vm.heap.get_field(recv, "rd_submenu").unwrap_or(Value::Null)
        }
        ("hasSubMenu", "()Z") => {
            Value::Int(matches!(vm.heap.get_field(recv, "rd_submenu"), Ok(Value::Obj(_))) as i32)
        }
        ("setIcon", _)
        | ("setShowAsAction", "(I)V")
        | ("setShowAsActionFlags", "(I)Landroid/view/MenuItem;")
        | ("setOnMenuItemClickListener", _)
        | ("setActionView", _)
        | ("setContentDescription", _)
        | ("setTooltipText", _)
        | ("setAlphabeticShortcut", _)
        | ("setNumericShortcut", _) => {
            if sig.ends_with("Landroid/view/MenuItem;") {
                Value::Obj(recv)
            } else {
                Value::Null
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(v))
}
