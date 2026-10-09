//! `rd` — CLI do RustyDroid (M0).
//!
//! Subcomandos:
//! - `rd inspect <apk>`      — componentes/permissões/features/assinatura (DoD do M0)
//! - `rd perm list|info|audit` — motor de permissões consultável via CLI
//! - `rd device list|show`   — perfis de device (piso: moto-e5)
//! - `rd behavior`           — comutadores por targetSdk 26→36
//! - `rd dex <apk>`          — demonstra o contrato NOT_IMPLEMENTED (Lei 1)
//!
//! Códigos de saída: 0 ok · 1 erro estruturado · 2 NOT_IMPLEMENTED.

use clap::{Parser, Subcommand};
use rd_apk::{Apk, RdError};
use rd_framework::fd_behavior::SwitchTable;
use rd_framework::fd_devices::DeviceTable;
use rd_framework::fd_permissions::{GrantState, PermissionEngine, PermissionTable};
use rd_framework::DeviceProfile;
use std::io::Write as _;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "rd",
    version,
    about = "RustyDroid — runtime Android em Rust, sem VM, headless-first, nativo para agentes de IA",
    long_about = None,
    after_help = "Erros seguem o contrato {code, cause, suggestion, module_id} — consumível por agentes."
)]
struct Cli {
    /// Diretório de dados versionados (default: ./data, env RD_DATA_DIR, fallback embutido)
    #[arg(long, global = true)]
    data: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Inspeciona um APK real: identidade, SDKs, componentes, permissões, features
    Inspect {
        path: PathBuf,
        /// Saída em JSON (contrato do agente)
        #[arg(long)]
        json: bool,
        /// Perfil de device para orçamentos (ex.: moto-e5)
        #[arg(long)]
        device: Option<String>,
    },
    /// Motor de permissões (Apêndice B — dado versionado)
    Perm {
        #[command(subcommand)]
        sub: PermSub,
    },
    /// Perfis de device e orçamentos de hardware
    Device {
        #[command(subcommand)]
        sub: DeviceSub,
    },
    /// Behavior changes por targetSdk 26→36 (PRF-07)
    Behavior {
        /// targetSdk do app (default: 36 — teto)
        #[arg(long)]
        target: Option<u32>,
        /// Filtrar por impacto (storage, notifications, ui, …)
        #[arg(long)]
        impact: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Parser de DEX (milestone M1) — demonstra o contrato NOT_IMPLEMENTED
    Dex {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum PermSub {
    /// Lista permissões declaradas + estado calculado pelo motor
    List {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Detalhe de uma permissão da tabela versionada
    Info {
        name: String,
        #[arg(long)]
        json: bool,
    },
    /// Auditoria estática (exported sem guarda, debuggable, cleartext…)
    Audit {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum DeviceSub {
    /// Lista perfis disponíveis
    List {
        #[arg(long)]
        json: bool,
    },
    /// Mostra um perfil em detalhe (orçamentos)
    Show {
        id: String,
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let code = run(cli);
    std::process::exit(code);
}

fn run(cli: Cli) -> i32 {
    // resolução do diretório de dados: --data > RD_DATA_DIR > ./data
    let data_path = data_dir(cli.data.as_deref());
    let data = data_path.as_deref();
    let result = match &cli.cmd {
        Cmd::Inspect { path, json, device } => cmd_inspect(path, *json, device.as_deref(), data),
        Cmd::Perm { sub } => match sub {
            PermSub::List { path, json } => cmd_perm_list(path, *json, data),
            PermSub::Info { name, json } => cmd_perm_info(name, *json, data),
            PermSub::Audit { path, json } => cmd_perm_audit(path, *json, data),
        },
        Cmd::Device { sub } => match sub {
            DeviceSub::List { json } => cmd_device_list(*json, data),
            DeviceSub::Show { id, json } => cmd_device_show(id, *json, data),
        },
        Cmd::Behavior {
            target,
            impact,
            json,
        } => cmd_behavior(*target, impact.as_deref(), *json, data),
        Cmd::Dex { json, .. } => {
            let e = RdError::not_implemented("rd-dex", "M1");
            if *json {
                println!("{}", to_json(&e));
            } else {
                eprintln!("rd: {e}");
            }
            return 2;
        }
    };
    match result {
        Ok(()) => 0,
        Err(e) => {
            let mut stderr = std::io::stderr();
            let _ = writeln!(stderr, "{}", to_json(&e));
            if e.code == "NOT_IMPLEMENTED" {
                2
            } else {
                1
            }
        }
    }
}

fn to_json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string_pretty(v).unwrap_or_else(|e| {
        format!("{{\"code\":\"INTERNAL\",\"cause\":\"{e}\",\"module_id\":\"rd-cli\"}}")
    })
}

/// Resolve o diretório de dados: --data > RD_DATA_DIR > ./data (fallback embutido).
fn data_dir(explicit: Option<&Path>) -> Option<PathBuf> {
    explicit
        .map(|p| p.to_path_buf())
        .or_else(|| std::env::var_os("RD_DATA_DIR").map(PathBuf::from))
        .or_else(|| {
            let cwd = PathBuf::from("data");
            cwd.exists().then_some(cwd)
        })
}

fn load_permissions(data: Option<&Path>) -> Result<PermissionTable, RdError> {
    match data.and_then(|d| d.join("permissions.toml").exists().then_some(d)) {
        Some(d) => {
            let s = std::fs::read_to_string(d.join("permissions.toml"))
                .map_err(|e| RdError::io(&e, "reading permissions.toml"))?;
            PermissionTable::from_toml(&s)
        }
        None => PermissionTable::embedded(),
    }
}

fn open_apk(path: &Path) -> Result<Apk, RdError> {
    Apk::open(path)
}

// ─── inspect ────────────────────────────────────────────────────────────────

fn cmd_inspect(
    path: &Path,
    json: bool,
    device: Option<&str>,
    data: Option<&Path>,
) -> Result<(), RdError> {
    let apk = open_apk(path)?;
    let table = load_permissions(data)?;
    let engine = PermissionEngine::install(&apk.manifest, table);

    let device_profile: Option<DeviceProfile> = match device {
        Some(id) => {
            let dt = DeviceTable::load(data)?;
            Some(
                dt.get(id)
                    .ok_or_else(|| {
                        RdError::missing_entry(format!("device profile {id:?}"))
                            .with_suggestion("run `rd device list`")
                    })?
                    .clone(),
            )
        }
        None => None,
    };

    if json {
        let mut v = serde_json::to_value(&apk)
            .map_err(|e| RdError::new("INTERNAL", e.to_string(), "rd-cli"))?;
        if let serde_json::Value::Object(map) = &mut v {
            map.insert(
                "permissions_state".into(),
                serde_json::to_value(engine.all_states())
                    .map_err(|e| RdError::new("INTERNAL", e.to_string(), "rd-cli"))?,
            );
            map.insert(
                "floor_status".into(),
                serde_json::to_value(apk.floor_status())
                    .map_err(|e| RdError::new("INTERNAL", e.to_string(), "rd-cli"))?,
            );
            if let Some(dp) = &device_profile {
                map.insert(
                    "device_budget".into(),
                    serde_json::to_value(dp)
                        .map_err(|e| RdError::new("INTERNAL", e.to_string(), "rd-cli"))?,
                );
            }
        }
        println!("{v}");
        return Ok(());
    }

    // ── saída humana ──
    let m = &apk.manifest;
    let file = path.display();
    let mut out = String::new();
    out.push_str(&format!("RustyDroid inspect — {file}\n"));
    out.push_str(&"=".repeat(64));
    out.push('\n');

    out.push_str(&format!(
        "package            {}  v{} ({})\n",
        m.package,
        m.version_name.as_deref().unwrap_or("?"),
        m.version_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "?".into())
    ));
    if let Some(label) = &m.application.label {
        let src = m
            .application
            .label_res
            .map(|r| format!(" (resolved @0x{r:08x})"))
            .unwrap_or_default();
        out.push_str(&format!("label              {label}{src}\n"));
    }
    if let Some(app) = &m.application.name {
        out.push_str(&format!("application class  {app}\n"));
    }
    out.push_str(&format!(
        "SDK                min {} · target {} · compile {}\n",
        m.min_sdk,
        m.target_sdk,
        m.compile_sdk
            .map(|c| c.to_string())
            .unwrap_or_else(|| "?".into())
    ));
    match apk.floor_status() {
        rd_apk::FloorStatus::Ok => {
            out.push_str(&format!(
                "api floor          OK (≥ {})\n",
                rd_apk::FLOOR_MIN_SDK
            ));
        }
        rd_apk::FloorStatus::BelowFloor { min_sdk, floor } => {
            out.push_str(&format!(
                "api floor          ⚠ BELOW_FLOOR(minSdk={min_sdk}, floor={floor}) — inspect ok; execução responde erro estruturado\n"
            ));
        }
    }
    out.push_str(&format!(
        "signing            {}\n",
        if apk.signing.is_signed() {
            apk.signing.schemes().join(", ")
        } else {
            "UNSIGNED (esquemas v1/v2/v3 não detectados)".into()
        }
    ));
    out.push_str(&format!(
        "content            {} entradas zip · {} dex · {} lib .so (abis: {}) · {} assets · {} res · arsc={}\n",
        apk.zip_entry_count,
        apk.dex_files.len(),
        apk.native_libs.len(),
        if apk.abis.is_empty() { "—".into() } else { apk.abis.join(",") },
        apk.assets.len(),
        apk.res_count,
        apk.resources_arsc,
    ));
    if let Some(dp) = &device_profile {
        out.push_str(&format!(
            "device budget      {} [{}]\n{}\n",
            dp.id,
            dp.label,
            dp.budget_summary()
        ));
    }

    // features
    if !m.uses_features.is_empty() {
        out.push_str("\nfeatures\n");
        for f in &m.uses_features {
            let what = f
                .name
                .as_deref()
                .or(f.gl_es_version.as_deref().map(|_| "opengles"))
                .unwrap_or("?");
            let version = f.gl_es_version.as_deref().unwrap_or("");
            out.push_str(&format!(
                "  {}{} (required={})\n",
                what,
                if version.is_empty() {
                    String::new()
                } else {
                    format!(" {version}")
                },
                f.required
            ));
        }
    }

    // permissões — estados do motor
    let states = engine.all_states();
    let count = |pred: fn(GrantState) -> bool| states.values().filter(|s| pred(**s)).count();
    out.push_str(&format!(
        "\npermissions ({} declaradas; tabela: {} permissões)\n",
        states.len(),
        PermissionTable::embedded().map(|t| t.len()).unwrap_or(0)
    ));
    out.push_str(&format!(
        "  auto-granted: {} · runtime-pending: {} · special: {} · unknown: {}\n",
        count(|s| s == GrantState::AutoGranted),
        count(|s| s == GrantState::RuntimePending),
        count(|s| s == GrantState::SpecialPending),
        count(|s| s == GrantState::Unknown),
    ));
    for group in ["dangerous (runtime-pending)", "special access", "unknown"] {
        let want = match group {
            "dangerous (runtime-pending)" => GrantState::RuntimePending,
            "special access" => GrantState::SpecialPending,
            _ => GrantState::Unknown,
        };
        let names: Vec<&String> = states
            .iter()
            .filter(|(_, s)| **s == want)
            .map(|(n, _)| n)
            .collect();
        if !names.is_empty() {
            out.push_str(&format!("  {group}:\n"));
            for n in &names {
                out.push_str(&format!("    {n}\n"));
            }
        }
    }

    // componentes
    let comps = &m.application.components;
    let mut by_kind = std::collections::BTreeMap::new();
    for c in comps {
        let entry = by_kind.entry(c.kind.to_string()).or_insert(0usize);
        *entry += 1;
    }
    let summary: Vec<String> = by_kind
        .into_iter()
        .map(|(k, n)| format!("{k}×{n}"))
        .collect();
    out.push_str(&format!(
        "\ncomponents ({}: {})\n",
        comps.len(),
        summary.join(" · ")
    ));
    for c in comps {
        let mut tags = Vec::new();
        if c.is_launcher {
            tags.push("LAUNCHER".into());
        }
        if c.exported_effective {
            tags.push("exported".into());
        }
        if let Some(p) = &c.permission {
            tags.push(format!("perm={p}"));
        }
        if c.exported_effective && c.permission.is_none() {
            tags.push("⚠ sem guarda".into());
        }
        out.push_str(&format!(
            "  {:<15} {}{}\n",
            c.kind.to_string(),
            if c.class_name.is_empty() {
                "(sem nome)"
            } else {
                &c.class_name
            },
            if tags.is_empty() {
                String::new()
            } else {
                format!("  [{}]", tags.join(", "))
            }
        ));
        for f in &c.intent_filters {
            let actions = if f.actions.is_empty() {
                "—".to_string()
            } else {
                f.actions.join(", ")
            };
            let cats = f.categories.join(", ");
            out.push_str(&format!(
                "  {:<15}   filter: actions=[{}] categories=[{}]\n",
                "", actions, cats
            ));
            for d in &f.data {
                let mut parts = Vec::new();
                for (k, v) in [
                    ("scheme", &d.scheme),
                    ("host", &d.host),
                    ("mime", &d.mime_type),
                    ("pathPrefix", &d.path_prefix),
                ] {
                    if let Some(v) = v {
                        parts.push(format!("{k}={v}"));
                    }
                }
                if !parts.is_empty() {
                    out.push_str(&format!("  {:<15}     data: {}\n", "", parts.join(" ")));
                }
            }
        }
    }

    print!("{out}");
    Ok(())
}

// ─── perm ───────────────────────────────────────────────────────────────────

fn cmd_perm_list(path: &Path, json: bool, data: Option<&Path>) -> Result<(), RdError> {
    let apk = open_apk(path)?;
    let table = load_permissions(data)?;
    let engine = PermissionEngine::install(&apk.manifest, table.clone());
    if json {
        let mut v = serde_json::to_value(engine.all_states())
            .map_err(|e| RdError::new("INTERNAL", e.to_string(), "rd-cli"))?;
        if let serde_json::Value::Object(map) = &mut v {
            map.insert(
                "target_sdk".into(),
                serde_json::json!(apk.manifest.target_sdk),
            );
            map.insert("table_size".into(), serde_json::json!(table.len()));
        }
        println!("{v}");
        return Ok(());
    }
    println!(
        "permissions de {} (target {}) — engine fd-permissions",
        apk.manifest.package, apk.manifest.target_sdk
    );
    for (name, state) in engine.all_states() {
        let def = table.get(name);
        let level = def
            .map(|d| format!("{:?}", d.level))
            .unwrap_or_else(|| "?".into());
        println!("  {:<28} {:<16} {level}", format!("{state:?}"), name);
    }
    Ok(())
}

fn cmd_perm_info(name: &str, json: bool, data: Option<&Path>) -> Result<(), RdError> {
    let table = load_permissions(data)?;
    let def = table.get(name).ok_or_else(|| {
        RdError::missing_entry(format!("permission {name:?}"))
            .with_suggestion("check data/permissions.toml — names follow android.permission.*")
    })?;
    if json {
        println!("{}", to_json(def));
        return Ok(());
    }
    println!("permission       {}", def.name);
    println!("level            {:?}", def.level);
    if let Some(g) = def.group_name() {
        println!("group            {g}");
    }
    if !def.flags.is_empty() {
        println!("flags            {}", def.flags.join(", "));
    }
    println!("since_api        {}", def.since_api);
    if !def.description.is_empty() {
        println!("description      {}", def.description);
    }
    Ok(())
}

fn cmd_perm_audit(path: &Path, json: bool, data: Option<&Path>) -> Result<(), RdError> {
    let apk = open_apk(path)?;
    let table = load_permissions(data)?;
    let engine = PermissionEngine::install(&apk.manifest, table);
    let findings = engine.audit(&apk.manifest);
    if json {
        println!("{}", to_json(&findings));
        return Ok(());
    }
    println!(
        "audit de {} — {} finding(s)\n",
        apk.manifest.package,
        findings.len()
    );
    for f in &findings {
        let sev = format!("{:?}", f.severity).to_uppercase();
        println!("  [{sev:<7}] {} — {}", f.code, f.message);
    }
    Ok(())
}

// ─── device ─────────────────────────────────────────────────────────────────

fn cmd_device_list(json: bool, data: Option<&Path>) -> Result<(), RdError> {
    let table = DeviceTable::load(data)?;
    if json {
        println!("{}", to_json(&table.device));
        return Ok(());
    }
    println!("perfis de device (piso marcado com ★):");
    for d in &table.device {
        let star = if d.is_hardware_floor() { " ★" } else { "" };
        println!(
            "  {:<18} {:<24} {}×{} @{}dpi · heap {} MB · api {}–{}{}",
            d.id,
            d.label,
            d.width_px,
            d.height_px,
            d.density_dpi,
            d.ram_app_heap_mb,
            d.min_api,
            d.max_api,
            star
        );
    }
    Ok(())
}

fn cmd_device_show(id: &str, json: bool, data: Option<&Path>) -> Result<(), RdError> {
    let table = DeviceTable::load(data)?;
    let d = table.get(id).ok_or_else(|| {
        RdError::missing_entry(format!("device profile {id:?}"))
            .with_suggestion("run `rd device list`")
    })?;
    if json {
        println!("{}", to_json(d));
        return Ok(());
    }
    println!("device            {} ({})", d.id, d.label);
    println!("class             {}", d.class);
    println!(
        "screen            {}×{} · {}dpi · {}\" · {} · {}Hz",
        d.width_px, d.height_px, d.density_dpi, d.diagonal_in, d.aspect_ratio, d.refresh_hz
    );
    println!(
        "cpu               {} cores · single-thread {}% de 1 core · quota total {:.2} cores",
        d.cpu_cores,
        (d.cpu_single_thread_factor * 100.0) as u32,
        d.cpu_quota_cores
    );
    println!(
        "gpu               {} (fill-rate: {})",
        d.gpu, d.fill_rate_budget
    );
    println!(
        "ram               device {} MB · processo ≤ {} MB · heap app ≤ {} MB",
        d.ram_device_mb, d.ram_process_cap_mb, d.ram_app_heap_mb
    );
    println!(
        "storage           {} GB · I/O: {}",
        d.storage_quota_gb, d.io_profile
    );
    println!(
        "os                fábrica API {} · suportado {}–{}",
        d.factory_api, d.min_api, d.max_api
    );
    if !d.flags.is_empty() {
        println!("flags             {}", d.flags.join(", "));
    }
    if !d.notes.is_empty() {
        println!("notes             {}", d.notes);
    }
    println!("\nbudgets           {}", d.budget_summary());
    Ok(())
}

// ─── behavior ───────────────────────────────────────────────────────────────

fn cmd_behavior(
    target: Option<u32>,
    impact: Option<&str>,
    json: bool,
    data: Option<&Path>,
) -> Result<(), RdError> {
    let table = SwitchTable::load(data)?;
    let target = target.unwrap_or(36);
    if !(26..=36).contains(&target) {
        return Err(RdError::new(
            "OUT_OF_RANGE",
            format!("target {target} fora do piso/teto (26–36)"),
            "fd-behavior",
        ));
    }
    let mut active = table.active_for(target);
    if let Some(imp) = impact {
        active.retain(|s| s.impact == imp);
    }
    if json {
        let v = serde_json::json!({
            "target_sdk": target,
            "active_switches": active,
            "inactive_count": table.inactive_for(target).len(),
        });
        println!("{v}");
        return Ok(());
    }
    println!(
        "behavior switches ativos com targetSdk {target} ({} de {})\n",
        active.len(),
        table.switch.len()
    );
    for s in &active {
        println!(
            "  @{}  {:<34} [{:<17}] {} ({})",
            s.api, s.title, s.impact, s.default_state, s.id
        );
        println!("       {}", s.summary);
    }
    Ok(())
}
