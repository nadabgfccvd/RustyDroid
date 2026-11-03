//! `android.content.pm.PackageManager`, backed by the installed APK's manifest.
//!
//! - `Context.getPackageManager()` returns an `ApplicationPackageManager`.
//! - **Own package.** `getPackageInfo` and `getApplicationInfo` return
//!   `PackageInfo`/`ApplicationInfo` objects whose public fields
//!   (`packageName`, `versionName`, `versionCode`, `applicationInfo`,
//!   `targetSdkVersion`, `minSdkVersion`, `dataDir`, `sourceDir`, `uid`, …)
//!   come from the manifest. `getLongVersionCode` is also served.
//!   `getActivityInfo`, `getServiceInfo` and `getProviderInfo` return
//!   component infos for classes present in the APK.
//! - **Other packages** throw `PackageManager$NameNotFoundException`, as on a
//!   device without them installed.
//! - `hasSystemFeature`: reports a fixed phone profile (touchscreen,
//!   camera, wifi, portrait screen, …). TV, watch, automotive and PC
//!   features are false.
//! - **Intent queries.** `queryIntent*` return an empty list;
//!   `resolveActivity` and `resolveContentProvider` return null. No other
//!   apps are installed.
//! - `getApplicationLabel`, `getPackagesForUid` and `isInstantApp` are
//!   served; `getDefaultActivityIcon` and `getDrawable` return null.

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::heap::{ElemKind, ObjRef};
use crate::value::Value;

use rd_apk::manifest::ComponentKind;

pub(crate) const PM: &str = "Landroid/content/pm/PackageManager;";
pub(crate) const APM: &str = "Landroid/app/ApplicationPackageManager;";
const PACKAGE_INFO: &str = "Landroid/content/pm/PackageInfo;";
const APP_INFO: &str = "Landroid/content/pm/ApplicationInfo;";
const APP_UID: i32 = 10_123;

/// Features of the emulated device (a phone with a camera).
const FEATURE_INFO: &str = "Landroid/content/pm/FeatureInfo;";
const PACKAGE_INSTALLER: &str = "Landroid/content/pm/PackageInstaller;";
const INSTALLER_SESSION: &str = "Landroid/content/pm/PackageInstaller$Session;";
const SESSION_PARAMS: &str = "Landroid/content/pm/PackageInstaller$SessionParams;";

const FEATURES: &[&str] = &[
    "android.hardware.touchscreen",
    "android.hardware.touchscreen.multitouch",
    "android.hardware.faketouch",
    "android.hardware.camera",
    "android.hardware.camera.any",
    "android.hardware.camera.autofocus",
    "android.hardware.camera.front",
    "android.hardware.screen.portrait",
    "android.hardware.screen.landscape",
    "android.hardware.wifi",
    "android.hardware.audio.output",
    "android.hardware.microphone",
    "android.hardware.sensor.accelerometer",
    "android.hardware.vulkan.version",
    "android.software.app_widgets",
    "android.software.home_screen",
    "android.software.input_methods",
    "android.software.webview",
];

fn exc(class: &str, msg: impl Into<String>) -> VmExit {
    VmExit::Exception(Throwable::new(class, msg.into()))
}

fn jstr(vm: &mut Engine, s: &str) -> Result<Value, VmExit> {
    Ok(Value::Obj(crate::intrinsics::alloc_string(
        vm,
        s.to_string(),
    )?))
}

fn sval(vm: &Engine, v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::Obj(r)) => vm.heap.as_str(*r).ok().map(str::to_string),
        _ => None,
    }
}

struct AppMeta {
    package: String,
    version_name: Option<String>,
    version_code: u64,
    min_sdk: u32,
    target_sdk: u32,
    label: String,
}

fn meta(vm: &Engine) -> AppMeta {
    match vm.fw.resources.as_ref() {
        Some(apk) => {
            let m = &apk.manifest;
            AppMeta {
                package: m.package.clone(),
                version_name: m.version_name.clone(),
                version_code: m.version_code.unwrap_or(1),
                min_sdk: m.min_sdk,
                target_sdk: m.target_sdk,
                label: m
                    .application
                    .label
                    .clone()
                    .unwrap_or_else(|| m.package.clone()),
            }
        }
        None => AppMeta {
            package: "app".into(),
            version_name: None,
            version_code: 1,
            min_sdk: 1,
            target_sdk: 1,
            label: "app".into(),
        },
    }
}

