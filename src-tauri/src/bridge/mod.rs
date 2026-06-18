//! Bridge Tauri — comandos tipados expostos ao frontend.
//! Os payloads vêm de `tk-contracts` (fonte única de tipos). Validação e
//! tratamento de erro acontecem neste boundary.

use std::sync::OnceLock;

use serde::Serialize;
use sysinfo;
use tauri::State;
use tk_analyzer::{AnalysisInput, AnalyzerEngine};
use tk_contracts::{
    BenchmarkSessionInfo, BottleneckReport, Diagnosis, HardwareInfo, HardwareSnapshot, NoiseProfile,
    OptimizationInfo, OptimizationRunInfo, PerfComparison, ProtectionState, RollbackOutcome,
    ScoreHistoryItem, SelfTestReport, SnapshotInfo, StartupItem,
};
use tk_core::AppContext;
use tk_monitor::SysinfoSampler;
use tk_optimize::{
    profiles::{executor::{build_action, ConfigAction}, evidence::ProfileEvidenceRepo, repo::ProfileRepo},
    recommendations::{Recommendation, RecommendationContext, RecommendationEngine},
    MeasurementPipeline, ProfileMeasureResult,
};
use tk_storage::session_source;

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
    // Usa apenas os últimos 30 ticks (~30 s) para análise consistente entre reanálises.
    // Sem esse limite, ticks ociosos acumulados inflam o score a cada clique.
    if window.len() > 30 {
        window.drain(..window.len() - 30);
    }
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

/// Cria um ponto de restauração manual (linha de base reversível). UX-008.
#[tauri::command]
pub async fn restore_point_create(state: State<'_, AppContext>) -> Cmd<SnapshotInfo> {
    let ctx = state.inner().clone();
    Ok(ctx.protection().create_manual_point().await?)
}

/// Exclui um ponto de restauração. UX-008.
#[tauri::command]
pub async fn restore_point_delete(state: State<'_, AppContext>, snapshot_id: i64) -> Cmd<()> {
    let ctx = state.inner().clone();
    ctx.protection().delete(snapshot_id).await?;
    Ok(())
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

    let fp = ctx.machine_fingerprint();
    let id = ctx.perf().save_session(&label, &result, Some(&fp), session_source::MANUAL).await?;
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

/// Desabilita um item de inicialização (HKCU) de forma reversível: cria um
/// snapshot e remove o valor. Reverter = restaurar o snapshot no Rollback Center.
/// Retorna o id do snapshot criado.
#[tauri::command]
pub async fn opt_disable_startup(state: State<'_, AppContext>, name: String) -> Cmd<i64> {
    let ctx = state.inner().clone();
    ctx.optimize()
        .disable_startup(&name)
        .await
        .map_err(|m| AppError { code: "optimize".into(), message: m })
}

/// Estado REAL dos módulos do TkSpeed. Estrutura aberta — novas capacidades
/// em V4.3/V4.4 não requerem mudança neste comando.
#[tauri::command]
pub async fn system_capabilities() -> Cmd<Vec<tk_optimize::capabilities::Capability>> {
    let caps = tokio::task::spawn_blocking(tk_optimize::capabilities::build)
        .await
        .map_err(|e| AppError { code: "capabilities".into(), message: format!("falha ao ler capacidades: {e}") })?;
    Ok(caps)
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
        let src = tk_perflab::PresentMonFrameSource::locate()
            .ok_or_else(|| "PresentMon não disponível nesta instalação.".to_string())?;
        tk_perflab::run_fps_capture(&src, &target_c, dur)
    })
    .await
    .map_err(|e| AppError { code: "perf".into(), message: format!("Tarefa de captura falhou: {e}") })?
    .map_err(|m| AppError { code: "perf_capture".into(), message: m })?;

    let fp = ctx.machine_fingerprint();
    let id = ctx.perf().save_session(&label, &result, Some(&fp), session_source::MANUAL).await?;
    Ok(session_info(id, &label, result))
}

