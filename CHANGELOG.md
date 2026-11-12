# Changelog

All notable changes to **RustyDroid** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
(see [`GOVERNANCE.md`](./GOVERNANCE.md) for pre-1.0 milestone versioning and MSRV policy).

## [Unreleased]

### Changed

- **Documentação revisada (2026-11-11):** o RD Sentinel foi retirado e o guia dele
  foi para `docs/archive/` (ADR 010, emenda). ADR 011 ganhou emenda: `virtual-devices`
  é opt-in na compilação, não `default`. O catálogo MCP tem 52 tools (não 47).
  Caminhos de teste desatualizados foram corrigidos. Os documentos históricos
  foram para `docs/archive/` (índice em `docs/archive/README.md`).

### Added

- **Página de releases viva no site (M12):** `docs/releases.md` é renderizada por
  `scripts/releases_live.py` a partir de `GET /repos/:owner/:repo/releases` no build do
  `docs.yml`, com uma linha por versão (data, nº de artefatos, tamanho), a lista de
  arquivos de cada payload e a instrução de verificação da procedência
  (`gh attestation verify`). Política de falha **diferente** da matriz de
  compatibilidade: uma lista de releases não envelhece, então se a API não responder o
  script sai sem tocar na página e o build publica o último snapshot versionado — só uma
  resposta válida e vazia troca o bloco por "nenhuma release publicada ainda".
  Self-test hermético `tests/scripts/releases_live_test.sh` (21 casos) no `ci.yml`.
- **`VectorDrawable` (`crates/rd-vm/src/vector_drawable.rs`) — apps AppCompat não abriam sem ele:**
  `NewPipe` morria antes da primeira tela com *"This app has been built with an incorrect
  configuration. Please configure your build for VectorDrawableCompat."* — que é o
  `AppCompatDrawableManager.checkVectorDrawableSetup()` fazendo `getDrawable(abc_vector_test)` e
  exigindo uma instância de `android.graphics.drawable.VectorDrawable`. Agora `getDrawable`
  resolve `res/*.xml` como vetor de verdade: `<vector>` + `<path>` + `<group>`
  (translate/scale/rotate/pivot) e o subconjunto SVG de `pathData` (`M L H V C S Q T A Z`,
  absolutos e relativos) achatado em polilinhas, com cores `@color` resolvidas no ARSC e
  `draw` escalando viewport → bounds. Fora do modelo (e dito em `rd_vgaps`): `<gradient>`,
  `clip-path` e stroke de polilinha (ícones só-contorno são preenchidos com a cor do stroke).
  Testes: 14 unitários + 4 de integração (classe, intrínseco, subtype, escala para bounds).
- **Matriz de compatibilidade viva no site (M12):** `nightly-compat.yml` continua só
  produzindo evidência (o artefato `nightly-compat-reports`, retenção de 14 dias) e o
  `docs.yml` segue como único escritor na `gh-pages`. O job `build` agora busca a última
  run **bem-sucedida** do nightly (`actions: read`) e `scripts/compat_live.py` renderiza
  `docs/compat-live.md` + `docs/compat-badge.svg` antes do `mdbook build`; um `schedule`
  diário (06:17 UTC, atrás do nightly das 04:43) republica o site sem depender de commit.
  Sem run, artefato expirado ou sem permissão, a página publica **"sem dados"** em vez de
  placar velho. Renderizador determinístico (sem `now()`) com self-test hermético
  `tests/scripts/compat_live_test.sh` (23 casos), rodando também no `ci.yml`.