fn alloc(vm: &mut Engine, class: &str, fields: Vec<(&str, Value)>) -> Result<ObjRef, VmExit> {
    let fields = fields
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    Ok(vm.heap.alloc_instance(class.to_string(), fields)?)
}

fn not_found(pkg: &str) -> VmExit {
    exc(
        "Landroid/content/pm/PackageManager$NameNotFoundException;",
        pkg,
    )
}

fn app_info(vm: &mut Engine) -> Result<ObjRef, VmExit> {
    let m = meta(vm);
    let data_dir = format!("/data/user/0/{}", m.package);
    let source_dir = format!("/data/app/{}/base.apk", m.package);
    let fields = vec![
        ("packageName", jstr(vm, &m.package)?),
        ("processName", jstr(vm, &m.package)?),
        ("className", Value::Null),
        ("dataDir", jstr(vm, &data_dir)?),
        ("sourceDir", jstr(vm, &source_dir)?),
        ("publicSourceDir", jstr(vm, &source_dir)?),
        (
            "nativeLibraryDir",
            jstr(vm, &format!("/data/app/{}/lib", m.package))?,
        ),
        ("uid", Value::Int(APP_UID)),
        ("minSdkVersion", Value::Int(m.min_sdk as i32)),
        ("targetSdkVersion", Value::Int(m.target_sdk as i32)),
        ("enabled", Value::Int(1)),
        ("flags", Value::Int(0x0000_0004 | 0x0000_0080 | 0x0040_0000)), // HAS_CODE | INSTALLED… | SUPPORTS_RTL-ish
        ("metaData", Value::Null),
        ("nonLocalizedLabel", jstr(vm, &m.label)?),
        ("labelRes", Value::Int(0)),
        ("icon", Value::Int(0)),
        ("theme", Value::Int(0)),
        ("category", Value::Int(-1)),
    ];
    alloc(vm, APP_INFO, fields)
}

/// Bits GET_* do AOSP PackageManager (getPackageInfo(String, int) e
/// PackageManager$PackageInfoFlags.getValue()) — issue #361: antes eram
/// ignorados e TODOS os arrays de PackageInfo vinham Null.
const GET_ACTIVITIES: u64 = 0x0000_0001;
const GET_RECEIVERS: u64 = 0x0000_0002;
const GET_SERVICES: u64 = 0x0000_0004;
const GET_PROVIDERS: u64 = 0x0000_0008;
const GET_PERMISSIONS: u64 = 0x0000_1000;
/// MATCH_DISABLED_COMPONENTS (0x200; inclui o legado GET_DISABLED_COMPONENTS):
/// sem o bit, componentes com enabled=false ficam fora (AOSP).
const MATCH_DISABLED_COMPONENTS: u64 = 0x0000_0200;
/// Bits de requestedPermissionsFlags (AOSP): REQUIRED=1, GRANTED=2. A VM
/// host concede tudo o que o manifesto pede (semântica de emulador), então
/// cada entrada nasce REQUIRED|GRANTED.
const REQUESTED_PERMISSION_REQUIRED: i32 = 1;
const REQUESTED_PERMISSION_GRANTED: i32 = 2;

/// Extrai o long de flags do 2º argumento de getPackageInfo: Int (forma
/// clássica) ou PackageManager$PackageInfoFlags (API 33 — backing field
/// `value`; alocado por PackageManager$PackageInfoFlags.of(long)).
fn package_info_flags(vm: &Engine, v: Option<&Value>) -> u64 {
    match v {
        Some(Value::Int(f)) => (*f as u32) as u64,
        Some(Value::Long(l)) => *l as u64,
        Some(Value::Obj(r)) => match vm.heap.get_field(*r, "value") {
            Ok(Value::Long(l)) => l as u64,
            Ok(Value::Int(i)) => (i as u32) as u64,
            _ => 0,
        },
        _ => 0,
    }
}

