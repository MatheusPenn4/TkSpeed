//! Bridge Tauri — comandos tipados expostos ao frontend.
//! Os payloads vêm de `tk-contracts` (fonte única de tipos). Validação e
//! tratamento de erro acontecem neste boundary.

use std::sync::OnceLock;

use serde::Serialize;
use tauri::State;
use tk_analyzer::{AnalysisInput, AnalyzerEngine};
use tk_contracts::{
    BenchmarkSessionInfo, BottleneckReport, Diagnosis, HardwareInfo, HardwareSnapshot, NoiseProfile,
    OptimizationInfo, OptimizationRunInfo, PerfComparison, ProtectionState, RollbackOutcome,
    ScoreHistoryItem, SelfTestReport, SnapshotInfo, StartupItem,
};
use tk_core::AppContext;
use tk_monitor::SysinfoSampler;

/// Erro serializável para o frontend. NUNCA carrega SQL, paths, stack traces ou
/// exceções técnicas — apenas `code` (programático) e `message` (amigável).
/// O detalhe técnico vai somente para o log.
#[derive(Debug, Serialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
}

// Erros de infraestrutura → mensagem genérica ao usuário, detalhe só no log.
impl From<tk_storage::StorageError> for AppError {
    fn from(e: tk_storage::StorageError) -> Self {
        tracing::error!("storage error: {e}");
        AppError {
            code: "storage".into(),
            message: "Falha ao acessar os dados locais. Tente novamente.".into(),
        }
    }
}

impl From<tk_rollback::RollbackError> for AppError {
    fn from(e: tk_rollback::RollbackError) -> Self {
        use tk_rollback::RollbackError as R;
        // Variantes de domínio têm texto próprio seguro; infra vira mensagem genérica.
        let (code, message) = match &e {
            R::NotFound(_) => ("rollback_not_found", "Snapshot não encontrado."),
            R::Integrity => (
                "rollback_integrity",
                "Integridade do snapshot inválida — rollback cancelado por segurança.",
            ),
            R::Validation(_) => (
                "rollback_validation",
                "Não foi possível confirmar a operação. Nenhuma alteração permanente foi feita.",
            ),
            _ => ("rollback", "Falha na operação de proteção. Tente novamente."),
        };
        tracing::error!("rollback error: {e}");
        AppError {
            code: code.into(),
            message: message.into(),
        }
    }
}

type Cmd<T> = Result<T, AppError>;

// ── Status de bootstrap (A3): permite à UI detectar falha de inicialização do
// serviço local mesmo quando o AppContext não foi registrado (sem State). ──
static BOOT_ERROR: OnceLock<String> = OnceLock::new();

/// Registra (uma vez) a mensagem de falha de bootstrap.
pub fn set_boot_error(msg: String) {
    let _ = BOOT_ERROR.set(msg);
}

/// Retorna a mensagem de falha de bootstrap, se houve. Não depende de State.
#[tauri::command]
pub fn bootstrap_status() -> Option<String> {
    BOOT_ERROR.get().cloned()
}

/// Detecção de hardware sob demanda (fallback do evento `hardware:info`).
#[tauri::command]
pub async fn get_hardware(state: State<'_, AppContext>) -> Cmd<HardwareInfo> {
    if let Some(hw) = state.hardware() {
        return Ok(hw);
    }
    // Fallback: detecta agora se o monitor ainda não populou.
    let mut sampler = SysinfoSampler::new();
    Ok(sampler.detect_hardware())
}

