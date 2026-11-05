# Fixture: RxJava 3 reduzido (DEX de biblioteca real)

Exceção documentada à política de não commitar binários de teste:
ver [`docs/adr/036-excecao-fixture-dex-rxjava.md`](../../../../../docs/adr/036-excecao-fixture-dex-rxjava.md).

| Arquivo | Conteúdo | SHA-256 |
|---|---|---|
| `classes.dex` | DEX gerado por R8 8.7.18 (`--release --min-api 26`) | `4023c2291dd57bf6d7718da62f0b837f3fa7b45555c3ee04550c4f9f284c3fac` |
| `keep.pro` | Regras keep (o que é mantido como raiz) | — |

Entradas (Maven Central / Google Maven):

| Artefato | SHA-256 do jar |
|---|---|
| `io.reactivex.rxjava3:rxjava:3.1.10` (Apache-2.0) | `e9f256f9e805572dd5fd45b140db360d7dc444d0c383049b2d3d7ebf05d84aab` |
| `org.reactivestreams:reactive-streams:1.0.4` (MIT-0, conforme POM) | `f75ca597789b3dac58f61857b9ac2e1034a68fa672db35055a8fb4509e325f28` |
| `com.android.tools:r8:8.7.18` (Google Maven) | `58366f77067207c39a17d469de7b05701d2877212a9c55201bcb0af43e59e903` |

Regenerar e validar: `scripts/gen-rxjava3-fixture.sh` (falha se algum hash diferir).

O que o DEX contém: só as classes alcançáveis a partir de `Observable.just/never/empty`,
`subscribe()` e `Disposable` (ex.: `ObservableJust`, `LambdaObserver`, `DisposableHelper`,
`Functions`). Não é a biblioteca inteira. Os testes estão em `crates/rd-vm/tests/jetpack_rxjava.rs`.

Avisos do R8 (`Missing class java.util.concurrent.*`): o R8 roda sem `android.jar`,
então as classes da plataforma Java ficam como referência externa. Em runtime, a VM
resolve essas classes (intrínsecos de `java.util.concurrent.atomic`). É esse caminho
que o teste cobre, e foi ele que expôs a falta de `lazySet`, fechada no mesmo PR.