/// Item (ActivityInfo/ServiceInfo/ProviderInfo) de um componente DECLARADO
/// no manifesto (issue #361) — a fonte de verdade é apk.manifest
/// (`exported_effective`/`enabled`/`authorities`/`permission`), não o
/// classpath: o componente pode não estar carregado no DEX do host.
fn manifest_component_info(
    vm: &mut Engine,
    info_class: &str,
    comp: &ComponentSpec,
) -> Result<ObjRef, VmExit> {
    let m = meta(vm);
    let ai = app_info(vm)?;
    let enabled = i32::from(comp.enabled != Some(false));
    let exported = i32::from(comp.exported_effective);
    let process = comp.process.clone().unwrap_or_else(|| m.package.clone());
    let fields = vec![
        ("name", jstr(vm, &comp.class_name)?),
        ("packageName", jstr(vm, &m.package)?),
        ("processName", jstr(vm, &process)?),
        ("applicationInfo", Value::Obj(ai)),
        ("enabled", Value::Int(enabled)),
        ("exported", Value::Int(exported)),
        ("metaData", Value::Null),
        (
            "authority",
            match &comp.authorities {
                Some(a) => jstr(vm, a)?,
                None => Value::Null,
            },
        ),
        (
            "permission",
            match &comp.permission {
                Some(p) => jstr(vm, p)?,
                None => Value::Null,
            },
        ),
        (
            "targetActivity",
            match &comp.target_activity {
                Some(t) => jstr(vm, t)?,
                None => Value::Null,
            },
        ),
        ("theme", Value::Int(0)),
        ("labelRes", Value::Int(0)),
        ("screenOrientation", Value::Int(-1)),
        ("launchMode", Value::Int(0)),
        ("flags", Value::Int(0)),
        ("configChanges", Value::Int(0)),
        ("softInputMode", Value::Int(0)),
    ];
    alloc(vm, info_class, fields)
}

/// Array tipado de componentes para os bits GET_* (issue #361). AOSP: flag
/// NÃO setada → null; flag setada → array alocado (MESMO VAZIO quando o
/// manifesto não declara nada daquele kind). MATCH_DISABLED_COMPONENTS
/// controla a inclusão de enabled=false (default: excluir). Alocações
/// passam pelo heap normal (orçadas — flagship tem 1.006 uses-permission).
fn component_array(
    vm: &mut Engine,
    info_class: &str,
    arr_desc: &str,
    kind: ComponentKind,
    include_disabled: bool,
) -> Result<Value, VmExit> {
    // coleta os dados do manifesto ANTES de alocar (o manifesto é emprestado
    // do engine; as alocações dos itens precisam de &mut)
    let specs: Vec<ComponentSpec> = vm
        .fw
        .resources
        .as_ref()
        .map(|apk| {
            apk.manifest
                .application
                .components
                .iter()
                .filter(|c| c.kind == kind)
                .filter(|c| include_disabled || c.enabled != Some(false))
                .map(ComponentSpec::of)
                .collect()
        })
        .unwrap_or_default();
    let mut items: Vec<Value> = Vec::with_capacity(specs.len());
    for spec in &specs {
        let item = manifest_component_info(vm, info_class, spec)?;
        items.push(Value::Obj(item));
    }
    crate::arrays::new_array(vm, arr_desc, ElemKind::Obj, items)
}

/// Snapshot imutável dos campos de componente que os itens precisam
/// (desacopla o empréstimo do manifesto das alocações no heap).
struct ComponentSpec {
    class_name: String,
    process: Option<String>,
    authorities: Option<String>,
    permission: Option<String>,
    target_activity: Option<String>,
    exported_effective: bool,
    enabled: Option<bool>,
}

impl ComponentSpec {
    fn of(c: &rd_apk::manifest::Component) -> Self {
        Self {
            class_name: c.class_name.clone(),
            process: c.process.clone(),
            authorities: c.authorities.clone(),
            permission: c.permission.clone(),
            target_activity: c.target_activity.clone(),
            exported_effective: c.exported_effective,
            enabled: c.enabled,
        }
    }
}

