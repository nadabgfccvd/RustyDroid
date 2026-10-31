//! Regression tests for issue #370: `XmlResourceParser.next()` returns
//! END_DOCUMENT idempotently, and `nextTag()` throws at END_DOCUMENT.

mod common;

use common::apkfix::{build_apk, build_axml, ArscFix, AxNode};
use common::*;
use rd_vm::value::Value;
use rd_vm::{Engine, ObjRef};

const RESOURCES: &str = "Landroid/content/res/Resources;";
const NEXT_I: &str = "()I";

fn resources(engine: &mut Engine) -> ObjRef {
    engine
        .heap
        .alloc_instance(RESOURCES.to_string(), Vec::new())
        .unwrap()
}

/// `<LinearLayout><TextView/></LinearLayout>` as `res/layout/root.xml`.
fn engine_with_layout() -> (Engine, i32) {
    let mut arsc = ArscFix::new();
    let rid = arsc.add_layout_file("root");
    let root = AxNode::new(
        "LinearLayout",
        vec![],
        vec![AxNode::new("TextView", vec![], vec![])],
    );
    let apk = build_apk(
        "com.test.xml370",
        "LMain;",
        arsc.build(),
        vec![("res/layout/root.xml", build_axml(&root))],
    );
    let mut engine = engine_of(&DexBuilder::new());
    engine.set_resources(rd_apk::Apk::from_bytes(apk).expect("APK sintético parseia"));
    (engine, rid as i32)
}

fn open_parser(engine: &mut Engine, id: i32) -> ObjRef {
    let res = resources(engine);
    match engine
        .invoke_instance(
            res,
            "getXml",
            "(I)Landroid/content/res/XmlResourceParser;",
            &[Value::Int(id)],
        )
        .unwrap_or_else(|e| panic!("getXml failed: {e:?}"))
    {
        Value::Obj(p) => p,
        other => panic!("expected parser, got {other:?}"),
    }
}

fn step(engine: &mut Engine, parser: ObjRef, name: &str) -> Value {
    engine.invoke_instance(parser, name, NEXT_I, &[]).unwrap()
}

#[test]
fn next_keeps_returning_end_document_after_the_end() {
    let (mut engine, id) = engine_with_layout();
    let p = open_parser(&mut engine, id);
    // START_TAG, START_TAG, END_TAG, END_TAG, then END_DOCUMENT forever.
    assert_eq!(step(&mut engine, p, "next"), Value::Int(2));
    assert_eq!(step(&mut engine, p, "next"), Value::Int(2));
    assert_eq!(step(&mut engine, p, "next"), Value::Int(3));
    assert_eq!(step(&mut engine, p, "next"), Value::Int(3));
    assert_eq!(step(&mut engine, p, "next"), Value::Int(1));
    assert_eq!(step(&mut engine, p, "next"), Value::Int(1));
    assert_eq!(step(&mut engine, p, "nextToken"), Value::Int(1));
}

#[test]
fn next_tag_skips_to_tags_and_throws_at_end_document() {
    let (mut engine, id) = engine_with_layout();
    let p = open_parser(&mut engine, id);
    assert_eq!(step(&mut engine, p, "nextTag"), Value::Int(2));
    assert_eq!(step(&mut engine, p, "nextTag"), Value::Int(2));
    assert_eq!(step(&mut engine, p, "nextTag"), Value::Int(3));
    assert_eq!(step(&mut engine, p, "nextTag"), Value::Int(3));
    // Past the last tag, nextTag() must throw XmlPullParserException.
    assert!(engine.invoke_instance(p, "nextTag", NEXT_I, &[]).is_err());
}
