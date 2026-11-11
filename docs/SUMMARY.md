# Summary

[Introduction](./index.md)

# Roadmap & Official Scoreboards

- [Roadmap (M0 → M12)](./ROADMAP.md)
- [Target Apps (end-to-end)](./target-apps.md)
- [Official Compatibility Matrix](./compat-matrix.md)
- [Appendices B–F — Implementation Plan & Verified State](./appendix-b-f-plan.md)
- [Android API Coverage — Appendix C](./api-coverage.md)
- [Appendix C — Public Android API Inventory (API 26–36)](./appendix-c-inventory.md)
- [Appendix C Literal Completion Ledger](./appendix-c-literal-completion.md)
- [AndroidX / Jetpack — Appendix D](./jetpack.md)
- [Google Play Services / Firebase — Appendix E](./gms.md)
- [Appendices D + E Executable Ledger](./appendix-d-e.md)
- [Language & Engine Registry — Appendix G](./engine-registry.md)

# MCP Agent Plane & Test Runner

- [MCP Tools Reference](./mcp-tools.md)
- [Screen Observation & Live Viewer](./mcp-observe.md)
- [Reliable MCP Automation & Diagnostics](./mcp-automation.md)
- [Test Suite Format & JUnit XML (`rd test`)](./suite-format.md)

# Runtime Subsystems & Verification

- [M9 — Advanced Components & Isolated NDK](./m9.md)
- [Real C, C++ & Rust APK Execution](./native-languages.md)
- [M10 — Programmable Virtual Devices](./m10.md)
- [M11 — WASM engine (slice S1)](./m11-wasm.md)
- [Generational Garbage Collector](./gc.md)
- [Fuzzing Binary Parsers](./FUZZING.md)
- [Appendix G Validation (2026-10-14)](./appendix-g-validation.md)
- [Appendix G Integration Validation (2026-10-20)](./appendix-g-integration.md)
- [Comprix v1.6.0 End-to-End via MCP](./comprix-e2e.md)
- [Comprix v1.6.0 — Launch Profile](./comprix-launch-profile.md)
- [Arquivo: documentos históricos](./archive/README.md)
  - [RD Sentinel (retirado, histórico)](./archive/rd-sentinel.md)
  - [Runtime Integration Validation (2026-10-20)](./archive/integration-validation.md)
  - [GC & Runtime Efficiency Verification (2026-10-20)](./archive/performance-gc-20261020.md)
- [Comprix v1.6.0 — MCP Latency Baseline](./comprix-latency.md)

# Architecture Decision Records (ADRs)

Toda decisão de arquitetura vive em `docs/adr/`. O índice abaixo é **gerado**
por `scripts/gen_adr_index.py` a partir dos arquivos — não edite à mão.

