//! Issue #229 (item 3): the M10 virtual-device shims must be **off** unless
//! virtual hardware was explicitly enabled.
//!
//! `default = ["advanced-components"]` includes `virtual-devices`, so `m10::call_*`
//! used to run unconditionally before `androidx::*`/`gms::*` in the DEX dispatch
//! chain. Every app — with no virtual device configured — therefore got the M10
//! answers for `android.location.*`, shadowing the framework host and splitting
//! state in two: the agent programmed a position through `gms.rs`, the app read
//! it through `m10`.
//!
//! Contract: with virtual hardware off, the M10 shim does not answer; when the
//! owner enables it (`Engine::m10_enable_virtual_devices`), it answers again.

use rd_vm::value::Value;
use rd_vm::{Engine, VmConfig};

const LOCATION_MANAGER: &str = "Landroid/location/LocationManager;";

fn provider(vm: &mut Engine, name: &str) -> Value {
    Value::Obj(vm.heap.alloc_string(name.to_string()).unwrap())
}

#[test]
fn m10_location_shims_are_not_dispatched_with_virtual_hardware_off() {
    let mut vm = Engine::new(vec![], VmConfig::default());
    assert!(
        !vm.fw.vdev.virtual_hardware_enabled,
        "o perfil padrão não tem hardware virtual"
    );
    let lm = vm
        .heap
        .alloc_instance(LOCATION_MANAGER.to_string(), vec![])
        .unwrap();
    let gps = provider(&mut vm, "gps");
    // M10 answers `isProviderEnabled("gps")` with the virtual GPS state. With
    // the gate on, the call must NOT reach m10, so either the framework host
    // answers something else or the gap is reported — never the M10 value
    // while the device is off.
    let off = vm.invoke_instance(
        lm,
        "isProviderEnabled",
        "(Ljava/lang/String;)Z",
        std::slice::from_ref(&gps),
    );
    match off {
        Ok(Value::Int(0)) => {}
        Err(_) => {}
        other => panic!("m10 respondeu com hardware virtual desligado: {other:?}"),
    }
}

#[test]
fn enabling_virtual_hardware_restores_the_m10_answer() {
    let mut vm = Engine::new(vec![], VmConfig::default());
    vm.m10_enable_virtual_devices();
    vm.fw.vdev.location.gps_enabled = true;
    let lm = vm
        .heap
        .alloc_instance(LOCATION_MANAGER.to_string(), vec![])
        .unwrap();
    let gps = provider(&mut vm, "gps");
    let on = vm
        .invoke_instance(
            lm,
            "isProviderEnabled",
            "(Ljava/lang/String;)Z",
            std::slice::from_ref(&gps),
        )
        .expect("com hardware virtual ligado o m10 responde");
    assert_eq!(on, Value::Int(1));
}
