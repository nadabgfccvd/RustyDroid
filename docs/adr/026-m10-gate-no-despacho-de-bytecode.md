# ADR 026 — `virtual_hardware_enabled` vale no despacho de bytecode (fecha a #229)

- **Status:** aceito · **Data:** 2026-11-03
- **Escopo:** `crates/rd-vm/src/interp.rs` (`resolve_or_run_host_invoke`,
  `call_host_shim_layer`, `inherited_android_host`), `crates/rd-vm/src/m10/mod.rs`
- **Relacionados:** ADR 011 (M10 virtual devices), Apêndice C (ausência de hardware
  devolve erro estruturado), issue #229 (ponto 3), PRs #251 (tentativa anterior,
  conflitante com a `main` atual)
- **Ordem:** decisão de dono registrada aqui antes do código (CONTRIBUTING).

## Contexto

A issue #229 deixou um único ponto aberto depois de #461 e #7c19d4d:

- `intrinsics.rs` consulta `vm.fw.vdev.virtual_hardware_enabled` antes de chamar o M10.
- `interp.rs` chamava `m10::call_static` / `m10::call_instance` **incondicionalmente**
  (dispatch de bytecode real, caminho de `do_invoke`).
- Consequência: uma sessão que nunca ligou os dispositivos virtuais (`rd app run`,
  motores de teste) respondia como se houvesse câmera, sensores e GPS. Isso viola o
  perfil padrão do Apêndice C ("nenhuma API ausente quebra o app em silêncio": o
  correto é o erro tipado de hardware ausente, não um dispositivo inventado).

Tentativas anteriores de gate quebraram `camera_305` e `dispatch_315` porque
aplicavam o gate **por módulo**. Os módulos M10 misturam hardware e software:
`camera.rs` também implementa `java.nio.ByteBuffer`, `sysconfig.rs` também
implementa clipboard/locale/UiMode/DisplayMetrics, `media.rs` também implementa
objetos de valor (`MediaMetadata`).

## Decisão

1. O gate vale no **despacho de bytecode** do interpretador (os três sítios de
   `interp.rs`) e é decidido por `m10::dispatch_enabled(vm, class)`:
   `virtual_hardware_enabled || !is_virtual_hardware_class(class)`.
2. A lista de classes é o contrato (`m10::is_virtual_hardware_class`): ela nomeia
   pacotes de **hardware** — `android/hardware/` (sensores, camera2, biometria,
   fingerprint, VirtualDisplay, USB), `android/location/`,
   `android/bluetooth/`, `android/nfc/`, `android/telephony/`, `android/net/`,
   `android/media/Audio*`, `MediaPlayer`, `MediaRecorder`, `MediaExtractor`,
   `Image*`, `projection/`, e `android/os/{BatteryManager,PowerManager,Vibrator}`.
3. Superfícies de **software** hospedadas nos mesmos módulos continuam
   respondendo sem o flag: ByteBuffer, clipboard, locale/UiMode/Configuration,
   DisplayMetrics. Elas não são hardware; tirá-las quebrava testes legítimos
   (`camera_305`, `dispatch_315`).
4. `Engine::m10_disable_virtual_devices()` entra como par simétrico do
   `m10_enable_virtual_devices()` já existente, para o agente/sessão poder voltar
   ao perfil de ausência de hardware sem recriar o motor.
5. `rd mcp` continua ligando os dispositivos virtuais na criação da sessão
   (`crates/rd-agent/src/mcp.rs`, `m10_tools.rs`). `rd app run` **não** liga —
   passa a receber o perfil do Apêndice C, que é o comportamento documentado.

## Consequências

- O caminho de bytecode e o caminho de intrínseco passam a concordar. Um app que
  depende de hardware sem sessão de agente recebe erro tipado em vez de valor
  inventado (Lei de Ouro).
- Nenhum teste existente foi removido: os testes que exercitam hardware virtual
  declaram isso com `m10_enable_virtual_devices()` (ex.: `dispatch_315`).
- Regressão coberta por `crates/rd-vm/tests/m10_gate_229.rs`: o mesmo probe em
  bytecode DEX devolve `Location` com o flag ligado e não devolve com o flag
  desligado; `ByteBuffer` responde nos dois casos; o classificador é testado
  classe a classe.
- Fora de escopo: decidir gates por *método* (ex.: `MediaPlayer` de valor) e
  expor o flag na CLI (`rd app run --virtual-hardware`). Se um app real precisar
  de hardware no modo CLI sem agente, isso é uma fase, não um caso especial.
