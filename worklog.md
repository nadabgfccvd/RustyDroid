# Worklog — RustyDroid

---
Task ID: 2
Agent: Explore
Task: Mapear APIs AXML/ARSC do rd-apk e superfícies do rd-vm para o LayoutInflater

Work Log:
- Li o repositório (worklog.md não existia — este é o primeiro registro) e mapeei crates/rd-apk por completo: axml.rs, arsc.rs, pool.rs, zip.rs, manifest.rs, lib.rs, error.rs, examples/ (dump_arsc.rs, dump_manifest_attrs.rs) e tests/integration_apk.rs.
- Mapeei crates/rd-vm: engine.rs (Engine/HostState não guardam APK), framework.rs inteiro (HostObj, dispatch call_host_instance/call_host_static, setContentView, findViewById, layout, attach_host_state, launch_app), interp.rs (pontos de despacho host: linhas ~1319 e ~1392; new-instance → attach_host_state linha ~363), value.rs, heap.rs (ObjRef/as_str/alloc_string/alloc_instance), err.rs, intrinsics.rs (alloc_string), classpath.rs (is_subtype/resolve_method).
- Verifiquei rd-cli/src/main.rs: cmd_app_run abre o APK fora da Engine (load_dex_files + launcher_of_apk) e só passa Vec<Dex> + package para launch_app — o APK/ARSC nunca chega ao Engine hoje.
- Chequei dependências: rd-vm/Cargo.toml tem rd-apk apenas como dev-dependency (dependência de produção rd-apk → rd-vm NÃO existe; workspace define "dependências só apontam para baixo": rd-apk → rd-dex → rd-vm → rd-framework).
- Inspecionei golden-apks/org.fdroid.fdroid.apk: tem resources.arsc (2 MB) mas ZERO entradas res/layout (nomes de recurso ofuscados) — testes do LayoutInflater precisarão de APK sintético com res/layout/*.xml + resources.arsc, ou AXML handcrafted (padrão dos testes unitários de axml.rs/arsc.rs).
- Li docs/ROADMAP.md (M3 round 2: LayoutInflater confirmado como próximo passo) e tests/app.rs (padrão de teste M3 com DexBuilder sintético).

Stage Summary:
- AXML: `rd_apk::axml::parse(&[u8]) -> RdResult<AxmlDocument>` (axml.rs:259). Documento = `AxmlDocument { root: XmlElement, resource_ids: Vec<u32>, strings: StringPool, namespaces: Vec<(String,String)> }`. `XmlElement { name, attrs, children, text }` com helpers `android_attr()/attr()/children_named()`. `XmlAttribute { ns: Option<String>, name, raw: Option<String>, value: AttrValue, res_id: Option<u32> }` — res_id vem do resource map indexado pelo índice do NOME do atributo no string pool (axml.rs:422).
- Enum de valores de atributo (exato, axml.rs:115): `AttrValue { Null, String(String), Reference(u32), Attribute(u32), Float(f32), Dimension(f32, ComplexUnit), Fraction(f32, ComplexUnit), Int(i32), Bool(bool), Raw { dtype: u8, data: u32 } }` + `ComplexUnit { Px, Dip, Sp, Pt, In, Mm, Percent, PercentParent, Unknown(u8) }`; conversores `as_bool/as_i32/as_u32/as_string/as_text`.
- ARSC: `Arsc::parse(&[u8]) -> RdResult<Arsc>` (arsc.rs:101); `Arsc { global_strings, packages: Vec<Package> }`; `Package { id, name, type_strings, key_strings, entries: BTreeMap<u32 /*resid*/, Vec<ResEntry>> }`; `ResEntry { config: ResConfigInfo, data_type: u8, data: u32, string: Option<String>, complex: bool, key_idx }`. Lookups existentes: `Arsc::resolve_string(res_id) -> Option<String>` (só STRING e cadeia REFERENCE, config "melhor" automática via specificity/is_default — SEM parâmetro de config), `Arsc::resolve_name(res_id) -> Option<String>` → "@type/key", `Arsc::package_for(res_id)`. NÃO EXISTE: resolve por nome (`R.string.foo` → id), resolve tipado (int/bool/dimension — data_type/data ficam crus em ResEntry; complex/bags não são desdobrados), índice nome→id. LayoutInflater precisará criar esses helpers (ou caminhar `entries` + `resolve_name`).
- Acesso a entradas: `Apk { zip: Zip, arsc: Option<Arsc>, ... }` (rd-apk/src/lib.rs:34-36); `Apk::container() -> &Zip`, `Apk::manifest_raw()`; `Zip::read(name)/read_entry/find/names/names_with_prefix("res/layout/")` — caminho pronto: zip.read(entry) → axml::parse.
- rd-vm: Engine (engine.rs:46) NÃO tem campo de APK/ARSC; HostState (framework.rs:104) só tem objects/clock_ms/main_looper/current_activity/package_name. `setContentView(I)V` hoje = NOT_IMPLEMENTED tipado (framework.rs:678-685); `setContentView(View)` (669) usa helper `set_content_view` (1092) → ensure_window + Window.content + `vm.layout(content, 0, 0, WINDOW_W)` (WINDOW_W=720/WINDOW_H=1440/ROW_H=48, linhas 46-49). `setText(int resId)` também é stub (857-865).
- Views host: `HostObj::View { id, x, y, w, h, visible, enabled, click_listener, parent, text, children, orientation }` (framework.rs:67-80); criação = `vm.heap.alloc_instance(desc, vec![])` + `vm.attach_host_state(desc, r)` (framework.rs:147, chamado do new-instance interp.rs:363); tipo real da view vem do DEX do app via is_subtype (constantes ACTIVITY/VIEW/TEXTVIEW/BUTTON/LINEARLAYOUT/FRAMELAYOUT em framework.rs:30-43). `findViewById` → `find_view_by_id` recursivo (framework.rs:558; dispatch 686; id é i32 via `args[0].as_int()`).
- Dispatch host: interp.rs chama `framework::call_host_instance(vm, mref_class, name, sig, recv, args) -> Result<Option<Value>, VmExit>` quando `mref.class.starts_with("Landroid/")` (interp.rs:1392-1406; static em 1319-1335 → framework.rs:603). Args chegam como `&[Value]` (Int/Long/Float/Double/Obj/Null); strings lidas com `vm.heap.as_str(r)` e devolvidas com `intrinsics::alloc_string(vm, s)`; None = método host não tratado.
- Manifest resolve @string via ARSC assim (manifest.rs:554-573): `arsc.and_then(|a| a.resolve_string(res).or_else(|| a.resolve_name(res)))` — padrão reutilizável para refs de atributo em layouts.
- Gaps para o M3 round 2 (decisões, não código): (1) rd-vm precisa acessar AXML/ARSC → ou dependência rd-apk em rd-vm + campo novo em HostState/Engine (ex.: arsc + leitor de layouts), ou injeção na API (Engine::new/launch_app/método set_resources); (2) resolver layout resId → entrada zip "res/layout/<name>.xml" requer índice nome→id no ARSC (não existe); (3) attributes android:layout_width/layout_height como Dimension precisam mapear ComplexUnit para px no modelo headless (layout atual usa px inteiros); (4) fixture de teste: APK sintético com res/layout + resources.arsc (golden APK F-Droid não tem layouts).

---
Task ID: 3
Agent: Z.ai Code (principal)
Task: M3 round 2 — LayoutInflater de layout XML real (AXML via rd-apk + Resources/ARSC) + issues/CI check prévio

Work Log:
- Pré-check (instrução permanente "Issues antes de M3"): API GitHub autenticada → 0 issues abertas (as 38 já fechadas); CI verde no HEAD fb37e28 (run 38026991552). Liberado para M3 round 2.
- Subagente Explore (Task ID 2) mapeou AXML/ARSC/zip do rd-apk e superfícies do rd-vm (relatório na seção anterior).
- rd-vm/Cargo.toml: rd-apk promovido a dependência de produção (rd-apk é a BASE do workspace; rd-vm→rd-apk segue apontando para baixo, sem ciclo).
- classpath.rs: builtin_hierarchy estendida com a hierarquia Android (Activity/Context/View/ViewGroup/TextView/Button/LinearLayout/FrameLayout/Intent/Bundle/Handler/Looper/Window) — APKs REAIS não definem android.* no DEX; is_subtype agora resolve sem DEX (habilita inflação em APK real).
- framework.rs: HostState.resources (Option<rd_apk::Apk>); Engine::set_resources + Engine::resources() (erro RESOURCES_MISSING tipado) + Engine::resolve_string (arsc, fallback nome qualificado); HostObj::View ganhou click_method (android:onClick); dispatch_touch refatorado em dispatch_click (listener OU click_method na ACTIVITY corrente); performClick reusa dispatch_click; dump_view mostra @id/nome via arsc.resolve_name; set_content_view/add_child/ensure_window viraram pub(crate); DENSITY 2.0 (720px = viewport 360dp).
- inflate.rs (NOVO): LayoutInflater headless — resid → @layout/key (Arsc::resolve_name) → res/layout/key.xml (ZIP, fallback configs) → axml::parse → árvore host recursiva; attrs id/text/orientation/visibility/enabled/onClick/layout_height; tags sem host falham INFLATE tipado; attrs desconhecidos ignorados com RD_FW_DEBUG.
- interp dispatch: setContentView(I)V substituiu o stub NOT_IMPLEMENTED; setText(I) resolve no arsc; novo braço getString(I).
- BUG LATENTE corrigido: braço findViewById exigia Value::Obj em args[0] (o resid é Int!) e devolvia Null SEMPRE — sem cobertura até agora; corrigido para ler o resid Int + guard de formato.
- rd-cli: cmd_app_run abre o APK UMA vez (dexes_of_apk + launcher_of(&Apk) + eng.set_resources(apk)) — antes relia o zip 3×; package agora vem do manifest (getPackageName correto com --activity explícito).
- tests/common/apkfix.rs (NOVO): fixtures byte a byte — CRC-32, ZIP STORED (local+central+EOCD), string pool UTF-16, builder AXML (elements + attrs tipados: String/Ref/Int/Bool/Dim), builder ARSC (pacote 0x7f + N tipos + config default), build_manifest_axml e build_apk.
- tests/app_xml.rs (NOVO): 6 testes de integração M3.2 — DoD (XML infla + toque via android:onClick muda texto), RESOURCES_MISSING sem resources, findViewById+setText(I)+getString(I) via arsc, tag não suportada → INFLATE tipado, visibility/enabled no hit-test, shape da árvore host.
- Smoke end-to-end manual (temporário): fixture APK com classes.dex gerado em /tmp, `rd app run` real executou lifecycle → setContentView(R.layout.main) → inflação → tap(10,170) → onBtn → dump com @id/tv/@id/btn e texto trocado; JSON com package do manifest. Teste temporário removido após validação.
- Docs: ROADMAP.md (M3 = concluído rounds 1+2 com checklist do round 2), compat-matrix.md + gerador xtask sincronizados (round-trip validado, diff vazio).

Stage Summary:
- M3 round 2 COMPLETO: setContentView(I) infla layout XML REAL do APK (AXML+ARSC), app trivial XML roda ponta a ponta com toque via android:onClick — DoD do M3 fechado nos dois rounds.
- 187 testes no workspace (0 falhas; +6 M3.2), clippy limpo, fmt ok, MSRV 1.76 ok, smoke CLI verificado.
- Decisões: rd-vm→rd-apk como dep de produção (sem ciclo); hierarquia Android builtin no classpath (chave p/ APKs reais); DENSITY 2.0 (viewport 360dp); attrs não modelados ignorados (semântica Android) com log debug; classes sem host falham tipadas.
- Pendências M4: render pixel + UI dump rico (get_ui_tree), configs específicas (land/locale/density) selecionáveis, weights/gravidade no layout.
