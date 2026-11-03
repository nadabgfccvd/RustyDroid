//! `javax.net` surface: the socket-factory constructors that networking
//! libraries call while **building** their configuration.
//!
//! The VM does not model a network stack yet, and it does not pretend to: the
//! factories exist (so `OkHttpClient.Builder`, `HttpsURLConnection` and friends
//! can finish their constructor), and the first real I/O operation answers the
//! structured `NOT_IMPLEMENTED` gap that the compat matrix already lists.
//!
//! That split matters for the agent: building a client is configuration, and an
//! app that fails during `Application.onCreate` because of a *configuration*
//! call cannot even reach its first screen. An app that fails when it actually
//! opens a socket is an app whose UI the agent can still drive.

use crate::engine::Engine;
use crate::err::{not_implemented, VmExit};
use crate::value::Value;

pub(crate) const SOCKET_FACTORY: &str = "Ljavax/net/SocketFactory;";
pub(crate) const SSL_SOCKET_FACTORY: &str = "Ljavax/net/ssl/SSLSocketFactory;";

/// `getDefault()` for both factories. The instance carries no state: it only
/// has to be a non-null object of the right class, which is what the caller
/// stores and later uses to open sockets.
pub(crate) fn static_intrinsic(
    vm: &mut Engine,
    class: &str,
    name: &str,
    _sig: &str,
) -> Result<Option<Value>, VmExit> {
    let target = match (class, name) {
        (SOCKET_FACTORY, "getDefault") => SOCKET_FACTORY,
        (SSL_SOCKET_FACTORY, "getDefault") => SSL_SOCKET_FACTORY,
        _ => return Ok(None),
    };
    let r = vm.heap.alloc_instance(target.to_string(), Vec::new())?;
    Ok(Some(Value::Obj(r)))
}

pub(crate) fn instance_intrinsic(
    _vm: &mut Engine,
    recv_class: &str,
    name: &str,
    _sig: &str,
) -> Result<Option<Value>, VmExit> {
    if recv_class != SOCKET_FACTORY && recv_class != SSL_SOCKET_FACTORY {
        return Ok(None);
    }
    if name == "createSocket" {
        // First real I/O: the structured gap, with the reason and the path
        // forward (Lei 1 — never a fake socket, never a silent null).
        return Err(not_implemented(format!(
            "{recv_class}->createSocket: o runtime não modela a pilha de rede TCP/TLS"
        ))
        .into());
    }
    Ok(None)
}