/// Retorna as top-5 recomendações do Recommendation Engine para esta máquina.
/// Usa evidência real, capacidades detectadas e histórico de benchmarks.
/// Sem heurísticas inventadas — só dados reais.
#[tauri::command]
pub async fn advisor_recommendations(state: State<'_, AppContext>) -> Cmd<Vec<Recommendation>> {
    let ctx = state.inner().clone();
    let fp = ctx.machine_fingerprint();

    // Capacidades do sistema detectadas em runtime.
    let capabilities = tk_optimize::capabilities::build();

    // Sessões de benchmark recentes (até 10) para contexto.
    let recent_sessions = ctx.perf().list_sessions(10).await.unwrap_or_default();

    // Perfil atualmente ativo.
    let active_profile = ProfileRepo::new(ctx.db.clone())
        .get_state("default")
        .await
        .ok()
        .and_then(|s| s.profile_id);

    // Evidência de config acumulada nesta máquina.
    let config_evidence = tk_optimize::EvidenceRepo::new(ctx.db.clone())
        .evidence_for_fingerprint(&fp)
        .await
        .unwrap_or_default();

    // Evidência de perfil acumulada nesta máquina.
    let profile_evidence = ProfileEvidenceRepo::new(ctx.db.clone())
        .list_by_fingerprint(&fp)
        .await
        .unwrap_or_default();

    // Configs executáveis nesta máquina (pode chamar powercfg — usa spawn_blocking).
    let applicable_config_ids: Vec<String> = tokio::task::spawn_blocking(|| {
        tk_optimize::ConfigRegistry::new()
            .all()
            .iter()
            .filter(|c| matches!(build_action(c.meta().id), Ok(ConfigAction::Executable(_))))
            .map(|c| c.meta().id.to_string())
            .collect()
    })
    .await
    .map_err(|e| AppError { code: "advisor".into(), message: format!("Falha ao verificar configs: {e}") })?;

    let rec_ctx = RecommendationContext {
        machine_fingerprint: fp,
        capabilities,
        recent_sessions,
        active_profile,
        active_config_ids: vec![], // expandir quando pipeline de config individual estiver pronto
        config_evidence,
        profile_evidence,
        applicable_config_ids,
    };

    Ok(RecommendationEngine::top_n(&rec_ctx, 5))
}

/// Aplica um perfil via MeasurementPipeline completo (before bench → ativar → after bench → evidência).
/// Para recomendações de perfil. Operação longa — frontend deve mostrar estado de carregamento.
#[tauri::command]
pub async fn advisor_apply_profile(
    state: State<'_, AppContext>,
    profile_id: String,
) -> Cmd<ProfileMeasureResult> {
    let ctx = state.inner().clone();
    MeasurementPipeline::new(ctx.db.clone())
        .activate_with_measure(&profile_id)
        .await
        .map_err(|e| {
            tracing::error!("advisor_apply_profile falhou: {e}");
            AppError { code: "advisor_apply".into(), message: e }
        })
}

// ───────────────────────── V4.5-F: Game Process Booster ─────────────────────────

/// Jogo detectado em execução.
#[derive(Debug, Serialize)]
pub struct DetectedGame {
    pub pid: u32,
    pub name: String,
    pub exe: String,
}

/// Resultado da elevação de prioridade.
#[derive(Debug, Serialize)]
pub struct BoostResult {
    pub pid: u32,
    pub success: bool,
    pub message: String,
}

/// Resultado da limpeza de standby memory.
#[derive(Debug, Serialize)]
pub struct RamFlushResult {
    pub freed_mb: f64,
    pub before_mb: f64,
    pub after_mb: f64,
    pub success: bool,
    pub message: String,
}

/// Nomes de executáveis de jogos conhecidos (minúsculos, sem .exe).
const KNOWN_GAMES: &[&str] = &[
    "valorant",
    "csgo",
    "cs2",
    "fortniteclient",
    "cod",
    "modernwarfare",
    "warzone",
    "r5apex",
    "league",       // LeagueClient.exe, League of Legends.exe
    "riotclient",
    "overwatch",
    "overwatch2",
    "eldenring",
    "cyberpunk2077",
    "rdr2",
    "gta5",         // era "gtav" — não batia com GTA5.exe
    "minecraft",
    "javaw",
    "dota2",
    "tslgame",      // PUBG
    "rocketleague",
    "rainbowsix",
    "deadbydaylight",
    "destiny2",
    "steam_api",    // jogos genéricos Steam que usam overlay
];

/// Detecta jogos conhecidos atualmente em execução.
#[tauri::command]
pub async fn detect_games() -> Cmd<Vec<DetectedGame>> {
    tokio::task::spawn_blocking(|| {
        use sysinfo::System;
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
        let mut games = Vec::new();
        for (pid, proc) in sys.processes() {
            let exe_name = proc
                .exe()
                .and_then(|p| p.file_stem())
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            // proc.name() sempre disponível; proc.exe() pode ser None em processos protegidos
            let proc_name = proc.name().to_string_lossy().to_lowercase();
            let combined = format!("{exe_name} {proc_name}");
            if KNOWN_GAMES.iter().any(|g| combined.contains(g)) {
                games.push(DetectedGame {
                    pid: pid.as_u32(),
                    name: proc.name().to_string_lossy().to_string(),
                    exe: proc.exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
                });
            }
        }
        Ok(games)
    })
    .await
    .map_err(|e| AppError { code: "detect_games".into(), message: format!("Falha ao listar processos: {e}") })?
}

