//! `<item>` with a `<menu>` child becomes a `SubMenu`, and `Menu.addSubMenu`
//! builds one in code. Covers: XML submenus and their items, the group
//! flattening next to a submenu, programmatic `addSubMenu` with the two-way
//! `MenuItem.getSubMenu` / `SubMenu.getItem` link, and the nesting cap.

#[macro_use]
mod common;

use common::apkfix::{a, build_apk, build_axml, ArscFix, AxNode, AxVal};
use common::*;
use rd_vm::framework::HostObj;
use rd_vm::value::Value;

const NEST_LEVELS: usize = 9; // items at levels 0..=8; level 8 is past the cap (8)

fn item(id: i32, title: &str, children: Vec<AxNode>) -> AxNode {
    let menu = if children.is_empty() {
        vec![]
    } else {
        vec![AxNode::new("menu", vec![], children)]
    };
    AxNode::new(
        "item",
        vec![
            a(true, "id", AxVal::Int(id)),
            a(true, "title", AxVal::Str(title.to_string())),
        ],
        menu,
    )
}

fn engine_with_menu(menu_root: AxNode) -> (rd_vm::Engine, u32) {
    let mut arsc = ArscFix::with_package("com.test.menu");
    let res = arsc.add_menu_file("main");
    let xml = build_axml(&menu_root);
    let apk = build_apk(
        "com.test.menu",
        "LMenuMain;",
        arsc.build(),
        vec![("res/menu/main.xml", xml)],
    );
    let mut e = engine_of(&DexBuilder::new());
    e.set_resources(rd_apk::Apk::from_bytes(apk).expect("synthetic APK parses"));
    (e, res)
}

fn new_menu(e: &mut rd_vm::Engine) -> rd_vm::heap::ObjRef {
    let m = e
        .heap
        .alloc_instance("Landroid/view/Menu;".into(), vec![])
        .unwrap();
    e.fw.objects.insert(m, HostObj::Menu { items: Vec::new() });
    m
}

fn call_obj(
    e: &mut rd_vm::Engine,
    recv: rd_vm::heap::ObjRef,
    name: &str,
    sig: &str,
    args: &[Value],
) -> rd_vm::heap::ObjRef {
    match e.invoke_instance(recv, name, sig, args).expect(name) {
        Value::Obj(o) => o,
        other => panic!("{name} did not return an object: {other:?}"),
    }
}

fn call_int(
    e: &mut rd_vm::Engine,
    recv: rd_vm::heap::ObjRef,
    name: &str,
    sig: &str,
    args: &[Value],
) -> i32 {
    match e.invoke_instance(recv, name, sig, args).expect(name) {
        Value::Int(i) => i,
        other => panic!("{name} did not return an int: {other:?}"),
    }
}

fn inflate(e: &mut rd_vm::Engine, res: u32) -> rd_vm::heap::ObjRef {
    let menu = new_menu(e);
    let inflater = e
        .heap
        .alloc_instance("Landroid/view/MenuInflater;".into(), vec![])
        .unwrap();
    e.invoke_instance(
        inflater,
        "inflate",
        "(ILandroid/view/Menu;)V",
        &[Value::Int(res as i32), Value::Obj(menu)],
    )
    .expect("MenuInflater.inflate");
    menu
}

#[test]
fn xml_item_with_menu_child_becomes_a_submenu() {
    let root = AxNode::new(
        "menu",
        vec![],
        vec![
            item(1, "Abrir", vec![]),
            item(
                2,
                "Mais",
                vec![item(3, "Copiar", vec![]), item(4, "Colar", vec![])],
            ),
            AxNode::new(
                "group",
                vec![a(true, "id", AxVal::Int(9))],
                vec![item(5, "Sair", vec![])],
            ),
        ],
    );
    let (mut e, res) = engine_with_menu(root);
    let menu = inflate(&mut e, res);

    // The submenu's parent item is a plain entry of the top menu; the group
    // is flattened next to it.
    assert_eq!(call_int(&mut e, menu, "size", "()I", &[]), 3);
    let ids: Vec<i32> = (0..3)
        .map(|i| {
            let it = call_obj(
                &mut e,
                menu,
                "getItem",
                "(I)Landroid/view/MenuItem;",
                &[Value::Int(i)],
            );
            call_int(&mut e, it, "getItemId", "()I", &[])
        })
        .collect();
    assert_eq!(ids, vec![1, 2, 5]);

    let parent = call_obj(
        &mut e,
        menu,
        "getItem",
        "(I)Landroid/view/MenuItem;",
        &[Value::Int(1)],
    );
    assert_eq!(call_int(&mut e, parent, "hasSubMenu", "()Z", &[]), 1);
    let sub = call_obj(
        &mut e,
        parent,
        "getSubMenu",
        "()Landroid/view/SubMenu;",
        &[],
    );
    assert_eq!(call_int(&mut e, sub, "size", "()I", &[]), 2);
    let child_ids: Vec<i32> = (0..2)
        .map(|i| {
            let it = call_obj(
                &mut e,
                sub,
                "getItem",
                "(I)Landroid/view/MenuItem;",
                &[Value::Int(i)],
            );
            call_int(&mut e, it, "getItemId", "()I", &[])
        })
        .collect();
    assert_eq!(child_ids, vec![3, 4]);

    // A plain item has no submenu.
    let plain = call_obj(
        &mut e,
        menu,
        "getItem",
        "(I)Landroid/view/MenuItem;",
        &[Value::Int(0)],
    );
    assert_eq!(call_int(&mut e, plain, "hasSubMenu", "()Z", &[]), 0);
}