/// (permissions, requestedPermissions, requestedPermissionsFlags) para
/// GET_PERMISSIONS — arrays de `m.declared_permissions` (PermissionInfo[],
/// <permission> do app), `m.uses_permissions` (String[]) e o paralelo int[]
/// REQUIRED|GRANTED (issue #361).
fn permissions_arrays(vm: &mut Engine) -> Result<(Value, Value, Value), VmExit> {
    let m = meta(vm);
    // coleta ANTES de alocar (mesmo padrão do component_array)
    let (ups, dps): (Vec<String>, Vec<(String, u32)>) = vm
        .fw
        .resources
        .as_ref()
        .map(|apk| {
            (
                apk.manifest
                    .uses_permissions
                    .iter()
                    .map(|up| up.name.clone())
                    .collect(),
                apk.manifest
                    .declared_permissions
                    .iter()
                    .map(|dp| (dp.name.clone(), dp.protection_level_raw.unwrap_or(0)))
                    .collect(),
            )
        })
        .unwrap_or_default();
    let mut requested: Vec<Value> = Vec::with_capacity(ups.len());
    let mut requested_flags: Vec<Value> = Vec::with_capacity(ups.len());
    for name in &ups {
        requested.push(jstr(vm, name)?);
        requested_flags.push(Value::Int(
            REQUESTED_PERMISSION_REQUIRED | REQUESTED_PERMISSION_GRANTED,
        ));
    }
    let mut declared: Vec<Value> = Vec::with_capacity(dps.len());
    for (name, protection) in &dps {
        let fields = vec![
            ("name", jstr(vm, name)?),
            ("packageName", jstr(vm, &m.package)?),
            ("protectionLevel", Value::Int(*protection as i32)),
        ];
        declared.push(Value::Obj(alloc(
            vm,
            "Landroid/content/pm/PermissionInfo;",
            fields,
        )?));
    }
    let req = crate::arrays::new_array(vm, "[Ljava/lang/String;", ElemKind::Obj, requested)?;
    let req_flags = crate::arrays::new_array(vm, "[I", ElemKind::Int, requested_flags)?;
    let decl = crate::arrays::new_array(
        vm,
        "[Landroid/content/pm/PermissionInfo;",
        ElemKind::Obj,
        declared,
    )?;
    Ok((decl, req, req_flags))
}

fn package_info(vm: &mut Engine, flags: u64) -> Result<ObjRef, VmExit> {
    let m = meta(vm);
    let ai = app_info(vm)?;
    let vn = match &m.version_name {
        Some(v) => jstr(vm, v)?,
        None => Value::Null,
    };
    let include_disabled = flags & MATCH_DISABLED_COMPONENTS != 0;
    // AOSP: activities cobre <activity> E <activity-alias>; receivers também
    // são ActivityInfo[]; flag não setada → null (issue #361)
    let activities = if flags & GET_ACTIVITIES != 0 {
        component_array(
            vm,
            "Landroid/content/pm/ActivityInfo;",
            "[Landroid/content/pm/ActivityInfo;",
            ComponentKind::Activity,
            include_disabled,
        )?
    } else {
        Value::Null
    };
    let aliases = if flags & GET_ACTIVITIES != 0 {
        component_array(
            vm,
            "Landroid/content/pm/ActivityInfo;",
            "[Landroid/content/pm/ActivityInfo;",
            ComponentKind::ActivityAlias,
            include_disabled,
        )?
    } else {
        Value::Null
    };
    let receivers = if flags & GET_RECEIVERS != 0 {
        component_array(
            vm,
            "Landroid/content/pm/ActivityInfo;",
            "[Landroid/content/pm/ActivityInfo;",
            ComponentKind::Receiver,
            include_disabled,
        )?
    } else {
        Value::Null
    };
    let services = if flags & GET_SERVICES != 0 {
        component_array(
            vm,
            "Landroid/content/pm/ServiceInfo;",
            "[Landroid/content/pm/ServiceInfo;",
            ComponentKind::Service,
            include_disabled,
        )?
    } else {
        Value::Null
    };
    let providers = if flags & GET_PROVIDERS != 0 {
        component_array(
            vm,
            "Landroid/content/pm/ProviderInfo;",
            "[Landroid/content/pm/ProviderInfo;",
            ComponentKind::Provider,
            include_disabled,
        )?
    } else {
        Value::Null
    };
    let (permissions, requested_perms, requested_flags) = if flags & GET_PERMISSIONS != 0 {
        permissions_arrays(vm)?
    } else {
        (Value::Null, Value::Null, Value::Null)
    };
    let fields = vec![
        ("packageName", jstr(vm, &m.package)?),
        ("versionName", vn),
        ("versionCode", Value::Int(m.version_code as u32 as i32)),
        ("rd_long_version", Value::Long(m.version_code as i64)),
        ("applicationInfo", Value::Obj(ai)),
        ("firstInstallTime", Value::Long(1_700_000_000_000)),
        ("lastUpdateTime", Value::Long(1_700_000_000_000)),
        // issue #361: arrays mesclados (Activity + ActivityAlias) em
        // activities, como o PackageParser do AOSP faz
        ("activities", merge_obj_arrays(vm, activities, aliases)?),
        ("services", services),
        ("providers", providers),
        ("receivers", receivers),
        ("permissions", permissions),
        ("requestedPermissions", requested_perms),
        ("requestedPermissionsFlags", requested_flags),
        ("signatures", Value::Null),
        ("signingInfo", Value::Null),
    ];
    alloc(vm, PACKAGE_INFO, fields)
}