/// Eleva a prioridade de um processo de jogo para Above Normal.
/// Não usa Realtime (proibido).
#[tauri::command]
pub async fn boost_game(pid: u32) -> Cmd<BoostResult> {
    tokio::task::spawn_blocking(move || {
        match tk_platform_win::process_win::set_above_normal_priority(pid) {
            Ok(()) => Ok(BoostResult { pid, success: true, message: "Prioridade elevada para Above Normal.".into() }),
            Err(e) => Ok(BoostResult { pid, success: false, message: e }),
        }
    })
    .await
    .map_err(|e| AppError { code: "boost_game".into(), message: format!("Falha ao elevar prioridade: {e}") })?
}

/// Restaura a prioridade de um processo para Normal.
#[tauri::command]
pub async fn restore_game_priority(pid: u32) -> Cmd<BoostResult> {
    tokio::task::spawn_blocking(move || {
        match tk_platform_win::process_win::set_normal_priority(pid) {
            Ok(()) => Ok(BoostResult { pid, success: true, message: "Prioridade restaurada para Normal.".into() }),
            Err(e) => Ok(BoostResult { pid, success: false, message: e }),
        }
    })
    .await
    .map_err(|e| AppError { code: "restore_priority".into(), message: format!("Falha ao restaurar prioridade: {e}") })?
}

/// Libera a standby list de RAM. Requer administrador.
/// Retorna MB livres antes/depois e MB liberados.
#[tauri::command]
pub async fn ram_flush_standby() -> Cmd<RamFlushResult> {
    if !tk_platform_win::elevation::is_elevated() {
        return Err(AppError {
            code: "elevation_required".into(),
            message: "Esta operação requer privilégios de administrador.".into(),
        });
    }
    tokio::task::spawn_blocking(|| {
        let before = tk_platform_win::memory::available_bytes();
        match tk_platform_win::memory::flush_standby() {
            Ok(freed) => {
                let after = tk_platform_win::memory::available_bytes();
                Ok(RamFlushResult {
                    freed_mb: freed as f64 / (1024.0 * 1024.0),
                    before_mb: before as f64 / (1024.0 * 1024.0),
                    after_mb: after as f64 / (1024.0 * 1024.0),
                    success: true,
                    message: format!("{:.0} MB liberados da standby list.", freed as f64 / (1024.0 * 1024.0)),
                })
            }
            Err(e) => Ok(RamFlushResult {
                freed_mb: 0.0,
                before_mb: before as f64 / (1024.0 * 1024.0),
                after_mb: before as f64 / (1024.0 * 1024.0),
                success: false,
                message: e,
            }),
        }
    })
    .await
    .map_err(|e| AppError { code: "ram_flush".into(), message: format!("Falha ao liberar standby: {e}") })?
}

// ───────────────────────── V4.6-A/B: GPU Detection ─────────────────────────

/// GPU detectada na máquina.
#[derive(Debug, Serialize)]
pub struct GpuDetectResult {
    pub name: String,
    pub vendor: String,
}

/// Detecta GPUs instaladas via chaves de classe de dispositivo do registro.
/// Leitura pura, sem alteração — não requer elevação.
#[tauri::command]
pub async fn gpu_detect() -> Cmd<Vec<GpuDetectResult>> {
    tokio::task::spawn_blocking(|| {
        let gpus = tk_platform_win::gpu::detect();
        Ok(gpus.into_iter().map(|g| GpuDetectResult { name: g.name, vendor: g.vendor }).collect())
    })
    .await
    .map_err(|e| AppError { code: "gpu_detect".into(), message: format!("Falha ao detectar GPU: {e}") })?
}

/// Resultado da limpeza de cache DNS.
#[derive(Debug, Serialize)]
pub struct DnsFlushResult {
    pub success: bool,
    pub message: String,
}

/// Limpa o cache DNS do sistema via `ipconfig /flushdns`.
/// One-shot, sem rollback (operação benigna e idempotente).
#[tauri::command]
pub async fn dns_flush() -> Cmd<DnsFlushResult> {
    tokio::task::spawn_blocking(|| {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            match std::process::Command::new("ipconfig")
                .arg("/flushdns")
                .creation_flags(0x08000000)
                .output()
            {
                Ok(out) if out.status.success() => Ok(DnsFlushResult {
                    success: true,
                    message: "Cache DNS limpo com sucesso. Reconexões de rede podem ser ligeiramente mais lentas por alguns segundos.".into(),
                }),
                Ok(_) => Ok(DnsFlushResult {
                    success: false,
                    message: "ipconfig /flushdns retornou erro.".into(),
                }),
                Err(e) => Ok(DnsFlushResult {
                    success: false,
                    message: format!("Falha ao executar ipconfig: {e}"),
                }),
            }
        }
        #[cfg(not(windows))]
        Ok(DnsFlushResult { success: false, message: "Indisponível fora do Windows.".into() })
    })
    .await
    .map_err(|e| AppError { code: "dns_flush".into(), message: format!("Falha: {e}") })?
}

