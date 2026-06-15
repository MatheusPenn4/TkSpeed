//! Repositórios SQLite (Repository pattern). Cada struct encapsula o acesso a
//! um agregado. Usam queries em runtime via `sqlx::query`/`query_scalar`.

use crate::{now_ms, Db, Result};
use sqlx::Row;
use tk_contracts::{
    BenchmarkMetric, BenchmarkResult, BenchmarkSessionInfo, Classification, Finding, MetricSample,
    OptDecision, OptimizationRunInfo, PerfComparison, ScoreBreakdown, ScoreHistoryItem, Severity,
    TkSpeedScore,
};

// ───────────────────────── Inventário de hardware ─────────────────────────

#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub category: String,
    pub name: String,
    pub details_json: String,
}

pub struct InventoryRepo {
    db: Db,
}

impl InventoryRepo {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Insere ou atualiza um componente (chave lógica = categoria + nome).
    pub async fn upsert(&self, item: &InventoryItem) -> Result<()> {
        let ts = now_ms();
        let existing: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM hardware_inventory WHERE category = ? AND name = ?",
        )
        .bind(&item.category)
        .bind(&item.name)
        .fetch_optional(&self.db)
        .await?;

        match existing {
            Some(id) => {
                sqlx::query(
                    "UPDATE hardware_inventory SET details_json = ?, last_seen = ? WHERE id = ?",
                )
                .bind(&item.details_json)
                .bind(ts)
                .bind(id)
                .execute(&self.db)
                .await?;
            }
            None => {
                sqlx::query(
                    "INSERT INTO hardware_inventory (category, name, details_json, first_seen, last_seen) \
                     VALUES (?, ?, ?, ?, ?)",
                )
                .bind(&item.category)
                .bind(&item.name)
                .bind(&item.details_json)
                .bind(ts)
                .bind(ts)
                .execute(&self.db)
                .await?;
            }
        }
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<InventoryItem>> {
        let rows = sqlx::query(
            "SELECT category, name, details_json FROM hardware_inventory ORDER BY category",
        )
        .fetch_all(&self.db)
        .await?;
        Ok(rows
            .iter()
            .map(|r| InventoryItem {
                category: r.get("category"),
                name: r.get("name"),
                details_json: r.get("details_json"),
            })
            .collect())
    }
}

// ───────────────────────── Métricas (telemetria) ─────────────────────────

pub struct MetricRepo {
    db: Db,
}

impl MetricRepo {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Persiste uma amostra (rollup já decidido pelo downsampler).
    pub async fn insert(&self, s: &MetricSample, rollup: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO metric_samples (ts, source, metric, value, unit, rollup) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(s.ts)
        .bind(s.source.as_str())
        .bind(&s.metric)
        .bind(s.value)
        .bind(&s.unit)
        .bind(rollup)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    /// Últimos N valores de uma (source, metric) para sparklines/histórico curto.
    pub async fn recent(&self, source: &str, metric: &str, limit: i64) -> Result<Vec<f64>> {
        let rows = sqlx::query(
            "SELECT value FROM metric_samples WHERE source = ? AND metric = ? \
             ORDER BY ts DESC LIMIT ?",
        )
        .bind(source)
        .bind(metric)
        .bind(limit)
        .fetch_all(&self.db)
        .await?;
        Ok(rows.iter().map(|r| r.get::<f64, _>("value")).collect())
    }
}

// ───────────────────────── Análise + Score ─────────────────────────

pub struct AnalysisRepo {
    db: Db,
}

impl AnalysisRepo {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn start_run(&self, trigger: &str) -> Result<i64> {
        let id = sqlx::query("INSERT INTO analysis_runs (started_at, trigger) VALUES (?, ?)")
            .bind(now_ms())
            .bind(trigger)
            .execute(&self.db)
            .await?
            .last_insert_rowid();
        Ok(id)
    }

