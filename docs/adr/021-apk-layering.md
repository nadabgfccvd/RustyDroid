# ADR 021 — Layering: executar dois APKs ao mesmo tempo (um por cima do outro)

- **Status:** proposto — aguarda decisão do dono (ver "Decisões necessárias")
- **Data:** 2026-11-02 · **Escopo:** `crates/rd-agent` (tools `launch`, `tap`, `swipe`, `input`), `crates/rd-render`, `crates/rd-cli` (`boot_app`), e um novo módulo de sessões
- **Relacionados:** objetivos adicionais do dono em `docs/ROADMAP.md` ("Objetivos adicionais do dono", 2026-11-02); ADR 016 (piso de API); M10 (dispositivos virtuais).

## Contexto

O dono pediu que dois APKs rodem ao mesmo tempo, com um por cima do outro. O caso de uso citado é um jogo clicker e um auto-clicker. O auto-clicker precisa enxergar a tela do jogo (ou pelo menos saber onde tocar) e injetar toques nela, enquanto o jogo segue respondendo.

Hoje o RustyDroid executa um app por vez: `launch` inicia um pacote, e `tap`, `swipe` e `input` atuam sobre ele. Não há um conceito de sessão múltipla nem de camada de entrada compartilhada. Esta ADR só decide o caminho; a implementação depende da escolha abaixo e não está nesta entrega.

## Opções

**A. Duas sessões com camada de entrada compartilhada (recomendada).**
Cada APK roda na sua própria sessão (VM, namespace de classes, diretório de dados e engine de permissões próprios). Uma terceira peça, o roteador de entrada, fica entre o dispositivo virtual e as sessões. A sessão de cima (o auto-clicker) recebe um canal de injeção de toques destinado à sessão de baixo (o jogo). Isso reaproveita a camada de entrada que já existe nas tools `tap` e `swipe`. A composição visual é feita pelo `rd-render`, com a sessão de cima em uma camada transparente.
- Prós: isolamento claro entre os dois APKs, o que mantém o modelo de permissões e de dados de cada um; a injeção de toques passa pelo mesmo caminho de um toque real.
- Contras: exige que o `rd-render` componha duas camadas; a sessão de cima precisa de uma janela transparente, o que pode não existir em todos os APKs.

**B. Um processo com dois "apps" dentro da mesma VM.**
Mais simples de implementar, mas quebra o isolamento: os dois APKs compartilham heap, classes e permissões. Não é aceitável para o modelo de segurança do projeto.

**C. Serviço externo de automação (fora do APK).**
O auto-clicker vira um script do próprio `rd` que chama `tap` em loop, sem rodar como APK. Resolve o caso clicker/auto-clicker em pouco tempo, mas não cumpre o pedido de "dois APKs rodando".

## Recomendação

Opção A, em duas etapas:
1. **Sessões múltiplas sem camada visual:** dois APKs em sessões separadas, com tools `session_list`, `session_launch` e `session_tap` (o toque é roteado para uma sessão alvo). Prova o isolamento e a injeção de toques.
2. **Camada visual:** composição de duas sessões no `rd-render`, com a sessão de cima transparente.

## Riscos

- **Memória:** o Moto E5 tem orçamento de 1 GB de RSS (ADR 015). Duas sessões dobram o custo. É preciso medir antes de prometer o caso clicker + auto-clicker no alvo.
- **Concorrência:** duas VMs no mesmo processo precisam respeitar a regra de estado thread-local da ADR 017.
- **Conteúdo do APK:** auto-clickers reais costumam usar Accessibility Service. O `rd` não tem essa API hoje; a opção A não a cobre, e a sessão de cima só funciona se o APK depender apenas de toque e de tela.

## Decisões necessárias do dono

1. A primeira entrega é a opção A em duas etapas (sessões primeiro, camada visual depois), ou prefere a opção C como atalho?
2. O auto-clicker de teste é um APK real (qual?) ou um APK de teste escrito para o `rd`?
3. O requisito é só a sessão de cima injetar toques, ou o auto-clicker também precisa ler a tela (por exemplo, `ui_tree`)?

## Fora de escopo desta ADR

- Instalação de bundles XAPK em sessões (depende do item XAPK, que já tem leitura de splits).
- Accessibility Service e APIs de automação de sistema.
