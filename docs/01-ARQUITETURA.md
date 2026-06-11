# 01 · Arquitetura

## 1. Visão arquitetural

TkSpeed adota **Clean Architecture + DDD** sobre um workspace Rust modular, com o frontend React como camada de apresentação desacoplada via IPC do Tauri.

```
┌───────────────────────────────────────────────────────────────┐
│                     PRESENTATION (React/TS)                     │
│   Views • Design System • Estado UI (Zustand) • Charts          │
└───────────────▲───────────────────────────────┬────────────────┘
                │  invoke() / events (IPC)        │
┌───────────────┴───────────────────────────────▼────────────────┐
│                  TAURI BRIDGE (src-tauri)                       │
│   Command handlers • Event emitters • Permissões • State        │
└───────────────▲───────────────────────────────┬────────────────┘
                │ chamadas tipadas (tk-contracts) │
┌───────────────┴───────────────────────────────▼────────────────┐
│                    APPLICATION (Use Cases)                      │
│   Orquestração • CQRS (Commands/Queries) • Sagas de otimização  │
└───────────────▲───────────────────────────────┬────────────────┘
                │                                 │
┌───────────────┴─────────────────┐ ┌─────────────▼───────────────┐
│        DOMAIN (Entidades)        │ │   INFRA / ADAPTERS          │
│ Bottleneck • Score • Snapshot •  │ │ WMI • PDH • LHM • Registry  │
│ Profile • OptimizationPlan       │ │ SQLite • FS • PowerShell    │
└──────────────────────────────────┘ └─────────────────────────────┘
```

### Regra de dependência
As setas de dependência apontam **para dentro**. `domain` não conhece `infra`. `application` depende de `domain` e de *traits* (portas); as implementações concretas (adapters) são injetadas na composição (`tk-core`).

## 2. Camadas

| Camada | Crate(s) | Responsabilidade | Conhece |
|---|---|---|---|
| Domain | `tk-contracts` (tipos), domínio dentro de cada módulo | Entidades, value objects, regras puras | nada externo |
| Application | use cases dentro de cada crate de módulo | Orquestrar domínio, CQRS | domain + ports |
| Infra/Adapters | `tk-storage`, coletores em `tk-monitor` | WMI, PDH, SQLite, Registry, FS | bibliotecas externas |
| Composition | `tk-core` | DI, bootstrap, lifecycle | tudo |
| Presentation | `apps/desktop`, `src-tauri` | UI + bridge | contracts |

## 3. Padrões aplicados

- **SOLID** — interfaces pequenas (ISP), inversão de dependência via traits Rust.
- **DDD** — *bounded contexts* por módulo (Monitoring, Analysis, Optimization, GameBoost, Benchmark, Rollback, Reporting).
- **CQRS** — separação entre *Commands* (mutam estado/sistema, sempre transacionais com rollback) e *Queries* (leitura de telemetria/histórico, otimizadas para throughput).
- **Event-Driven** — `TkMonitor` publica amostras num *event bus* interno (broadcast Tokio); UI assina via eventos Tauri.
- **Saga / Unit of Work** — toda otimização é uma saga: `snapshot → apply → verify → (commit | compensate)`.
- **Repository** — persistência abstraída por traits (`SnapshotRepository`, `MetricRepository`...).
- **Strategy** — coletores de métrica e regras de análise são plugáveis.
- **Plugin (futuro)** — módulos de otimização carregáveis via ABI estável (WASM/dynamic).

## 4. Bounded Contexts (DDD)

```
Monitoring   → amostras de telemetria em tempo real (alta frequência)
Diagnostics  → análise, detecção de gargalo, score
Optimization → planos, aplicação transacional, perfis de energia/rede/gamer
GameBoost    → detecção de jogo, perfis, suspensão de processos
Benchmark    → suites de teste, scores brutos
Rollback     → snapshots, restauração, auditoria
Reporting    → composição de relatórios
DigitalTwin  → série temporal histórica, baseline, regressões
Identity/Licensing → licença, tier, ativação (offline-first)
```

## 5. Concorrência e threading

- **Runtime async**: Tokio multi-thread.
- **Coleta de telemetria**: tarefas dedicadas por fonte (CPU/GPU/RAM/IO/Net/Temp) com cadências independentes (ex.: temperatura 1s, FPS 250ms, disco 2s). Backpressure via `tokio::sync::broadcast` com *lag policy* (descarta amostra antiga).
- **Aplicação de otimizações**: pool de trabalho separado; operações privilegiadas serializadas por um *mutex de sistema* para evitar conflito de escrita no Registry/serviços.
- **UI nunca bloqueia**: todo comando longo retorna `task_id` e emite progresso por evento.

## 6. Comunicação Frontend ↔ Backend

- **Comandos** (`#[tauri::command]`): request/response tipado, definido em `tk-contracts` e espelhado em TS por geração de tipos (`ts-rs`).
- **Eventos** (`app.emit`): streaming de telemetria e progresso (`metrics:sample`, `optimize:progress`, `gameboost:state`).
- **Canais** (`tauri::ipc::Channel`): para streams de alta frequência sem poluir o event bus global.

## 7. Decisões arquiteturais (ADR resumido)

| ADR | Decisão | Razão |
|---|---|---|
| 001 | Tauri sobre Electron | ~10–20× menor footprint, Rust nativo, WebView2 já presente no Win11 |
| 002 | Workspace Rust multi-crate | Bounded contexts isolados, compilação incremental, testabilidade |
| 003 | SQLite + WAL | Local-first, transacional, zero servidor, ideal p/ série temporal moderada |
| 004 | LibreHardwareMonitor via interop | Cobertura de sensores superior a WMI puro |
| 005 | Privilégio sob demanda | App roda *non-elevated*; eleva só para ações que exigem (UAC) |
| 006 | Telemetria local por padrão | Privacidade como diferencial; opt-in para qualquer envio |

Veja [`15-ESPECIFICACAO-TECNICA.md`](15-ESPECIFICACAO-TECNICA.md) para detalhes de implementação.
