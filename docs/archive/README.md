# Arquivo de documentos históricos

Documentos que deixaram de ser guias vigentes. Ficam aqui, e não apagados, para
preservar o raciocínio e as evidências. Cada arquivo tem um aviso no topo com o
motivo e o substituto. O histórico completo também está no `git log`.

| Documento | Motivo do arquivamento | Vigente |
|---|---|---|
| [`rd-sentinel.md`](rd-sentinel.md) | A ferramenta `tools/rd-sentinel/` foi removida em `08ae54f`; workflows em `46af515` (#436). | [ADR 010](../adr/010-rd-sentinel-anti-regression-bot.md) (emenda de 2026-11-11) |
| [`integration-validation.md`](integration-validation.md) | Checkpoint de 2026-10-20 do branch `feat/comprix-e2e` (PR #160). | [Appendix G integration](../appendix-g-integration.md), [Comprix E2E](../comprix-e2e.md) |
| [`performance-gc-20261020.md`](performance-gc-20261020.md) | Medição de 2026-10-20 em uma máquina e commit específicos. | [GC](../gc.md), [ADR 006](../adr/006-generational-gc-and-live-frame-roots.md) |

Evidência bruta referenciada por estes documentos continua fora de `docs/`
(por exemplo, `tests/performance/20261020/`).
