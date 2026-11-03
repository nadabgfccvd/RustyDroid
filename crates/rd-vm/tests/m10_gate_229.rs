#![cfg(feature = "virtual-devices")]
//! Issue #229, point 3 — `virtual_hardware_enabled` must be honoured by the
//! **bytecode dispatch path**, not only by the intrinsic path.
//!
//! Before this suite, `interp::resolve_or_run_host_invoke` and
//! `interp::call_host_shim_layer` dispatched every M10 submodule
//! unconditionally, while `intrinsics::call_static_intrinsic` already checked
//! `vm.fw.vdev.virtual_hardware_enabled`. An engine that never enabled virtual
//! devices (`rd app run`, unit-test engines) therefore saw a virtual camera,
//! virtual sensors and a virtual location provider — the opposite of the
//! Appendix C absent-hardware profile.
//!
//! The contract now is:
//! 1. **Hardware surfaces** (`m10::is_virtual_hardware_class`) only answer when
//!    the session enabled virtual devices (ADR 011, `rd mcp`, tests that drive
//!    peripherals).
//! 2. **Software surfaces** hosted by the same modules (`java.nio.ByteBuffer`,
//!    clipboard, locale/UI-mode configuration) keep answering either way: they
//!    are not hardware, and gating them is what broke `camera_305` and
//!    `dispatch_315` in the earlier attempts.

mod common;

use common::*;
use rd_vm::m10;
use rd_vm::{Engine, Value};

const LOCATION_MANAGER: &str = "Landroid/location/LocationManager;";
const LOCATION: &str = "Landroid/location/Location;";
const BYTE_BUFFER: &str = "Ljava/nio/ByteBuffer;";

fn object(value: Value) -> rd_vm::ObjRef {
    match value {
        Value::Obj(r) => r,
        other => panic!("expected object, got {other:?}"),
    }
}

/// DEX probe: two static methods that reach the host through **real bytecode**
/// (`invoke-virtual` / `invoke-static`), which is the path issue #229 is about.
fn probe() -> Engine {
    let mut b = DexBuilder::new();

    // Object loc(LocationManager m)
    let p_ret_location = b.proto_idx(LOCATION, vec!["Ljava/lang/String;".into()]);
    let m_get_last = b.method_idx(LOCATION_MANAGER, p_ret_location, "getLastKnownLocation");
    let s_gps = b.intern("gps") as u16;

    // Object bb(int size)
    let p_alloc = b.proto_idx("Ljava/nio/ByteBuffer;", vec!["I".into()]);
    let m_alloc = b.method_idx(BYTE_BUFFER, p_alloc, "allocate");

    let cls = b.class("LProbe;", "Ljava/lang/Object;");
    // regs=2, ins=1 → parameter lands in v1.
    let mut loc_code = op21c(0x1a, 0, s_gps);
    loc_code.extend(op35c(0x6e, 2, m_get_last, [1, 0, 0, 0, 0]));
    loc_code.extend(op11x(0x0c, 0));
    loc_code.extend(op11x(0x11, 0));
    b.direct(
        cls,
        "loc",
        "Ljava/lang/Object;",
        vec![LOCATION_MANAGER],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(2, 1, 2, loc_code)),
    );

    let mut bb_code = op35c(0x71, 1, m_alloc, [1, 0, 0, 0, 0]);
    bb_code.extend(op11x(0x0c, 0));
    bb_code.extend(op11x(0x11, 0));
    b.direct(
        cls,
        "bb",
        "Ljava/lang/Object;",
        vec!["I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(2, 1, 1, bb_code)),
    );

    engine_of(&b)
}

/// The virtual location provider exists only when the session asked for it.
#[test]
fn bytecode_location_provider_is_absent_without_virtual_devices_and_present_with_them() {
    let mut e = probe();
    assert!(
        !e.fw.vdev.virtual_hardware_enabled,
        "a fresh engine must default to the Appendix C absent-hardware profile"
    );
    let manager = e
        .heap
        .alloc_instance(LOCATION_MANAGER.to_string(), Vec::new())
        .expect("alloc LocationManager");

    let without = e.invoke_static(
        "LProbe;",
        "loc",
        "(Landroid/location/LocationManager;)Ljava/lang/Object;",
        &[Value::Obj(manager)],
    );
    let saw_virtual_location = matches!(&without, Ok(Value::Obj(r)) if e.heap.class_of(*r).map(|c| c == LOCATION).unwrap_or(false));
    assert!(
        !saw_virtual_location,
        "M10 location must not answer through bytecode while virtual_hardware_enabled is false, got {without:?}"
    );

    e.m10_enable_virtual_devices();
    let with = e
        .invoke_static(
            "LProbe;",
            "loc",
            "(Landroid/location/LocationManager;)Ljava/lang/Object;",
            &[Value::Obj(manager)],
        )
        .expect("the virtual provider must answer once virtual devices are enabled");
    let fix = object(with);
    assert_eq!(
        e.heap.class_of(fix).expect("class of location"),
        LOCATION,
        "the virtual provider returns a Location"
    );
}

/// Software surfaces that merely *live* in the M10 modules must not be gated:
/// `java.nio.ByteBuffer` is `camera.rs`'s neighbour, not a camera.
#[test]
fn bytecode_software_surfaces_keep_answering_without_virtual_devices() {
    let mut e = probe();
    assert!(!e.fw.vdev.virtual_hardware_enabled);

    let buffer = object(
        e.invoke_static("LProbe;", "bb", "(I)Ljava/lang/Object;", &[Value::Int(16)])
            .expect("ByteBuffer.allocate is software and must answer without virtual devices"),
    );
    assert_eq!(e.heap.class_of(buffer).expect("class"), BYTE_BUFFER);
    // The M10 module hosting `camera.rs` owns the ByteBuffer state, so read it
    // back through the same layer the gate protects.
    assert_eq!(
        m10::call_instance(&mut e, BYTE_BUFFER, "capacity", "()I", buffer, &[])
            .expect("capacity")
            .expect("handled"),
        Value::Int(16)
    );
}

/// The classifier that drives the gate is the contract: hardware prefixes are
/// virtual devices, everything else hosted by M10 is software.
#[test]
fn hardware_class_classifier_covers_the_peripheral_packages() {
    for class in [
        "Landroid/hardware/SensorManager;",
        "Landroid/hardware/camera2/CameraManager;",
        "Landroid/hardware/biometrics/BiometricPrompt;",
        "Landroid/location/LocationManager;",
        "Landroid/bluetooth/BluetoothAdapter;",
        "Landroid/nfc/NfcAdapter;",
        "Landroid/telephony/TelephonyManager;",
        "Landroid/net/wifi/WifiManager;",
        "Landroid/media/AudioRecord;",
        "Landroid/media/MediaPlayer;",
        "Landroid/media/projection/MediaProjectionManager;",
        "Landroid/media/ImageReader;",
        "Landroid/os/BatteryManager;",
        "Landroid/os/PowerManager;",
    ] {
        assert!(
            m10::is_virtual_hardware_class(class),
            "{class} models virtual hardware and must be gated"
        );
    }
    for class in [
        "Ljava/nio/ByteBuffer;",
        "Landroid/content/ClipboardManager;",
        "Landroid/content/ClipData;",
        "Landroid/app/UiModeManager;",
        "Landroid/content/res/Configuration;",
        "Landroid/util/DisplayMetrics;",
    ] {
        assert!(
            !m10::is_virtual_hardware_class(class),
            "{class} is software and must keep answering"
        );
    }
}
