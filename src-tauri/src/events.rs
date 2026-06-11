//! Loop de monitoramento: coleta telemetria (sysinfo) → emite eventos para a UI
//! e persiste amostras (downsample). É a camada de integração: o `tk-monitor`
//! não conhece o Tauri; aqui ligamos o sampler ao `AppHandle` e ao `AppContext`.

use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tk_contracts::HardwareInfo;
use tk_core::AppContext;
use tk_monitor::{to_samples, SysinfoSampler};
use tk_storage::InventoryItem;

/// Persiste 1 amostra a cada N ticks (downsample → rollup s10).
const PERSIST_EVERY_TICKS: u64 = 10;

/// Inicia a tarefa de monitoramento. A UI assina `metrics:tick` (sem polling).
pub fn start_monitor(app: AppHandle, ctx: AppContext) {
    tauri::async_runtime::spawn(async move {
        let mut sampler = SysinfoSampler::new();

        // Hardware (uma vez): guarda no contexto, emite para a UI e persiste.
        let hw = sampler.detect_hardware();
        ctx.set_hardware(hw.clone());
        let _ = app.emit("hardware:info", &hw);
        persist_hardware(&ctx, &hw).await;

        let mut ticker = tokio::time::interval(Duration::from_secs(1));
        let mut n: u64 = 0;

        loop {
            ticker.tick().await;
            let tick = sampler.read();
            // Alimenta a janela de análise (em memória) e emite para a UI.
            ctx.push_tick(tick.clone());
            if let Err(e) = app.emit("metrics:tick", &tick) {
                tracing::warn!("falha ao emitir metrics:tick: {e}");
            }

            n += 1;
            if n % PERSIST_EVERY_TICKS == 0 {
                let repo = ctx.metrics();
                for s in to_samples(&tick) {
                    if let Err(e) = repo.insert(&s, "s10").await {
                        tracing::warn!("falha ao persistir métrica: {e}");
                    }
                }
            }
        }
    });
}

/// Tarefa periódica de manutenção do banco (retenção + VACUUM).
/// Roda imediatamente no startup e depois a cada 12h.
pub fn start_housekeeping(ctx: AppContext) {
    tauri::async_runtime::spawn(async move {
        let policy = tk_storage::housekeeping::RetentionPolicy::default();
        let mut ticker = tokio::time::interval(Duration::from_secs(12 * 3600));
        loop {
            ticker.tick().await; // a primeira chamada dispara de imediato
            match tk_storage::housekeeping::run(&ctx.db, &policy).await {
                Ok(r) => tracing::info!(
                    metrics_deleted = r.metrics_deleted,
                    audit_deleted = r.audit_deleted,
                    runs_deleted = r.analysis_runs_deleted,
                    snapshots_deleted = r.snapshots_deleted,
                    "housekeeping concluído"
                ),
                Err(e) => tracing::warn!("housekeeping falhou: {e}"),
            }
        }
    });
}

async fn persist_hardware(ctx: &AppContext, hw: &HardwareInfo) {
    let inv = ctx.inventory();

    let items = [
        InventoryItem {
            category: "cpu".into(),
            name: hw.cpu_name.clone(),
            details_json: serde_json::json!({ "cores": hw.cpu_cores }).to_string(),
        },
        InventoryItem {
            category: "ram".into(),
            name: format!("{:.0} GB", hw.ram_total_gb),
            details_json: serde_json::json!({ "total_gb": hw.ram_total_gb }).to_string(),
        },
    ];
    for it in &items {
        if let Err(e) = inv.upsert(it).await {
            tracing::warn!("falha ao persistir inventário: {e}");
        }
    }

    for d in &hw.disks {
        let it = InventoryItem {
            category: "storage".into(),
            name: format!("{} ({})", d.name, d.mount),
            details_json: serde_json::to_string(d).unwrap_or_default(),
        };
        let _ = inv.upsert(&it).await;
    }
}