/// Concatena dois arrays de objetos (um pode ser Null) num array novo do
/// mesmo descritor — mescla de <activity> + <activity-alias> em activities.
fn merge_obj_arrays(vm: &mut Engine, a: Value, b: Value) -> Result<Value, VmExit> {
    let (desc, items) = match (a, b) {
        (Value::Null, Value::Null) => {
            return Ok(Value::Null);
        }
        (Value::Null, v @ Value::Obj(_)) | (v @ Value::Obj(_), Value::Null) => {
            return Ok(v);
        }
        (Value::Obj(x), Value::Obj(y)) => {
            let desc = vm
                .heap
                .class_of(x)
                .unwrap_or("[Ljava/lang/Object;")
                .to_string();
            let mut items: Vec<Value> = Vec::new();
            for r in [x, y] {
                let (elems, _) = vm.heap.as_array_elems(r)?;
                items.extend_from_slice(elems);
            }
            (desc, items)
        }
        _ => return Ok(Value::Null),
    };
    crate::arrays::new_array(vm, &desc, ElemKind::Obj, items)
}

fn component_info(
    vm: &mut Engine,
    info_class: &str,
    comp: Option<&Value>,
) -> Result<Value, VmExit> {
    let m = meta(vm);
    let (pkg, cls) = match comp {
        Some(Value::Obj(c)) => {
            let get = |f: &str| match vm.heap.get_field(*c, f) {
                Ok(Value::Obj(s)) => vm.heap.as_str(s).ok().map(str::to_string),
                _ => None,
            };
            (
                get("rd_pkg").unwrap_or_default(),
                get("rd_cls").unwrap_or_default(),
            )
        }
        _ => return Err(exc("Ljava/lang/NullPointerException;", "component == null")),
    };
    let desc = format!("L{};", cls.replace('.', "/"));
    if pkg != m.package || vm.cp.find_class(&desc).is_none() {
        return Err(not_found(&format!("{pkg}/{cls}")));
    }
    let ai = app_info(vm)?;
    let fields = vec![
        ("name", jstr(vm, &cls)?),
        ("packageName", jstr(vm, &pkg)?),
        ("processName", jstr(vm, &pkg)?),
        ("applicationInfo", Value::Obj(ai)),
        ("enabled", Value::Int(1)),
        ("exported", Value::Int(0)),
        ("metaData", Value::Null),
        ("authority", Value::Null),
        ("permission", Value::Null),
        ("theme", Value::Int(0)),
        ("labelRes", Value::Int(0)),
        ("screenOrientation", Value::Int(-1)),
        ("launchMode", Value::Int(0)),
        ("flags", Value::Int(0)),
        ("configChanges", Value::Int(0)),
        ("softInputMode", Value::Int(0)),
    ];
    Ok(Value::Obj(alloc(vm, info_class, fields)?))
}

/// `Context.getPackageManager()` (any Context receiver).
pub fn package_manager(vm: &mut Engine) -> Result<Value, VmExit> {
    Ok(Value::Obj(alloc(vm, APM, Vec::new())?))
}

/// `Context.getApplicationInfo()` — the SHORT route (AOSP: same
/// ApplicationInfo the PackageManager would hand back for the own package;
/// LadderActivity and friends call it directly on the Context, without going
/// through getPackageManager()).
pub(crate) fn context_app_info(vm: &mut Engine) -> Result<ObjRef, VmExit> {
    app_info(vm)
}

