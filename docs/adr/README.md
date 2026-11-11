# Índice de Architecture Decision Records

<!-- GERADO por scripts/gen_adr_index.py a partir de docs/adr/*.md.
     Não edite à mão: rode `python3 scripts/gen_adr_index.py --write`. -->

39 registros de decisão. Cada ADR é imutável; uma decisão nova
que revisa uma anterior entra como ADR novo que a referencia.

| ADR | Decisão | Data | Status |
| --- | --- | --- | --- |
| [001](001-mcp-sem-rmcp-msrv.md) | Servidor MCP do rd-agent sem rmcp (JSON-RPC 2.0 próprio, zero deps) | — | aceito e implementado; revisto pelo ADR 023 |
| [002](002-zip-duplicate-entries.md) | Entradas ZIP duplicadas: aceitar, sinalizar e contar por nome único | 2026-10-11 | aceito |
| [003](003-appendix-g-engine-registry.md) | Registro de engines separado de execução | 2026-10-14 | aceito para registro/detecção; novos runtimes continuam… |
| [003](003-m8-compat-host.md) | M8 compat host: interceptação seletiva + estado real consultável | — | aceito e implementado |
| [004](004-m9-native-isolation.md) | M9 isolated native execution and advanced components | — | Accepted for implementation; acceptance remains subject… |
| [005](005-sqlite-subprocess.md) | Real SQLite through an isolated child process | — | accepted |
| [006](006-generational-gc-and-live-frame-roots.md) | non-moving generations and live-frame GC roots | — | implemented; differential/regression and real-APK valida… |
| [007](007-cranelift-leaf-jit.md) | opt-in Cranelift integer & control-flow JIT (`MethodJit`) | — | accepted and upgraded from straight-line leaf prototype… |
| [008](008-15gb-apk-and-1500ms-cold-start.md) | 15 GB APK support with ≤ 1,500 ms cold start on the Moto E5 floor | 2026-10-21 | Accepted |
| [009](009-process-cold-26ms.md) | Process-cold < 26 ms to first frame (live mode, all heavy features) | — | Accepted |
| [010](010-rd-sentinel-anti-regression-bot.md) | RD Sentinel: Dependabot-like anti-regression bot beyond CI | — | retirada |
| [011](011-m10-virtual-devices.md) | M10 virtual devices, sensors, media & deterministic agent controls | 2026-10-25 | aceito e implementado |
| [012](012-appendix-d-e-registries.md) | Registros executáveis dos Apêndices D e E | 2026-10-25 | aceito; runtime de shims em implementação incremental so… |
| [013](013-m11-engine-execution-strategy.md) | Estratégia de execução de engines do M11 | 2026-10-25 | proposto |
| [014](014-lazyarsc-lazystringpool.md) | LazyArsc + LazyStringPool: decodificação sob demanda de resources.arsc | 2026-10-25 | proposto |
| [015](015-moto-e5-process-rss-1gb.md) | moto-e5 process RSS cap raised from 512 MB to 1024 MB | 2026-10-25 | accepted. This is an owner decision |
| [016](016-api-floor-9.md) | Piso de API de execução: minSdk 26 → 9 (tier T0 best-effort) | 2026-10-25 | proposto |
| [017](017-host-thread-local-state-engine-boundary.md) | Estado host thread-local: fronteira de `Engine`, pacote e sessão | 2026-10-25 | proposto |
| [018](018-restore-rd-sentinel-after-accidental-deletion.md) | Restaurar o RD Sentinel após exclusão acidental (`4489813`) | — | histórico |
| [019](019-renderer-tiny-skia-pendente.md) | Renderer M4: tiny-skia + shaping (spec) vs. rasterizador próprio atual | — | Aceito |
| [020](020-m11-s2-react-native.md) | M11 S2: React Native (LANG-05) — estratégia de execução | 2026-11-01 | proposto |
| [021](021-apk-layering.md) | Layering: executar dois APKs ao mesmo tempo (um por cima do outro) | 2026-11-02 | proposto |
| [022](022-xapk-bundles.md) | Suporte a XAPK (bundle de splits) | — | aceito |
| [023](023-revisa-adr-001-multiplas-sessoes.md) | Revisão do ADR 001: múltiplas sessões por processo (etapa 1 do layering) | 2026-11-02 | aceito |
| [024](024-inventario-thread-local-e-caminho-para-engines-concorrentes.md) | Inventário do estado thread-local do host e caminho para engines concorrentes | 2026-11-03 | proposto |
| [025](025-room-runtime-sobre-sqlite-framework.md) | Runtime do Room no VM: `init` real sobre o SQLite de framework | 2026-11-03 | aceito na etapa 1 |
| [026](026-m10-gate-no-despacho-de-bytecode.md) | `virtual_hardware_enabled` vale no despacho de bytecode (fecha a #229) | 2026-11-03 | aceito |
| [027](027-reflexao-class-metadados-genericos.md) | Metadados de `Class` para reflexão do Kotlin: sem falsear | 2026-11-03 | aceito |
| [028](028-memoria-compacta-valores-e-nomes-internados.md) | Memória compacta: valores de 16 bytes, nomes internados, código decodificado menor | 2026-11-03 | aceito |
| [029](029-composicao-rss-e-proximos-passos-de-memoria.md) | Composição do RSS e próximos passos de memória | 2026-11-03 | proposto |
| [030](030-sessoes-compartilhamento-e-alvo-de-memoria.md) | Sessões simultâneas: custo marginal medido e caminho para o alvo | 2026-11-04 | proposto; atualizado em 2026-11-04. Passo 1 da decisão |
| [031](031-dex-e-apk-compartilhados-entre-sessoes.md) | DEX e APK compartilhados entre sessões do `rd mcp` | 2026-11-04 | aceito e implementado nesta PR |
| [032](032-cache-de-codigo-compartilhado-entre-sessoes.md) | Cache de código decodificado compartilhado entre sessões | 2026-11-04 | aceito e implementado nesta PR |
| [033](033-layout-de-campos-por-classe.md) | Layout de campos de instância por classe | 2026-11-04 | proposto |
| [034](034-caminho-quente-de-invoke.md) | Caminho quente de `invoke` (cache por site e alocações por chamada) | 2026-11-04 | proposto |
| [035](035-identidades-inteiras-no-caminho-quente.md) | Identidades inteiras (classe, campo, método) no caminho quente | 2026-11-04 | proposto |
| [036](036-excecao-fixture-dex-rxjava.md) | Exceção datada: fixture DEX reduzido do RxJava 3 | 2026-11-05 | proposta, aguarda aplicação do label `sentinel:policy-ap… |
| [037](037-stubs-android-dirigidos-por-tabela.md) | Stubs Android dirigidos por tabela, sem relocação e com erro delgado | 2026-11-10 | implementado |
| [038](038-gate-de-benchmark-invariante-ao-runner.md) | Gate de benchmark invariante à classe de runner (calibração normalizada) | 2026-11-10 | implementado |
