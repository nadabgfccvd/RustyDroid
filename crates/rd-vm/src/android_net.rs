//! `android.net` connectivity: `ConnectivityManager`, `Network`,
//! `NetworkCapabilities` and `NetworkInfo`.
//!
//! The VM presents a single always-on, validated Wi-Fi network — the same
//! assumption `WifiManager` makes (`m8::SERVICES` hands out a connected
//! `WifiInfo`). F-Droid asks for the active network before every repo update
//! and falls back to "no network" if `getActiveNetwork` is missing, so the
//! manager has to answer like a connected device:
//!
//! - `getActiveNetwork` → a `Network` handle (id 1, `getNetworkHandle` = 1)
//! - `getActiveNetworkInfo` → a connected `NetworkInfo` (type `WIFI`)
//! - `getNetworkCapabilities` → `TRANSPORT_WIFI` + `INTERNET` + `VALIDATED`
//! - `isActiveNetworkMetered` → `false` (Wi-Fi, unmetered)
//! - `registerNetworkCallback` fires `onAvailable`/`onCapabilitiesChanged`
//!   immediately (the network is already up) and `unregisterNetworkCallback`
//!   is a no-op — mirroring how the activity manager dispatches lifecycle
//!   callbacks synchronously.
//!
//! `NetworkInfo` is legacy but still what many apps read: `getState`/
//! `getDetailedState` return the AOSP enums, so `NetworkInfo.State.CONNECTED`
//! comparisons work.

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::heap::ObjRef;
use crate::value::Value;

const CONNECTIVITY: &str = "Landroid/net/ConnectivityManager;";
const NETWORK: &str = "Landroid/net/Network;";
const CAPABILITIES: &str = "Landroid/net/NetworkCapabilities;";
const NETWORK_INFO: &str = "Landroid/net/NetworkInfo;";
const INFO_STATE: &str = "Landroid/net/NetworkInfo$State;";
const INFO_DETAILED: &str = "Landroid/net/NetworkInfo$DetailedState;";
const NETWORK_REQUEST: &str = "Landroid/net/NetworkRequest;";

/// `TRANSPORT_WIFI`.
const TRANSPORT_WIFI: i32 = 1;
/// `NetworkCapabilities.NET_CAPABILITY_INTERNET`.
const CAP_INTERNET: i32 = 12;
/// `NetworkCapabilities.NET_CAPABILITY_VALIDATED`.
const CAP_VALIDATED: i32 = 16;
/// `NetworkCapabilities.NET_CAPABILITY_NOT_METERED`.
const CAP_NOT_METERED: i32 = 11;

fn exc(class: &str, msg: impl Into<String>) -> VmExit {
    VmExit::Exception(Throwable::new(class, msg))
}
fn jstr(vm: &mut Engine, s: impl Into<String>) -> Result<Value, VmExit> {
    Ok(Value::Obj(crate::intrinsics::alloc_string(vm, s.into())?))
}
fn alloc(vm: &mut Engine, class: &str, fields: Vec<(&str, Value)>) -> Result<ObjRef, VmExit> {
    let fields: Vec<(String, Value)> = fields
        .into_iter()
        .map(|(name, v)| (name.to_string(), v))
        .collect();
    vm.heap
        .alloc_instance(class.to_string(), fields)
        .map_err(crate::err::oom_throwable)
}
fn obj_arg(args: &[Value], i: usize) -> Option<ObjRef> {
    match args.get(i) {
        Some(Value::Obj(r)) => Some(*r),
        _ => None,
    }
}
fn int_arg(args: &[Value], i: usize) -> i32 {
    args.get(i).and_then(|v| v.as_int().ok()).unwrap_or(0)
}

/// A host enum constant, cached in `vm.statics` like the AOSP surfaces.
fn enum_value(vm: &mut Engine, class: &str, name: &str, ordinal: i32) -> Result<Value, VmExit> {
    let key = (class.to_string(), name.to_string());
    if let Some(v) = vm.statics.get(&key) {
        return Ok(v.clone());
    }
    let r = alloc(
        vm,
        class,
        vec![
            ("rd_name", Value::Null),
            ("rd_ord", Value::Int(ordinal)),
            ("name", Value::Null),
            ("ordinal", Value::Int(ordinal)),
        ],
    )?;
    let n = jstr(vm, name)?;
    let _ = vm.heap.put_field(r, "rd_name", n.clone());
    let _ = vm.heap.put_field(r, "name", n);
    let v = Value::Obj(r);
    vm.statics.insert(key, v.clone());
    Ok(v)
}

