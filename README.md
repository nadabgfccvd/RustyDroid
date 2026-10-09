<p align="center">
  <img src="logo.png" alt="RustyDroid" width="180" />
</p>

<h1 align="center">🦀 RustyDroid</h1>

<p align="center">
  <strong>Rust-native Android runtime — run real APKs on Linux.<br/>No VM. No KVM. No guest kernel.</strong><br/><br/>
  <em>Runtime Android em Rust — executa APKs reais no Linux.<br/>Sem VM. Sem KVM. Sem kernel convidado.</em>
</p>

<p align="center">
  <a href="https://github.com/nadabgfccvd/RustyDroid/actions/workflows/ci.yml"><img src="https://github.com/nadabgfccvd/RustyDroid/actions/workflows/ci.yml/badge.svg" alt="CI"/></a>
  <img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue" alt="License: MIT OR Apache-2.0"/>
  <img src="https://img.shields.io/badge/spec-v2.1-8A2BE2" alt="Spec v2.1"/>
  <img src="https://img.shields.io/badge/platform-Linux%20x86__64-lightgrey" alt="Platform: Linux x86_64"/>
  <img src="https://img.shields.io/badge/API-26%20%E2%86%92%2036-brightgreen" alt="Android API 26→36"/>
  <img src="https://img.shields.io/badge/built%20with-AI%20assistance-FF6F00" alt="Built with AI assistance"/>
</p>

---

## ✨ What is RustyDroid?

RustyDroid is a **"Wine for Android"**: instead of emulating an entire Android OS
(QEMU-style, slow without KVM), it parses APKs directly and executes them natively:

- 📦 **Parses** the APK (ZIP, binary `AndroidManifest.xml`, `resources.arsc`, `classes.dex`)
- ⚙️ **Executes** Dalvik bytecode in a **Rust VM** (interpreter → Cranelift JIT later)
- 🧩 **Implements** `android.*` framework APIs natively in Rust
- 🎨 **Renders** the UI in software (tiny-skia): headless PNG, desktop window, web canvas
- 🤖 **Drives everything through MCP** — built for AI agents from day one

### Why it's different

| | Traditional emulator | **RustyDroid** |
|---|---|---|
| Needs KVM/VMX | ✅ required for speed | ❌ never |
| Guest OS in RAM | 2–4 GB | **0** (no guest OS) |
| Boot time | minutes | **seconds** (no boot at all) |
| Runs in a plain container | rarely | ✅ designed for it |
| AI-agent control plane | adb, screen scraping | **MCP + ~40 structured tools** |
| Perf baseline | modern desktop | **Motorola Moto E5 (2 GB RAM)** floor |

### 🤖 Built with AI — heavily

> **This project is developed with heavy AI assistance** — specification, architecture,
> code and CI. Every change passes through review gates (tests, fuzzing, benchmarks)
> before landing. We believe in being transparent about it: **AI-assisted, human-directed.**

---

## 🗺️ Roadmap (M0 → M12)

| Phase | Delivers |
|---|---|
| **M0** | Workspace + CI + APK inspector + permission engine (full Android permission matrix as data) |
| **M1** | DEX parser 100% + disassembler (validated against baksmali, fuzzed) |
| **M2** | Minimal Dalvik VM (all core opcodes, real APK methods run) |
| **M3** | Essential framework: Activity lifecycle, Handler/Looper, LayoutInflater, Resources |
| **M4** | Software rendering + UI dump (uiautomator-compatible) |
| **M5** | **AI-agent mode v1**: MCP server with ~20 tools |
| **M6** | Test executor: scenario DSL, JUnit XML, automated compat matrix |
| **M7** | Performance + **Moto E5 floor gate** (release blocked if budgets blown) |
| **M8** | Real-world compat: RecyclerView, Fragments, OkHttp shim, SQLite/Room, Services |
| **M9** | Advanced components (widgets, IME, a11y) + NDK (`.so` via ELF/JNI) |
| **M10** | Virtual devices: camera, GPS, sensors, biometrics — programmable by the agent |
| **M11** | Engines: Flutter → React Native → Mono → WASM |
| **M12** | Full GitHub ecosystem: docs site, live compat matrix, releases → **v1.0** |

**Compatibility floor:** `minSdkVersion 26` (Android 8.0 Oreo) · **ceiling:** latest stable (`targetSdk 36`).
APKs below the floor are inspected fine but refuse to run with a structured `BELOW_FLOOR` error — never a crash.

---

## 📦 Planned repository layout

```
rustydroid/
├── crates/
│   ├── rd-apk/        # ZIP, AXML, resources.arsc, signatures, manifest model
│   ├── rd-dex/        # DEX 100% + disassembler + fuzz targets
│   ├── rd-vm/         # Dalvik VM in Rust (interpreter → Cranelift JIT)
│   ├── rd-framework/  # android.* natively in Rust, by domain
│   ├── rd-render/     # headless / window / web renderers + UI dump
│   └── rd-agent/      # MCP (rmcp) + JSON-RPC + scenario DSL + coverage
├── data/              # permissions.toml, api-coverage.toml, devices.toml (incl. moto-e5)
├── golden-apks/       # open-source APK test suite (download script, no binaries)
├── docs/              # ROADMAP, compat-matrix (live), ADRs, guide
└── xtask/             # bench, compat-report, golden-run, mcp-doc-gen
```

## 📄 Full specification

Everything above is specified in detail in **[`RustyDroid.md`](./RustyDroid.md)** (v2.1):

- **~85 selectable modules** with installation checklist & presets
- **Complete Android permissions matrix** (protection levels, groups, roles, AppOps)
- **`android.*` API surface** mapped by domain + Jetpack/Play Services strategy
- **28 languages/engines** that can live inside an APK — and how each one runs
- **GitHub project plan**: CI/CD, governance, issue templates, release automation

## 🤝 Contributing

PRs and issues are welcome! Start with [`CONTRIBUTING.md`](./CONTRIBUTING.md).
**Found an API that doesn't work?** Open a `compat-gap` issue — it feeds the compatibility scoreboard automatically.

## 🔒 Security

See [`SECURITY.md`](./SECURITY.md).

## ⚖️ License

Dual-licensed under [`MIT`](./LICENSE-MIT) OR [`Apache-2.0`](./LICENSE-APACHE) — your choice.

---

<details>
<summary>🇧🇷 Sobre (PT-BR)</summary>

**RustyDroid** é uma "Wine para Android": em vez de emular um sistema Android inteiro
(lento sem KVM), ele parseia APKs diretamente e executa o bytecode Dalvik numa **VM
escrita em Rust**, com as APIs `android.*` implementadas nativamente, renderização por
software e um **modo dedicado a agentes de IA (MCP)** com ~40 ferramentas estruturadas.

- **Piso de performance:** Motorola Moto E5 (Snapdragon 425, 2 GB RAM) — todos os
  budgets de performance são calibrados nele, não em emuladores modernos
- **Piso de API:** `minSdkVersion 26` · **teto:** `targetSdk 36`
- **Desenvolvido pesadamente com auxílio de IA** — especificação, código e CI, com
  revisão humana e gates de teste
- A especificação completa (v2.1) está em [`RustyDroid.md`](./RustyDroid.md)

</details>

---

<p align="center">
  <sub>🦀 RustyDroid — Android without Android. Made with 🤖 AI assistance + ☕ human review.</sub>
</p>