    pub async fn finish_run(&self, run_id: i64, summary_json: &str) -> Result<()> {
        sqlx::query("UPDATE analysis_runs SET finished_at = ?, summary_json = ? WHERE id = ?")
            .bind(now_ms())
            .bind(summary_json)
            .bind(run_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn add_finding(&self, run_id: i64, f: &Finding) -> Result<()> {
        sqlx::query(
            "INSERT INTO findings (run_id, kind, severity, title, impact, solution) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(run_id)
        .bind(&f.kind)
        .bind(f.severity.as_str())
        .bind(&f.title)
        .bind(&f.impact)
        .bind(&f.solution)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn save_score(&self, run_id: i64, score: &TkSpeedScore) -> Result<i64> {
        let breakdown_json = serde_json::to_string(&score.breakdown)?;
        let id = sqlx::query(
            "INSERT INTO scores (run_id, ts, total, classification, breakdown_json, score_version) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(run_id)
        .bind(now_ms())
        .bind(score.total as i64)
        .bind(score.classification.as_str())
        .bind(&breakdown_json)
        .bind(&score.score_version)
        .execute(&self.db)
        .await?
        .last_insert_rowid();
        Ok(id)
    }

    /// Histórico de scores (mais recentes primeiro) para consulta de análises passadas.
    pub async fn score_history(&self, limit: i64) -> Result<Vec<ScoreHistoryItem>> {
        let rows = sqlx::query(
            "SELECT ts, total, classification FROM scores ORDER BY ts DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await?;
        Ok(rows
            .iter()
            .map(|r| ScoreHistoryItem {
                ts: r.get("ts"),
                total: r.get::<i64, _>("total") as u16,
                classification: parse_classification(&r.get::<String, _>("classification")),
            })
            .collect())
    }

    /// Score mais recente persistido (para o Dashboard ao abrir).
    pub async fn latest_score(&self) -> Result<Option<TkSpeedScore>> {
        let row = sqlx::query(
            "SELECT total, classification, breakdown_json, score_version \
             FROM scores ORDER BY ts DESC LIMIT 1",
        )
        .fetch_optional(&self.db)
        .await?;

        let Some(row) = row else { return Ok(None) };
        let breakdown_json: String = row.get("breakdown_json");
        let breakdown: ScoreBreakdown = serde_json::from_str(&breakdown_json)?;
        let total: i64 = row.get("total");
        let classification = parse_classification(&row.get::<String, _>("classification"));

        Ok(Some(TkSpeedScore {
            total: total as u16,
            classification,
            breakdown,
            score_version: row.get("score_version"),
        }))
    }
}

// ───────────────────────── Snapshots (rollback) ─────────────────────────

#[derive(Debug, Clone)]
pub struct SnapshotRow {
    pub id: i64,
    pub ts: i64,
    pub reason: String,
    pub integrity_hash: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct SnapshotEntryRow {
    pub target_type: String,
    pub target_key: String,
    pub old_value_json: Option<String>,
    pub new_value_json: Option<String>,
}

pub struct SnapshotRepo {
    db: Db,
}

impl SnapshotRepo {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        reason: &str,
        integrity_hash: &str,
        machine_fingerprint: Option<&str>,
    ) -> Result<i64> {
        let id = sqlx::query(
            "INSERT INTO snapshots (ts, reason, integrity_hash, status, machine_fingerprint) \
             VALUES (?, ?, ?, 'active', ?)",
        )
        .bind(now_ms())
        .bind(reason)
        .bind(integrity_hash)
        .bind(machine_fingerprint)
        .execute(&self.db)
        .await?
        .last_insert_rowid();
        Ok(id)
    }

    pub async fn add_entry(&self, snapshot_id: i64, e: &SnapshotEntryRow) -> Result<()> {
        sqlx::query(
            "INSERT INTO snapshot_entries (snapshot_id, target_type, target_key, old_value_json, new_value_json) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(snapshot_id)
        .bind(&e.target_type)
        .bind(&e.target_key)
        .bind(&e.old_value_json)
        .bind(&e.new_value_json)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn set_status(&self, snapshot_id: i64, status: &str) -> Result<()> {
        sqlx::query("UPDATE snapshots SET status = ? WHERE id = ?")
            .bind(status)
            .bind(snapshot_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn get(&self, snapshot_id: i64) -> Result<Option<SnapshotRow>> {
        let row = sqlx::query(
            "SELECT id, ts, reason, integrity_hash, status FROM snapshots WHERE id = ?",
        )
        .bind(snapshot_id)
        .fetch_optional(&self.db)
        .await?;
        Ok(row.map(|r| SnapshotRow {
            id: r.get("id"),
            ts: r.get("ts"),
            reason: r.get("reason"),
            integrity_hash: r.get("integrity_hash"),
            status: r.get("status"),
        }))
    }

    pub async fn entries(&self, snapshot_id: i64) -> Result<Vec<SnapshotEntryRow>> {
        let rows = sqlx::query(
            "SELECT target_type, target_key, old_value_json, new_value_json \
             FROM snapshot_entries WHERE snapshot_id = ? ORDER BY id",
        )
        .bind(snapshot_id)
        .fetch_all(&self.db)
        .await?;
        Ok(rows
            .iter()
            .map(|r| SnapshotEntryRow {
                target_type: r.get("target_type"),
                target_key: r.get("target_key"),
                old_value_json: r.get("old_value_json"),
                new_value_json: r.get("new_value_json"),
            })
            .collect())
    }

    pub async fn count(&self) -> Result<i64> {
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM snapshots")
            .fetch_one(&self.db)
            .await?;
        Ok(n)
    }

    pub async fn list(&self, limit: i64) -> Result<Vec<SnapshotRow>> {
        let rows = sqlx::query(
            "SELECT id, ts, reason, integrity_hash, status FROM snapshots ORDER BY ts DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await?;
        Ok(rows
            .iter()
            .map(|r| SnapshotRow {
                id: r.get("id"),
                ts: r.get("ts"),
                reason: r.get("reason"),
                integrity_hash: r.get("integrity_hash"),
                status: r.get("status"),
            })
            .collect())
    }
}

// ───────────────────────── Auditoria (append-only) ─────────────────────────

pub struct AuditRepo {
    db: Db,
}

impl AuditRepo {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn log(&self, actor: &str, action: &str, details_json: &str) -> Result<()> {
        sqlx::query("INSERT INTO audit_log (ts, actor, action, details_json) VALUES (?, ?, ?, ?)")
            .bind(now_ms())
            .bind(actor)
            .bind(action)
            .bind(details_json)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Timestamp da última ocorrência de uma ação (ex.: "rollback.completed").
    pub async fn last_ts(&self, action: &str) -> Result<Option<i64>> {
        let ts: Option<i64> = sqlx::query_scalar(
            "SELECT ts FROM audit_log WHERE action = ? ORDER BY ts DESC LIMIT 1",
        )
        .bind(action)
        .fetch_optional(&self.db)
        .await?;
        Ok(ts)
    }
}

// ───────────────────────── Performance Lab ─────────────────────────

pub struct PerfRepo {
    db: Db,
}

impl PerfRepo {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Persiste uma sessão de benchmark (+ suas métricas). Retorna o id da sessão.
    pub async fn save_session(
        &self,
        label: &str,
        r: &BenchmarkResult,
        machine_fingerprint: Option<&str>,
        source: &str,
    ) -> Result<i64> {
        let id = sqlx::query(
            "INSERT INTO benchmark_sessions \
             (ts, kind, suite_version, duration_ms, target, conditions_json, runs, confidence, contaminated, temp_start, temp_end, machine_fingerprint, source) \
             VALUES (?, ?, ?, ?, ?, '{}', ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(now_ms())
        .bind(&r.kind)
        .bind(&r.suite_version)
        .bind(r.duration_ms)
        .bind(label)
        .bind(r.runs as i64)
        .bind(r.confidence as i64)
        .bind(r.contaminated as i64)
        .bind(r.temp_start_c)
        .bind(r.temp_end_c)
        .bind(machine_fingerprint)
        .bind(source)
        .execute(&self.db)
        .await?
        .last_insert_rowid();

        for m in &r.metrics {
            sqlx::query(
                "INSERT INTO benchmark_metrics (session_id, metric, value, unit, stddev, samples) \
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(id)
            .bind(&m.metric)
            .bind(m.value)
            .bind(&m.unit)
            .bind(m.stddev)
            .bind(m.samples as i64)
            .execute(&self.db)
            .await?;
        }
        Ok(id)
    }

    async fn session_metrics(&self, session_id: i64) -> Result<Vec<BenchmarkMetric>> {
        let rows = sqlx::query(
            "SELECT metric, value, unit, stddev, samples FROM benchmark_metrics \
             WHERE session_id = ? ORDER BY id",
        )
        .bind(session_id)
        .fetch_all(&self.db)
        .await?;
        Ok(rows
            .iter()
            .map(|r| BenchmarkMetric {
                metric: r.get("metric"),
                value: r.get("value"),
                unit: r.get("unit"),
                stddev: r.get("stddev"),
                samples: r.get::<i64, _>("samples") as u32,
            })
            .collect())
    }

    pub async fn list_sessions(&self, limit: i64) -> Result<Vec<BenchmarkSessionInfo>> {
        let rows = sqlx::query(
            "SELECT id, ts, kind, suite_version, COALESCE(target, '') AS label, confidence, contaminated \
             FROM benchmark_sessions ORDER BY ts DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let id: i64 = row.get("id");
            let metrics = self.session_metrics(id).await?;
            let confidence = row.get::<i64, _>("confidence") as u8;
            let contaminated = row.get::<i64, _>("contaminated") != 0;
            out.push(BenchmarkSessionInfo {
                id,
                ts: row.get("ts"),
                kind: row.get("kind"),
                suite_version: row.get("suite_version"),
                label: row.get("label"),
                metrics,
                confidence,
                stable: confidence >= 70 && !contaminated,
                contaminated,
            });
        }
        Ok(out)
    }

    /// Métricas (médias) de todas as sessões de uma suite — base do perfil de ruído.
    pub async fn metrics_by_suite(&self, suite: &str) -> Result<Vec<Vec<BenchmarkMetric>>> {
        let ids: Vec<i64> = sqlx::query_scalar(
            "SELECT id FROM benchmark_sessions WHERE suite_version = ? ORDER BY ts",
        )
        .bind(suite)
        .fetch_all(&self.db)
        .await?;
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            out.push(self.session_metrics(id).await?);
        }
        Ok(out)
    }

    /// Sessões filtradas por source (mais recentes primeiro).
    /// Fallback legado: registros sem source explícito usam o DEFAULT 'manual' do schema.
    pub async fn sessions_by_source(&self, source: &str, limit: i64) -> Result<Vec<BenchmarkSessionInfo>> {
        let rows = sqlx::query(
            "SELECT id, ts, kind, suite_version, COALESCE(target, '') AS label, confidence, contaminated \
             FROM benchmark_sessions WHERE COALESCE(source, 'manual') = ? ORDER BY ts DESC LIMIT ?",
        )
        .bind(source)
        .bind(limit)
        .fetch_all(&self.db)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let id: i64 = row.get("id");
            let metrics = self.session_metrics(id).await?;
            let confidence = row.get::<i64, _>("confidence") as u8;
            let contaminated = row.get::<i64, _>("contaminated") != 0;
            out.push(BenchmarkSessionInfo {
                id,
                ts: row.get("ts"),
                kind: row.get("kind"),
                suite_version: row.get("suite_version"),
                label: row.get("label"),
                metrics,
                confidence,
                stable: confidence >= 70 && !contaminated,
                contaminated,
            });
        }
        Ok(out)
    }

    /// Reconstrói o `BenchmarkResult` de uma sessão (para comparação).
    pub async fn get_result(&self, session_id: i64) -> Result<Option<BenchmarkResult>> {
        let row = sqlx::query(
            "SELECT kind, suite_version, duration_ms, runs, confidence, contaminated, temp_start, temp_end \
             FROM benchmark_sessions WHERE id = ?",
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?;

        let Some(row) = row else { return Ok(None) };
        let metrics = self.session_metrics(session_id).await?;
        let confidence = row.get::<i64, _>("confidence") as u8;
        let contaminated = row.get::<i64, _>("contaminated") != 0;
        Ok(Some(BenchmarkResult {
            kind: row.get("kind"),
            suite_version: row.get("suite_version"),
            duration_ms: row.get("duration_ms"),
            runs: row.get::<i64, _>("runs") as u32,
            metrics,
            confidence,
            stable: confidence >= 70 && !contaminated,
            contaminated,
            temp_start_c: row.get::<Option<f64>, _>("temp_start"),
            temp_end_c: row.get::<Option<f64>, _>("temp_end"),
        }))
    }

    pub async fn save_comparison(
        &self,
        before_id: i64,
        after_id: i64,
        comp: &PerfComparison,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO comparisons (ts, before_session, after_session, verdict_json) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(now_ms())
        .bind(before_id)
        .bind(after_id)
        .bind(serde_json::to_string(comp)?)
        .execute(&self.db)
        .await?;
        Ok(())
    }
}

// ───────────────────────── Optimization Engine ─────────────────────────

pub struct OptRepo {
    db: Db,
}

impl OptRepo {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Persiste uma execução do pipeline. Retorna o id atribuído.
    pub async fn save_run(
        &self,
        info: &OptimizationRunInfo,
        snapshot_id: i64,
        machine_fingerprint: Option<&str>,
        source: &str,
    ) -> Result<i64> {
        let decision = match info.decision {
            OptDecision::Keep => "keep",
            OptDecision::Revert => "revert",
            OptDecision::Inconclusive => "inconclusive",
        };
        let id = sqlx::query(
            "INSERT INTO optimization_runs \
             (ts, optimization_id, snapshot_id, before_session, after_session, status, decision, confidence, evidence_json, machine_fingerprint, source) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(info.ts)
        .bind(&info.optimization_id)
        .bind(snapshot_id)
        .bind(info.before_session)
        .bind(info.after_session)
        .bind(&info.status)
        .bind(decision)
        .bind(info.confidence as i64)
        .bind(serde_json::to_string(info)?)
        .bind(machine_fingerprint)
        .bind(source)
        .execute(&self.db)
        .await?
        .last_insert_rowid();
        Ok(id)
    }

    /// Execuções filtradas por source (mais recentes primeiro).
    /// Fallback legado: registros sem source usam o DEFAULT 'optimization_catalog' do schema.
    pub async fn runs_by_source(&self, source: &str, limit: i64) -> Result<Vec<OptimizationRunInfo>> {
        let rows = sqlx::query(
            "SELECT id, evidence_json FROM optimization_runs \
             WHERE COALESCE(source, 'optimization_catalog') = ? ORDER BY ts DESC LIMIT ?",
        )
        .bind(source)
        .bind(limit)
        .fetch_all(&self.db)
        .await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let json: String = r.get("evidence_json");
            let mut info: OptimizationRunInfo = serde_json::from_str(&json)?;
            info.id = r.get("id");
            out.push(info);
        }
        Ok(out)
    }

    /// Lista execuções (mais recentes primeiro). Reconstrói a partir do evidence_json.
    pub async fn list_runs(&self, limit: i64) -> Result<Vec<OptimizationRunInfo>> {
        let rows = sqlx::query(
            "SELECT id, evidence_json FROM optimization_runs ORDER BY ts DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let json: String = r.get("evidence_json");
            let mut info: OptimizationRunInfo = serde_json::from_str(&json)?;
            info.id = r.get("id");
            out.push(info);
        }
        Ok(out)
    }

    /// Snapshot associado a uma execução (para rollback manual).
    pub async fn run_snapshot(&self, run_id: i64) -> Result<Option<i64>> {
        let v: Option<i64> =
            sqlx::query_scalar("SELECT snapshot_id FROM optimization_runs WHERE id = ?")
                .bind(run_id)
                .fetch_optional(&self.db)
                .await?;
        Ok(v)
    }

    pub async fn set_status(&self, run_id: i64, status: &str) -> Result<()> {
        sqlx::query("UPDATE optimization_runs SET status = ? WHERE id = ?")
            .bind(status)
            .bind(run_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

// ───────────────────────── helpers ─────────────────────────

fn parse_classification(s: &str) -> Classification {
    match s {
        "critico" => Classification::Critico,
        "regular" => Classification::Regular,
        "bom" => Classification::Bom,
        "excelente" => Classification::Excelente,
        "elite" => Classification::Elite,
        _ => Classification::Critico,
    }
}

#[allow(dead_code)]
fn severity_from_str(s: &str) -> Severity {
    match s {
        "info" => Severity::Info,
        "low" => Severity::Low,
        "medium" => Severity::Medium,
        "high" => Severity::High,
        "critical" => Severity::Critical,
        _ => Severity::Info,
    }
}