fn network(vm: &mut Engine) -> Result<ObjRef, VmExit> {
    alloc(vm, NETWORK, vec![("rd_net_id", Value::Int(1))])
}

fn capabilities(vm: &mut Engine) -> Result<ObjRef, VmExit> {
    alloc(
        vm,
        CAPABILITIES,
        vec![("rd_transport", Value::Int(TRANSPORT_WIFI))],
    )
}

fn network_info(vm: &mut Engine) -> Result<ObjRef, VmExit> {
    alloc(
        vm,
        NETWORK_INFO,
        vec![
            ("rd_type", Value::Int(1)), // ConnectivityManager.TYPE_WIFI
            ("rd_connected", Value::Int(1)),
            ("rd_info", Value::Int(0)), // NON_DEFAULT (0) vs DEFAULT: not modelled
        ],
    )
}

fn connectivity(
    vm: &mut Engine,
    name: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let _ = recv;
    Ok(Some(match name {
        "getActiveNetwork" => Value::Obj(network(vm)?),
        "getActiveNetworkInfo" => Value::Obj(network_info(vm)?),
        "isActiveNetworkMetered" => Value::Int(0),
        "isActiveNetworkValidated" | "isDefaultNetworkActive" => Value::Int(1),
        "getNetworkCapabilities" => Value::Obj(capabilities(vm)?),
        "getLinkProperties" => Value::Null,
        // The device has one always-on network, so a registered callback sees
        // `onAvailable` right away (documented in the module header).
        "registerNetworkCallback" | "registerDefaultNetworkCallback" => {
            if let Some(cb) = obj_arg(args, args.len().saturating_sub(1)) {
                let net = network(vm)?;
                let caps = capabilities(vm)?;
                let _ = vm.invoke_instance(
                    cb,
                    "onAvailable",
                    "(Landroid/net/Network;)V",
                    &[Value::Obj(net)],
                );
                let _ = vm.invoke_instance(
                    cb,
                    "onCapabilitiesChanged",
                    "(Landroid/net/Network;Landroid/net/NetworkCapabilities;)V",
                    &[Value::Obj(net), Value::Obj(caps)],
                );
            }
            Value::Null
        }
        "unregisterNetworkCallback" | "bindProcessToNetwork" | "setProcessDefaultNetwork" => {
            Value::Null
        }
        "getBoundNetworkForProcess" | "getProcessDefaultNetwork" => Value::Obj(network(vm)?),
        "requestNetwork" => Value::Null,
        "getAllNetworks" => {
            let arr = vm
                .heap
                .alloc_array(crate::heap::ElemKind::Obj, 1, NETWORK)
                .map_err(crate::err::oom_throwable)?;
            *vm.heap.array_elems_mut(arr)? = vec![Value::Obj(network(vm)?)];
            Value::Obj(arr)
        }
        "getNetworkInfo" => Value::Obj(network_info(vm)?),
        "getActiveNetworkInfoForUid" => Value::Obj(network_info(vm)?),
        "isTetheringSupported" => Value::Int(0),
        "getRestrictBackgroundStatus" => Value::Int(1),
        _ => return Ok(None),
    }))
}

fn network_instance(
    vm: &mut Engine,
    name: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let id = vm
        .heap
        .get_field(recv, "rd_net_id")
        .ok()
        .and_then(|v| v.as_int().ok())
        .unwrap_or(1);
    Ok(Some(match name {
        "getNetworkHandle" => Value::Long(i64::from(id)),
        "toString" => jstr(vm, format!("Network {{networkId: {id}, netId: {id}}}"))?,
        "hashCode" => Value::Int(id),
        "equals" => Value::Int(i32::from(match args.first() {
            Some(Value::Obj(o)) => {
                vm.heap
                    .get_field(*o, "rd_net_id")
                    .ok()
                    .and_then(|v| v.as_int().ok())
                    == Some(id)
            }
            _ => false,
        })),
        "getNetId" => Value::Int(id),
        _ => return Ok(None),
    }))
}

