//! Evidence Engine Foundation — acumula evidência histórica por (fingerprint, config_id).
//!
//! Responde às perguntas:
//!   "Quantas vezes esta otimização funcionou nesta máquina?"
//!   "Qual o ganho médio observado?"
//!   "Qual a confiança histórica?"
//!
//! Regras de atualização:
//!   Keep + comparação confiável → record_success() → incrementa executions + successful
//!   Revert / Inconclusive       → sem atualização

use serde::Serialize;
use sqlx::Row;
use tk_contracts::{PerfComparison, PerfVerdict};
use tk_storage::{now_ms, Db};

type Result<T> = std::result::Result<T, tk_storage::StorageError>;

/// Registro de evidência acumulada por (fingerprint, config_id).
#[derive(Debug, Clone, Serialize)]
pub struct EvidenceRecord {
    pub fingerprint: String,
    pub config_id: String,
    pub source: String,
    pub benchmark_relevance: Vec<String>,
    pub executions: u32,
    pub successful_executions: u32,
    pub average_gain: f64,
    pub confidence: u8,
    pub updated_at: i64,
}

/// Confidence placeholder baseada no número de execuções.
/// Algoritmo avançado fica para o Evidence Engine completo (V4.x).
///
/// | execuções | confidence |
/// |-----------|------------|
/// | 0         | 0%         |
/// | 1–2       | 25%        |
/// | 3–4       | 50%        |
/// | 5–9       | 75%        |
/// | 10+       | 95%        |
pub fn confidence_for_executions(n: u32) -> u8 {
    match n {
        0 => 0,
        1..=2 => 25,
        3..=4 => 50,
        5..=9 => 75,
        _ => 95,
    }
}

/// Extrai o ganho percentual primário de uma comparação confiável.
/// Usa `cpu_multi` como métrica primária; cai para a primeira linha disponível.
/// Retorna `None` se a comparação não é confiável, sem linhas, ou o veredito não é Gain.
pub fn extract_primary_gain(comp: &PerfComparison) -> Option<f64> {
    if !comp.reliable {
        return None;
    }
    let row = comp
        .rows
        .iter()
        .find(|r| r.metric == "cpu_multi")
        .or_else(|| comp.rows.first())?;
    if matches!(row.verdict, PerfVerdict::Gain) {
        Some(row.delta_pct)
    } else {
        None
    }
}

pub struct EvidenceRepo {
    db: Db,
}

impl EvidenceRepo {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Busca o registro atual para (fingerprint, config_id), se existir.
    pub async fn get(&self, fingerprint: &str, config_id: &str) -> Result<Option<EvidenceRecord>> {
        let row = sqlx::query(
            "SELECT fingerprint, config_id, source, benchmark_relevance, \
             executions, successful_executions, average_gain, confidence, updated_at \
             FROM config_evidence WHERE fingerprint = ? AND config_id = ?",
        )
        .bind(fingerprint)
        .bind(config_id)
        .fetch_optional(&self.db)
        .await?;
        Ok(row.as_ref().map(parse_row))
    }

    /// Registra uma execução bem-sucedida (Keep + comparação confiável).
    /// Cria o registro se for a primeira vez; caso contrário acumula.
    pub async fn record_success(
        &self,
        fingerprint: &str,
        config_id: &str,
        source: &str,
        benchmark_relevance: &[String],
        gain: f64,
    ) -> Result<()> {
        let relevance_json =
            serde_json::to_string(benchmark_relevance).map_err(tk_storage::StorageError::Json)?;
        let existing = self.get(fingerprint, config_id).await?;

        match existing {
            None => {
                let conf = confidence_for_executions(1) as i64;
                sqlx::query(
                    "INSERT INTO config_evidence \
                     (fingerprint, config_id, source, benchmark_relevance, \
                      executions, successful_executions, average_gain, confidence, updated_at) \
                     VALUES (?, ?, ?, ?, 1, 1, ?, ?, ?)",
                )
                .bind(fingerprint)
                .bind(config_id)
                .bind(source)
                .bind(&relevance_json)
                .bind(gain)
                .bind(conf)
                .bind(now_ms())
                .execute(&self.db)
                .await?;
            }
            Some(prev) => {
                let new_exec = prev.executions + 1;
                let new_success = prev.successful_executions + 1;
                // Média incremental: avg_new = (avg_old × (n−1) + gain) / n
                let new_avg = (prev.average_gain * prev.successful_executions as f64 + gain)
                    / new_success as f64;
                let conf = confidence_for_executions(new_exec) as i64;
                sqlx::query(
                    "UPDATE config_evidence SET \
                     executions = ?, successful_executions = ?, average_gain = ?, \
                     confidence = ?, source = ?, benchmark_relevance = ?, updated_at = ? \
                     WHERE fingerprint = ? AND config_id = ?",
                )
                .bind(new_exec as i64)
                .bind(new_success as i64)
                .bind(new_avg)
                .bind(conf)
                .bind(source)
                .bind(&relevance_json)
                .bind(now_ms())
                .bind(fingerprint)
                .bind(config_id)
                .execute(&self.db)
                .await?;
            }
        }
        Ok(())
    }