<!-- adr:begin -->
- [Índice de ADRs](./adr/README.md)
- [ADR 001 — Servidor MCP do rd-agent sem rmcp (JSON-RPC 2.0 próprio, zero deps)](./adr/001-mcp-sem-rmcp-msrv.md)
- [ADR 002 — Entradas ZIP duplicadas: aceitar, sinalizar e contar por nome único](./adr/002-zip-duplicate-entries.md)
- [ADR 003 — Registro de engines separado de execução](./adr/003-appendix-g-engine-registry.md)
- [ADR 003 — M8 compat host: interceptação seletiva + estado real consultável](./adr/003-m8-compat-host.md)
- [ADR 004 — M9 isolated native execution and advanced components](./adr/004-m9-native-isolation.md)
- [ADR 005 — Real SQLite through an isolated child process](./adr/005-sqlite-subprocess.md)
- [ADR 006 — non-moving generations and live-frame GC roots](./adr/006-generational-gc-and-live-frame-roots.md)
- [ADR 007 — opt-in Cranelift integer & control-flow JIT (`MethodJit`)](./adr/007-cranelift-leaf-jit.md)
- [ADR 008 — 15 GB APK support with ≤ 1,500 ms cold start on the Moto E5 floor](./adr/008-15gb-apk-and-1500ms-cold-start.md)
- [ADR 009 — Process-cold < 26 ms to first frame (live mode, all heavy features)](./adr/009-process-cold-26ms.md)
- [ADR 010 — RD Sentinel: Dependabot-like anti-regression bot beyond CI](./adr/010-rd-sentinel-anti-regression-bot.md)
- [ADR 011 — M10 virtual devices, sensors, media & deterministic agent controls](./adr/011-m10-virtual-devices.md)
- [ADR 012 — Registros executáveis dos Apêndices D e E](./adr/012-appendix-d-e-registries.md)
- [ADR 013 — Estratégia de execução de engines do M11](./adr/013-m11-engine-execution-strategy.md)
- [ADR 014 — LazyArsc + LazyStringPool: decodificação sob demanda de resources.arsc](./adr/014-lazyarsc-lazystringpool.md)
- [ADR 015 — moto-e5 process RSS cap raised from 512 MB to 1024 MB](./adr/015-moto-e5-process-rss-1gb.md)
- [ADR 016 — Piso de API de execução: minSdk 26 → 9 (tier T0 best-effort)](./adr/016-api-floor-9.md)
- [ADR 017 — Estado host thread-local: fronteira de `Engine`, pacote e sessão](./adr/017-host-thread-local-state-engine-boundary.md)
- [ADR 018 — Restaurar o RD Sentinel após exclusão acidental (`4489813`)](./adr/018-restore-rd-sentinel-after-accidental-deletion.md)
- [ADR 019 — Renderer M4: tiny-skia + shaping (spec) vs. rasterizador próprio atual](./adr/019-renderer-tiny-skia-pendente.md)
- [ADR 020 — M11 S2: React Native (LANG-05) — estratégia de execução](./adr/020-m11-s2-react-native.md)
- [ADR 021 — Layering: executar dois APKs ao mesmo tempo (um por cima do outro)](./adr/021-apk-layering.md)
- [ADR 022 — Suporte a XAPK (bundle de splits)](./adr/022-xapk-bundles.md)
- [ADR 023 — Revisão do ADR 001: múltiplas sessões por processo (etapa 1 do layering)](./adr/023-revisa-adr-001-multiplas-sessoes.md)
- [ADR 024 — Inventário do estado thread-local do host e caminho para engines concorrentes](./adr/024-inventario-thread-local-e-caminho-para-engines-concorrentes.md)
- [ADR 025 — Runtime do Room no VM: `init` real sobre o SQLite de framework](./adr/025-room-runtime-sobre-sqlite-framework.md)
- [ADR 026 — `virtual_hardware_enabled` vale no despacho de bytecode (fecha a #229)](./adr/026-m10-gate-no-despacho-de-bytecode.md)
- [ADR 027 — Metadados de `Class` para reflexão do Kotlin: sem falsear](./adr/027-reflexao-class-metadados-genericos.md)
- [ADR 028 — Memória compacta: valores de 16 bytes, nomes internados, código decodificado menor](./adr/028-memoria-compacta-valores-e-nomes-internados.md)
- [ADR 029 — Composição do RSS e próximos passos de memória](./adr/029-composicao-rss-e-proximos-passos-de-memoria.md)
- [ADR 030 — Sessões simultâneas: custo marginal medido e caminho para o alvo](./adr/030-sessoes-compartilhamento-e-alvo-de-memoria.md)
- [ADR 031 — DEX e APK compartilhados entre sessões do `rd mcp`](./adr/031-dex-e-apk-compartilhados-entre-sessoes.md)
- [ADR 032 — Cache de código decodificado compartilhado entre sessões](./adr/032-cache-de-codigo-compartilhado-entre-sessoes.md)
- [ADR 033 — Layout de campos de instância por classe](./adr/033-layout-de-campos-por-classe.md)
- [ADR 034 — Caminho quente de `invoke` (cache por site e alocações por chamada)](./adr/034-caminho-quente-de-invoke.md)
- [ADR 035 — Identidades inteiras (classe, campo, método) no caminho quente](./adr/035-identidades-inteiras-no-caminho-quente.md)
- [ADR 036 — Exceção datada: fixture DEX reduzido do RxJava 3](./adr/036-excecao-fixture-dex-rxjava.md)
- [ADR 037 — Stubs Android dirigidos por tabela, sem relocação e com erro delgado](./adr/037-stubs-android-dirigidos-por-tabela.md)
- [ADR 038 — Gate de benchmark invariante à classe de runner (calibração normalizada)](./adr/038-gate-de-benchmark-invariante-ao-runner.md)
<!-- adr:end -->
