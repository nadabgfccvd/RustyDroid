# ADR 027 — Metadados de `Class` para reflexão do Kotlin: sem falsear

- **Status:** aceito · **Data:** 2026-11-03
- **Escopo:** `crates/rd-vm/src/intrinsics.rs` (`class_intrinsic`, `type_parameters_of`),
  `crates/rd-vm/src/jlclass.rs` (`isAnnotation`, `isSynthetic`),
  `crates/rd-vm/src/jannotations.rs` (`class_generic_signature`),
  `crates/rd-vm/src/classpath.rs` (`AssertionError` na hierarquia embutida)
- **Relacionados:** ADR 025 (launch do F-Droid), teste `fdroid_oncreate_e2e`,
  Apêndice C (API ausente vira erro estruturado, não silêncio; `NOT_IMPLEMENTED`)
- **Ordem:** decisão registrada antes do código de reflexão genérica (CONTRIBUTING).

## Contexto

Com o APK real do F-Droid, `App.onCreate` passa pelo Ktor. Na inicialização do
`ContentEncodingKt`, o Kotlin reflect percorre o modelo Java das classes
(`ReflectJavaClass`). A cadeia pediu, nesta ordem:

1. `Class.getTypeParameters()` (nas classes `Cloneable`, `Unit`, `ProgressListener`,
   `HttpSend`, `Attributes`, `HttpTimeoutConfig`);
2. `Class.isAnnotation()`;
3. `AssertionError.<init>(Object)`.

Nenhum desses membros tinha implementação. O primeiro não pode ser feito por um
array vazio fixo: para uma classe genérica como `java.util.List`, isso devolve um
metadado falso sem erro, e o Kotlin reflect constrói tipos errados em silêncio.

## Decisão

1. **`getTypeParameters()` responde com a verdade que a VM sabe, ou falha com
   `NOT_IMPLEMENTED`.** Nunca devolve array vazio para uma classe que pode ser
   genérica.
   - Arrays e primitivos: array vazio (não têm parâmetros).
   - Classe DEX: lê o `dalvik.annotation.Signature` (visibilidade SYSTEM). Sem
     `<...>` na assinatura: array vazio. Com `<...>`: `NOT_IMPLEMENTED`, porque
     `TypeVariable` não é modelado.
   - Classe de plataforma: array vazio só para a lista fechada
     `NON_GENERIC_HOST_TYPES` (Object, String, Cloneable, Serializable, Runnable,
     CharSequence, números e boxes, Throwable e três subclasses). Qualquer outra:
     `NOT_IMPLEMENTED`.
2. **Flags de `Class` vêm das flags do DEX.** `isAnnotation` usa `ACC_ANNOTATION`
   (0x2000) e `isSynthetic` usa `ACC_SYNTHETIC` (0x1000) da definição da classe.
   Para classes de plataforma sem DEX, a flag é falsa.
3. **`AssertionError` é `Error`.** Construtor `(Object)`: a mensagem é
   `String.valueOf(detail)` (null vira `"null"`); um `Throwable` como detalhe também
   vira a causa.

## Consequências

- O F-Droid passa de `App.onCreate` ainda lançando (`ExceptionInInitializerError`)
  para `onCreate` concluindo sem exceção. O teste E2E `fdroid_oncreate_e2e` passa.
- Classes de plataforma genéricas (por exemplo `java.util.List`) passam a falhar de
  forma visível em `getTypeParameters()` até haver metadado genérico real para elas.
  Esse é o custo aceito.
- Reflexão completa (`getDeclaredMethods` com tipos genéricos, `TypeVariable`,
  `ParameterizedType`) continua fora do escopo. É a próxima frente se um app exigir.

## Testes

- `launch_gap_tests` (intrinsics.rs): array vazio para `String`, `Cloneable`, `[I`, `I`;
  `NOT_IMPLEMENTED` para `java.util.List`; `AssertionError(Object)` com texto e com null.
- `fdroid_oncreate_e2e`: `App.onCreate` do F-Droid real não lança.