- **`android.webkit.CookieManager` (`crates/rd-vm/src/webkit_cookies.rs`):** process-wide `getInstance()`, `setCookie`/`getCookie` with the RFC 6265 subset APKs use (host-only vs `Domain=`, `Path=`, `Secure` refused over plain http, `HttpOnly`, `Max-Age` on the virtual clock), `removeAllCookies`/`removeSessionCookies` answering through the `ValueCallback`, `setAcceptCookie`/`acceptCookie`, `flush`, third-party-cookie setters. Not modelled, and documented as such: public-suffix rejection, SameSite, partitioned (CHIPS) cookies. Test: `crates/rd-vm/tests/webkit_cookies.rs` (7).
- **`Boolean.getBoolean(String)`** reads the platform system-property table and parses it like the JDK (`true` case-insensitive, anything else — including an absent property — is `false`).
- **`java.util.EnumMap`:** the host Map model over `Landroid`-style enum-keyed maps (`<init>(Class)`, `<init>(EnumMap)`, `put/get/remove/containsKey/size/isEmpty/clear/keySet/values/entrySet` through the shared `map_intrinsic`), plus the `Map` interface in the builtin hierarchy — without it the desugared `j$/util/stream/d7` `<clinit>` died and every later use of that class answered `NoClassDefFoundError`. Also `RD_TRACE_CLINIT=1`, which prints the class and the original cause the first time a `<clinit>` fails (the JLS 12.4.2 error alone hides it).
- **`java.util.EnumSet` completion:** host `Set` over an `Ljava/util/EnumSet;` instance (hierarchy → `AbstractSet` → `AbstractCollection`), with the JDK's `IllegalArgumentException` for a duplicate element and `NullPointerException` for null. Factories: `of(1..5 elements)` + varargs `of(first, rest...)`, `copyOf(Collection)`/`copyOf(EnumSet)`, `noneOf(Class)`, `allOf(Class)` (all constants in declaration order via DEX `class_data` static fields), `range(first, last)` (inclusive slice, reversed → IAE) and `complementOf(set)` (all constants minus the set). Enum classes outside the DEX stay a typed gap, never invented. Test: `crates/rd-vm/tests/enum_set.rs` (13).
- **`java.util.ResourceBundle` (class-based bundles, `crates/rd-vm/src/resbundle.rs`):** `getBundle(baseName[, locale[, loader]])` resolves `base_lang_COUNTRY` → `base_lang` → `base` (the JDK candidate order) and instantiates the guest class; `getObject`/`handleGetObject`/`getString`/`getStringArray`/`containsKey`/`getLocale` read the pairs from the guest `getContents()` (cached per bundle). This is how `PrettyTime` localizes its units in real APKs — without it NewPipe's `Application.onCreate` died right after the okhttp client was built. `.properties` bundles and `getKeys` enumeration are **not** modelled and say so. `ListResourceBundle`/`ResourceBundle` are in the builtin hierarchy (guest bundles are DEX subclasses). Test: `crates/rd-vm/tests/resource_bundle.rs`.
- **`java.util.Collections` completion:** `reverseOrder()` (singleton, JDK identity) and `reverseOrder(Comparator)`, `nCopies`, `frequency`, `fill`, `swap`, `rotate`, `copy`, `disjoint`, `replaceAll` — with the JDK's exception classes at the boundaries. Test: `crates/rd-vm/tests/collections_completion.rs`.
- **ARSC array resources:** `Arsc::resolve_array` resolves `string-array`/`integer-array`/`array` bags (both the aapt2 `0x02000000|index` and the `0x01000000|index` naming found in the golden APKs, parent chain followed), and `TypedArray.getTextArray(int)` materializes the `CharSequence[]`. Without it no preference screen with a list could inflate.
- **`Collections.binarySearch(List[, Comparator])`** over host and guest lists, with the JDK's `-(insertion point) - 1` encoding for a missing key. Test: `crates/rd-vm/tests/collections_binary_search.rs`.
- **`javax.net.SocketFactory` / `SSLSocketFactory` (`crates/rd-vm/src/jnet.rs`):** `getDefault()` returns a real object of the right class so HTTP clients can finish their construction (okhttp builds its client in `Application.onCreate`); the first `createSocket` answers the structured `NOT_IMPLEMENTED` network gap, never a fake socket. Test: `crates/rd-vm/tests/jnet_socket_factory.rs`.
- **M4 renderer (ADR 019 accepted):** `rd app shot` paints with `tiny-skia` (anti-aliased paths) and shapes text with `rustybuzz` over the bundled DejaVu Sans (`crates/rd-render/assets/`). Golden lock `golden/screens/trivial-app.png` (`tests/golden_screenshot.rs`, regression only, not Android fidelity).
- **XAPK bundles (ADR 022):** inspect, base split, OBB listing, and dex disasm across splits.
- **XAPK install (ADR 022):** `install` accepts `.xapk`, validates every split, installs the base, and returns `bundle.splits` in the response. DEX of other splits is merged into execution (`bundle.split_dex_merged`). Native libs of other splits are not loaded yet (`native_libs_pending`; `base_only_execution` stays true while they are pending).
- **XAPK bundles (`rd-apk::xapk`):** a `.xapk` (ZIP with `manifest.json`, no root
  `AndroidManifest.xml`) opens as its `base` split. `rd inspect` (text and `--json`) lists
  every split, its dex/lib counts and ABIs, and the OBB expansions. Missing split files and
  a bundle without a base split are reported as structured errors. Dex disassembly and
  `rd dex disasm` covers the DEX of every split. `rd test` and installation still cover the base split only.
