# 15 · Especificação Técnica

## 1. Avaliação de stack (e alternativas)

| Necessidade | Escolha | Alternativas consideradas | Veredito |
|---|---|---|---|
| Shell desktop | **Tauri 2** | Electron, .NET MAUI/WPF, Qt | Tauri vence: WebView2 já no Win11, footprint ~10–20× menor que Electron, Rust nativo para acesso a SO. Electron pesaria demais para um produto que promete leveza. WPF/.NET seria ótimo em integração Windows mas perde no frontend moderno/portável e no apelo de UI. |
| Backend/core | **Rust** | C#/.NET, C++ | Rust: segurança de memória (crítico num app que mexe no SO), performance, ótimo FFI com Windows API, concorrência sem data races. C++ daria poder igual sem as garantias. C# seria produtivo mas runtime maior e menos controle. |
| Frontend | **React 18 + TS + Vite** | Svelte, SolidJS, Vue | React: ecossistema, contratação, libs de chart/animação. Solid/Svelte são mais leves, mas React+Vite é rápido o suficiente e reduz risco de equipe. |
| Banco local | **SQLite (WAL)** | sled, RocksDB, DuckDB | SQLite: maduro, transacional, SQL p/ Digital Twin, zero servidor. DuckDB seria interessante p/ analytics pesado — overkill agora; pode ser adicionado p/ Twin no futuro. |
| Sensores HW | **LibreHardwareMonitorLib + WMI + PDH + ETW** | só WMI, OpenHardwareMonitor | LHM tem a melhor cobertura de sensores (temp/voltagem/clock). WMI/PDH p/ contadores de SO. ETW p/ FPS/eventos. Combinação cobre tudo. |
| Charts | **uPlot / visx / recharts** | chart.js, d3 puro | uPlot p/ tempo real (altíssima performance), visx p/ visual custom premium. |
| Estado UI | **Zustand + TanStack Query** | Redux Toolkit, Jotai | Zustand p/ estado local simples; TanStack Query p/ cache de comandos. |

**Conclusão:** a stack pedida (Tauri + Rust + React + SQLite) é, de fato, a melhor combinação para os requisitos (leveza, acesso ao SO, segurança, UI premium). Recomendações adicionais: `ts-rs` (tipos), `uPlot` (charts RT), `DuckDB` como evolução futura do Twin se o analytics crescer.

## 2. Crates e dependências-chave (Rust)

```toml
# workspace
tokio        # runtime async multi-thread
serde        # serialização
sqlx         # SQLite async + migrations (ou rusqlite p/ sync simples)
thiserror / anyhow  # erros
tracing      # logs estruturados
ts-rs        # geração de tipos TS a partir de structs Rust
windows      # bindings oficiais Windows API (registry, services, power)
wmi          # consultas WMI
sysinfo      # fallback de info de sistema
# interop LibreHardwareMonitor via .NET host (clr) ou wrapper
```

## 3. Contratos (exemplo `tk-contracts`)

```rust
#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MetricSample {
    pub ts: i64,
    pub source: MetricSource,   // Cpu|Gpu|Ram|Ssd|Net|Temp...
    pub metric: String,
    pub value: f64,
    pub unit: String,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Diagnosis {
    pub run_id: i64,
    pub findings: Vec<Finding>,
    pub score: TkSpeedScore,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Finding {
    pub kind: String,
    pub severity: Severity,     // Info|Low|Medium|High|Critical
    pub title: String,
    pub impact: String,
    pub solution: String,
}
```

## 4. Trait central de otimização

