//! `javax.net.SocketFactory` / `SSLSocketFactory`: construction is configuration,
//! I/O is the modelled gap.
//!
//! OkHttp builds its client eagerly in `Application.onCreate`
//! (`new OkHttpClient.Builder()` stores `SocketFactory.getDefault()`), so a
//! `NOT_IMPLEMENTED` at construction time stopped the app before its first
//! screen — the agent could not even see the UI. The contract below is the one
//! the compat matrix promises: the factory exists and is a real object of the
//! right class; the first `createSocket` answers the structured gap instead of
//! returning a fake socket or a silent null.

use rd_vm::err::VmExit;
use rd_vm::value::Value;
use rd_vm::{Engine, VmConfig};

const SOCKET_FACTORY: &str = "Ljavax/net/SocketFactory;";
const SSL_SOCKET_FACTORY: &str = "Ljavax/net/ssl/SSLSocketFactory;";

fn engine() -> Engine {
    Engine::new(vec![], VmConfig::default())
}

#[test]
fn get_default_returns_an_instance_of_the_requested_factory() {
    for class in [SOCKET_FACTORY, SSL_SOCKET_FACTORY] {
        let mut e = engine();
        let v = rd_vm::intrinsics::call_static_intrinsic(
            &mut e,
            class,
            "getDefault",
            "()Ljava/net/SocketFactory;",
            &[],
        )
        .expect("getDefault executa")
        .expect("getDefault responde");
        let Value::Obj(r) = v else {
            panic!("{class}: esperava objeto, veio {v:?}")
        };
        assert_eq!(e.heap.class_of(r).unwrap(), class);
    }
}

#[test]
fn ssl_socket_factory_is_a_socket_factory() {
    let mut e = engine();
    let v = rd_vm::intrinsics::call_static_intrinsic(
        &mut e,
        SSL_SOCKET_FACTORY,
        "getDefault",
        "()Ljavax/net/SocketFactory;",
        &[],
    )
    .unwrap()
    .unwrap();
    let Value::Obj(r) = v else { panic!() };
    // okhttp assigns the SSLSocketFactory to a SocketFactory field.
    assert!(e.cp.is_subtype(e.heap.class_of(r).unwrap(), SOCKET_FACTORY));
}

#[test]
fn create_socket_reports_the_structured_network_gap() {
    let mut e = engine();
    let factory = rd_vm::intrinsics::call_static_intrinsic(
        &mut e,
        SOCKET_FACTORY,
        "getDefault",
        "()Ljavax/net/SocketFactory;",
        &[],
    )
    .unwrap()
    .unwrap();
    let Value::Obj(r) = factory else { panic!() };
    let host = e.heap.alloc_string("example.org".to_string()).unwrap();
    match e.invoke_instance(
        r,
        "createSocket",
        "(Ljava/lang/String;I)Ljava/net/Socket;",
        &[Value::Obj(host), Value::Int(443)],
    ) {
        Err(VmExit::Error(err)) => {
            assert_eq!(err.code, "NOT_IMPLEMENTED");
            assert!(err.cause.contains("pilha de rede"), "{}", err.cause);
        }
        other => panic!("esperava o gap estruturado de rede, veio {other:?}"),
    }
}
