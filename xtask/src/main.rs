//! xtask — tarefas de build do RustyDroid (GH-01).
//!
//! `cargo xtask compat-report` → gera `docs/compat-matrix.md` a partir dos
//! dados versionados + estado real das crates. Nunca manual: o placar oficial
//! do projeto é gerado.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

fn main() {
    if let Err(e) = run() {
        eprintln!("xtask error: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()))
        // xtask roda de dentro de xtask/ — sobe para a raiz
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    match args.next().as_deref() {
        Some("compat-report") => {
            let out = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| root.join("docs").join("compat-matrix.md"));
            compat_report(&root, &out)
        }
        Some("--help") | Some("help") | None => {
            println!("xtask — RustyDroid build tasks");
            println!();
            println!("USAGE: cargo xtask <COMMAND>");
            println!();
            println!("COMMANDS:");
            println!("  compat-report [out.md]  gera docs/compat-matrix.md (placar oficial)");
            Ok(())
        }
        Some(other) => bail!("unknown task {other:?} — try `cargo xtask --help`"),
    }
}

/// Estado real das crates (fonte da verdade: este arquivo, junto do roadmap).
/// issue #33: sincronizado com M2 (42b6532) — rd-dex e rd-vm NÃO são mais stub.
const CRATE_STATUS: &[(&str, &str, &str)] = &[
    (
        "rd-apk",
        "M0",
        "ZIP + AXML + arsc + assinaturas + manifest model completo",
    ),
    (
        "rd-framework",
        "M0",
        "fd-permissions (motor completo + gating maxSdk/since_api) · fd-devices · fd-behavior",
    ),
    (
        "rd-cli",
        "M0–M2",
        "inspect · perm · device · behavior · dex disasm · vm exec (contrato de erro JSON)",
    ),
    (
        "rd-dex",
        "M1",
        "parser DEX completo + disassembler smali validado vs baksmali + fuzz targets",
    ),
    (
        "rd-vm",
        "M2",
        "interpretador Dalvik mínimo — métodos puros de APK real (golden harness vs JVM) + intrinsics",
    ),
    ("rd-render", "M4", "stub NOT_IMPLEMENTED"),
    ("rd-agent", "M5", "stub NOT_IMPLEMENTED"),
    ("rd-jni", "M3+", "stub NOT_IMPLEMENTED"),
    ("rd-ndk", "M9", "stub NOT_IMPLEMENTED"),
];

fn compat_report(root: &Path, out: &Path) -> Result<()> {
    let perm_toml = std::fs::read_to_string(root.join("data/permissions.toml"))
        .context("data/permissions.toml ausente")?;
    let devices_toml = std::fs::read_to_string(root.join("data/devices.toml"))
        .context("data/devices.toml ausente")?;
    let switches_toml = std::fs::read_to_string(root.join("data/behavior-switches.toml"))
        .context("data/behavior-switches.toml ausente")?;

    let perms: toml::Value = toml::from_str(&perm_toml)?;
    let devices: toml::Value = toml::from_str(&devices_toml)?;
    let switches: toml::Value = toml::from_str(&switches_toml)?;

    let perm_count = perms
        .get("permission")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let role_count = perms
        .get("role")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let device_count = devices
        .get("device")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let switch_count = switches
        .get("switch")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let mut md = String::new();
    md.push_str("# compat-matrix — placar oficial do RustyDroid\n\n");
    md.push_str("> **GERADO** por `cargo xtask compat-report` — não editar à mão.\n");
    md.push_str("> Lei de Ouro: nenhuma API ausente quebra o app em silêncio — ");
    md.push_str("tudo responde `NOT_IMPLEMENTED(id)` e aparece aqui.\n\n");
    md.push_str("## Estado por fase (roadmap M0–M12)\n\n");
    md.push_str("| Fase | Entrega | Status |\n|---|---|---|\n");
    md.push_str("| M0 | workspace · rd-apk inspect · motor de permissões · devices · behavior | **✅ implementado** |\n");
    md.push_str("| M1 | rd-dex 100% + disassembler | **✅ implementado** |\n");
    md.push_str("| M2 | VM Dalvik mínima | **✅ implementado** |\n");
    for (fase, entrega) in [
        ("M3", "framework essencial headless"),
        ("M4", "render + UI dump"),
        ("M5", "agente MCP v1 (~20 tools)"),
        ("M6", "executor de testes + compat automatizada"),
        ("M7", "performance + gate piso E5"),
        ("M8", "compat real (views/rede/dados)"),
        ("M9", "componentes avançados + NDK"),
        ("M10", "dispositivos virtuais"),
        ("M11", "engines (Flutter/RN/Mono/wasm)"),
        ("M12", "ecossistema GH completo"),
    ] {
        md.push_str(&format!("| {fase} | {entrega} | ⬜ |\n"));
    }

    md.push_str("\n## Crates\n\n| Crate | Fase | Escopo |\n|---|---|---|\n");
    for (name, fase, escopo) in CRATE_STATUS {
        md.push_str(&format!("| `{name}` | {fase} | {escopo} |\n"));
    }

    md.push_str("\n## Dados versionados (M0)\n\n");
    md.push_str(&format!(
        "- permissions.toml: **{perm_count}** permissões + {role_count} roles (Apêndice B)\n"
    ));
    md.push_str(&format!(
        "- devices.toml: **{device_count}** perfis (piso: `moto-e5`)\n"
    ));
    md.push_str(&format!(
        "- behavior-switches.toml: **{switch_count}** comutadores targetSdk 26→36 (PRF-07)\n"
    ));
    md.push_str("\n## Compatibilidade por módulo (checklist PARTE 0)\n\n");
    md.push_str("- Núcleo (0.0): parcial (CORE-01..06 M0; rd-dex M1; VM M2 — métodos puros executam)\n");
    md.push_str("- Framework/render/agente: `NOT_IMPLEMENTED(module_id)` estruturado (M3+)\n");

    std::fs::create_dir_all(out.parent().unwrap_or(Path::new(".")))?;
    std::fs::write(out, &md).with_context(|| format!("writing {}", out.display()))?;
    println!("compat-matrix gerada: {}", out.display());
    Ok(())
}