/// Análise completa: usa a janela real de telemetria, gera findings + score,
/// persiste no SQLite (run + findings + score) e retorna o diagnóstico.
#[tauri::command]
pub async fn analyze_full(state: State<'_, AppContext>) -> Cmd<Diagnosis> {
    // AppContext é Clone (Arc internamente) → owned, sem borrow através de await.
    let ctx = state.inner().clone();

    // Janela de telemetria; se vazia (logo após abrir), faz uma leitura imediata.
    let mut window = ctx.recent_window();
    if window.is_empty() {
        let mut sampler = SysinfoSampler::new();
        window.push(sampler.read());
    }

    let input = AnalysisInput::from_window(&window);

    let run_id = ctx.analysis().start_run("manual").await?;
    let diagnosis = AnalyzerEngine::new().run_full(run_id, &input);

    // Persistência (histórico).
    for f in &diagnosis.findings {
        ctx.analysis().add_finding(run_id, f).await?;
    }
    ctx.analysis().save_score(run_id, &diagnosis.score).await?;
    let summary = serde_json::json!({
        "findings": diagnosis.findings.len(),
        "score": diagnosis.score.total,
        "samples": input.samples,
    })
    .to_string();
    ctx.analysis().finish_run(run_id, &summary).await?;
    let _ = ctx.audit().log("user", "analyze.completed", &summary).await;

    Ok(diagnosis)
}

/// Histórico de análises (scores passados), mais recentes primeiro.
#[tauri::command]
pub async fn get_history(state: State<'_, AppContext>, limit: Option<i64>) -> Cmd<Vec<ScoreHistoryItem>> {
    let ctx = state.inner().clone();
    let items = ctx.analysis().score_history(limit.unwrap_or(50)).await?;
    Ok(items)
}

// ───────────────────────── Proteção (Snapshots / Rollback) ─────────────────────────

/// Estado de proteção para o Dashboard (último snapshot, último rollback, status).
#[tauri::command]
pub async fn protection_state(state: State<'_, AppContext>) -> Cmd<ProtectionState> {
    let ctx = state.inner().clone();
    Ok(ctx.protection().protection_state().await?)
}

/// Lista snapshots (mais recentes primeiro).
#[tauri::command]
pub async fn protection_list(state: State<'_, AppContext>, limit: Option<i64>) -> Cmd<Vec<SnapshotInfo>> {
    let ctx = state.inner().clone();
    Ok(ctx.protection().list(limit.unwrap_or(50)).await?)
}

/// Operação-piloto: cria snapshot, aplica a alteração segura e valida.
#[tauri::command]
pub async fn protection_apply_demo(state: State<'_, AppContext>) -> Cmd<SnapshotInfo> {
    let ctx = state.inner().clone();
    Ok(ctx.protection().apply_demo().await?)
}

/// Executa o rollback de um snapshot e verifica a restauração.
#[tauri::command]
pub async fn protection_rollback(state: State<'_, AppContext>, snapshot_id: i64) -> Cmd<RollbackOutcome> {
    let ctx = state.inner().clone();
    Ok(ctx.protection().rollback(snapshot_id).await?)
}

/// Autoteste completo: snapshot → aplicação → verificação → rollback → verificação.
#[tauri::command]
pub async fn protection_selftest(state: State<'_, AppContext>) -> Cmd<SelfTestReport> {
    let ctx = state.inner().clone();
    Ok(ctx.protection().selftest().await?)
}

// ───────────────────────── Performance Lab ─────────────────────────

/// Executa um benchmark (`kind`: "cpu" | "ram" | "io" | "complete") em thread
/// bloqueante e persiste a sessão. Retorna a sessão salva (id + métricas).
#[tauri::command]
pub async fn perf_run_benchmark(
    state: State<'_, AppContext>,
    kind: String,
    label: String,
    runs: Option<u32>,
) -> Cmd<BenchmarkSessionInfo> {
    let ctx = state.inner().clone();
    let n = runs.unwrap_or(5).clamp(3, 10);
    let kind_for_run = kind.clone();

    let result = tokio::task::spawn_blocking(move || tk_perflab::run_benchmark(&kind_for_run, n))
        .await
        .map_err(|e| {
            tracing::error!("benchmark task falhou: {e}");
            AppError { code: "perf".into(), message: "Falha ao executar o benchmark.".into() }
        })?;

    let id = ctx.perf().save_session(&label, &result).await?;
    tracing::info!("benchmark salvo (session={id}, kind={kind}, runs={n}, label={label})");
    Ok(session_info(id, &label, result))
}