```rust
pub trait Optimization: Send + Sync {
    fn id(&self) -> &'static str;
    fn category(&self) -> Category;
    fn risk_level(&self) -> RiskLevel;       // Safe | Moderate | Advanced
    fn requires_elevation(&self) -> bool;

    /// Descreve o que mudará, sem aplicar.
    fn preview(&self, ctx: &SystemCtx) -> Result<Preview>;

    /// Captura estado atual para rollback.
    fn capture(&self, ctx: &SystemCtx) -> Result<Vec<SnapshotEntry>>;

    /// Aplica a mudança.
    fn apply(&self, ctx: &mut SystemCtx) -> Result<ApplyOutcome>;

    /// Verifica pós-condições.
    fn verify(&self, ctx: &SystemCtx) -> Result<bool>;

    /// Reverte usando o snapshot.
    fn revert(&self, ctx: &mut SystemCtx, entries: &[SnapshotEntry]) -> Result<()>;
}
```

## 5. Saga de otimização (pseudo-Rust)

```rust
pub async fn run_plan(plan: Plan, ctx: &mut SystemCtx) -> Result<PlanResult> {
    if plan.requires_elevation() { permission_broker.ensure_elevated()?; }

    let snap = snapshot_repo.create(plan.reason())?;
    audit.log("optimize.intent", &plan);

    let mut applied: Vec<&dyn Optimization> = vec![];
    for op in plan.ops() {
        let entries = op.capture(ctx)?;
        snapshot_repo.add_entries(snap.id, &entries)?;
        match op.apply(ctx).and_then(|_| op.verify(ctx)) {
            Ok(true)  => applied.push(op),
            _ => {
                // compensação: reverte em ordem inversa
                for done in applied.iter().rev() { done.revert(ctx, &snap.entries_of(done.id()))?; }
                snapshot_repo.mark_restored(snap.id);
                audit.log("optimize.rolledback", &op.id());
                return Err(Error::Compensated);
            }
        }
    }
    snapshot_repo.activate(snap.id);
    audit.log("optimize.committed", &plan);
    Ok(PlanResult::committed(snap.id))
}
```

## 6. Bridge Tauri (exemplo)

```rust
#[tauri::command]
async fn analyze_full(state: tauri::State<'_, App>) -> Result<Diagnosis, AppError> {
    state.analyzer.run_full().await.map_err(Into::into)
}

#[tauri::command]
async fn optimize_apply(plan: Plan, state: tauri::State<'_, App>) -> Result<PlanResult, AppError> {
    state.optimizer.run_plan(plan).await.map_err(Into::into)
}
```

Telemetria via canal:
```rust
#[tauri::command]
async fn subscribe_metrics(channel: tauri::ipc::Channel<MetricSample>, state: State<'_, App>) {
    let mut rx = state.bus.subscribe();
    tokio::spawn(async move { while let Ok(s) = rx.recv().await { let _ = channel.send(s); } });
}
```

## 7. Estratégia de testes

- **Domínio:** unit tests puros (score, baseline, regras de gargalo) — sem SO.
- **Adapters:** testes de integração com mocks/feature-gated; coleta real validada em CI Windows.
- **Sagas/rollback:** testes em **VM Windows descartável** (snapshot → apply → revert → asserção de estado), idempotência do revert.
- **Frontend:** Vitest + Testing Library; Playwright p/ e2e do ciclo.
- **Health budget:** teste que falha o build se idle CPU/RAM excede orçamento.

## 8. Build, assinatura e distribuição

- CI (GitHub Actions, runner Windows): `cargo build --release`, `pnpm build`, `tauri build`.
- Assinatura Authenticode dos binários + assinatura do updater.
- Instalador MSI (WiX) e NSIS; publicação no site + Microsoft Store.
- `cargo audit`/`cargo deny`/`pnpm audit` no pipeline; SBOM por release.

## 9. Logs

- `tracing` → arquivos rotacionados em `%APPDATA%\TkSpeed\logs\` (JSON estruturado).
- Níveis por módulo; redaction de dados sensíveis.
- Crash handler grava minidump (opt-in para envio).

## 10. Orçamento de performance da ferramenta

| Métrica | Alvo |
|---|---|
| CPU idle (monitorando) | < 1% |
| RAM residente | < 150 MB |
| Disco (DB) | crescimento < 50 MB/mês com retenção padrão |
| Tempo de cold start | < 1.5 s até dashboard interativo |
