//! Regression suite for issues `#229` and `#232`:
//! - `#229`: `m10::location` must never hijack `com.google.android.gms.tasks.Task` /
//!   `Tasks` or split `FusedLocationProviderClient` / `androidx.biometric` state,
//!   and `m10` dispatch must honor `virtual_hardware_enabled`.
//! - `#232`: `Engine::invoke_static`, `Engine::invoke_instance`, `Engine::new_instance`,
//!   and `interp::inherited_android_host` must cover `m9`, `m10`, `androidx`, `gms`,
//!   `ViewModel`, `Landroidx/*`, and `Lcom/google/android/material/*` superclasses.

use rd_testsupport::vm::*;
use rd_vm::androidx::BiometricOutcome as AxBiometricOutcome;
use rd_vm::{Engine, ObjRef, Value, VmConfig, VmExit};

fn jstr(e: &mut Engine, s: &str) -> Value {
    Value::Obj(e.heap.alloc_string(s.to_string()).expect("alloc_string"))
}

fn as_obj(v: Value) -> ObjRef {
    match v {
        Value::Obj(r) => r,
        other => panic!("expected Value::Obj, got {other:?}"),
    }
}

#[test]
fn issue_229_dex_gms_tasks_fcm_and_exception_work_with_and_without_m10() {
    // Build real Dalvik bytecode executing GMS/Firebase calls through `interp::do_invoke`:
    //   Probe.fcmToken():
    //     v0 = FirebaseMessaging.getInstance()
    //     v0 = v0.getToken()
    //     v0 = v0.getResult()
    //     return (String) v0
    //   Probe.failedTaskIsSuccessful():
    //     v0 = new IllegalStateException("boom")
    //     v0 = Tasks.forException(v0)
    //     return v0.isSuccessful()
    //   Probe.failedTaskException():
    //     v0 = new IllegalStateException("boom")
    //     v0 = Tasks.forException(v0)
    //     return v0.getException()
    //   Probe.pendingTaskIsComplete():
    //     v0 = new TaskCompletionSource()
    //     v0 = v0.getTask()
    //     return v0.isComplete()
    let mut b = DexBuilder::new();
    let probe = b.class("LProbe;", "Ljava/lang/Object;");

    let fcm_cls = "Lcom/google/firebase/messaging/FirebaseMessaging;";
    let task_cls = "Lcom/google/android/gms/tasks/Task;";
    let tasks_cls = "Lcom/google/android/gms/tasks/Tasks;";
    let tcs_cls = "Lcom/google/android/gms/tasks/TaskCompletionSource;";
    let ise_cls = "Ljava/lang/IllegalStateException;";

    let p_get_inst = b.proto_idx(fcm_cls, vec![]);
    let m_get_inst = b.method_idx(fcm_cls, p_get_inst, "getInstance");
    let p_get_tok = b.proto_idx(task_cls, vec![]);
    let m_get_tok = b.method_idx(fcm_cls, p_get_tok, "getToken");
    let p_get_res = b.proto_idx("Ljava/lang/Object;", vec![]);
    let m_get_res = b.method_idx(task_cls, p_get_res, "getResult");
    let t_str = b.type_idx("Ljava/lang/String;");

    let mut u_fcm = Vec::new();
    u_fcm.extend(op35c(0x71, 0, m_get_inst, [0; 5]));
    u_fcm.extend(op11x(0x0c, 0));
    u_fcm.extend(op35c(0x6e, 1, m_get_tok, [0, 0, 0, 0, 0]));
    u_fcm.extend(op11x(0x0c, 0));
    u_fcm.extend(op35c(0x6e, 1, m_get_res, [0, 0, 0, 0, 0]));
    u_fcm.extend(op11x(0x0c, 0));
    u_fcm.extend(op21c(0x1f, 0, t_str));
    u_fcm.extend(op11x(0x11, 0));
    let c_fcm = b.code(1, 0, 1, u_fcm);

    let t_ise = b.type_idx(ise_cls);
    let s_boom = b.intern("boom") as u16;
    let p_ise_init = b.proto_idx("V", vec!["Ljava/lang/String;".into()]);
    let m_ise_init = b.method_idx(ise_cls, p_ise_init, "<init>");
    let p_for_exc = b.proto_idx(task_cls, vec!["Ljava/lang/Exception;".into()]);
    let m_for_exc = b.method_idx(tasks_cls, p_for_exc, "forException");
    let p_bool = b.proto_idx("Z", vec![]);
    let m_is_succ = b.method_idx(task_cls, p_bool, "isSuccessful");
    let m_is_comp = b.method_idx(task_cls, p_bool, "isComplete");
    let p_get_exc = b.proto_idx("Ljava/lang/Exception;", vec![]);
    let m_get_exc = b.method_idx(task_cls, p_get_exc, "getException");

    let mut u_fail_ok = Vec::new();
    u_fail_ok.extend(op21c(0x22, 0, t_ise));
    u_fail_ok.extend(op21c(0x1a, 1, s_boom));
    u_fail_ok.extend(op35c(0x70, 2, m_ise_init, [0, 1, 0, 0, 0]));
    u_fail_ok.extend(op35c(0x71, 1, m_for_exc, [0, 0, 0, 0, 0]));
    u_fail_ok.extend(op11x(0x0c, 0));
    u_fail_ok.extend(op35c(0x6e, 1, m_is_succ, [0, 0, 0, 0, 0]));
    u_fail_ok.extend(op11x(0x0a, 0));
    u_fail_ok.extend(op11x(0x0f, 0));
    let c_fail_ok = b.code(2, 0, 2, u_fail_ok);

    let mut u_fail_exc = Vec::new();
    u_fail_exc.extend(op21c(0x22, 0, t_ise));
    u_fail_exc.extend(op21c(0x1a, 1, s_boom));
    u_fail_exc.extend(op35c(0x70, 2, m_ise_init, [0, 1, 0, 0, 0]));
    u_fail_exc.extend(op35c(0x71, 1, m_for_exc, [0, 0, 0, 0, 0]));
    u_fail_exc.extend(op11x(0x0c, 0));
    u_fail_exc.extend(op35c(0x6e, 1, m_get_exc, [0, 0, 0, 0, 0]));
    u_fail_exc.extend(op11x(0x0c, 0));
    u_fail_exc.extend(op11x(0x11, 0));
    let c_fail_exc = b.code(2, 0, 2, u_fail_exc);

    let mut u_fail_throw = Vec::new();
    u_fail_throw.extend(op21c(0x22, 0, t_ise));
    u_fail_throw.extend(op21c(0x1a, 1, s_boom));
    u_fail_throw.extend(op35c(0x70, 2, m_ise_init, [0, 1, 0, 0, 0]));
    u_fail_throw.extend(op35c(0x71, 1, m_for_exc, [0, 0, 0, 0, 0]));
    u_fail_throw.extend(op11x(0x0c, 0));
    u_fail_throw.extend(op35c(0x6e, 1, m_get_res, [0, 0, 0, 0, 0]));
    u_fail_throw.extend(op11x(0x0c, 0));
    u_fail_throw.extend(op11x(0x11, 0));
    let c_fail_throw = b.code(2, 0, 2, u_fail_throw);

    let t_tcs = b.type_idx(tcs_cls);
    let p_void = b.proto_idx("V", vec![]);
    let m_tcs_init = b.method_idx(tcs_cls, p_void, "<init>");
    let m_get_task = b.method_idx(tcs_cls, p_get_tok, "getTask");
    let mut u_pend = Vec::new();
    u_pend.extend(op21c(0x22, 0, t_tcs));
    u_pend.extend(op35c(0x70, 1, m_tcs_init, [0, 0, 0, 0, 0]));
    u_pend.extend(op35c(0x6e, 1, m_get_task, [0, 0, 0, 0, 0]));
    u_pend.extend(op11x(0x0c, 0));
    u_pend.extend(op35c(0x6e, 1, m_is_comp, [0, 0, 0, 0, 0]));
    u_pend.extend(op11x(0x0a, 0));
    u_pend.extend(op11x(0x0f, 0));
    let c_pend = b.code(1, 0, 1, u_pend);

    // Lexical order for direct methods on `LProbe;`:
    b.direct(
        probe,
        "failedTaskException",
        "Ljava/lang/Exception;",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(c_fail_exc),
    );
    b.direct(
        probe,
        "failedTaskGetResultThrows",
        "Ljava/lang/Object;",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(c_fail_throw),
    );
    b.direct(
        probe,
        "failedTaskIsSuccessful",
        "Z",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(c_fail_ok),
    );
    b.direct(
        probe,
        "fcmToken",
        "Ljava/lang/String;",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(c_fcm),
    );
    b.direct(
        probe,
        "pendingTaskIsComplete",
        "Z",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(c_pend),
    );

    let dex_bytes = b.finish();
    for enable_m10 in [false, true] {
        let dex = rd_dex::Dex::parse(dex_bytes.clone()).expect("parse dex");
        let mut e = Engine::new(vec![dex], VmConfig::default());
        #[cfg(feature = "virtual-devices")]
        if enable_m10 {
            e.m10_enable_virtual_devices();
        }

        let tok = e
            .invoke_static("LProbe;", "fcmToken", "()Ljava/lang/String;", &[])
            .expect("fcmToken");
        let tok_ref = as_obj(tok);
        let tok_str = e.heap.as_str(tok_ref).expect("token str");
        assert!(!tok_str.is_empty(), "FCM token must be non-empty");

        assert_eq!(
            e.invoke_static("LProbe;", "failedTaskIsSuccessful", "()Z", &[])
                .expect("failedTaskIsSuccessful"),
            Value::Int(0),
            "Tasks.forException must report isSuccessful() == false (enable_m10={enable_m10})"
        );

        let exc_val = e
            .invoke_static(
                "LProbe;",
                "failedTaskException",
                "()Ljava/lang/Exception;",
                &[],
            )
            .expect("failedTaskException");
        let exc_ref = as_obj(exc_val);
        assert_eq!(
            e.heap.class_of(exc_ref).expect("class_of"),
            "Ljava/lang/IllegalStateException;"
        );

        match e.invoke_static(
            "LProbe;",
            "failedTaskGetResultThrows",
            "()Ljava/lang/Object;",
            &[],
        ) {
            Err(VmExit::Exception(t)) => {
                assert_eq!(
                    t.class,
                    "Lcom/google/android/gms/tasks/RuntimeExecutionException;"
                );
            }
            other => panic!("expected RuntimeExecutionException, got {other:?}"),
        }

        assert_eq!(
            e.invoke_static("LProbe;", "pendingTaskIsComplete", "()Z", &[])
                .expect("pendingTaskIsComplete"),
            Value::Int(0),
            "pending TaskCompletionSource must report isComplete() == false"
        );
    }
}

