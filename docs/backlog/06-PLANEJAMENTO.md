# 06 · Planejamento de Execução

## 1. Caminho crítico do MVP

A sequência que **não pode atrasar** sem empurrar a data do MVP. Cada item bloqueia o próximo.

```
TK-S0111  AppContext / bootstrap
   ↓
TK-S0121  SQLite + migrations
   ↓
TK-S0122  Repositórios base
   ├──────────────────────────────┐
   ↓                               ↓
TK-S0611  SnapshotStore        TK-S0211  Coletor base + bus
   ↓                               ↓
TK-S0621  Saga (commit/comp.)  TK-S0212  Adapter PDH (CPU/RAM/IO/net)
   ↓                               ↓
[pilot tweak validado em VM]   TK-S0311  BottleneckEngine
                                   ↓
                               TK-S0411  TkSpeed Score
                                   ↓
                               TK-S0331  analyze_full
                                   ↓
                               TK-S0721  Dashboard com dados reais  ← DEMO do MVP
```

> **Nota de escopo:** conforme o roadmap, o **catálogo de otimizações reais** vive na Fase 2 (Alpha). O MVP valida a **infraestrutura de mutação reversível** (saga + snapshot) aplicando **1 tweak-piloto** (`power.high_performance`) em VM — provando o ciclo *aplicar → medir → reverter* end-to-end sem ainda abrir o catálogo completo.

**Os dois P0 mais arriscados do caminho:** `TK-S0621` (saga) e `TK-S0611` (snapshot) — risco 🔴. Atacar cedo, com testes em VM, é a maior redução de risco do projeto.

## 2. Dependências técnicas (grafo resumido)

| Story | Depende de | Habilita |
|---|---|---|
| TK-S0111 | — | tudo |
| TK-S0112 | S0111 | toda UI tipada |
| TK-S0121 | S0111 | S0122, S0131, S1311 |
| TK-S0122 | S0121 | repos de métricas/score/snapshot |
| TK-S0211 | S0111, S0122 | S0212, S0213, S0214, S0311 |
| TK-S0212 | S0211 | S0311 (dados de CPU/RAM/IO) |
| TK-S0311 | S0211, S0122 | S0411, S0331 |
| TK-S0411 | S0311 | S0331, S0721 |
| TK-S0611 | S0122 | S0621, S0631, S0812 |
| TK-S0621 | S0611 | S0812 (tweaks), Game Boost, Perfis |
| TK-S0711 | S0111 | todas as telas |
| TK-S0214 | S0211, S0112 | S0721, S0722 |

**Regra de ouro:** nenhuma mutação de SO (Fase 2+) começa antes de `TK-S0621` + `TK-S0611` estarem **verdes em VM**.

## 3. Ordem ideal de implementação

1. **Fundação primeiro** (E01): bootstrap → SQLite → repos → contratos/ts-rs → logs → CI. *Sem isto, nada é testável.*
2. **Risco alto cedo** (E06): SnapshotStore + Saga em paralelo à fundação tardia. *Reduz o maior risco do produto.*
3. **Leitura antes de escrita** (E02 → E03 → E04): monitoramento → diagnóstico → score. *Valor visível e somente-leitura (seguro).*
4. **Medição** (E05): benchmark para comparar antes/depois.
5. **UI integrando tudo** (E07): dashboard/telas com dados reais.
6. **Só então mutação real** (Fase 2/E08): catálogo de tweaks sobre a saga já validada.

## 4. Sprint Planning — 8 semanas (4 sprints × 2 semanas)

**Time-base assumido:** 2 eng. Rust (A, B), 1 eng. Frontend (FE), apoio de Design/QA parcial. Velocity-alvo ~35 pts/sprint. *(Ajustar ao time real.)*

### Sprint 1 (Sem. 1–2) — Fundação · ~33 pts
| Story | Pts | Dono | Risco |
|---|--:|---|:--:|
| TK-S0111 AppContext/lifecycle | 5 | A | 🟢 |
| TK-S0121 SQLite + migrations | 5 | A | 🟢 |
| TK-S0112 Contratos + ts-rs | 5 | B/FE | 🟡 |
| TK-S0131 Logging estruturado | 5 | B | 🟢 |
| TK-S0141 CI Windows | 5 | B | 🟢 |
| TK-S0711 Design System (packages/ui) | 8 | FE | 🟢 |
> **Meta:** app inicia, persiste, loga, CI verde, UI base. Fim do sprint = "esqueleto vivo".