- **M1 differential gate (`scripts/dex_baksmali_diff.py`):** versioned comparison of
  `rd dex disasm` against baksmali 2.5.2 (jar pinned by SHA-256). Runs per method on every
  golden APK in `nightly-compat.yml`; the gate fails on any opcode-sequence divergence.
  Golden corpus: F-Droid client, NewPipe, Tasks (296,907 methods, all identical).

### Fixed

- **Guard de pilha nativa agora é relativo à pilha real (#637):** o piso de
  `detect_native_stack_budget` era `256 KiB` fixos, então numa thread menor que
  ~512 KiB o orçamento calculado era maior que a própria pilha — o guard em bytes
  nunca disparava antes do guard page do SO e o processo morria com `SIGABRT`
  (`has overflowed its stack`), que é exatamente o que derrubava o
  `stack_overflow_313` em CI. O piso passou a ser `min(64 KiB, VMA/8)` e
  `enter_stack_frame` deriva também um **teto de profundidade** do orçamento
  (`~4 KiB` por frame) para que o erro seja lançado muito antes de a pilha acabar,
  independentemente do tamanho de frame. Teste de regressão novo cobre 256/320/512 KiB
  (antes: SIGABRT; agora: `Ljava/lang/StackOverflowError;`).
- **Link morto `archive/README.html` no site (M12):** o mdBook renderiza
  `docs/archive/README.md` como `archive/index.html`, então o link de `docs/index.md`
  para `./archive/README.md` — correto na visão do repositório — virava
  `archive/README.html` no HTML e respondia **404** no site publicado. `scripts/book_links.py`
  agora reescreve `X/README.html` → `X/index.html` quando é esse o arquivo que existe, e
  **falha o build** quando não existe nenhum dos dois (link morto não passa em silêncio).
  Achado no crawl das 97 páginas do site; 7 casos novos no `book_links_test.sh`.
- **VM, `android.view.ScaleGestureDetector` (`crates/rd-vm/src/view_scale_gesture.rs`):** the class was not modelled, so `new ScaleGestureDetector(Context, OnScaleGestureListener)` answered `NOT_FOUND` inside `PreviewView.<init>` (CameraX). The error aborted the `Recomposer` frame before `applyChanges` and left the Comprix Scanner tab without its preview. The constructor and `onTouchEvent` (consumed, as AOSP) are modelled; `isInProgress()` is `false` (the virtual input is single-pointer). Quick scale, stylus scaling and the scale-factor getters stay typed `NOT_FOUND`, documented as limits. Tests: `crates/rd-vm/tests/android_scale_gesture.rs` (the constructor test fails without the hook, with the same `NOT_FOUND` as the app).
- **VM, `Resources.NotFoundException` hierarchy (`crates/rd-vm/src/classpath.rs`):** the exception `android_res` throws for an unknown resource id had no superclass entry, so `checkcast Throwable` raised `ClassCastException` and the coroutine machinery aborted the process (`FATAL EXCEPTION: main`, Comprix Scanner). Now `RuntimeException`, as AOSP. Test: `crates/rd-vm/tests/android_res_not_found.rs` (fails without the entry with the same message as the app).
- **VM, framework colors (`android.R.color`):** `black` (`0x0106000c`), `white` (`0x0106000b`) and `darker_gray` (`0x01060000`) resolve to the documented ARGB values. Before, `PreviewView` (CameraX) failed on `Resource ID #0x106000c` in the Scanner tab. Other framework ids stay `NotFoundException`.
- **VM, `PackageManager` `metaData`:** `ComponentInfo.metaData` and `ApplicationInfo.metaData` are read from the manifest `<meta-data>` (AOSP: `null` when none, a `Bundle` otherwise). CameraX finds its default `CameraXConfig.Provider` through the `MetadataHolderService` meta-data, which the VM returned as `null`.
- **VM, `android.os.HandlerThread`:** constructor, `start()` (creates a Looper with its own queue), `getLooper()`, `quit()`, `quitSafely()` and `getThreadId()`. Messages posted through `new Handler(looper)` run in the same virtual-time order as the main Looper. The thread body is the Looper drain, not a `Looper.loop()` green thread (one deterministic consumer; documented in `handler_thread.rs`).
- **VM, `java.util.concurrent.ThreadPoolExecutor`:** the four JDK constructors and the configuration setters build the same inline `ExecutorService` as the `Executors` factories (single-threaded VM). Pool parameters are accepted and not applied; the getters are not modelled and stay a typed `NOT_FOUND`.
- **VM, `Context.getDrawable(int)` (API 21+):** was `NOT_FOUND` on the `Context` family (the model served strings/colors/arrays through `Resources`, never drawables). A color resource now materializes a `ColorDrawable` (same host state `android_drawable.rs` uses); a *file* drawable (vector `res/*.xml`, PNG) raises a structured `NOT_IMPLEMENTED` naming the resolved path — the Android original never returns null there, so a silent null would be a lie. Real effect: NewPipe's `MainActivity` now dies at `res/hq.xml` with a precise message instead of an unresolved-method error.
- **VM, `android.app.NotificationChannel` null arguments:** the DEX null literal (`const/4 v0, 0`) reaches a model as `Int(0)`, and the m9 model only accepted `Value::Null`, so `setGroup(null)`/`setName(null)`/`setDescription(null)` answered `NullPointerException: text`. AOSP accepts all three, and an unset name reads back as `null` — `Channel.name`/`description` are `Option<String>` now. This was the NewPipe stop right after the ResourceBundle work (`NotificationChannelCompat$Api26Impl.setGroup`).
- **VM, `sun.misc.Unsafe` operands were read one index too high:** the receiver is not part of the intrinsic argument slice, so `obj`/`offset` are `args[0]`/`args[1]`, and the value (or the CAS `expect`/`update` pair) follows. `putInt`/`putLong`/`putObject` therefore stored a default value and `compareAndSetInt`/`Long`/`Object` compared the wrong word and always answered `false`. The visible damage was a hang, not an error: okhttp's `ConcurrentHashMap.initTable()` retries its `compareAndSetInt(this, SIZECTL, sc, -1)` until the fuel budget dies inside `Application.onCreate` (NewPipe 0.27). Also added `arrayBaseOffset`/`arrayIndexScale`/`addressSize`/`pageSize` and array slots (`offset >= 16` → element `(offset - 16) / 8`), so `tabAt`-style CAS on a table array works. Test: `crates/rd-vm/tests/unsafe_cas.rs`.
- **VM, field hiding (JLS 8.3):** instance fields were keyed by name only, so a subclass field with the same name as an inherited one shared a single slot. Real APK (NewPipe 0.27): `SwitchPreference.mListener` and `Preference.mListener` collapsed, and `Preference.notifyChanged()` dereferenced the wrong object inside `Application.onCreate`. Ambiguous names now use a `declClass#name` slot; unambiguous fields are unchanged. Test: `crates/rd-vm/tests/field_hiding.rs`.
- **VM, `android.view.InflateException`** is in the builtin hierarchy, so the framework's own throwable constructors materialize instead of failing with `NOT_FOUND`.
- **CLI, uncaught exceptions:** `rd app run` now prints the `Caused by:` chain of the escaped throwable (guest frames included when `--trace` is on), which is what made the NewPipe failures diagnosable.
- `SharedPreferencesImpl` is now a subtype of `SharedPreferences` (checkcast on the F-Droid cold start).
- **VM, okhttp shim (F-Droid cold start):** the `<init>` shim for `Lokhttp3/*` applies only to classes that are modeled (`is_modeled_okhttp`), so `ConnectionSpec$Builder` keeps its real fields instead of returning null.
- **VM, reflection:** `Class.getEnclosingClass` and `Class.getDeclaringClass` answer named member classes (`Outer$Inner` → `Outer`) from the class name. `getEnclosingMethod` and `getEnclosingConstructor` return null: the EnclosingMethod attribute is not modeled yet.
- **VM, collections:** `LinkedHashMap.<init>(IFZ)` runs when the receiver is a DEX subclass, not only the host map class.
- **VM, dispatch:** `invoke-super` whose target is an interface default method starts the lookup in that interface (ktor `HttpClientEngineBase.install`). Regression test: `crates/rd-vm/tests/invoke_super_interface.rs`.
- **Android host:** static `Application.getProcessName()` returns the package name. `PackageManager` component and application enabled-state setters are accepted as no-ops, and their getters return `COMPONENT_ENABLED_STATE_DEFAULT`.
- **`rd-dex` 32x decoding:** `move/16`, `move-wide/16` and `move-object/16` were decoded as
  2 code units instead of 3, misaligning the rest of the method. The interpreter and JIT
  had the same size bug. Added a regression test.
- **`rd-dex` field names:** `iput-short` (0x5f) printed as `index@N` instead of the field
  reference, because the iget/iput range stopped at 0x5e.

- **M10 Virtual Devices (`#201`, [ADR 011](./docs/adr/011-m10-virtual-devices.md), [`docs/m10.md`](./docs/m10.md)):**
  Programmable deterministic virtual camera (synthetic RGBA frames + Camera2/CameraX shims),
  GPS/Location (NMEA + GPX/waypoints), sensors (accelerometer, gyroscope, light, proximity,
  rotation vector, step counter), biometrics/fingerprint (`BiometricPrompt` + `FingerprintManager`),
  media/audio/MediaSession (`AudioManager`, `AudioRecord` PCM waveforms, `MediaPlayer` virtual clock),
  and network/connectivity/system configuration (`ConnectivityManager`, `WifiManager`, `BluetoothAdapter`,
  `BatteryManager`, locale, night mode, font scale, orientation) controllable via MCP `virtual_device`.
- **Appendices D & E Executable Registries (`#197`, `#198`, [ADR 012](./docs/adr/012-appendix-d-e-registries.md)):**
  Versioned TOML catalogs (`data/jetpack.toml` — 46 AndroidX/Jetpack libraries; `data/gms.toml` —
  21 Google Play Services / Firebase services in honest "no-GMS" mode), `cargo xtask` report/spec
  generators ([`docs/jetpack.md`](./docs/jetpack.md), [`docs/gms.md`](./docs/gms.md)), CLI `rd jetpack` /
  `rd gms` subcommands, and initial AndroidX runtime shims (`crates/rd-vm/src/androidx.rs`).
- **M11 Engine Execution Strategy (`#199`, [ADR 013](./docs/adr/013-m11-engine-execution-strategy.md)):**
  Accepted execution roadmap ordering (`WASM → React Native → Mono → Flutter`), MSRV 1.76 constraints,
  documented framework bridge contract, and golden-flow evidence gates.
- **LazyArsc Architecture (`#202`, [ADR 014](./docs/adr/014-lazyarsc-lazystringpool.md)):**
  Design for on-demand `resources.arsc` and `StringPool` decoding within Moto E5 budgets.
- **Appendix C Deep AOSP Audit Series (`#192`, `#196`, `#200`, `#204`):**
  Modeled-subset AOSP conformance verification, edge-case fixes, and regression suites across 43+
  `android.*` surfaces ([`docs/api-coverage.md`](./docs/api-coverage.md),
  [`docs/appendix-c-literal-completion.md`](./docs/appendix-c-literal-completion.md)).
- **Runtime & Engine Detection Fixes (`#205`, `#207`):**
  `Context.getAssets()` / `AssetManager` reading real APK `assets/` entries, and `rd-engine-registry`
  detection rules for real Tauri (`libtauri_android.so` / `libwry.so`), Defold (`libdmengine.so`),
  and TWA (`<application>` `<meta-data>` `asset_statements`) builds.
- **M12 Community, Governance & Docs Site Foundation (`GH-02`, `GH-03`, `GH-04`):**
  [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md), [`GOVERNANCE.md`](./GOVERNANCE.md),
  [`SUPPORT.md`](./SUPPORT.md), [`README.en.md`](./README.en.md), `CHANGELOG.md`,
  [`.github/ISSUE_TEMPLATE/perf-regression.yml`](./.github/ISSUE_TEMPLATE/perf-regression.yml),
  [`.github/FUNDING.yml`](./.github/FUNDING.yml), and `mdBook` configuration (`docs/book.toml`,
  `docs/SUMMARY.md`, `docs/index.md`).

## [0.9.0] - 2026-10-21

### Added

- **M9 Advanced Components & Isolated NDK Execution ([ADR 004](./docs/adr/004-m9-native-isolation.md), [`docs/m9.md`](./docs/m9.md)):**
  - `SYS-09` Notifications (`NotificationChannel`, `Builder`, `POST_NOTIFICATIONS` API 33+, `PendingIntent` mutability, `NotificationListenerService`), IME (`InputConnection`, `InputMethodService`), and AppWidgets (`AppWidgetProvider`, `RemoteViews`).
  - `SYS-10` Accessibility (`AccessibilityService`, node trees, actions) and Shortcuts (`ShortcutManager`).
  - `LANG-02` Bounded ELF32/64 loader (`x86_64`, `AArch64`, `ARM`) + JNI table (`JNIEnv`/`JavaVM`, `RegisterNatives`, `JNI_OnLoad`) executing guest `.so` libraries inside an isolated Unicorn subprocess worker (`crates/rd-ndk`), verified across 5 real F-Droid APKs × 3 ABIs (`15/15` checks) plus source-built C/C++/Rust demonstration APKs ([`docs/native-languages.md`](./docs/native-languages.md)).
- **M8 Real-World Compatibility Host ([ADR 003](./docs/adr/003-m8-compat-host.md)):**
  Selective framework interception (`crates/rd-vm/src/m8.rs`) for `RecyclerView`, legacy + AndroidX `FragmentManager`, `AlertDialog`, deterministic offline `OkHttpClient`, in-memory + isolated subprocess `SQLite`/`Room` ([ADR 005](./docs/adr/005-sqlite-subprocess.md)), `Services`/`BroadcastReceiver`/deep links/`JobScheduler`, `WorkManager`, and stateful `SharedPreferences`.
- **Generational GC & Live-Frame Roots ([ADR 006](./docs/adr/006-generational-gc-and-live-frame-roots.md), [`docs/gc.md`](./docs/gc.md)):**
  Non-moving generational mark/sweep collector with remembered-set write barriers, weak `WeakHashMap` keys, and mid-call collection within the 256 MB Moto E5 heap cap.
- **Experimental Cranelift JIT ([ADR 007](./docs/adr/007-cranelift-leaf-jit.md)):**
  Opt-in `--features jit` (`RD_JIT=on`) backend in `rd-jni`/`rd-vm` with fuel parity and interpreter fallback.
- **Large-APK & Cold-Start Architecture ([ADR 008](./docs/adr/008-15gb-apk-and-1500ms-cold-start.md), [ADR 009](./docs/adr/009-process-cold-26ms.md)):**
  Bounded `pread64` `PagedZip` LRU cache, `< 26 ms` snapshot-backed warm/live startup, and `rd perf` phase telemetry against the Motorola Moto E5 floor (`1,500 ms` emulated cold-start budget).
- **RD Sentinel Anti-Regression Bot ([ADR 010](./docs/adr/010-rd-sentinel-anti-regression-bot.md), [`docs/archive/rd-sentinel.md`](./docs/archive/rd-sentinel.md)):**
  Automated diff-risk scoring, static safety invariants, protected-path enforcement, fuzz/mutation/perf campaigns, and SARIF reporting.
- **Expanded MCP Control Plane ([`docs/mcp-observe.md`](./docs/mcp-observe.md), [`docs/mcp-automation.md`](./docs/mcp-automation.md)):**
  Screen observation without screenshots (`observe`, `tap_text`, subscribable `rustydroid://screen` resources, `--live` browser viewer), guarded selectors, incremental logs, and typed diagnostics across 47 tools.
- **Appendix G Engine Registry ([ADR 003](./docs/adr/003-appendix-g-engine-registry.md), [`docs/engine-registry.md`](./docs/engine-registry.md)):**
  28 stable language/engine records (`data/engines.toml`), `rd engine list|detect|check`, and honest `NOT_IMPLEMENTED` execution boundaries.

## [0.6.0] - 2026-10-12

### Added

- **M6 Test Executor ([`docs/suite-format.md`](./docs/suite-format.md)):** `rd test suite.json` scenario runner with JUnit XML (`--junit`), JSON reports, isolated agent sessions, and `cargo xtask compat-report` scoreboard generation ([`docs/compat-matrix.md`](./docs/compat-matrix.md)).
- **M5 MCP Agent Server v1 ([ADR 001](./docs/adr/001-mcp-sem-rmcp-msrv.md), [`docs/mcp-tools.md`](./docs/mcp-tools.md)):** Zero-dependency JSON-RPC 2.0 `rd mcp` server over stdio with package management, input injection, UI tree/screenshot inspection, permissions, batch execution, and `--describe-tools`.
- **M4 Software Renderer & UI Dump:** Deterministic `tiny-skia` software rendering (`crates/rd-render`), PNG encoding, and `uiautomator`-compatible accessibility/UI hierarchy dumps.
- **M3 Essential Android Framework:** `Activity` lifecycle (`onCreate`/`onStart`/`onResume`), `Looper`/`Handler`, binary XML `LayoutInflater`, and `Resources` resolution.
- **M2 Dalvik Bytecode VM:** Pure-Rust Dalvik interpreter (`crates/rd-vm`) passing `150/150` JVM golden vectors, exception tables, virtual dispatch, fuel budgets, and 256 MB arena `VM_OOM` enforcement.
- **M1 Complete DEX Parser & Disassembler:** Fuzz-hardened `rd-dex` parser + `smali` disassembler validated against `baksmali 2.5.2` across 24,913 classes, plus `cargo-fuzz` targets ([`docs/FUZZING.md`](./docs/FUZZING.md)).
- **M0 Workspace, APK Inspector & Permission Engine:** Cargo workspace, `rd-apk` (ZIP, AXML, ARSC, v1/v2/v3 signing, duplicate-entry handling per [ADR 002](./docs/adr/002-zip-duplicate-entries.md)), `data/permissions.toml` (175 Android permissions), `data/devices.toml` (`moto-e5` floor), `data/behavior-switches.toml` (API 26→36), and GitHub Actions CI.