pub fn instance_intrinsic(
    vm: &mut Engine,
    recv_class: &str,
    mref_class: &str,
    name: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if name == "getPackageManager" {
        if mref_class.starts_with("Landroid/") && recv_class != APM {
            return Ok(Some(package_manager(vm)?));
        }
        return Ok(None);
    }
    if matches!(
        recv_class,
        PACKAGE_INSTALLER | INSTALLER_SESSION | SESSION_PARAMS
    ) {
        return installer_intrinsic(vm, recv_class, name, recv, args);
    }
    if recv_class == PACKAGE_INFO && name == "getLongVersionCode" {
        return Ok(Some(match vm.heap.get_field(recv, "rd_long_version") {
            Ok(v @ Value::Long(_)) => v,
            _ => Value::Long(0),
        }));
    }
    if recv_class != APM && mref_class != PM && mref_class != APM {
        return Ok(None);
    }
    let own = |vm: &Engine, v: Option<&Value>| -> Result<(), VmExit> {
        let p = sval(vm, v)
            .ok_or_else(|| exc("Ljava/lang/NullPointerException;", "packageName == null"))?;
        if p == meta(vm).package {
            Ok(())
        } else {
            Err(not_found(&p))
        }
    };
    Ok(Some(match name {
        "hasSystemFeature" => {
            let f = sval(vm, args.first()).unwrap_or_default();
            Value::Int(FEATURES.contains(&f.as_str()) as i32)
        }
        // The same profile `hasSystemFeature` answers, as the array the
        // settings/first-run screens walk (`FeatureInfo.name`).
        "getSystemAvailableFeatures" => Value::Obj(system_features(vm)?),
        "getPackageInstaller" => Value::Obj(alloc(vm, PACKAGE_INSTALLER, Vec::new())?),
        "getPackageInfo" => {
            own(vm, args.first())?;
            // issue #361: extrai os bits GET_* (Int clássico ou
            // PackageInfoFlags API 33) e popula os arrays do manifesto
            let flags = package_info_flags(vm, args.get(1));
            Value::Obj(package_info(vm, flags)?)
        }
        "getApplicationInfo" => {
            own(vm, args.first())?;
            Value::Obj(app_info(vm)?)
        }
        "getApplicationLabel" => {
            let l = meta(vm).label;
            jstr(vm, &l)?
        }
        "getActivityInfo" => component_info(vm, "Landroid/content/pm/ActivityInfo;", args.first())?,
        "getServiceInfo" => component_info(vm, "Landroid/content/pm/ServiceInfo;", args.first())?,
        "getProviderInfo" => component_info(vm, "Landroid/content/pm/ProviderInfo;", args.first())?,
        "getReceiverInfo" => component_info(vm, "Landroid/content/pm/ActivityInfo;", args.first())?,
        "queryIntentActivities"
        | "queryIntentServices"
        | "queryBroadcastReceivers"
        | "queryIntentContentProviders"
        | "queryIntentActivityOptions"
        | "getInstalledPackages"
        | "getInstalledApplications" => {
            let r = alloc(vm, "Ljava/util/ArrayList;", Vec::new())?;
            vm.fw
                .objects
                .insert(r, crate::framework::HostObj::List { elems: Vec::new() });
            Value::Obj(r)
        }
        "resolveActivity"
        | "resolveService"
        | "resolveContentProvider"
        | "getDefaultActivityIcon"
        | "getDrawable"
        | "getApplicationIcon"
        | "getLaunchIntentForPackage" => Value::Null,
        "getPackagesForUid" => {
            let p = meta(vm).package;
            let s = jstr(vm, &p)?;
            let a = vm
                .heap
                .alloc_array(crate::heap::ElemKind::Obj, 1, "Ljava/lang/String;")?;
            *vm.heap.array_elems_mut(a)? = vec![s];
            Value::Obj(a)
        }
        "getNameForUid" => {
            let p = meta(vm).package;
            jstr(vm, &p)?
        }
        // Estado de componente/app não é modelado: set é no-op (void) e o get
        // devolve COMPONENT_ENABLED_STATE_DEFAULT (0), como um app sem overrides.
        "setComponentEnabledSetting" | "setApplicationEnabledSetting" => Value::Null,
        "getComponentEnabledSetting" | "getApplicationEnabledSetting" => Value::Int(0),
        "isInstantApp" | "isSafeMode" => Value::Int(0),
        "checkPermission" => Value::Int(0), // PERMISSION_GRANTED; runtime grants are modelled elsewhere
        "getInstallerPackageName" => Value::Null,
        _ => return Ok(None),
    }))
}

/// `PackageManager.getSystemAvailableFeatures()`: one `FeatureInfo` per entry of
/// [`FEATURES`], so `hasSystemFeature(name)` and the array agree.
fn system_features(vm: &mut Engine) -> Result<ObjRef, VmExit> {
    let mut infos = Vec::with_capacity(FEATURES.len());
    for name in FEATURES {
        let n = jstr(vm, name)?;
        infos.push(Value::Obj(alloc(
            vm,
            FEATURE_INFO,
            vec![
                ("name", n),
                ("version", Value::Int(0)),
                ("reqGlEsVersion", Value::Int(0)),
                ("flags", Value::Int(0)),
            ],
        )?));
    }
    let arr = vm
        .heap
        .alloc_array(crate::heap::ElemKind::Obj, infos.len(), FEATURE_INFO)
        .map_err(crate::err::oom_throwable)?;
    *vm.heap.array_elems_mut(arr)? = infos;
    Ok(arr)
}

