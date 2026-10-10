# Harness golden "execução real vs VM" (M2)

Critério de saída do M2 (docs/ROADMAP.md): *"métodos puros de APK real retornam
valores corretos (golden vectors vs execução real)"*. Este pipeline gera e
compara esses vetores: os mesmos métodos puros são executados na **JVM real**
(fonte de verdade) e na **VM Dalvik mínima do RustyDroid** (`rd vm exec` sobre
um DEX compilado do mesmo fonte), e cada resultado é comparado campo a campo.

## Pipeline (4 estágios)

```
cases/*.java ──javac/ecj──► classes 17 ──D8──► build/classes.dex
      │                                        │
      ▼                                        ▼
GoldenRunner (JVM real)                    rd vm exec … --json  (por caso)
      │                                        │
      ▼                                        ▼
build/expected.tsv  ◄──── comparador python3 ────► build/actual.jsonl
                        (PASS/FAIL + contagem; exit 0 só se 100% PASS)
```

1. **`fetch-tools.sh`** — baixa as ferramentas para `cache/` (gitignored).
   - `ecj` (Eclipse Compiler for Java, roda em JRE puro) — **só se `javac` não
     existir no PATH** (CI do GitHub tem JDK; ambiente local costuma ter só JRE).
     Pinnado: **ecj 3.42.0** (Maven Central; fallback 3.41.0/3.40.0 — cada
     versão é validada no download; 404 → tenta vizinha).
   - `r8` (contém o dexer **D8**) — sempre necessário. Pinnado: **r8 8.5.35**
     (Google Maven `dl.google.com/android/maven2/…`; fallback 8.3.37).
     Nota: o r8 NÃO é publicado no repo1.maven.org — o home oficial é o
     Maven/Google Maven; isso foi verificado com `curl -sI` (404 lá, 200 aqui).
   - User-Agent identificado, `--retry 3`, validação de tamanho, mensagens claras.
2. **`build.sh`** — compila `cases/*.java` para bytecode **17** (`javac
   --release 17` ou `java -jar cache/ecj.jar -17 -nowarn -proc:none`) e dexa
   com `D8 --release --min-api 26` (piso API do projeto) → `build/classes.dex`.
3. **`run-golden.sh [caminho-do-rd]`** (default `../../target/release/rd`):
   - Passo 1: `java -cp build/classes br.rustydroid.vm.GoldenRunner` →
     `build/expected.tsv` (via reflexão, boxing manual dos literais conforme a sig).
   - Passo 2: para cada linha, invoca
     `rd vm exec build/classes.dex --class 'Lbr/rustydroid/vm/<Classe>;' --method <m> --sig '<s>' --args=<a> --json`
     e guarda a resposta bruta em `build/actual.jsonl`.
     (Forma `--args=VALOR`: na forma separada `--args -7`, o clap consome o
     `-7` como flag — detalhe do harness, não da VM.)
   - Passo 3: comparação campo a campo (python3 embutido), relatório
     PASS/FAIL por caso + contagem final; `build/report.txt` guarda o detalhe.
     **Exit 0 só se 100% PASS.**

## Contrato do TSV (`build/expected.tsv`)

Uma linha por caso: `classe|metodo|sig|argsRepr|resultadoRepr`

- `argsRepr` — EXATAMENTE os literais Java da tabela (gramática do
  `rd_vm::parse_arg_value`: `42`, `-7`, `1L`, `1.5f`, `2.5`, `'c'`, `"texto"`,
  `true`, `null`); a mesma string é passada ao `--args` no lado da VM, então os
  dois lados recebem valores idênticos. Regra: literais não contêm `,` nem `|`.
- `resultadoRepr`:
  - primitivos: `String.valueOf` estilo Java — int/long/byte/short decimais,
    boolean `true`/`false`, **char cru**, float/double via
    `Float.toString`/`Double.toString`;
  - String: conteúdo literal (a tabela atual não tem casos retornando String —
    ver "Limitações conhecidas");
  - `void`: `<void>`; `null`: `null`;
  - exceção escapada: `EXCEPTION:NomeClasse:mensagem` (getMessage; null → vazio).

### Normalizações da comparação (decisões documentadas)

| Tipo | JVM imprime | VM (JSON) devolve | Comparador |
|---|---|---|---|
| I/J/F/D/B/S | decimal / Java float format | string idêntica (`java_float`/`java_double` no repr.rs) | igualdade direta |
| Z | `true`/`false` | `"1"`/`"0"` (int cru, `type:"int"`) | `1→true`, `0→false` |
| C | caractere cru | código numérico | `chr(código)` |
| void | `<void>` | `{"type":"null","value":null}` | aceito como `<void>` |
| exceção | `EXCEPTION:Classe:msg` | `{"status":"exception","class","message"}` | **compara APENAS a classe** |

A mensagem da exceção é texto livre e difere entre a JVM/ART e a VM mínima
(ex.: `"/ by zero"` vs `"divide by zero"`), então o golden compara a **classe**;
o conteúdo de mensagens é coberto pelos casos `customMsgLen`/`customHash`
(length + hashCode da mensagem, que têm de bater bit a bit).

## Cobertura dos casos (~5 classes, todos `public static` puros, sem
dependências cruzadas; `golden/vm/cases/`)

- **Arith** — add/sub/mul/div/rem int e long (incl. negativos, `MIN/-1`,
  wrapping `65536*65536`), shifts com contagem via variável (incl. mascaramento
  `1<<33`), bitwise/not/neg, casts int↔long↔float↔double↔byte/char/short (incl.
  NaN/Infinity → `0`/`Long.MAX`), float/double com `Infinity`/`NaN`,
  comparações `<`/`>=` com NaN (cmpl/cmpg), boolean, `Math.sqrt/min/max/abs`.