#[cfg(feature = "virtual-devices")]
#[test]
fn issue_229_fused_location_and_biometric_bridge_through_dex() {
    // DEX bytecode calling LocationServices.getFusedLocationProviderClient(ctx)
    // -> getLastLocation() -> getResult() -> Location.getLatitude(), plus
    // BiometricManager.from(ctx).canAuthenticate().
    let mut b = DexBuilder::new();
    let probe = b.class("LLocBioProbe;", "Ljava/lang/Object;");

    let loc_svcs = "Lcom/google/android/gms/location/LocationServices;";
    let fused_cls = "Lcom/google/android/gms/location/FusedLocationProviderClient;";
    let task_cls = "Lcom/google/android/gms/tasks/Task;";
    let loc_cls = "Landroid/location/Location;";
    let bm_cls = "Landroidx/biometric/BiometricManager;";

    let p_fused = b.proto_idx(fused_cls, vec!["Landroid/content/Context;".into()]);
    let m_fused = b.method_idx(loc_svcs, p_fused, "getFusedLocationProviderClient");
    let p_last = b.proto_idx(task_cls, vec![]);
    let m_last = b.method_idx(fused_cls, p_last, "getLastLocation");
    let p_res = b.proto_idx("Ljava/lang/Object;", vec![]);
    let m_res = b.method_idx(task_cls, p_res, "getResult");
    let t_loc = b.type_idx(loc_cls);
    let p_lat = b.proto_idx("D", vec![]);
    let m_lat = b.method_idx(loc_cls, p_lat, "getLatitude");

    // regs: v0, v1 (wide double), v2 = ctx
    let mut u_lat = Vec::new();
    u_lat.extend(op35c(0x71, 1, m_fused, [2, 0, 0, 0, 0]));
    u_lat.extend(op11x(0x0c, 0));
    u_lat.extend(op35c(0x6e, 1, m_last, [0, 0, 0, 0, 0]));
    u_lat.extend(op11x(0x0c, 0));
    u_lat.extend(op35c(0x6e, 1, m_res, [0, 0, 0, 0, 0]));
    u_lat.extend(op11x(0x0c, 0));
    u_lat.extend(op21c(0x1f, 0, t_loc));
    u_lat.extend(op35c(0x6e, 1, m_lat, [0, 0, 0, 0, 0]));
    u_lat.extend(op11x(0x0b, 0)); // move-result-wide v0..v1
    u_lat.extend(op11x(0x10, 0)); // return-wide v0
    let c_lat = b.code(3, 1, 1, u_lat);

    let p_bm_from = b.proto_idx(bm_cls, vec!["Landroid/content/Context;".into()]);
    let m_bm_from = b.method_idx(bm_cls, p_bm_from, "from");
    let p_can = b.proto_idx("I", vec![]);
    let m_can = b.method_idx(bm_cls, p_can, "canAuthenticate");
    let mut u_bio = Vec::new();
    u_bio.extend(op35c(0x71, 1, m_bm_from, [1, 0, 0, 0, 0]));
    u_bio.extend(op11x(0x0c, 0));
    u_bio.extend(op35c(0x6e, 1, m_can, [0, 0, 0, 0, 0]));
    u_bio.extend(op11x(0x0a, 0));
    u_bio.extend(op11x(0x0f, 0));
    let c_bio = b.code(2, 1, 1, u_bio);

    b.direct(
        probe,
        "canAuth",
        "I",
        vec!["Landroid/content/Context;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(c_bio),
    );
    b.direct(
        probe,
        "fusedLat",
        "D",
        vec!["Landroid/content/Context;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(c_lat),
    );

    let mut e = engine_of(&b);
    let app = e.new_instance("Landroid/app/Application;").expect("app");

    // Default `virtual_hardware_enabled == false`: BiometricManager uses Appendix D default (12 = NO_HARDWARE).
    assert_eq!(
        e.invoke_static(
            "LLocBioProbe;",
            "canAuth",
            "(Landroid/content/Context;)I",
            &[Value::Obj(app)],
        )
        .expect("canAuth default"),
        Value::Int(12)
    );

    // Enable M10 + set Diamantina fix (-18.2444, -43.6003): FusedLocationProviderClient + Location.getLatitude() agree!
    e.m10_enable_virtual_devices();
    e.m10_set_location(-18.2444, -43.6003, Some(1280.0), Some(4.0), None, None)
        .expect("m10_set_location");
    let Value::Double(lat) = e
        .invoke_static(
            "LLocBioProbe;",
            "fusedLat",
            "(Landroid/content/Context;)D",
            &[Value::Obj(app)],
        )
        .expect("fusedLat")
    else {
        panic!("expected Double latitude");
    };
    assert!((lat - (-18.2444)).abs() < 1e-4, "got lat={lat}");

    // Programming `androidx_set_biometric` updates M10 biometrics too (zero split-brain).
    e.androidx_set_biometric(11, AxBiometricOutcome::Failed);
    assert_eq!(
        e.invoke_static(
            "LLocBioProbe;",
            "canAuth",
            "(Landroid/content/Context;)I",
            &[Value::Obj(app)],
        )
        .expect("canAuth 11"),
        Value::Int(11)
    );
    e.androidx_set_biometric(0, AxBiometricOutcome::Success);
    assert_eq!(
        e.invoke_static(
            "LLocBioProbe;",
            "canAuth",
            "(Landroid/content/Context;)I",
            &[Value::Obj(app)],
        )
        .expect("canAuth 0"),
        Value::Int(0)
    );
}

#[test]
fn issue_232_engine_new_instance_and_invoke_instance_cover_gms_and_androidx() {
    let mut e = Engine::new(Vec::new(), VmConfig::default());

    // Exact reproducer #1 from #232: `vm.new_instance("Lcom/google/android/gms/tasks/TaskCompletionSource;")`
    let tcs = e
        .new_instance("Lcom/google/android/gms/tasks/TaskCompletionSource;")
        .expect("new_instance TaskCompletionSource");
    let task = as_obj(
        e.invoke_instance(tcs, "getTask", "()Lcom/google/android/gms/tasks/Task;", &[])
            .expect("tcs.getTask"),
    );
    assert_eq!(
        e.invoke_instance(task, "isComplete", "()Z", &[])
            .expect("isComplete before"),
        Value::Int(0)
    );
    let payload = jstr(&mut e, "hello-232");
    e.invoke_instance(
        tcs,
        "setResult",
        "(Ljava/lang/Object;)V",
        std::slice::from_ref(&payload),
    )
    .expect("tcs.setResult");
    assert_eq!(
        e.invoke_instance(task, "isComplete", "()Z", &[])
            .expect("isComplete after"),
        Value::Int(1)
    );
    assert_eq!(
        e.invoke_instance(task, "getResult", "()Ljava/lang/Object;", &[])
            .expect("task.getResult"),
        payload
    );

    // AndroidX ArrayMap via `new_instance` + `invoke_instance`
    let am = e
        .new_instance("Landroidx/collection/ArrayMap;")
        .expect("new_instance ArrayMap");
    let k = jstr(&mut e, "k");
    let v = jstr(&mut e, "v");
    e.invoke_instance(
        am,
        "put",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        &[k.clone(), v.clone()],
    )
    .expect("ArrayMap.put");
    assert_eq!(
        e.invoke_instance(
            am,
            "get",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            std::slice::from_ref(&k),
        )
        .expect("ArrayMap.get"),
        v
    );
}

#[test]
fn issue_232_dex_subclasses_inherit_androidx_viewmodel_livedata_and_material_hosts() {
    // Build DEX with:
    // 1. `LMyVM; extends Landroidx/lifecycle/ViewModel;` (NO DEX for ViewModel!)
    //    - `<init>()V` calls `invoke-direct Landroidx/lifecycle/ViewModel;-><init>()V`
    //    - overrides `onCleared()V` incrementing `LMyVM;->cleared:I`
    // 2. `LMyLiveData; extends Landroidx/lifecycle/MutableLiveData;` (NO DEX for MutableLiveData!)
    //    - `<init>()V` calls `invoke-direct Landroidx/lifecycle/MutableLiveData;-><init>()V`
    // 3. `LMyBtn; extends Lcom/google/android/material/button/MaterialButton;` (NO DEX for MaterialButton!)
    //    - `<init>(Landroid/content/Context;)V` calls `invoke-direct MaterialButton;-><init>(Context)V`
    // 4. `LSubRunner;` invoking inherited methods declared on the DEX subclass (`LMyVM;->setTagIfAbsent`,
    //    `LMyVM;->getTag`, `LMyLiveData;->setValue`, `LMyLiveData;->getValue`, `LMyBtn;->setText`, `LMyBtn;->getText`).
    let mut b = DexBuilder::new();

    // 1. LMyVM;
    let vm_base = "Landroidx/lifecycle/ViewModel;";
    let my_vm = b.class("LMyVM;", vm_base);
    b.static_field(my_vm, "cleared", "I", SVal::Int(0));
    let f_cleared = b.field_idx("LMyVM;", "I", "cleared");
    let p_void = b.proto_idx("V", vec![]);
    let m_vm_init = b.method_idx(vm_base, p_void, "<init>");
    let mut u_vm_ctor = op35c(0x70, 1, m_vm_init, [0, 0, 0, 0, 0]);
    u_vm_ctor.extend(op10x(0x0e));
    let c_vm_ctor = b.code(1, 1, 1, u_vm_ctor);
    b.direct(my_vm, "<init>", "V", vec![], ACC_PUBLIC, Some(c_vm_ctor));

    let mut u_cleared = op21c(0x60, 0, f_cleared);
    u_cleared.extend(op22b(0xd8, 0, 0, 1));
    u_cleared.extend(op21c(0x67, 0, f_cleared));
    u_cleared.extend(op10x(0x0e));
    let c_cleared = b.code(2, 1, 0, u_cleared);
    b.r#virtual(my_vm, "onCleared", "V", vec![], ACC_PUBLIC, Some(c_cleared));

    // 2. LMyLiveData;
    let mld_base = "Landroidx/lifecycle/MutableLiveData;";
    let my_ld = b.class("LMyLiveData;", mld_base);
    let m_ld_init = b.method_idx(mld_base, p_void, "<init>");
    let mut u_ld_ctor = op35c(0x70, 1, m_ld_init, [0, 0, 0, 0, 0]);
    u_ld_ctor.extend(op10x(0x0e));
    let c_ld_ctor = b.code(1, 1, 1, u_ld_ctor);
    b.direct(my_ld, "<init>", "V", vec![], ACC_PUBLIC, Some(c_ld_ctor));

    // 3. LMyBtn;
    let mbtn_base = "Lcom/google/android/material/button/MaterialButton;";
    let my_btn = b.class("LMyBtn;", mbtn_base);
    let p_ctx_void = b.proto_idx("V", vec!["Landroid/content/Context;".into()]);
    let m_mbtn_init = b.method_idx(mbtn_base, p_ctx_void, "<init>");
    let mut u_btn_ctor = op35c(0x70, 2, m_mbtn_init, [0, 1, 0, 0, 0]);
    u_btn_ctor.extend(op10x(0x0e));
    let c_btn_ctor = b.code(2, 2, 2, u_btn_ctor);
    b.direct(
        my_btn,
        "<init>",
        "V",
        vec!["Landroid/content/Context;"],
        ACC_PUBLIC,
        Some(c_btn_ctor),
    );

    // 4. LSubRunner;
    let runner = b.class("LSubRunner;", "Ljava/lang/Object;");
    let t_my_vm = b.type_idx("LMyVM;");
    let m_my_vm_init = b.method_idx("LMyVM;", p_void, "<init>");
    let p_set_tag = b.proto_idx(
        "Ljava/lang/Object;",
        vec!["Ljava/lang/String;".into(), "Ljava/lang/Object;".into()],
    );
    let m_set_tag = b.method_idx("LMyVM;", p_set_tag, "setTagIfAbsent");
    let p_get_tag = b.proto_idx("Ljava/lang/Object;", vec!["Ljava/lang/String;".into()]);
    let m_get_tag = b.method_idx("LMyVM;", p_get_tag, "getTag");
    let m_clear = b.method_idx("LMyVM;", p_void, "clear");
    let s_key = b.intern("scope") as u16;
    let s_val = b.intern("Job1") as u16;

    // testViewModel(): new LMyVM; -> setTagIfAbsent("scope", "Job1") -> clear() -> getTag("scope")
    let mut u_rvm = Vec::new();
    u_rvm.extend(op21c(0x22, 0, t_my_vm));
    u_rvm.extend(op35c(0x70, 1, m_my_vm_init, [0, 0, 0, 0, 0]));
    u_rvm.extend(op21c(0x1a, 1, s_key));
    u_rvm.extend(op21c(0x1a, 2, s_val));
    u_rvm.extend(op35c(0x6e, 3, m_set_tag, [0, 1, 2, 0, 0]));
    u_rvm.extend(op35c(0x6e, 1, m_clear, [0, 0, 0, 0, 0]));
    u_rvm.extend(op35c(0x6e, 2, m_get_tag, [0, 1, 0, 0, 0]));
    u_rvm.extend(op11x(0x0c, 0));
    u_rvm.extend(op11x(0x11, 0));
    let c_rvm = b.code(3, 0, 3, u_rvm);

    // testLiveData(): new LMyLiveData; -> setValue("Live!") -> getValue()
    let t_my_ld = b.type_idx("LMyLiveData;");
    let m_my_ld_init = b.method_idx("LMyLiveData;", p_void, "<init>");
    let p_set_val = b.proto_idx("V", vec!["Ljava/lang/Object;".into()]);
    let m_set_val = b.method_idx("LMyLiveData;", p_set_val, "setValue");
    let p_get_val = b.proto_idx("Ljava/lang/Object;", vec![]);
    let m_get_val = b.method_idx("LMyLiveData;", p_get_val, "getValue");
    let s_live = b.intern("Live!") as u16;
    let mut u_rld = Vec::new();
    u_rld.extend(op21c(0x22, 0, t_my_ld));
    u_rld.extend(op35c(0x70, 1, m_my_ld_init, [0, 0, 0, 0, 0]));
    u_rld.extend(op21c(0x1a, 1, s_live));
    u_rld.extend(op35c(0x6e, 2, m_set_val, [0, 1, 0, 0, 0]));
    u_rld.extend(op35c(0x6e, 1, m_get_val, [0, 0, 0, 0, 0]));
    u_rld.extend(op11x(0x0c, 0));
    u_rld.extend(op11x(0x11, 0));
    let c_rld = b.code(2, 0, 2, u_rld);

    // testMaterialBtn(ctx): new LMyBtn(ctx) -> setText("Click") -> getText()
    let t_my_btn = b.type_idx("LMyBtn;");
    let m_my_btn_init = b.method_idx("LMyBtn;", p_ctx_void, "<init>");
    let p_set_txt = b.proto_idx("V", vec!["Ljava/lang/CharSequence;".into()]);
    let m_set_txt = b.method_idx("LMyBtn;", p_set_txt, "setText");
    let p_get_txt = b.proto_idx("Ljava/lang/CharSequence;", vec![]);
    let m_get_txt = b.method_idx("LMyBtn;", p_get_txt, "getText");
    let s_click = b.intern("Click") as u16;
    // regs: v0, v1, v2=ctx
    let mut u_rbtn = Vec::new();
    u_rbtn.extend(op21c(0x22, 0, t_my_btn));
    u_rbtn.extend(op35c(0x70, 2, m_my_btn_init, [0, 2, 0, 0, 0]));
    u_rbtn.extend(op21c(0x1a, 1, s_click));
    u_rbtn.extend(op35c(0x6e, 2, m_set_txt, [0, 1, 0, 0, 0]));
    u_rbtn.extend(op35c(0x6e, 1, m_get_txt, [0, 0, 0, 0, 0]));
    u_rbtn.extend(op11x(0x0c, 0));
    u_rbtn.extend(op11x(0x11, 0));
    let c_rbtn = b.code(3, 1, 2, u_rbtn);

    b.direct(
        runner,
        "testLiveData",
        "Ljava/lang/Object;",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(c_rld),
    );
    b.direct(
        runner,
        "testMaterialBtn",
        "Ljava/lang/CharSequence;",
        vec!["Landroid/content/Context;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(c_rbtn),
    );
    b.direct(
        runner,
        "testViewModel",
        "Ljava/lang/Object;",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(c_rvm),
    );

    let mut e = engine_of(&b);
    let tag = as_obj(
        e.invoke_static("LSubRunner;", "testViewModel", "()Ljava/lang/Object;", &[])
            .expect("testViewModel"),
    );
    assert_eq!(e.heap.as_str(tag).expect("tag str"), "Job1");
    assert_eq!(
        e.statics.get(&("LMyVM;".into(), "cleared".into())),
        Some(&Value::Int(1)),
        "ViewModel.clear() must dispatch virtual DEX override LMyVM.onCleared()"
    );

    let ld_val = as_obj(
        e.invoke_static("LSubRunner;", "testLiveData", "()Ljava/lang/Object;", &[])
            .expect("testLiveData"),
    );
    assert_eq!(e.heap.as_str(ld_val).expect("ld str"), "Live!");

    let app = e.new_instance("Landroid/app/Application;").expect("app");
    let btn_txt = as_obj(
        e.invoke_static(
            "LSubRunner;",
            "testMaterialBtn",
            "(Landroid/content/Context;)Ljava/lang/CharSequence;",
            &[Value::Obj(app)],
        )
        .expect("testMaterialBtn"),
    );
    assert_eq!(e.heap.as_str(btn_txt).expect("btn str"), "Click");
}
