//! Guest-visible `android.util.Printer` wrappers: DEX `invoke-interface`
//! reaches StringBuilderPrinter (including its append-before-null-check
//! quirk), while the stream/writer/log printers delegate to their targets.

mod common;

use common::*;
use rd_vm::value::Value;
use rd_vm::{Engine, HostOut, ObjRef, VmExit};
use std::sync::{Arc, Mutex};

const PROBE: &str = "LPrinterProbe;";
const PRINTER: &str = "Landroid/util/Printer;";
const LOG_PRINTER: &str = "Landroid/util/LogPrinter;";
const STREAM_PRINTER: &str = "Landroid/util/PrintStreamPrinter;";
const WRITER_PRINTER: &str = "Landroid/util/PrintWriterPrinter;";
const BUILDER_PRINTER: &str = "Landroid/util/StringBuilderPrinter;";
const PRINTLN: &str = "(Ljava/lang/String;)V";

fn build_probe() -> DexBuilder {
    let mut b = DexBuilder::new();
    let probe = b.class(PROBE, "Ljava/lang/Object;");
    let println_proto = b.proto_idx("V", vec!["Ljava/lang/String;".to_string()]);
    let println = b.method_idx(PRINTER, println_proto, "println");
    b.direct(
        probe,
        "print",
        "V",
        vec![PRINTER, "Ljava/lang/String;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(3, 2, 2, {
            // regs=3, ins=2: v1=printer, v2=message.
            let mut code = op35c(0x72, 2, println, [1, 2, 0, 0, 0]);
            code.extend(op10x(0x0e)); // return-void
            code
        })),
    );
    b
}

fn instance(engine: &mut Engine, class: &str) -> ObjRef {
    engine
        .heap
        .alloc_instance(class.to_string(), Vec::new())
        .unwrap()
}

fn construct(engine: &mut Engine, class: &str, sig: &str, args: &[Value]) -> ObjRef {
    let obj = instance(engine, class);
    engine.invoke_instance(obj, "<init>", sig, args).unwrap();
    obj
}

fn text(engine: &mut Engine, value: &str) -> Value {
    Value::Obj(engine.heap.alloc_string(value.to_string()).unwrap())
}

fn string_of(engine: &mut Engine, obj: ObjRef) -> String {
    let value = engine
        .invoke_instance(obj, "toString", "()Ljava/lang/String;", &[])
        .unwrap();
    let Value::Obj(s) = value else {
        panic!("toString should return a String, got {value:?}");
    };
    engine.heap.as_str(s).unwrap().to_string()
}

fn print_via_interface(
    engine: &mut Engine,
    printer: ObjRef,
    message: Value,
) -> Result<Value, VmExit> {
    engine.invoke_static(
        PROBE,
        "print",
        "(Landroid/util/Printer;Ljava/lang/String;)V",
        &[Value::Obj(printer), message],
    )
}

#[test]
fn printers_delegate_to_builder_writer_stream_and_log() {
    let mut engine = engine_of(&build_probe());

    // StringBuilderPrinter through the Printer interface, including the
    // conditional newline and the null quirk (append "null", then NPE).
    let builder = construct(&mut engine, "Ljava/lang/StringBuilder;", "()V", &[]);
    let printer = construct(
        &mut engine,
        BUILDER_PRINTER,
        "(Ljava/lang/StringBuilder;)V",
        &[Value::Obj(builder)],
    );
    let hello = text(&mut engine, "hello");
    print_via_interface(&mut engine, printer, hello).unwrap();
    let already = text(&mut engine, "already\n");
    print_via_interface(&mut engine, printer, already).unwrap();
    let empty = text(&mut engine, "");
    print_via_interface(&mut engine, printer, empty).unwrap();
    assert_eq!(string_of(&mut engine, builder), "hello\nalready\n\n");
    let error = print_via_interface(&mut engine, printer, Value::Null).unwrap_err();
    assert!(
        matches!(error, VmExit::Exception(ref t) if t.class == "Ljava/lang/NullPointerException;"),
        "null message should NPE after appending, got {error:?}"
    );
    assert_eq!(string_of(&mut engine, builder), "hello\nalready\n\nnull");

    // PrintWriterPrinter over a StringWriter (flushed before reading).
    let string_writer = construct(&mut engine, "Ljava/io/StringWriter;", "()V", &[]);
    let print_writer = construct(
        &mut engine,
        "Ljava/io/PrintWriter;",
        "(Ljava/io/Writer;)V",
        &[Value::Obj(string_writer)],
    );
    let writer_printer = construct(
        &mut engine,
        WRITER_PRINTER,
        "(Ljava/io/PrintWriter;)V",
        &[Value::Obj(print_writer)],
    );
    let line = text(&mut engine, "writer line");
    engine
        .invoke_instance(writer_printer, "println", PRINTLN, &[line])
        .unwrap();
    engine
        .invoke_instance(print_writer, "flush", "()V", &[])
        .unwrap();
    assert_eq!(string_of(&mut engine, string_writer), "writer line\n");

    // PrintStreamPrinter over a ByteArrayOutputStream.
    let bytes = construct(&mut engine, "Ljava/io/ByteArrayOutputStream;", "()V", &[]);
    let print_stream = construct(
        &mut engine,
        "Ljava/io/PrintStream;",
        "(Ljava/io/OutputStream;)V",
        &[Value::Obj(bytes)],
    );
    let stream_printer = construct(
        &mut engine,
        STREAM_PRINTER,
        "(Ljava/io/PrintStream;)V",
        &[Value::Obj(print_stream)],
    );
    let line = text(&mut engine, "stream line");
    print_via_interface(&mut engine, stream_printer, line).unwrap();
    assert_eq!(string_of(&mut engine, bytes), "stream line\n");

    // LogPrinter formats through the host log channel (captured here).
    let logs: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    rd_vm::set_app_stdout(HostOut::Buffer(logs.clone()));
    let tag = text(&mut engine, "Probe");
    let log_printer = construct(
        &mut engine,
        LOG_PRINTER,
        "(ILjava/lang/String;)V",
        &[Value::Int(3), tag], // Log.DEBUG
    );
    let message = text(&mut engine, "logged");
    let result = engine.invoke_instance(log_printer, "println", PRINTLN, &[message]);
    rd_vm::set_app_stdout(HostOut::Std);
    result.unwrap();
    assert_eq!(
        String::from_utf8(logs.lock().unwrap().clone()).unwrap(),
        "D/Probe: logged\n"
    );
}