// ───────────────────────── V4.7-F2: Perceived Performance (read-only) ─────────────────────────

/// App de overhead detectado em execução (overlay, peripheral manager, etc.).
/// Somente leitura — não altera processos.
#[derive(Debug, Serialize)]
pub struct HeavyAppDetected {
    pub name: String,
    pub exe: String,
    pub impact: String,       // "baixo" | "medio" | "alto"
    pub description: String,
}

struct HeavyAppDef {
    exe_fragment: &'static str,
    name: &'static str,
    impact: &'static str,
    description: &'static str,
}

const HEAVY_APP_DEFS: &[HeavyAppDef] = &[
    HeavyAppDef { exe_fragment: "discord", name: "Discord", impact: "alto",
        description: "Overlay e cliente de voz. Injeta hooks em jogos e consome ~150 MB de RAM." },
    HeavyAppDef { exe_fragment: "gameoverlayui", name: "Steam Overlay", impact: "medio",
        description: "Overlay da Steam injetado em jogos. Pode causar stutters em títulos sensíveis." },
    HeavyAppDef { exe_fragment: "nvidia share", name: "GeForce Overlay", impact: "medio",
        description: "NVIDIA Share / ShadowPlay. Captura e overlay da GeForce Experience em segundo plano." },
    HeavyAppDef { exe_fragment: "nvdisplay.container", name: "NVIDIA Container", impact: "baixo",
        description: "Serviços de driver NVIDIA. Necessário para GPU funcionar, mas pode ser atualizado." },
    HeavyAppDef { exe_fragment: "gamebar", name: "Xbox Game Bar", impact: "medio",
        description: "Overlay do Windows com captura de tela e FPS counter. Consume CPU quando ativo." },
    HeavyAppDef { exe_fragment: "gamebarftserver", name: "Xbox Game Bar (FT Server)", impact: "medio",
        description: "Componente do Xbox Game Bar. Ativo mesmo com overlay fechado." },
    HeavyAppDef { exe_fragment: "overwolf", name: "Overwolf", impact: "alto",
        description: "Plataforma de overlay e apps de jogos. Injeta hooks em processos — alto consumo de CPU/RAM." },
    HeavyAppDef { exe_fragment: "curseforge", name: "CurseForge", impact: "baixo",
        description: "Gerenciador de mods. Mantém serviço de sync ativo em segundo plano." },
    HeavyAppDef { exe_fragment: "msicenter", name: "MSI Center", impact: "baixo",
        description: "Software de controle MSI (RGB, ventoinhas). Consome CPU em polling contínuo." },
    HeavyAppDef { exe_fragment: "armourycrate", name: "Armoury Crate (ASUS)", impact: "baixo",
        description: "Software ASUS para controle de RGB e overclock. Múltiplos processos em segundo plano." },
    HeavyAppDef { exe_fragment: "razer synapse", name: "Razer Synapse", impact: "baixo",
        description: "Software Razer para periféricos. Mantém múltiplos serviços ativos em segundo plano." },
    HeavyAppDef { exe_fragment: "rzsdkserver", name: "Razer SDK Server", impact: "baixo",
        description: "Componente do Razer Synapse para integração com jogos." },
    HeavyAppDef { exe_fragment: "lghub", name: "Logitech G HUB", impact: "baixo",
        description: "Software de periféricos Logitech. Usa Electron — ~150 MB de RAM e polling de USB." },
    HeavyAppDef { exe_fragment: "icue", name: "Corsair iCUE", impact: "baixo",
        description: "Software Corsair para RGB e ventoinhas. Alto uso de CPU durante atualizações de efeito." },
];

/// Detecta processos de overhead conhecidos (overlays, managers de periféricos) em execução.
/// Análise somente leitura — não altera nenhum processo.
#[tauri::command]
pub async fn detect_heavy_apps() -> Cmd<Vec<HeavyAppDetected>> {
    tokio::task::spawn_blocking(|| {
        use sysinfo::System;
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

        let mut found: Vec<HeavyAppDetected> = Vec::new();
        let mut seen_defs: std::collections::HashSet<usize> = std::collections::HashSet::new();

        for (_pid, proc) in sys.processes() {
            let exe_lower = proc
                .exe()
                .and_then(|p| p.file_stem())
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            let name_lower = proc.name().to_string_lossy().to_lowercase();
            let combined = format!("{exe_lower} {name_lower}");

            for (idx, def) in HEAVY_APP_DEFS.iter().enumerate() {
                if seen_defs.contains(&idx) { continue; }
                if combined.contains(def.exe_fragment) {
                    seen_defs.insert(idx);
                    found.push(HeavyAppDetected {
                        name: def.name.to_string(),
                        exe: exe_lower.clone(),
                        impact: def.impact.to_string(),
                        description: def.description.to_string(),
                    });
                }
            }
        }

        // Ordena por impacto: alto → medio → baixo
        found.sort_by_key(|a| match a.impact.as_str() {
            "alto" => 0u8,
            "medio" => 1,
            _ => 2,
        });

        Ok(found)
    })
    .await
    .map_err(|e| AppError { code: "detect_heavy_apps".into(), message: format!("Falha ao listar processos: {e}") })?
}