#[test]
fn xml_submenu_nesting_is_capped() {
    // Items at levels 0..=8, each level nested in the previous one's <menu>.
    // Levels 0..8 hold a submenu; the level-8 item is past the cap (8) and
    // must not carry one, which bounds hostile XML.
    let mut node = item((NEST_LEVELS - 1) as i32, "L8", vec![]);
    for lvl in (0..NEST_LEVELS - 1).rev() {
        node = item(lvl as i32, &format!("L{lvl}"), vec![node]);
    }
    let root = AxNode::new("menu", vec![], vec![node]);
    let (mut e, res) = engine_with_menu(root);
    let mut m = inflate(&mut e, res);
    for _ in 0..NEST_LEVELS - 1 {
        let it = call_obj(
            &mut e,
            m,
            "getItem",
            "(I)Landroid/view/MenuItem;",
            &[Value::Int(0)],
        );
        assert_eq!(call_int(&mut e, it, "hasSubMenu", "()Z", &[]), 1);
        m = call_obj(&mut e, it, "getSubMenu", "()Landroid/view/SubMenu;", &[]);
    }
    let deepest = call_obj(
        &mut e,
        m,
        "getItem",
        "(I)Landroid/view/MenuItem;",
        &[Value::Int(0)],
    );
    assert_eq!(
        call_int(&mut e, deepest, "getItemId", "()I", &[]),
        (NEST_LEVELS - 1) as i32
    );
    assert_eq!(call_int(&mut e, deepest, "hasSubMenu", "()Z", &[]), 0);
}

#[test]
fn programmatic_add_submenu_links_both_ways() {
    let (mut e, _res) = engine_with_menu(AxNode::new("menu", vec![], vec![]));
    let menu = new_menu(&mut e);
    let title = e.heap.alloc_string("Mais".into()).unwrap();
    let sub = call_obj(
        &mut e,
        menu,
        "addSubMenu",
        "(Ljava/lang/CharSequence;)Landroid/view/SubMenu;",
        &[Value::Obj(title)],
    );
    assert_eq!(call_int(&mut e, menu, "size", "()I", &[]), 1);

    let parent = call_obj(
        &mut e,
        menu,
        "getItem",
        "(I)Landroid/view/MenuItem;",
        &[Value::Int(0)],
    );
    assert_eq!(call_int(&mut e, parent, "hasSubMenu", "()Z", &[]), 1);
    let via_item = call_obj(
        &mut e,
        parent,
        "getSubMenu",
        "()Landroid/view/SubMenu;",
        &[],
    );
    assert_eq!(
        via_item, sub,
        "MenuItem.getSubMenu returns the SubMenu that addSubMenu gave"
    );
    let back = call_obj(&mut e, sub, "getItem", "()Landroid/view/MenuItem;", &[]);
    assert_eq!(
        back, parent,
        "SubMenu.getItem returns the item that opens it"
    );

    // Items added to the submenu live in the submenu, not in the top menu.
    let t = e.heap.alloc_string("Copiar".into()).unwrap();
    call_obj(
        &mut e,
        sub,
        "add",
        "(Ljava/lang/CharSequence;)Landroid/view/MenuItem;",
        &[Value::Obj(t)],
    );
    assert_eq!(call_int(&mut e, sub, "size", "()I", &[]), 1);
    assert_eq!(call_int(&mut e, menu, "size", "()I", &[]), 1);
}
