// Evita abrir console extra no Windows em release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bridge;
mod events;

use tauri::{Emitter, Manager};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

/// Mensagem amigável usada quando o serviço local não inicia (sem vazar detalhes).
const BOOTSTRAP_ERROR_MSG: &str = "Não foi possível iniciar o serviço local do TkSpeed. \
Verifique as permissões da pasta de dados e reabra o aplicativo.";

fn main() {
    // Logging deve existir ANTES de qualquer coisa; o guard precisa viver até o app sair.
    let _log_guard = init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let db_path = resolve_db_path();
                match tk_core::AppContext::bootstrap(&db_path).await {
                    Ok(ctx) => {
                        handle.manage(ctx.clone());
                        events::start_monitor(handle.clone(), ctx.clone());
                        events::start_housekeeping(ctx);
                        tracing::info!("TkSpeed pronto");
                    }
                    Err(e) => {
                        // Detalhe técnico só no log; usuário recebe mensagem amigável + evento.
                        tracing::error!("falha no bootstrap: {e:#}");
                        bridge::set_boot_error(BOOTSTRAP_ERROR_MSG.to_string());
                        let _ = handle.emit("app:error", BOOTSTRAP_ERROR_MSG.to_string());
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bridge::bootstrap_status,
            bridge::get_hardware,
            bridge::analyze_full,
            bridge::get_history,
            bridge::protection_state,
            bridge::protection_list,
            bridge::protection_apply_demo,
            bridge::protection_rollback,
            bridge::protection_selftest,
            bridge::restore_point_create,
            bridge::restore_point_delete,
            bridge::perf_run_benchmark,
            bridge::perf_list_sessions,
            bridge::perf_compare,
            bridge::perf_noise_floor,
            bridge::perf_capture_fps,
            bridge::perf_capture_fps_demo,
            bridge::perf_latest_fps_session,
            bridge::perf_hardware_snapshot,
            bridge::perf_detect_bottleneck,
            bridge::system_capabilities,
            bridge::opt_catalog,
            bridge::opt_run,
            bridge::opt_history,
            bridge::opt_rollback,
            bridge::opt_startup_analysis,
            bridge::opt_disable_startup,
            bridge::advisor_recommendations,
            bridge::advisor_apply_profile,
            bridge::detect_games,
            bridge::boost_game,
            bridge::restore_game_priority,
            bridge::ram_flush_standby,
            bridge::gpu_detect,
            bridge::dns_flush,
            bridge::detect_heavy_apps,
            bridge::detect_installed_games,
            bridge::game_assignments_list,
            bridge::game_assignment_save,
            bridge::game_assignment_delete,
            bridge::game_run_record,
            bridge::game_runs_list,
            bridge::monitor_live_snapshot,
            bridge::driver_health,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o TkSpeed");
}

/// Inicializa logs em arquivo JSON com rotação diária e retenção de 14 arquivos
/// em `%APPDATA%\TkSpeed\logs`. Em debug, também espelha no stdout.
/// Retorna o guard do appender não-bloqueante (deve viver enquanto o app roda).
fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let log_dir = tkspeed_data_dir().join("logs");
    std::fs::create_dir_all(&log_dir).ok();

    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("tkspeed")
        .filename_suffix("log")
        .max_log_files(14)
        .build(&log_dir)
        .expect("falha ao criar o appender de log");
    let (non_blocking, guard) = tracing_appender::non_blocking(appender);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let stdout_layer = cfg!(debug_assertions).then(|| fmt::layer());

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().json().with_ansi(false).with_writer(non_blocking))
        .with(stdout_layer)
        .init();

    tracing::info!("logging inicializado em {}", log_dir.display());
    guard
}

/// Diretório único de dados do TkSpeed: %APPDATA%\TkSpeed (fallback: temp).
/// DB e logs vivem aqui (B3 — caminhos unificados).
fn tkspeed_data_dir() -> std::path::PathBuf {
    std::env::var("APPDATA")
        .map(|p| std::path::PathBuf::from(p).join("TkSpeed"))
        .unwrap_or_else(|_| std::env::temp_dir().join("TkSpeed"))
}

fn resolve_db_path() -> String {
    let dir = tkspeed_data_dir();
    std::fs::create_dir_all(&dir).ok();
    dir.join("tkspeed.db").to_string_lossy().into_owned()
}