// ───────────────────────── V5.0-A: Game Center ─────────────────────────

/// Jogo instalado retornado ao frontend.
#[derive(Debug, Serialize)]
pub struct InstalledGameInfo {
    pub id: String,
    pub name: String,
    pub exe: String,
    pub launcher: String,
    pub install_path: String,
}

/// Atribuição de perfil por jogo.
#[derive(Debug, Serialize)]
pub struct GameAssignment {
    pub db_id: i64,
    pub exe_match: String,
    pub game_name: String,
    pub profile_id: String,
}

/// Entrada de histórico de aplicação de perfil para um jogo.
#[derive(Debug, Serialize)]
pub struct GameRun {
    pub id: i64,
    pub ts: i64,
    pub exe_match: String,
    pub game_name: String,
    pub profile_id: String,
    pub source: String,
}

fn db_err(e: sqlx::Error) -> AppError {
    tracing::error!("game_center db: {e}");
    AppError { code: "db".into(), message: "Erro ao acessar dados locais.".into() }
}

/// Detecta jogos instalados via Steam, Epic, Riot, Battle.net, EA, Ubisoft, GOG e Rockstar.
/// Somente leitura — sem modificações no sistema.
#[tauri::command]
pub async fn detect_installed_games() -> Cmd<Vec<InstalledGameInfo>> {
    tokio::task::spawn_blocking(|| {
        let games = tk_platform_win::games::detect_installed_games();
        Ok(games
            .into_iter()
            .map(|g| InstalledGameInfo {
                id: g.id,
                name: g.name,
                exe: g.exe,
                launcher: g.launcher,
                install_path: g.install_path,
            })
            .collect())
    })
    .await
    .map_err(|e| AppError { code: "detect_installed_games".into(), message: format!("Falha: {e}") })?
}

