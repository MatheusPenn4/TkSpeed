# TkSpeed · Backlog de Engenharia

Backlog profissional derivado da documentação de arquitetura, PRD, ADRs e roadmap.
Estrutura: **EPIC → FEATURE → USER STORY → TASKS → CRITÉRIOS DE ACEITE → DEPENDÊNCIAS → ESTIMATIVA**.

## Índice

| Doc | Conteúdo |
|---|---|
| [00 · Convenções](00-CONVENCOES.md) | IDs, prioridade, esforço, risco, DoD global, mapa de pontos |
| [01 · Fase 1 — MVP](01-FASE1-MVP.md) | Plataforma/SQLite/Logs, Monitoramento, Diagnóstico, Score, Benchmark, Rollback, Dashboard |
| [02 · Fase 2 — Alpha](02-FASE2-ALPHA.md) | Otimizações reais, Game Boost, Relatórios, Histórico |
| [03 · Fase 3 — Beta](03-FASE3-BETA.md) | Perfis, Licenciamento, Atualizador |
| [04 · Fase 4 — v1.0](04-FASE4-V1.md) | Marketplace, Plugins, Digital Twin |
| [05 · Fase 5 — v2.0](05-FASE5-V2.md) | TkAI, Cloud opcional, Enterprise |
| [06 · Planejamento](06-PLANEJAMENTO.md) | Caminho crítico, dependências, ordem ideal, sprints (8 sem.), paralelismo |
| [`backlog.csv`](backlog.csv) | Importável para Jira / Linear / Azure DevOps |

## Como importar

- **Jira:** Settings → System → External System Import → CSV. Mapear colunas `Issue Type`, `Epic Name`/`Parent`, `Summary`, `Priority`, `Story Points`, `Labels`, `Components`.
- **Linear:** Settings → Import/Export → CSV. Mapear `Title`, `Description`, `Priority`, `Estimate`, `Labels`, `Parent issue`.
- **Azure DevOps:** Boards → import Work Items (CSV). Mapear `Work Item Type` (Epic/Feature/User Story/Task), `Title`, `Priority`, `Story Points`, `Tags`.

> O CSV usa um esquema neutro (coluna `IssueType` com Epic/Feature/Story/Task e coluna `ParentId`) que mapeia para os três sistemas.
