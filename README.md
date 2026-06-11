# TkSpeed

> Plataforma profissional de análise, monitoramento, otimização, benchmarking e aceleração de computadores Windows.

**TkSpeed não é um limpador de PC.** É uma plataforma premium de performance para Windows — diagnóstico profissional, monitoramento em tempo real, análise de gargalos, otimizações 100% reversíveis, benchmarking interno, Game Boost inteligente e um *Digital Twin* histórico da máquina.

---

## Stack

| Camada | Tecnologia |
|---|---|
| Shell Desktop | **Tauri 2.x** (WebView2 no Windows) |
| Backend / Core | **Rust** (multi-thread, async via Tokio) |
| Frontend | **React 18 + TypeScript + Vite** |
| Estado UI | Zustand + TanStack Query |
| Banco local | **SQLite** (via `sqlx`/`rusqlite`) + WAL |
| Telemetria HW | LibreHardwareMonitorLib, WMI, PDH (Performance Counters), Windows API |
| Relatórios | HTML (template) → PDF (engine `printpdf` / `wkhtmltopdf` opcional) |
| Updates | Tauri Updater (assinado) |

> A análise completa de stack e alternativas está em [`docs/15-ESPECIFICACAO-TECNICA.md`](docs/15-ESPECIFICACAO-TECNICA.md).

## Documentação

| # | Documento | Conteúdo |
|---|---|---|
| 00 | [Visão Geral](docs/00-VISAO-GERAL.md) | Produto, posicionamento, concorrentes |
| 01 | [Arquitetura](docs/01-ARQUITETURA.md) | Clean Architecture, DDD, CQRS, diagramas |
| 02 | [Módulos](docs/02-MODULOS.md) | TkCore, TkMonitor, TkAnalyzer, etc. |
| 03 | [Estrutura de Pastas](docs/03-ESTRUTURA-PASTAS.md) | Monorepo completo |
| 04 | [Banco de Dados](docs/04-BANCO-DADOS.md) | Schema SQLite + migrations |
| 05 | [Fluxo de Dados](docs/05-FLUXO-DADOS.md) | IPC, eventos, pipelines |
| 06 | [Design System](docs/06-DESIGN-SYSTEM.md) | Tokens, componentes, efeitos |
| 07 | [Roadmap](docs/07-ROADMAP.md) | Fases 1 a 5 |
| 08 | [Segurança](docs/08-SEGURANCA.md) | Modelo de ameaças, privilégios, assinatura |
| 09 | [Rollback](docs/09-ROLLBACK.md) | Snapshots, transações, restauração |
| 10 | [Monetização](docs/10-MONETIZACAO.md) | Free/Pro/Studio, licenciamento |
| 11 | [Escalabilidade](docs/11-ESCALABILIDADE.md) | Plugins, multi-máquina, cloud opcional |
| 12 | [Casos de Uso](docs/12-CASOS-DE-USO.md) | User stories + fluxos |
| 13 | [TkSpeed Score](docs/13-TKSPEED-SCORE.md) | Modelo de pontuação 0–1000 |
| 14 | [Digital Twin](docs/14-DIGITAL-TWIN.md) | Perfil histórico da máquina |
| 15 | [Especificação Técnica](docs/15-ESPECIFICACAO-TECNICA.md) | Detalhamento de engenharia |
| 16 | [Wireframes](docs/16-WIREFRAMES.md) | Telas principais (baixa fidelidade) |

### Produto, Negócio & Estratégia → [`docs/business/`](docs/business/README.md)

| # | Documento | Conteúdo |
|---|---|---|
| B01 | [PRD](docs/business/01-PRD.md) | Visão, personas, RF/RNF, KPIs |
| B02 | [ADR](docs/business/02-ADR.md) | Decisões técnicas (Rust, Tauri, Saga, Rollback…) |
| B03 | [Licenciamento](docs/business/03-LICENCIAMENTO.md) | Free/Pro/Studio/Enterprise, anti-fraude |
| B04 | [Diferenciação](docs/business/04-DIFERENCIACAO.md) | Concorrentes + posicionamento |
| B05 | [Plugins](docs/business/05-PLUGINS.md) | SDK, WASM, sandbox, marketplace |
| B06 | [TkAI](docs/business/06-TKAI-IA.md) | Estratégia de IA híbrida local-first |
| B07 | [Plano de Lançamento](docs/business/07-PLANO-LANCAMENTO.md) | MVP→Alpha→Beta→1.0→2.0 |
| B08 | [Matriz de Riscos](docs/business/08-MATRIZ-RISCOS.md) | Técnicos/legais/segurança/comerciais |
| B09 | [Mercado & Receita](docs/business/09-MERCADO-RECEITA.md) | TAM/SAM/SOM, projeções |
| B10 | [Documento de Investimento](docs/business/10-INVESTOR-DECK.md) | Pitch executivo (capstone) |

### Performance Lab (fundação de medição) → [`docs/performance-lab/`](docs/performance-lab/README.md)

Design do `TkPerformanceLab`: arquitetura, catálogo de métricas, estratégia de benchmark/baseline/comparação (com margem de erro), detectores de gargalo, wireframe e backlog. **Princípio:** nenhum ganho é afirmado sem evidência reproduzível.

### TkOptimization Engine (Fase 2) → [`docs/optimization-engine/`](docs/optimization-engine/README.md)

Plataforma de engenharia de performance: 7 módulos (Engine/Catalog/Executor/Validator/Evidence/Advisor/Profiles), catálogo (SAFE/MODERATE/ADVANCED/EXPERIMENTAL), pipelines de execução/validação/rollback e backlog por slices. **Loop fechado:** snapshot → bench antes → aplicar → bench depois → Confidence Engine → comparar → manter ou reverter. Reaproveita Rollback + PerfLab + Analyzer.

### Distribuição Alpha → [`docs/release/`](docs/release/ALPHA-RELEASE.md)

Runbook de release (build, instalador NSIS/MSI, empacotamento, distribuição, checklist) + [matriz de testes](docs/release/TEST-MATRIX.md). Artefatos do pacote em [`release/`](release/) (LEIA-ME, CHANGELOG, licença Alpha, feedback, script de diagnóstico).

### Backlog de Engenharia → [`docs/backlog/`](docs/backlog/README.md)

Epics→Features→Stories→Tasks por fase + planejamento de 8 semanas + [`backlog.csv`](docs/backlog/backlog.csv) (importável para Jira/Linear/Azure DevOps).

## Estrutura do repositório

```
tkspeed/
├── apps/desktop/          # Frontend React + Vite
├── src-tauri/             # Shell Tauri + bridge de comandos
├── crates/                # Workspace Rust (núcleo modular)
│   ├── tk-core/
│   ├── tk-monitor/
│   ├── tk-analyzer/
│   ├── tk-optimizer/
│   ├── tk-gameboost/
│   ├── tk-benchmark/
│   ├── tk-rollback/
│   ├── tk-report/
│   ├── tk-storage/
│   └── tk-contracts/
├── docs/
└── packages/ui/           # Design system compartilhado
```

## Princípio inegociável

> **Toda alteração no sistema é reversível.** Nenhuma otimização é aplicada sem `snapshot → log → plano de rollback`. Segurança é prioridade absoluta.
