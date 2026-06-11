# 03 · Estrutura de Pastas

Monorepo: frontend (pnpm workspace) + backend (Cargo workspace), unidos pelo Tauri.

```
tkspeed/
├── README.md
├── package.json                  # raiz pnpm workspace
├── pnpm-workspace.yaml
├── tsconfig.base.json
│
├── apps/
│   └── desktop/                  # Frontend React + Vite
│       ├── index.html
│       ├── vite.config.ts
│       ├── tsconfig.json
│       ├── src/
│       │   ├── main.tsx
│       │   ├── App.tsx
│       │   ├── app/              # rotas, providers, layout
│       │   │   ├── router.tsx
│       │   │   ├── providers.tsx
│       │   │   └── shell/        # AppShell, Sidebar, TitleBar
│       │   ├── features/         # 1 pasta por feature (vertical slice)
│       │   │   ├── dashboard/
│       │   │   ├── analysis/
│       │   │   ├── monitoring/
│       │   │   ├── gameboost/
│       │   │   ├── benchmark/
│       │   │   ├── history/      # Digital Twin
│       │   │   ├── reports/
│       │   │   ├── rollback/
│       │   │   ├── diagnostics/  # Central de Diagnóstico
│       │   │   └── settings/
│       │   ├── shared/
│       │   │   ├── ipc/          # wrappers tipados de invoke/eventos
│       │   │   ├── hooks/
│       │   │   ├── stores/       # Zustand
│       │   │   └── lib/
│       │   └── styles/
│       └── public/
│
├── packages/
│   └── ui/                       # Design System (componentes + tokens)
│       ├── src/
│       │   ├── tokens/           # cores, espaçamento, tipografia, motion
│       │   ├── primitives/       # Button, Card, GlassPanel, Gauge...
│       │   ├── charts/           # gráficos em tempo real
│       │   └── index.ts
│       └── package.json
│
├── src-tauri/                    # Shell Tauri
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── capabilities/             # permissões Tauri v2
│   │   └── default.json
│   ├── icons/
│   └── src/
│       ├── main.rs               # entrypoint
│       ├── bridge/               # #[tauri::command] handlers
│       │   ├── mod.rs
│       │   ├── monitor_cmd.rs
│       │   ├── analyze_cmd.rs
│       │   ├── optimize_cmd.rs
│       │   ├── gameboost_cmd.rs
│       │   ├── benchmark_cmd.rs
│       │   ├── rollback_cmd.rs
│       │   └── report_cmd.rs
│       └── events.rs             # emitters de telemetria/progresso
│
├── crates/                       # Cargo workspace (núcleo)
│   ├── tk-contracts/             # tipos compartilhados (DTOs, ts-rs)
│   ├── tk-core/                  # bootstrap, DI, lifecycle, permissões
│   ├── tk-monitor/               # coletores de telemetria
│   │   └── src/adapters/         # lhm, wmi, pdh, etw_fps
│   ├── tk-analyzer/              # bottleneck engine, auditores
│   ├── tk-optimizer/             # catálogo + sagas de otimização
│   │   └── src/optimizations/    # 1 arquivo por tweak
│   ├── tk-gameboost/             # detector + perfis
│   ├── tk-benchmark/             # suites
│   ├── tk-rollback/              # snapshots, quarentena, restore
│   ├── tk-report/                # render HTML/PDF
│   ├── tk-storage/               # SQLite (sqlx), repos, migrations
│   │   └── migrations/           # *.sql versionadas
│   └── tk-platform-win/          # wrappers Windows API (registry, services, power)
│
├── docs/                         # esta documentação
├── scripts/                      # build, sign, release
└── installer/                    # NSIS/MSI config
```

## Convenções

- **Vertical slice no frontend:** cada `features/<x>/` tem `components/`, `hooks/`, `api.ts`, `store.ts`, `<X>Page.tsx`.
- **1 tweak = 1 arquivo** em `crates/tk-optimizer/src/optimizations/` implementando o trait `Optimization`.
- **1 coletor = 1 adapter** em `crates/tk-monitor/src/adapters/`.
- **Migrations versionadas** `NNNN_descricao.sql`, nunca editadas após release.
- **Contratos primeiro:** todo tipo cruzando IPC vive em `tk-contracts` com `#[derive(TS)]` (ts-rs) → gera `.d.ts` para o frontend.