/// Lista todas as atribuições de perfil salvas.
#[tauri::command]
pub async fn game_assignments_list(state: State<'_, AppContext>) -> Cmd<Vec<GameAssignment>> {
    use sqlx::Row;
    let ctx = state.inner().clone();
    let rows = sqlx::query(
        "SELECT id, game_name, exe_match, config_json FROM game_profiles WHERE enabled = 1 ORDER BY game_name",
    )
    .fetch_all(&ctx.db)
    .await
    .map_err(db_err)?;

    Ok(rows
        .iter()
        .map(|r| {
            let db_id: i64 = r.get("id");
            let game_name: String = r.get("game_name");
            let exe_match: String = r.get("exe_match");
            let config_json: String = r.get("config_json");
            let profile_id = serde_json::from_str::<serde_json::Value>(&config_json)
                .ok()
                .and_then(|v| v["profile_id"].as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            GameAssignment { db_id, game_name, exe_match, profile_id }
        })
        .collect())
}

/// Salva (cria ou substitui) a atribuição de perfil para um jogo.
#[tauri::command]
pub async fn game_assignment_save(
    state: State<'_, AppContext>,
    exe_match: String,
    game_name: String,
    profile_id: String,
) -> Cmd<()> {
    let ctx = state.inner().clone();
    let config_json = serde_json::json!({ "profile_id": profile_id }).to_string();
    sqlx::query("DELETE FROM game_profiles WHERE exe_match = ?")
        .bind(&exe_match)
        .execute(&ctx.db)
        .await
        .map_err(db_err)?;
    sqlx::query(
        "INSERT INTO game_profiles (game_name, exe_match, config_json, enabled) VALUES (?, ?, ?, 1)",
    )
    .bind(&game_name)
    .bind(&exe_match)
    .bind(&config_json)
    .execute(&ctx.db)
    .await
    .map_err(db_err)?;
    let _ = ctx.audit().log("user", "game.assignment.save", &format!("{exe_match} → {profile_id}")).await;
    Ok(())
}

/// Remove a atribuição de perfil de um jogo.
#[tauri::command]
pub async fn game_assignment_delete(state: State<'_, AppContext>, exe_match: String) -> Cmd<()> {
    let ctx = state.inner().clone();
    sqlx::query("DELETE FROM game_profiles WHERE exe_match = ?")
        .bind(&exe_match)
        .execute(&ctx.db)
        .await
        .map_err(db_err)?;
    let _ = ctx.audit().log("user", "game.assignment.delete", &exe_match).await;
    Ok(())
}

/// Registra a aplicação de um perfil para um jogo (histórico).
#[tauri::command]
pub async fn game_run_record(
    state: State<'_, AppContext>,
    exe_match: String,
    game_name: String,
    profile_id: String,
    source: String,
) -> Cmd<()> {
    let ctx = state.inner().clone();
    let ts = tk_storage::now_ms();
    sqlx::query(
        "INSERT INTO game_runs (ts, exe_match, game_name, profile_id, source) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(ts)
    .bind(&exe_match)
    .bind(&game_name)
    .bind(&profile_id)
    .bind(&source)
    .execute(&ctx.db)
    .await
    .map_err(db_err)?;
    Ok(())
}

/// Busca histórico de ativações de perfil para um jogo (mais recentes primeiro).
#[tauri::command]
pub async fn game_runs_list(
    state: State<'_, AppContext>,
    exe_match: String,
) -> Cmd<Vec<GameRun>> {
    use sqlx::Row;
    let ctx = state.inner().clone();
    let rows = sqlx::query(
        "SELECT id, ts, exe_match, game_name, profile_id, source FROM game_runs WHERE exe_match = ? ORDER BY ts DESC LIMIT 20",
    )
    .bind(&exe_match)
    .fetch_all(&ctx.db)
    .await
    .map_err(db_err)?;

    Ok(rows
        .iter()
        .map(|r| GameRun {
            id: r.get("id"),
            ts: r.get("ts"),
            exe_match: r.get("exe_match"),
            game_name: r.get("game_name"),
            profile_id: r.get("profile_id"),
            source: r.try_get("source").unwrap_or_else(|_| "Manual".into()),
        })
        .collect())
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

    let fp = ctx.machine_fingerprint();
    let id = ctx.perf().save_session(&label, &result, Some(&fp), session_source::MANUAL).await?;
    Ok(session_info(id, &label, result))
}

/// Retorna a sessão de captura de FPS mais recente (suite fps-1.0.0).
/// Usado pelo Game Center e Mission Control para exibir FPS sem requerer nova captura.
#[tauri::command]
pub async fn perf_latest_fps_session(state: State<'_, AppContext>) -> Cmd<Option<BenchmarkSessionInfo>> {
    let ctx = state.inner().clone();
    let sessions = ctx.perf().list_sessions(20).await?;
    let latest = sessions.into_iter().find(|s| s.suite_version == "fps-1.0.0");
    Ok(latest)
}

// ───────────────────────── V5.0-B: Monitor em Tempo Real ───────────────────

#[derive(Debug, Serialize)]
pub struct CpuLive {
    pub usage_pct: f32,
    pub clock_mhz: Option<f32>,
    pub temp_c: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct GpuLive {
    pub name: String,
    pub usage_pct: f32,
    pub vram_used_mb: f32,
    pub vram_total_mb: f32,
    pub clock_mhz: Option<f32>,
    pub temp_c: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct RamLive {
    pub used_gb: f32,
    pub free_gb: f32,
    pub total_gb: f32,
    pub usage_pct: f32,
}

/// Resultado da análise — texto em português, sem jargão técnico.
#[derive(Debug, Serialize)]
pub struct LiveAnalysis {
    /// "excelente" | "bom" | "atencao" | "critico"
    pub status: String,
    /// "Healthy" | "CpuBound" | "GpuBound" | "MemoryBound" | "ThermalBound" | "BackgroundInterference"
    pub bottleneck: String,
    pub headline: String,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct LiveSnapshot {
    pub ts_ms: u64,
    pub cpu: CpuLive,
    pub gpu: Option<GpuLive>,
    pub ram: RamLive,
    pub analysis: LiveAnalysis,
    pub heavy_apps: Vec<HeavyAppDetected>,
    pub running_games: Vec<DetectedGame>,
}

fn compute_live_analysis(
    cpu_pct: f32,
    cpu_temp: Option<f32>,
    gpu: Option<&GpuLive>,
    ram_free_gb: f32,
    heavy_apps: &[HeavyAppDetected],
) -> LiveAnalysis {
    // 1. Superaquecimento
    let cpu_hot = cpu_temp.map(|t| t > 90.0).unwrap_or(false);
    let gpu_hot = gpu.and_then(|g| g.temp_c).map(|t| t > 85.0).unwrap_or(false);
    if cpu_hot || gpu_hot {
        let what = match (cpu_hot, gpu_hot) {
            (true, true)  => "processador e placa de vídeo estão",
            (true, false) => "processador está",
            _             => "placa de vídeo está",
        };
        return LiveAnalysis {
            status:     "critico".into(),
            bottleneck: "ThermalBound".into(),
            headline:   format!("Superaquecimento — {what} em temperatura elevada."),
            detail:     "Verifique a ventilação do gabinete e a pasta térmica.".into(),
        };
    }

    // 2. RAM esgotada
    if ram_free_gb < 1.0 {
        return LiveAnalysis {
            status:     "critico".into(),
            bottleneck: "MemoryBound".into(),
            headline:   "Memória RAM quase esgotada.".into(),
            detail:     format!("Apenas {ram_free_gb:.1} GB disponível. Feche aplicativos em segundo plano."),
        };
    }

    // 3. CPU Bound
    let gpu_pct = gpu.map(|g| g.usage_pct).unwrap_or(0.0);
    if cpu_pct > 90.0 && gpu_pct < 70.0 {
        return LiveAnalysis {
            status:     if cpu_pct > 96.0 { "critico" } else { "atencao" }.into(),
            bottleneck: "CpuBound".into(),
            headline:   "Seu processador está limitando o desempenho neste momento.".into(),
            detail:     "A GPU tem capacidade disponível, mas a CPU não consegue alimentá-la.".into(),
        };
    }

    // 4. GPU Bound (positivo em jogos)
    if gpu_pct > 95.0 {
        return LiveAnalysis {
            status:     "bom".into(),
            bottleneck: "GpuBound".into(),
            headline:   "Sua placa de vídeo está trabalhando no máximo.".into(),
            detail:     "Isso é o ideal em jogos — a GPU está sendo totalmente aproveitada.".into(),
        };
    }

    // 5. Interferência em segundo plano
    let high_impact: Vec<&str> = heavy_apps
        .iter()
        .filter(|a| a.impact == "alto")
        .map(|a| a.name.as_str())
        .take(2)
        .collect();
    if !high_impact.is_empty() {
        return LiveAnalysis {
            status:     "atencao".into(),
            bottleneck: "BackgroundInterference".into(),
            headline:   format!("{} está consumindo recursos em segundo plano.", high_impact.join(", ")),
            detail:     "Feche esses aplicativos para liberar desempenho.".into(),
        };
    }

    // 6. Saudável
    let status = if cpu_pct < 50.0 && ram_free_gb > 4.0 { "excelente" } else { "bom" };
    LiveAnalysis {
        status:     status.into(),
        bottleneck: "Healthy".into(),
        headline:   "Nenhum gargalo detectado.".into(),
        detail:     "O sistema está funcionando bem.".into(),
    }
}

/// Snapshot ao vivo: CPU %, clock, temp; GPU uso/VRAM/temp; RAM; diagnóstico em português.
/// Leva ~200 ms por chamada (delta de CPU).
#[tauri::command]
pub async fn monitor_live_snapshot() -> Cmd<LiveSnapshot> {
    tokio::task::spawn_blocking(|| {
        use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System};

        let mut sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
        );
        std::thread::sleep(std::time::Duration::from_millis(200));
        sys.refresh_cpu_all();

        let cpu_pct  = sys.global_cpu_usage();
        let cpu_freq = sys.cpus().first().map(|c| c.frequency() as f32);

        let gb           = 1_073_741_824.0_f32;
        let ram_total_gb = sys.total_memory() as f32 / gb;
        let ram_used_gb  = sys.used_memory()  as f32 / gb;
        let ram_free_gb  = ram_total_gb - ram_used_gb;
        let ram_pct      = if ram_total_gb > 0.0 { ram_used_gb / ram_total_gb * 100.0 } else { 0.0 };

        let hw       = tk_perflab::hardware_snapshot();
        let cpu_temp = hw.cpu_temp_c.map(|t| t as f32);
        let gpu = hw.gpu.map(|g| GpuLive {
            name:          g.name,
            usage_pct:     g.usage_pct     as f32,
            vram_used_mb:  g.vram_used_mb  as f32,
            vram_total_mb: g.vram_total_mb as f32,
            clock_mhz:     g.clock_mhz.map(|c| c as f32),
            temp_c:        g.temp_c.map(|t| t as f32),
        });

        // Processos pesados (reutiliza sys)
        let mut heavy: Vec<HeavyAppDetected> = Vec::new();
        let mut seen_h = std::collections::HashSet::new();
        for (_pid, proc) in sys.processes() {
            let exe = proc.exe()
                .and_then(|p| p.file_stem())
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            let combined = format!("{exe} {}", proc.name().to_string_lossy().to_lowercase());
            for (idx, def) in HEAVY_APP_DEFS.iter().enumerate() {
                if !seen_h.contains(&idx) && combined.contains(def.exe_fragment) {
                    seen_h.insert(idx);
                    heavy.push(HeavyAppDetected {
                        name:        def.name.to_string(),
                        exe:         exe.clone(),
                        impact:      def.impact.to_string(),
                        description: def.description.to_string(),
                    });
                }
            }
        }
        heavy.sort_by_key(|a| match a.impact.as_str() { "alto" => 0u8, "medio" => 1, _ => 2 });

        // Jogos em execução (reutiliza sys)
        const GAME_EXES: &[(&str, &str)] = &[
            ("valorant",          "VALORANT"),
            ("cs2",               "Counter-Strike 2"),
            ("csgo",              "CS:GO"),
            ("fortniteclient",    "Fortnite"),
            ("league of legends", "League of Legends"),  // file_stem com espaços
            ("leagueclient",      "League of Legends"),  // launcher
            ("dota2",             "Dota 2"),
            ("r5apex",            "Apex Legends"),
            ("modernwarfare",     "Call of Duty"),
            ("cod",               "Call of Duty"),
            ("rainbowsix",        "Rainbow Six Siege"),
            ("tslgame",           "PUBG"),
            ("rocketleague",      "Rocket League"),
            ("overwatch",         "Overwatch 2"),
            ("eldenring",         "Elden Ring"),
            ("cyberpunk2077",     "Cyberpunk 2077"),
            ("rdr2",              "Red Dead Redemption 2"),
            ("gta5",              "GTA V"),
            ("javaw",             "Minecraft"),
            ("minecraft",         "Minecraft"),
            ("deadbydaylight",    "Dead by Daylight"),
            ("destiny2",          "Destiny 2"),
            ("battlefront",       "Star Wars Battlefront II"),
            ("squadgame",         "Squad"),
            ("tarkov",            "Escape from Tarkov"),
        ];
        let mut running_games: Vec<DetectedGame> = Vec::new();
        let mut seen_g = std::collections::HashSet::new();
        for (pid, proc) in sys.processes() {
            let exe = proc.exe()
                .and_then(|p| p.file_stem())
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            // Combina exe + proc.name() para detectar jogos mesmo sem acesso ao caminho do exe
            let pname = proc.name().to_string_lossy().to_lowercase();
            let combined = format!("{exe} {pname}");
            for (fragment, name) in GAME_EXES {
                if !seen_g.contains(fragment) && combined.contains(fragment) {
                    seen_g.insert(*fragment);
                    running_games.push(DetectedGame {
                        pid:  pid.as_u32(),
                        name: name.to_string(),
                        exe:  proc.exe().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default(),
                    });
                }
            }
        }

        let analysis = compute_live_analysis(cpu_pct, cpu_temp, gpu.as_ref(), ram_free_gb, &heavy);

        let ts_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Ok(LiveSnapshot {
            ts_ms,
            cpu: CpuLive { usage_pct: cpu_pct, clock_mhz: cpu_freq, temp_c: cpu_temp },
            gpu,
            ram: RamLive { used_gb: ram_used_gb, free_gb: ram_free_gb, total_gb: ram_total_gb, usage_pct: ram_pct },
            analysis,
            heavy_apps: heavy,
            running_games,
        })
    })
    .await
    .map_err(|e| AppError { code: "monitor".into(), message: format!("Falha ao coletar métricas: {e}") })?
}

// ───────────────────────── V5.0-C: Saúde dos Drivers ───────────────────────

/// Driver de dispositivo detectado, exposto ao frontend.
/// SOMENTE LEITURA — o TkSpeed não baixa, instala nem atualiza drivers.
#[derive(Debug, Serialize)]
pub struct DriverHealthInfo {
    /// "gpu" | "network" | "audio"
    pub category: String,
    pub name: String,
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub date: Option<String>,
}

/// Resumo por categoria, com status amigável e sem alarmismo.
#[derive(Debug, Serialize)]
pub struct DriverHealthReport {
    pub gpu: Vec<DriverHealthInfo>,
    pub network: Vec<DriverHealthInfo>,
    pub audio: Vec<DriverHealthInfo>,
}

/// Detecta drivers de GPU, Rede e Áudio (somente leitura, sem elevação).
/// Não compara versões nem sugere downloads — apenas informa o que está instalado.
#[tauri::command]
pub async fn driver_health() -> Cmd<DriverHealthReport> {
    use tk_platform_win::drivers::{self, DriverCategory};
    tokio::task::spawn_blocking(|| {
        let found = drivers::detect();
        let mut gpu = Vec::new();
        let mut network = Vec::new();
        let mut audio = Vec::new();
        for d in found {
            let (cat_str, bucket) = match d.category {
                DriverCategory::Gpu => ("gpu", &mut gpu),
                DriverCategory::Network => ("network", &mut network),
                DriverCategory::Audio => ("audio", &mut audio),
            };
            bucket.push(DriverHealthInfo {
                category: cat_str.to_string(),
                name: d.name,
                vendor: d.vendor,
                version: d.version,
                date: d.date,
            });
        }
        Ok(DriverHealthReport { gpu, network, audio })
    })
    .await
    .map_err(|e| AppError { code: "driver_health".into(), message: format!("Falha ao ler drivers: {e}") })?
}