/// Monta a `BenchmarkSessionInfo` retornada à UI a partir de um resultado salvo.
fn session_info(id: i64, label: &str, r: tk_contracts::BenchmarkResult) -> BenchmarkSessionInfo {
    BenchmarkSessionInfo {
        id,
        ts: tk_storage::now_ms(),
        kind: r.kind,
        suite_version: r.suite_version,
        label: label.into(),
        metrics: r.metrics,
        confidence: r.confidence,
        stable: r.stable,
        contaminated: r.contaminated,
    }
}

/// Fotografia ao vivo do hardware (GPU/temperaturas/RAM) para o dashboard.
#[tauri::command]
pub async fn perf_hardware_snapshot() -> Cmd<HardwareSnapshot> {
    // NVML + sysinfo são síncronos/curtos → thread bloqueante para não prender o runtime.
    let snap = tokio::task::spawn_blocking(tk_perflab::hardware_snapshot)
        .await
        .map_err(|e| {
            tracing::error!("hardware snapshot falhou: {e}");
            AppError { code: "perf".into(), message: "Falha ao ler o hardware.".into() }
        })?;
    Ok(snap)
}

// ───────────────────────── Optimization Engine (Fase 2) ─────────────────────────

/// Catálogo de otimizações disponíveis.
#[tauri::command]
pub async fn opt_catalog(state: State<'_, AppContext>) -> Cmd<Vec<OptimizationInfo>> {
    Ok(state.inner().clone().optimize().catalog_info())
}

/// Executa o loop completo (snapshot→bench→aplicar→bench→Confidence→decidir→manter/reverter).
#[tauri::command]
pub async fn opt_run(state: State<'_, AppContext>, id: String) -> Cmd<OptimizationRunInfo> {
    let ctx = state.inner().clone();
    ctx.optimize()
        .run(&id)
        .await
        .map_err(|m| AppError { code: "optimize".into(), message: m })
}

/// Histórico de execuções (com evidência).
#[tauri::command]
pub async fn opt_history(state: State<'_, AppContext>) -> Cmd<Vec<OptimizationRunInfo>> {
    let ctx = state.inner().clone();
    ctx.optimize()
        .history(50)
        .await
        .map_err(|m| AppError { code: "optimize".into(), message: m })
}

/// Reverte manualmente uma execução mantida.
#[tauri::command]
pub async fn opt_rollback(state: State<'_, AppContext>, run_id: i64) -> Cmd<()> {
    let ctx = state.inner().clone();
    ctx.optimize()
        .rollback_run(run_id)
        .await
        .map_err(|m| AppError { code: "optimize".into(), message: m })
}

/// Análise de inicialização (somente leitura).
#[tauri::command]
pub async fn opt_startup_analysis(state: State<'_, AppContext>) -> Cmd<Vec<StartupItem>> {
    Ok(state.inner().clone().optimize().startup_items())
}

/// Detecta o gargalo atual amostrando ~2s (CPU/GPU/RAM).
#[tauri::command]
pub async fn perf_detect_bottleneck() -> Cmd<BottleneckReport> {
    let report = tokio::task::spawn_blocking(tk_perflab::detect_bottleneck)
        .await
        .map_err(|e| {
            tracing::error!("detect bottleneck falhou: {e}");
            AppError { code: "perf".into(), message: "Falha ao detectar gargalo.".into() }
        })?;
    Ok(report)
}

/// Lista as sessões de benchmark (mais recentes primeiro).
#[tauri::command]
pub async fn perf_list_sessions(state: State<'_, AppContext>) -> Cmd<Vec<BenchmarkSessionInfo>> {
    let ctx = state.inner().clone();
    Ok(ctx.perf().list_sessions(50).await?)
}