fn capabilities_instance(
    vm: &mut Engine,
    name: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let transport = vm
        .heap
        .get_field(recv, "rd_transport")
        .ok()
        .and_then(|v| v.as_int().ok())
        .unwrap_or(TRANSPORT_WIFI);
    Ok(Some(match name {
        "hasTransport" => Value::Int(i32::from(int_arg(args, 0) == transport)),
        "hasCapability" => Value::Int(i32::from(matches!(
            int_arg(args, 0),
            CAP_INTERNET | CAP_VALIDATED | CAP_NOT_METERED
        ))),
        "getLinkDownstreamBandwidthKbps" => Value::Int(86_000),
        "getLinkUpstreamBandwidthKbps" => Value::Int(86_000),
        "getTransportInfo" => Value::Int(0),
        "toString" => jstr(
            vm,
            "NetworkCapabilities: INTERNET&VALIDATED&NOT_METERED&WIFI",
        )?,
        _ => return Ok(None),
    }))
}

fn network_info_instance(
    vm: &mut Engine,
    name: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let _ = args;
    let connected = vm
        .heap
        .get_field(recv, "rd_connected")
        .ok()
        .and_then(|v| v.as_int().ok())
        .unwrap_or(1)
        != 0;
    Ok(Some(match name {
        "isConnected" | "isAvailable" | "isConnectedOrConnecting" => {
            Value::Int(i32::from(connected))
        }
        "isFailover" | "isRoaming" | "isRoamingByPlatform" => Value::Int(0),
        "getType" => Value::Int(1),
        "getSubtype" => Value::Int(0),
        "getTypeName" => jstr(vm, "WIFI")?,
        "getSubtypeName" => jstr(vm, "")?,
        "getState" => enum_value(vm, INFO_STATE, "CONNECTED", 1)?,
        "getDetailedState" => enum_value(vm, INFO_DETAILED, "CONNECTED", 7)?,
        "getReason" => jstr(vm, "")?,
        "getExtraInfo" => jstr(vm, "rustydroid")?,
        "reconnect" | "disconnect" => Value::Null,
        "toString" => jstr(vm, "NetworkInfo: type: WIFI[], state: CONNECTED/CONNECTED")?,
        _ => return Ok(None),
    }))
}

/// `NetworkRequest.Builder` — the request object is only carried around.
fn network_request(
    vm: &mut Engine,
    name: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let _ = args;
    Ok(Some(match name {
        "<init>"
        | "addCapability"
        | "addTransportType"
        | "removeCapability"
        | "removeTransportType"
        | "setNetworkSpecifier" => Value::Obj(recv),
        "build" => Value::Obj(alloc(vm, NETWORK_REQUEST, Vec::new())?),
        _ => return Ok(None),
    }))
}

pub(crate) fn instance_intrinsic(
    vm: &mut Engine,
    recv_class: &str,
    name: &str,
    _sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match recv_class {
        CONNECTIVITY => connectivity(vm, name, recv, args),
        NETWORK => network_instance(vm, name, recv, args),
        CAPABILITIES => capabilities_instance(vm, name, recv, args),
        NETWORK_INFO => network_info_instance(vm, name, recv, args),
        NETWORK_REQUEST => network_request(vm, name, recv, args),
        "Landroid/net/NetworkRequest$Builder;" => network_request(vm, name, recv, args),
        INFO_STATE | INFO_DETAILED => Ok(Some(match name {
            "toString" => {
                let n = match vm.heap.get_field(recv, "rd_name") {
                    Ok(Value::Obj(s)) => vm.heap.as_str(s).map(str::to_string).unwrap_or_default(),
                    _ => String::new(),
                };
                jstr(vm, n)?
            }
            "name" => vm.heap.get_field(recv, "rd_name").unwrap_or(Value::Null),
            "ordinal" => vm.heap.get_field(recv, "rd_ord").unwrap_or(Value::Int(0)),
            _ => return Ok(None),
        })),
        // Everything else under `android.net` belongs to another host module
        // (`android_stub_surfaces`, `android_pm`, ...). Declining here lets the
        // dispatch chain continue instead of claiming the class.
        _ => Ok(None),
    }
}