// ---------------------------------------------------------------------------
// PackageInstaller
// ---------------------------------------------------------------------------

/// Next install-session id handed out by `createSession`.
const SESSION_NEXT: &str = "rd_pi_next";
/// Sessions: id (`i32`), session object (`ObjRef`) and its staged file path.
fn installer_intrinsic(
    vm: &mut Engine,
    recv_class: &str,
    name: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    Ok(Some(match recv_class {
        PACKAGE_INSTALLER => match name {
            "<init>" => Value::Null,
            // Callback registration: the VM delivers session callbacks
            // synchronously from `commit`, so nothing is stored.
            "registerSessionCallback" | "unregisterSessionCallback" => Value::Null,
            "getMySessions" | "getAllSessions" | "getActiveStageSessions" | "getStagedSessions" => {
                let r = alloc(vm, "Ljava/util/ArrayList;", Vec::new())?;
                vm.fw
                    .objects
                    .insert(r, crate::framework::HostObj::List { elems: Vec::new() });
                Value::Obj(r)
            }
            "createSession" => {
                let next = vm
                    .statics
                    .get(&(PACKAGE_INSTALLER.to_string(), SESSION_NEXT.to_string()))
                    .and_then(|v| v.as_int().ok())
                    .unwrap_or(1);
                vm.statics.insert(
                    (PACKAGE_INSTALLER.to_string(), SESSION_NEXT.to_string()),
                    Value::Int(next + 1),
                );
                let vfs = vm.fw.vfs.clone();
                crate::jfile::mkdirs_pub(&vfs, "/data/local/tmp/sessions");
                let path = format!("/data/local/tmp/sessions/{next}.apk");
                let path_ref = crate::intrinsics::alloc_string(vm, path)?;
                let session = alloc(
                    vm,
                    INSTALLER_SESSION,
                    vec![
                        ("rd_session_id", Value::Int(next)),
                        ("rd_path", Value::Obj(path_ref)),
                        ("rd_committed", Value::Int(0)),
                    ],
                )?;
                let sessions = session_list(vm)?;
                push_session(vm, sessions, next, session);
                Value::Int(next)
            }
            "openSession" => {
                let id = args.first().and_then(|v| v.as_int().ok()).unwrap_or(-1);
                match find_session(vm, id) {
                    Some(s) => Value::Obj(s),
                    None => {
                        return Err(exc(
                            "Ljava/lang/IllegalArgumentException;",
                            format!("Session {id} does not exist"),
                        ))
                    }
                }
            }
            "abandonSession" => {
                let id = args.first().and_then(|v| v.as_int().ok()).unwrap_or(-1);
                if let Some(s) = find_session(vm, id) {
                    let _ = vm.heap.put_field(s, "rd_abandoned", Value::Int(1));
                }
                Value::Null
            }
            "getSessionInfo" => {
                let id = args.first().and_then(|v| v.as_int().ok()).unwrap_or(-1);
                match find_session(vm, id) {
                    Some(s) => Value::Obj(s),
                    None => {
                        return Err(exc(
                            "Ljava/lang/IllegalArgumentException;",
                            format!("Session {id} does not exist"),
                        ))
                    }
                }
            }
            "uninstall" | "installExistingPackage" => Value::Null,
            "setPermissionsResult" => Value::Null,
            _ => return Ok(None),
        },
        SESSION_PARAMS => match name {
            "<init>" => Value::Null,
            // Every setter is a no-op that keeps the recorded mode/package.
            "setAppPackageName" => {
                if let Some(pkg) = args.first() {
                    crate::err::put_field_result(vm.heap.put_field(
                        recv,
                        "rd_package",
                        pkg.clone(),
                    ))?;
                }
                Value::Null
            }
            "setSize" => {
                if let Some(size) = args.first() {
                    crate::err::put_field_result(vm.heap.put_field(recv, "rd_size", size.clone()))?;
                }
                Value::Null
            }
            "getAppPackageName" => vm.heap.get_field(recv, "rd_package").unwrap_or(Value::Null),
            _ => Value::Null,
        },
        INSTALLER_SESSION => match name {
            "getSessionId" => Value::Int(
                vm.heap
                    .get_field(recv, "rd_session_id")
                    .ok()
                    .and_then(|v| v.as_int().ok())
                    .unwrap_or(-1),
            ),
            "openWrite" => {
                let path = match vm.heap.get_field(recv, "rd_path") {
                    Ok(Value::Obj(p)) => vm.heap.as_str(p).map(str::to_string).unwrap_or_default(),
                    _ => String::new(),
                };
                let stream = alloc(vm, crate::jfile::FOS, Vec::new())?;
                let path_ref = crate::intrinsics::alloc_string(vm, path)?;
                crate::jfile::stream_intrinsic(
                    vm,
                    crate::jfile::FOS,
                    "<init>",
                    "(Ljava/lang/String;Z)V",
                    stream,
                    &[Value::Obj(path_ref), Value::Int(0)],
                )?;
                Value::Obj(stream)
            }
            "openRead" => {
                let path = match vm.heap.get_field(recv, "rd_path") {
                    Ok(Value::Obj(p)) => vm.heap.as_str(p).map(str::to_string).unwrap_or_default(),
                    _ => String::new(),
                };
                if !crate::jfile::path_exists(&vm.fw.vfs, &path) {
                    return Ok(Some(Value::Null));
                }
                let stream = alloc(vm, "Ljava/io/FileInputStream;", Vec::new())?;
                let path_ref = crate::intrinsics::alloc_string(vm, path)?;
                crate::jfile::stream_intrinsic(
                    vm,
                    "Ljava/io/FileInputStream;",
                    "<init>",
                    "(Ljava/lang/String;)V",
                    stream,
                    &[Value::Obj(path_ref)],
                )?;
                Value::Obj(stream)
            }
            // The staged bytes are already durable in the VM's virtual disk, so
            // `fsync`/`commit` only record the state transition.
            "fsync" | "close" | "abandon" => Value::Null,
            "commit" => {
                crate::err::put_field_result(vm.heap.put_field(
                    recv,
                    "rd_committed",
                    Value::Int(1),
                ))?;
                Value::Null
            }
            "getNames" | "getAppPackageName" => Value::Null,
            "toString" => {
                let id = vm
                    .heap
                    .get_field(recv, "rd_session_id")
                    .ok()
                    .and_then(|v| v.as_int().ok())
                    .unwrap_or(-1);
                crate::intrinsics::alloc_string(vm, format!("Session {{id={id}}}"))
                    .map(Value::Obj)?
            }
            _ => return Ok(None),
        },
        _ => return Ok(None),
    }))
}

