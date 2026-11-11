# Desvios do modelo de execução (Zoom nativo)

Itens que a VM aceita com comportamento aproximado. Cada um é observável no código pelo comentário citado. Documento escrito à mão (o `compat-matrix.md` é gerado por `cargo xtask compat-report`).

| Desvio | Comportamento | Onde |
|---|---|---|
| `ReferenceQueue.remove()` sem timeout num worker de executor vazio | Parkeia o worker (bloqueio para sempre). O GC não acorda o worker depois. Fora de worker, devolve `null` (como antes). | `intrinsics.rs` (`exec_as_worker`, braço `ReferenceQueue`) |
| `Object.clone()` | Cópia rasa de array ou de instância `Cloneable`. Estado de host (`fw.objects`) não é copiado. | `intrinsics.rs` (`object_shallow_clone`) |
| `StrictMode.{Thread,Vm}Policy.Builder` | Métodos fluentes não mudam nada (o device virtual não impõe StrictMode). `build()` devolve uma policy nova. | `intrinsics.rs` (`STRICT_POLICY_BUILDERS`) |
| Drawable de arquivo (`res/…`) | Vira `VectorDrawable` se a raiz for `<vector>` (AXML ou texto), senão `Drawable` base. Nenhum dos dois pinta. Rasterizar vetores exige um renderizador próprio. | `android_res.rs` (`getDrawable`) |
| `Icon.createWithResource` | Handle opaco sem pixels. | `intrinsics.rs` (`static_intrinsic`) |
| `ShortcutInfo.Builder` / `ShortcutManager` | Builder fluente; `setDynamicShortcuts` devolve `true`. Sem launcher, nada aparece no device virtual. | `intrinsics.rs` |
| `MessageQueue.addIdleHandler` | Handlers rodam quando o Looper principal fica ocioso (uma vez por ociosidade). | `framework.rs` (`run_main_idle_handlers`) |
| Limite de regex (`delegate_size_limit`) | 2 MiB → 8 MiB (`REGEX_NFA_LIMIT`). Continua fixo contra padrões hostis. | `jregex.rs`, `jlstring.rs` |
| `WebView.setDataDirectorySuffix` | No-op (sem processo WebView separado). | `android_stub_surfaces.rs` |
| `BluetoothGattCallback()` | Construtor sem estado; sem BLE os callbacks não disparam. | `intrinsics.rs` |
| `PriorityBlockingQueue` | Mesma ordenação de `PriorityQueue`. `put`/`take` bloqueantes não são modelados. | `jpriorityqueue.rs` |
| `ConcurrentLinkedQueue` | FIFO não bloqueante, sem `null` (mesmo modelo host do `LinkedBlockingQueue`). | `intrinsics.rs` |
| `Process.setThreadPriority` | No-op; `getThreadPriority` devolve 0. | `m8.rs` |
| `Looper.getQueue` / `MessageQueue` | Singleton por VM da fila principal. | `framework.rs` |
| `Activity.isFinishing` | Lê o flag `finished` já mantido pelo `finish()`. | `framework.rs` |
| `Window.getContext` | Contexto de criação da janela (a Activity). | `framework.rs`, `window.rs` |
| `Class` implementa `java.lang.reflect.Type` | Hierarquia host (necessário ao `TypeToken` do Gson). | `classpath.rs` |
| `Class.getModifiers()` de classe membro | Usa `accessFlags` de `dalvik.annotation.InnerClass` (inclui `static`). | `jannotations.rs`, `jlclass.rs` |
| `Objects.toString` | Igual à JDK (nulo → default ou `"null"`). | `intrinsics.rs` |
| `Object.equals/hashCode/toString` via interface | Quando o receptor não sobrescreve, usa a identidade de `Object` (JLS 9.2). | `interp.rs` |
| `Runtime.exec` / `ProcessBuilder` | Lança `IOException`: o VM não executa comandos do host (apps que testam root com `su`/`sh` tratam a falha). | `intrinsics.rs` (`Runtime`) |
| `Activity.set/getRequestedOrientation` | Registra o valor (padrão -1, `SCREEN_ORIENTATION_UNSPECIFIED`); a tela não gira. | `framework.rs` |
| `Activity.isInMultiWindowMode` / `isInPictureInPictureMode` | Sempre `false` (janela única, sem PiP). | `framework.rs` |
| `Color.colorToHSV` / `RGBToHSV` / `HSVToColor` | Float como no AOSP. Acromáticos (preto, cinza) saem com matiz 0; o AOSP divide 0/0 nesse caso. A confirmar contra o AOSP. | `graphics.rs` |
| `Locale.getDefault(Category)` / `setDefault(Category, …)` | Um único locale padrão para todas as categorias. | `android_res.rs` |
| `CompoundButton` (checked, tint list/mode, button drawable) | Guardados em `view_props` pelo fallback genérico, sem render. `getButtonTintList`/`getButtonTintMode` começam `null`. | `view_props.rs` |
| `TextView.getCompoundDrawables(Relative)` | Devolve 4 posições nulas. Drawables de texto definidos pelo app não são guardados. | `view_props.rs` |
| `Resources.getColorStateList` | Cor simples vira `ColorStateList.valueOf`. Seletor XML (`res/color/*.xml`) falha com `NOT_IMPLEMENTED` e o caminho. | `android_res.rs` (`color_state_list_of`) |
| `Field.getGenericType` / `Class.getGenericSuperclass` | Sem `Signature`, devolvem a classe crua. Com tipo parametrizado, falham com `NOT_IMPLEMENTED` (sem `ParameterizedType`). | `intrinsics.rs`, `jlclass.rs` |
| `finish()` durante o `onCreate` do lançamento | A destruição é adiada até depois do pós-`onCreate`; a Activity não recebe `onStart`/`onResume`, como no `ActivityThread`. | `framework.rs` (`launch_activity_inner`) |
| Intent de lançamento | Herda action, data, mime, categories e flags do intent de origem; componente, pacote e seletor não são herdados. | `framework.rs` (`start_activity_with`) |
