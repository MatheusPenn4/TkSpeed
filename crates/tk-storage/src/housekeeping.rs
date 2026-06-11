//! Housekeeping — política de retenção e limpeza periódica do banco.
//!
//! Evita o crescimento infinito do SQLite. As tabelas com cascata
//! (`findings`/`scores` → `analysis_runs`; `snapshot_entries` → `snapshots`)
//! são limpas automaticamente via `ON DELETE CASCADE`. Roda no startup e a
//! cada 12h (ver `src-tauri/src/events.rs`).

use crate::{now_ms, Db, Result};

const DAY_MS: i64 = 86_400_000;

/// Política de retenção (documentada). Valores conservadores para o MVP.
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    /// Telemetria granular (metric_samples): janela curta.
    pub metrics_days: i64,
    /// Trilha de auditoria.
    pub audit_days: i64,
    /// Execuções de análise (e, por cascata, findings/scores).
    pub analysis_days: i64,
    /// Máximo de snapshots preservados (os mais recentes). 'active' nunca é apagado.
    pub max_snapshots: i64,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            metrics_days: 7,
            audit_days: 90,
            analysis_days: 90,
            max_snapshots: 50,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct HousekeepingReport {
    pub metrics_deleted: u64,
    pub audit_deleted: u64,
    pub analysis_runs_deleted: u64,
    pub snapshots_deleted: u64,
}

/// Aplica a política de retenção e recupera espaço (`VACUUM`).
pub async fn run(db: &Db, policy: &RetentionPolicy) -> Result<HousekeepingReport> {
    let now = now_ms();
    let mut report = HousekeepingReport::default();

    report.metrics_deleted = sqlx::query("DELETE FROM metric_samples WHERE ts < ?")
        .bind(now - policy.metrics_days * DAY_MS)
        .execute(db)
        .await?
        .rows_affected();

    report.audit_deleted = sqlx::query("DELETE FROM audit_log WHERE ts < ?")
        .bind(now - policy.audit_days * DAY_MS)
        .execute(db)
        .await?
        .rows_affected();

    // findings/scores caem por ON DELETE CASCADE.
    report.analysis_runs_deleted = sqlx::query("DELETE FROM analysis_runs WHERE started_at < ?")
        .bind(now - policy.analysis_days * DAY_MS)
        .execute(db)
        .await?
        .rows_affected();

    // Mantém os N snapshots mais recentes; preserva os 'active' (reversões pendentes).
    // snapshot_entries caem por ON DELETE CASCADE.
    report.snapshots_deleted = sqlx::query(
        "DELETE FROM snapshots WHERE status != 'active' AND id NOT IN \
         (SELECT id FROM snapshots ORDER BY ts DESC LIMIT ?)",
    )
    .bind(policy.max_snapshots)
    .execute(db)
    .await?
    .rows_affected();

    // Recupera espaço em disco (não pode rodar dentro de transação — aqui é autocommit).
    sqlx::query("VACUUM").execute(db).await?;

    Ok(report)
}
