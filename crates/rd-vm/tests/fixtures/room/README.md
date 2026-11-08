# Fixture: Room 2.6.1 (banco v1 → v2 com migração)

Fontes Java em `src/com/example/roomfx/`. O DEX **não é versionado** (`.dex` é
bloqueante no RD Sentinel; ver [ADR 036](../../../../../docs/adr/036-excecao-fixture-dex-rxjava.md)
para a única exceção existente, que cobre o fixture do RxJava).

Regenerar (gera `target/room-fixture/classes.dex` e imprime o SHA-256):

```sh
scripts/gen-room-fixture.sh
RD_ROOM_FIXTURE_DEX=target/room-fixture/classes.dex \
  cargo test -p rd-vm --test jetpack_room -- --ignored
```

| Entrada | Versão | Licença |
|---|---|---|
| `androidx.room:room-compiler` (processador, javac) | 2.6.1 | Apache-2.0 |
| `androidx.room:room-runtime`, `room-migration`, `room-common` | 2.6.1 | Apache-2.0 |
| `androidx.sqlite:sqlite-framework` | 2.4.0 | Apache-2.0 |
| `android.jar` (platform-35) | API 35 | SDK do Android |
| `com.android.tools:r8` (D8) | 8.7.18 | Apache-2.0 |

Os jars vêm do Google Maven (`dl.google.com/dl/android/maven2`); o AndroidX não está
no Maven Central. `scripts/maven_resolve.py` resolve as dependências transitivas.

Cenários: `V1.seed` cria `item(id, name)` com uma linha; `V2.withMigration` abre com
`Migration(1,2)` (`ALTER TABLE ... ADD COLUMN qty`); `V2.withoutMigration` abre sem
migração e sem `fallbackToDestructiveMigration`, que deve recusar a abertura.