- **Control** — fib/fatorial/collatz/loopSum/gcd (loops+branches),
  `switch` contíguo (packed-switch) e esparso (sparse-switch, incl. caso
  negativo), comparações encadeadas, ternários.
- **Arrays** — `new-array`/aget/aput int/long/char, `array-length`,
  `ArrayIndexOutOfBoundsException` capturada, `NegativeArraySizeException` capturada.
- **Strings** — `hashCode` sobre constantes (algoritmo da spec), length/isEmpty,
  `concat`, `charAt` (incl. fora dos limites capturado), `substring`,
  `StringBuilder` append(int/long/String/char/boolean)+toString,
  `String.valueOf` — conteúdo validado por length/hashCode/equals.
- **Statics** — `static final` com ConstantValue (→ static_values do DEX),
  `<clinit>` real com contas, leitura/escrita de statics dentro da chamada,
  método `void`.
- **Exceptions** — try/catch de ArithmeticException (div por zero),
  IllegalArgumentException, NullPointerException (receiver null via argumento),
  exceção custom (`Boom extends RuntimeException`) materializada com
  getMessage(), rethrow para outra classe, escapes sem catch (vetor
  `EXCEPTION:`), catch amplo por `RuntimeException`.

Como rodar:

```bash
cd golden/vm
./fetch-tools.sh        # 1x (cache/ é reutilizado)
./build.sh              # compila + dexa
./run-golden.sh         # JVM real vs rd vm exec — exit 0 = 100% PASS
# ou com binário explícito:
./run-golden.sh ../../target/release/rd
```

No CI (`.github/workflows/ci.yml`, JDK presente): o ecj é pulado
automaticamente; os 3 scripts rodam em sequência. Nada do pipeline precisa de
sudo, gradle ou Android SDK — só JRE 17+, python3, curl e o binário `rd`.

## Política GH-05 — nenhum binário commitado

- `golden/vm/cases/` contém apenas **texto** (fontes Java + este README + scripts) — commitável.
- `golden/vm/cache/` (jars ecj/r8) e `golden/vm/build/` (classes, classes.dex,
  tsv/jsonl) são **gitignored** (ver `.gitignore`: `/golden/vm/cache/`,
  `/golden/vm/build/`).
- O DEX é reconstruído de fonte a cada `build.sh` — não há binário de teste
  versionado em lugar nenhum do repo (mesma política do `golden-apks/fetch.sh`).

## Limitações conhecidas (e decisões tomadas)

1. **Retornos String não entram na tabela golden.** O CLI hoje renderiza todo
   retorno de objeto como `{"type":"object","value":"<object>"}` — o conteúdo
   da string fica no heap da VM e não chega ao JSON
   (`result_json` trata `Value::Obj(_)` genérico em crates/rd-cli/src/main.rs;
   strings do heap são `HeapObj::Str` no rd-vm). Em vez de comparar `<object>`
   (que não validaria nada), os casos String retornam **primitivos derivados**
   (length/hashCode/equals do resultado) — o bytecode executa
   `concat`/`substring`/`toString` normalmente; só o valor final do vetor é
   primitivo. Quando o CLI passar a renderizar `HeapObj::Str` com o conteúdo,
   basta acrescentar casos que retornam String à tabela (o GoldenRunner já
   sabe imprimir: conteúdo literal).
2. **Literal `null` no bytecode do usuário (`String s = null; s.length()`).**
   O javac/d8 emite `const/4 v0, 0` para null e a VM mínima ainda não
   reinterpreta `Int(0)` como referência null (`VM_TYPE_ERROR: esperado
   referência, got Int(0)`). Pela spec Dalvik, const/4 0 usado em contexto de
   referência É null — bug real do M2 anotado para o rd-vm. O caso
   `nullStringLen` contorna recebendo `null` via `--args 'null'`
   (`parse_arg_value` → `Value::Null`), que exerce o mesmo NPE+catch.
3. **Concatenação com `+` de strings é proibida nos casos** (javac 21 gera
   `invokedynamic makeConcatWithConstants` — fora do escopo M2). Usamos
   `String.concat` e `StringBuilder` explícitos. Idem: sem lambdas/streams/
   autoboxing/varargs/synchronized/instanceof/switch-de-string.
4. **Mensagens de exceção**: comparação só pela classe (ver tabela acima).
5. **Determinismo**: nenhum caso pode depender do estado deixado por outro —
   o lado VM executa cada caso num processo novo, o lado JVM roda tudo num
   processo só. (Por isso `Statics.setAndRead` é determinístico por invocação.)

## Arquivos

| Arquivo | Papel |
|---|---|
| `fetch-tools.sh` | baixa ecj/r8 para `cache/` (valida versões; pula ecj se houver javac) |
| `build.sh` | compila (javac/ecj) + dexa (D8) + valida `classes.dex` |
| `run-golden.sh` | GoldenRunner → expected.tsv; rd vm exec → actual.jsonl; compara |
| `cases/Arith.java` | aritmética/casts/IEEE/boolean (opcodes núcleo) |
| `cases/Control.java` | loops/branches/switches/ternários |
| `cases/Arrays.java` | arrays + AIOOBE/NASE capturadas |
| `cases/Strings.java` | intrínsecos String/StringBuilder (retornos primitivos) |
| `cases/Statics.java` | statics, `<clinit>`, static_values, void |
| `cases/Exceptions.java` | try/catch/throw/rethrow + classe custom `Boom` |
| `cases/GoldenRunner.java` | tabela de casos + reflexão + TSV (lado JVM) |
| `cache/`, `build/` | gitignored (GH-05) |