fn session_list(vm: &mut Engine) -> Result<ObjRef, VmExit> {
    let key = (PACKAGE_INSTALLER.to_string(), "rd_sessions".to_string());
    if let Some(Value::Obj(l)) = vm.statics.get(&key).cloned() {
        return Ok(l);
    }
    let list = alloc(vm, "Ljava/util/LinkedHashMap;", Vec::new())?;
    vm.fw
        .objects
        .insert(list, crate::framework::HostObj::List { elems: Vec::new() });
    vm.statics.insert(key, Value::Obj(list));
    Ok(list)
}

fn push_session(vm: &mut Engine, list: ObjRef, id: i32, session: ObjRef) {
    if let Some(crate::framework::HostObj::List { elems }) = vm.fw.objects.get_mut(&list) {
        elems.push(Value::Int(id));
        elems.push(Value::Obj(session));
    }
}

fn find_session(vm: &mut Engine, id: i32) -> Option<ObjRef> {
    let list = session_list(vm).ok()?;
    let elems = match vm.fw.objects.get(&list) {
        Some(crate::framework::HostObj::List { elems }) => elems.clone(),
        _ => return None,
    };
    let (pairs, _) = elems.as_chunks::<2>();
    pairs
        .iter()
        .find(|pair| pair[0].as_int().ok() == Some(id))
        .and_then(|pair| pair[1].as_ref().ok().flatten())
}

#[cfg(test)]
mod tests {
    use super::FEATURES;
    #[test]
    fn phone_profile_excludes_tv() {
        assert!(FEATURES.contains(&"android.hardware.camera.any"));
        assert!(!FEATURES.contains(&"android.software.leanback"));
    }
}