    /// Toda evidência de todas as máquinas para uma configuração específica.
    pub async fn evidence_for_config(&self, config_id: &str) -> Result<Vec<EvidenceRecord>> {
        let rows = sqlx::query(
            "SELECT fingerprint, config_id, source, benchmark_relevance, \
             executions, successful_executions, average_gain, confidence, updated_at \
             FROM config_evidence WHERE config_id = ? ORDER BY updated_at DESC",
        )
        .bind(config_id)
        .fetch_all(&self.db)
        .await?;
        Ok(rows.iter().map(parse_row).collect())
    }

    /// Toda evidência de uma máquina específica (ordenada por confidence desc, ganho desc).
    pub async fn evidence_for_fingerprint(&self, fingerprint: &str) -> Result<Vec<EvidenceRecord>> {
        let rows = sqlx::query(
            "SELECT fingerprint, config_id, source, benchmark_relevance, \
             executions, successful_executions, average_gain, confidence, updated_at \
             FROM config_evidence WHERE fingerprint = ? \
             ORDER BY confidence DESC, average_gain DESC",
        )
        .bind(fingerprint)
        .fetch_all(&self.db)
        .await?;
        Ok(rows.iter().map(parse_row).collect())
    }

    /// Sumário de toda evidência acumulada no sistema.
    pub async fn evidence_summary(&self, limit: i64) -> Result<Vec<EvidenceRecord>> {
        let rows = sqlx::query(
            "SELECT fingerprint, config_id, source, benchmark_relevance, \
             executions, successful_executions, average_gain, confidence, updated_at \
             FROM config_evidence ORDER BY confidence DESC, average_gain DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await?;
        Ok(rows.iter().map(parse_row).collect())
    }
}

fn parse_row(r: &sqlx::sqlite::SqliteRow) -> EvidenceRecord {
    let relevance_json: String = r.get("benchmark_relevance");
    let benchmark_relevance: Vec<String> =
        serde_json::from_str(&relevance_json).unwrap_or_default();
    EvidenceRecord {
        fingerprint: r.get("fingerprint"),
        config_id: r.get("config_id"),
        source: r.get("source"),
        benchmark_relevance,
        executions: r.get::<i64, _>("executions") as u32,
        successful_executions: r.get::<i64, _>("successful_executions") as u32,
        average_gain: r.get("average_gain"),
        confidence: r.get::<i64, _>("confidence") as u8,
        updated_at: r.get("updated_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tk_contracts::{ComparisonRow, PerfVerdict};

    // ── confidence_for_executions ──

    #[test]
    fn confidence_zero_executions() {
        assert_eq!(confidence_for_executions(0), 0);
    }

    #[test]
    fn confidence_one_execution() {
        assert_eq!(confidence_for_executions(1), 25);
    }

    #[test]
    fn confidence_two_executions() {
        assert_eq!(confidence_for_executions(2), 25);
    }

    #[test]
    fn confidence_three_executions() {
        assert_eq!(confidence_for_executions(3), 50);
    }

    #[test]
    fn confidence_four_executions() {
        assert_eq!(confidence_for_executions(4), 50);
    }

    #[test]
    fn confidence_five_executions() {
        assert_eq!(confidence_for_executions(5), 75);
    }

    #[test]
    fn confidence_nine_executions() {
        assert_eq!(confidence_for_executions(9), 75);
    }

    #[test]
    fn confidence_ten_executions() {
        assert_eq!(confidence_for_executions(10), 95);
    }

    #[test]
    fn confidence_hundred_executions() {
        assert_eq!(confidence_for_executions(100), 95);
    }

    // ── extract_primary_gain ──

    fn make_comp(reliable: bool, metric: &str, verdict: PerfVerdict, delta: f64) -> PerfComparison {
        PerfComparison {
            before_id: 1,
            after_id: 2,
            rows: vec![ComparisonRow {
                metric: metric.into(),
                before: 100.0,
                after: 100.0 + delta,
                delta_pct: delta,
                margin_pct: 2.0,
                verdict,
                unit: "score".into(),
            }],
            summary: String::new(),
            confidence: 80,
            reliable,
        }
    }

    #[test]
    fn extract_gain_reliable_cpu_multi() {
        let comp = make_comp(true, "cpu_multi", PerfVerdict::Gain, 8.5);
        assert_eq!(extract_primary_gain(&comp), Some(8.5));
    }

    #[test]
    fn extract_gain_none_when_unreliable() {
        let comp = make_comp(false, "cpu_multi", PerfVerdict::Gain, 8.5);
        assert_eq!(extract_primary_gain(&comp), None);
    }

    #[test]
    fn extract_gain_none_when_not_gain_verdict() {
        let comp = make_comp(true, "cpu_multi", PerfVerdict::NoChange, 1.0);
        assert_eq!(extract_primary_gain(&comp), None);
    }

    #[test]
    fn extract_gain_falls_back_to_first_row() {
        let comp = make_comp(true, "fps", PerfVerdict::Gain, 12.0);
        assert_eq!(extract_primary_gain(&comp), Some(12.0));
    }

    #[test]
    fn extract_gain_none_on_empty_rows() {
        let comp = PerfComparison {
            before_id: 1,
            after_id: 2,
            rows: vec![],
            summary: String::new(),
            confidence: 80,
            reliable: true,
        };
        assert_eq!(extract_primary_gain(&comp), None);
    }

    #[test]
    fn extract_gain_none_on_loss_verdict() {
        let comp = make_comp(true, "cpu_multi", PerfVerdict::Loss, -3.0);
        assert_eq!(extract_primary_gain(&comp), None);
    }

    // ── EvidenceRepo (integração com SQLite) ──

    async fn make_repo() -> (EvidenceRepo, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ev.db");
        let db = tk_storage::open(path.to_str().unwrap()).await.unwrap();
        (EvidenceRepo::new(db), dir)
    }

    const FP: &str = "aabbccdd";
    const CFG: &str = "power_plan_high_performance";
    const SRC: &str = "optimization_catalog";

    #[tokio::test]
    async fn no_evidence_before_any_record() {
        let (repo, _dir) = make_repo().await;
        assert!(repo.get(FP, CFG).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn record_creates_on_first_call() {
        let (repo, _dir) = make_repo().await;
        repo.record_success(FP, CFG, SRC, &[], 5.0).await.unwrap();
        let ev = repo.get(FP, CFG).await.unwrap().unwrap();
        assert_eq!(ev.executions, 1);
        assert_eq!(ev.successful_executions, 1);
        assert!((ev.average_gain - 5.0).abs() < 1e-6);
        assert_eq!(ev.confidence, 25);
        assert_eq!(ev.source, SRC);
    }

    #[tokio::test]
    async fn record_accumulates_executions() {
        let (repo, _dir) = make_repo().await;
        repo.record_success(FP, CFG, SRC, &[], 4.0).await.unwrap();
        repo.record_success(FP, CFG, SRC, &[], 6.0).await.unwrap();
        let ev = repo.get(FP, CFG).await.unwrap().unwrap();
        assert_eq!(ev.executions, 2);
        assert_eq!(ev.successful_executions, 2);
        assert!((ev.average_gain - 5.0).abs() < 1e-6, "média de 4+6 = 5.0");
    }

    #[tokio::test]
    async fn average_gain_correct_after_many_records() {
        let (repo, _dir) = make_repo().await;
        let gains = [2.0_f64, 4.0, 6.0, 8.0, 10.0];
        for g in &gains {
            repo.record_success(FP, CFG, SRC, &[], *g).await.unwrap();
        }
        let ev = repo.get(FP, CFG).await.unwrap().unwrap();
        let expected: f64 = gains.iter().sum::<f64>() / gains.len() as f64; // 6.0
        assert!((ev.average_gain - expected).abs() < 1e-6);
        assert_eq!(ev.executions, 5);
        assert_eq!(ev.confidence, 75);
    }

    #[tokio::test]
    async fn confidence_reaches_95_at_10_executions() {
        let (repo, _dir) = make_repo().await;
        for _ in 0..10 {
            repo.record_success(FP, CFG, SRC, &[], 5.0).await.unwrap();
        }
        let ev = repo.get(FP, CFG).await.unwrap().unwrap();
        assert_eq!(ev.confidence, 95);
        assert_eq!(ev.executions, 10);
    }

    #[tokio::test]
    async fn different_fingerprints_are_independent() {
        let (repo, _dir) = make_repo().await;
        repo.record_success("fp_ryzen", CFG, SRC, &[], 5.0).await.unwrap();
        repo.record_success("fp_intel", CFG, SRC, &[], 3.0).await.unwrap();

        let ryzen = repo.get("fp_ryzen", CFG).await.unwrap().unwrap();
        let intel = repo.get("fp_intel", CFG).await.unwrap().unwrap();
        assert!((ryzen.average_gain - 5.0).abs() < 1e-6);
        assert!((intel.average_gain - 3.0).abs() < 1e-6);
        assert_eq!(ryzen.executions, 1);
        assert_eq!(intel.executions, 1);
    }

    #[tokio::test]
    async fn different_configs_are_independent() {
        let (repo, _dir) = make_repo().await;
        repo.record_success(FP, "timer_resolution", SRC, &[], 5.0).await.unwrap();
        repo.record_success(FP, "power_plan_high_performance", SRC, &[], 8.0).await.unwrap();

        let timer = repo.get(FP, "timer_resolution").await.unwrap().unwrap();
        let power = repo.get(FP, "power_plan_high_performance").await.unwrap().unwrap();
        assert_eq!(timer.executions, 1);
        assert_eq!(power.executions, 1);
        assert!((timer.average_gain - 5.0).abs() < 1e-6);
        assert!((power.average_gain - 8.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn benchmark_relevance_stored_and_retrieved() {
        let (repo, _dir) = make_repo().await;
        let relevance = vec!["fps".to_string(), "cpu_multi".to_string()];
        repo.record_success(FP, CFG, SRC, &relevance, 5.0).await.unwrap();
        let ev = repo.get(FP, CFG).await.unwrap().unwrap();
        assert_eq!(ev.benchmark_relevance, relevance);
    }

    #[tokio::test]
    async fn evidence_for_config_returns_all_fingerprints() {
        let (repo, _dir) = make_repo().await;
        repo.record_success("fp_1", "visual_effects_gaming", SRC, &[], 3.0).await.unwrap();
        repo.record_success("fp_2", "visual_effects_gaming", SRC, &[], 5.0).await.unwrap();
        repo.record_success("fp_1", "power_plan_high_performance", SRC, &[], 4.0).await.unwrap();

        let ev = repo.evidence_for_config("visual_effects_gaming").await.unwrap();
        assert_eq!(ev.len(), 2);
        assert!(ev.iter().all(|e| e.config_id == "visual_effects_gaming"));
    }

    #[tokio::test]
    async fn evidence_for_fingerprint_returns_all_configs() {
        let (repo, _dir) = make_repo().await;
        repo.record_success(FP, "timer_resolution", SRC, &[], 5.0).await.unwrap();
        repo.record_success(FP, "power_plan_high_performance", SRC, &[], 8.0).await.unwrap();
        repo.record_success("fp_other", "timer_resolution", SRC, &[], 2.0).await.unwrap();

        let ev = repo.evidence_for_fingerprint(FP).await.unwrap();
        assert_eq!(ev.len(), 2);
        assert!(ev.iter().all(|e| e.fingerprint == FP));
    }

    #[tokio::test]
    async fn evidence_summary_ordered_by_confidence_desc() {
        let (repo, _dir) = make_repo().await;
        // cfg_b terá 5 execuções (confidence 75), cfg_a terá 1 (confidence 25)
        repo.record_success(FP, "cfg_a", SRC, &[], 2.0).await.unwrap();
        for _ in 0..5 {
            repo.record_success(FP, "cfg_b", SRC, &[], 10.0).await.unwrap();
        }

        let summary = repo.evidence_summary(50).await.unwrap();
        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0].config_id, "cfg_b", "maior confidence deve aparecer primeiro");
    }

    #[tokio::test]
    async fn evidence_summary_respects_limit() {
        let (repo, _dir) = make_repo().await;
        for i in 0..5 {
            repo.record_success(&format!("fp_{i}"), CFG, SRC, &[], 5.0).await.unwrap();
        }
        let summary = repo.evidence_summary(3).await.unwrap();
        assert_eq!(summary.len(), 3);
    }
}