### Sprint 2 (Sem. 3–4) — Monitoramento · ~36 pts
| Story | Pts | Dono | Risco |
|---|--:|---|:--:|
| TK-S0122 Repositórios base | 8 | A | 🟢 |
| TK-S0211 Coletor base + bus | 8 | B | 🟡 |
| TK-S0212 Adapter PDH | 8 | B | 🟡 |
| TK-S0214 Stream de métricas p/ UI | 5 | A/FE | 🟢 |
| TK-S0712 Titlebar custom | 2 | FE | 🟢 |
| TK-S0722 Tela Monitoramento (início) | 8→5 | FE | 🟢 |
> **Meta:** métricas reais ao vivo no app. Substituir o mock `useLiveMetrics`.

### Sprint 3 (Sem. 5–6) — Diagnóstico, Score & Snapshot · ~39 pts
| Story | Pts | Dono | Risco |
|---|--:|---|:--:|
| TK-S0311 BottleneckEngine | 8 | B | 🟡 |
| TK-S0411 TkSpeed Score | 8 | A | 🟡 |
| TK-S0331 analyze_full + progresso | 5 | B | 🟢 |
| TK-S0611 SnapshotStore + integridade | 8 | A | 🔴 |
| TK-S0723 Tela Diagnóstico | 5 | FE | 🟢 |
| TK-S0213 Adapter LHM (temp/clock) | 8 | B | 🔴 |
> **Meta:** scan completo → findings + score na tela. Snapshot pronto p/ a saga. *(LHM pode escorregar p/ buffer se a interop atrasar — não está no caminho crítico.)*

### Sprint 4 (Sem. 7–8) — Saga, Rollback, Benchmark & Demo · ~36 pts
| Story | Pts | Dono | Risco |
|---|--:|---|:--:|
| TK-S0621 Saga (commit/compensate) | 8 | A | 🔴 |
| TK-S0612 Quarentena | 5 | A | 🟡 |
| TK-S0631 Tela Rollback | 5 | FE | 🟢 |
| TK-S0511 Suites de benchmark | 8 | B | 🟡 |
| TK-S0512 Comparação antes/depois | 5 | B | 🟢 |
| TK-S0721 Dashboard com dados reais | 5 | FE | 🟢 |
> **Meta (DEMO do MVP):** rodar tweak-piloto → medir benchmark antes/depois → reverter 100% via UI de Rollback.

### Buffer / backlog de capacidade (puxar conforme sobra)
`TK-S0142` health gate · `TK-S0132` crash handler · `TK-S0321` startup analyzer · `TK-S0221` inventário · `TK-S0412` drill-down do score · `TK-S0724` tela Benchmark.

## 5. O que pode ser desenvolvido em paralelo

Três trilhas com baixo acoplamento após a fundação (Sprint 1):

```
TRILHA RUST-CORE (A)      TRILHA RUST-SENSORES/ANÁLISE (B)   TRILHA FRONTEND (FE)
─────────────────────     ──────────────────────────────    ─────────────────────
AppContext, SQLite,       Coletor base, PDH, LHM,           Design System, Shell,
Repos, Snapshot, Saga,    BottleneckEngine, Score,          Monitoramento, Diagnóstico,
Quarentena                analyze_full, Benchmark           Rollback, Dashboard
```

**Pontos de sincronização (handshakes):**
- `tk-contracts` (S0112) deve estabilizar cedo → FE trabalha contra tipos, não contra implementação.
- `S0214` (stream) é o encontro Rust-core ↔ FE.
- `S0331`/`S0411` (analyze+score) alimentam `S0721`/`S0723` (telas) — FE pode usar fixtures até o backend ligar.

**Paralelizável entre fases:** documentação de Licenciamento/Plugins/TkAI e specs de backend de cloud podem ser refinadas por Produto/Arquitetura **durante** as Fases 1–2, sem bloquear engenharia.

**NÃO paralelizar:** Saga (S0621) e qualquer catálogo de tweaks (Fase 2) — os tweaks dependem da saga validada. Forçar paralelismo aqui é o erro clássico que gera dívida e risco de "PC quebrado".

## 6. Métricas de acompanhamento (por sprint)

- Burndown de pontos; % do caminho crítico concluído.
- Cobertura de testes do domínio (score/saga/bottleneck).
- Health budget no CI (verde/vermelho).
- Nº de reverts bem-sucedidos em VM (deve ser 100%).