/// Compara duas sessões (antes vs depois) com margem de erro.
#[tauri::command]
pub async fn perf_compare(
    state: State<'_, AppContext>,
    before_id: i64,
    after_id: i64,
) -> Cmd<PerfComparison> {
    let ctx = state.inner().clone();

    let before = ctx.perf().get_result(before_id).await?.ok_or(AppError {
        code: "perf".into(),
        message: "Sessão 'antes' não encontrada.".into(),
    })?;
    let after = ctx.perf().get_result(after_id).await?.ok_or(AppError {
        code: "perf".into(),
        message: "Sessão 'depois' não encontrada.".into(),
    })?;

    // Guard de comparabilidade (mesma versão da suite).
    if before.suite_version != after.suite_version {
        return Err(AppError {
            code: "perf_incomparable".into(),
            message: "Sessões de versões diferentes da suite não são comparáveis.".into(),
        });
    }

    // Margem dinâmica: aprende o ruído da máquina a partir do histórico da suite.
    let history = ctx.perf().metrics_by_suite(&before.suite_version).await?;
    let noise = tk_perflab::build_noise_profile(&before.suite_version, &history);

    let mut comp = tk_perflab::compare(&before, &after, &noise);
    comp.before_id = before_id;
    comp.after_id = after_id;
    let _ = ctx.perf().save_comparison(before_id, after_id, &comp).await;
    Ok(comp)
}

/// Perfil de ruído da máquina para uma suite (demonstração do noise floor).
#[tauri::command]
pub async fn perf_noise_floor(state: State<'_, AppContext>, suite: String) -> Cmd<NoiseProfile> {
    let ctx = state.inner().clone();
    let history = ctx.perf().metrics_by_suite(&suite).await?;
    Ok(tk_perflab::build_noise_profile(&suite, &history))
}

/// Captura de FPS/frametime de um jogo via PresentMon (real). Requer o
/// `PresentMon*.exe` em %APPDATA%\TkSpeed\tools e app elevado (admin).
#[tauri::command]
pub async fn perf_capture_fps(
    state: State<'_, AppContext>,
    target: String,
    duration_secs: Option<u64>,
) -> Cmd<BenchmarkSessionInfo> {
    let ctx = state.inner().clone();
    let dur = duration_secs.unwrap_or(30).clamp(5, 300) * 1000;
    let label = format!("FPS · {target}");
    let target_c = target.clone();

    let result = tokio::task::spawn_blocking(move || {
        let src = tk_perflab::PresentMonFrameSource::locate().ok_or_else(|| {
            "PresentMon não encontrado. Baixe o PresentMon-x64.exe e coloque em \
             %APPDATA%\\TkSpeed\\tools\\."
                .to_string()
        })?;
        tk_perflab::run_fps_capture(&src, &target_c, dur)
    })
    .await
    .map_err(|e| AppError { code: "perf".into(), message: format!("Tarefa de captura falhou: {e}") })?
    .map_err(|m| AppError { code: "perf_capture".into(), message: m })?;

    let id = ctx.perf().save_session(&label, &result).await?;
    Ok(session_info(id, &label, result))
}

/// Demonstra o pipeline de FPS com um trace SINTÉTICO (não é medição real).
/// Serve para validar 1%/0.1% low + confiança sem depender do PresentMon.
#[tauri::command]
pub async fn perf_capture_fps_demo(state: State<'_, AppContext>) -> Cmd<BenchmarkSessionInfo> {
    let ctx = state.inner().clone();
    let label = "FPS · DEMO (sintético)".to_string();
    let result = tokio::task::spawn_blocking(|| {
        let src = tk_perflab::ReplayFrameSource { frametimes: tk_perflab::demo_trace() };
        tk_perflab::run_fps_capture(&src, "demo", 60_000)
    })
    .await
    .map_err(|e| AppError { code: "perf".into(), message: format!("Tarefa falhou: {e}") })?
    .map_err(|m| AppError { code: "perf_capture".into(), message: m })?;

    let id = ctx.perf().save_session(&label, &result).await?;
    Ok(session_info(id, &label, result))
}
