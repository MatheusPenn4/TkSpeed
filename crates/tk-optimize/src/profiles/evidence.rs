//! Profile Evidence — agrega resultados de benchmark por (profile_id, fingerprint).
//!
//! Regras:
//!   record_success(gain): executions++, successful_executions++, avg_gain atualizado,
//!                         confidence = confidence_for_executions(successful_executions)
//!   record_failure():     executions++ apenas (sem alteração de successful/avg/confidence)

use serde::Serialize;
use tk_storage::{now_ms, Db, StorageError};

use crate::evidence::confidence_for_executions;

type Result<T> = std::result::Result<T, StorageError>;

/// Registro de evidência agregada de um perfil em uma máquina específica.
#[derive(Debug, Clone, Serialize)]
pub struct ProfileEvidenceRecord {
    pub profile_id: String,
    pub fingerprint: String,
    pub executions: u32,
    pub successful_executions: u32,
    pub average_gain: f64,
    pub confidence: u8,
    pub updated_at: i64,
}

pub struct ProfileEvidenceRepo {
    db: Db,
}

impl ProfileEvidenceRepo {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Registra uma execução bem-sucedida (benchmark Keep + confiável).
    /// Incrementa executions + successful_executions e atualiza média e confidence.
    pub async fn record_success(
        &self,
        profile_id: &str,
        fingerprint: &str,
        gain: f64,
    ) -> Result<()> {
        let existing = self.get(profile_id, fingerprint).await?;
        let now = now_ms();

        let (new_exec, new_success, new_avg, new_conf) = match existing {
            None => {
                let conf = confidence_for_executions(1);
                (1u32, 1u32, gain, conf)
            }
            Some(rec) => {
                let n = rec.successful_executions + 1;
                let new_avg = (rec.average_gain * (n - 1) as f64 + gain) / n as f64;
                let conf = confidence_for_executions(n);
                (rec.executions + 1, n, new_avg, conf)
            }
        };

        sqlx::query(
            "INSERT INTO profile_evidence \
             (profile_id, fingerprint, executions, successful_executions, \
              average_gain, confidence, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(profile_id, fingerprint) DO UPDATE SET \
               executions           = excluded.executions, \
               successful_executions = excluded.successful_executions, \
               average_gain         = excluded.average_gain, \
               confidence           = excluded.confidence, \
               updated_at           = excluded.updated_at",
        )
        .bind(profile_id)
        .bind(fingerprint)
        .bind(new_exec as i64)
        .bind(new_success as i64)
        .bind(new_avg)
        .bind(new_conf as i64)
        .bind(now)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Registra uma execução sem sucesso (benchmark Revert ou ativação sem ganho).
    /// Incrementa executions apenas — successful_executions e avg_gain não são alterados.
    pub async fn record_failure(&self, profile_id: &str, fingerprint: &str) -> Result<()> {
        let now = now_ms();
        sqlx::query(
            "INSERT INTO profile_evidence \
             (profile_id, fingerprint, executions, successful_executions, \
              average_gain, confidence, updated_at) \
             VALUES (?, ?, 1, 0, 0.0, 0, ?) \
             ON CONFLICT(profile_id, fingerprint) DO UPDATE SET \
               executions = executions + 1, \
               updated_at = excluded.updated_at",
        )
        .bind(profile_id)
        .bind(fingerprint)
        .bind(now)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Retorna o registro de evidência para um par (profile_id, fingerprint).
    pub async fn get(
        &self,
        profile_id: &str,
        fingerprint: &str,
    ) -> Result<Option<ProfileEvidenceRecord>> {
        let row = sqlx::query(
            "SELECT profile_id, fingerprint, executions, successful_executions, \
             average_gain, confidence, updated_at \
             FROM profile_evidence WHERE profile_id = ? AND fingerprint = ?",
        )
        .bind(profile_id)
        .bind(fingerprint)
        .fetch_optional(&self.db)
        .await?;

        use sqlx::Row;
        Ok(row.as_ref().map(|r| ProfileEvidenceRecord {
            profile_id: r.get("profile_id"),
            fingerprint: r.get("fingerprint"),
            executions: r.get::<i64, _>("executions") as u32,
            successful_executions: r.get::<i64, _>("successful_executions") as u32,
            average_gain: r.get("average_gain"),
            confidence: r.get::<i64, _>("confidence") as u8,
            updated_at: r.get("updated_at"),
        }))
    }

    /// Retorna todos os registros de um perfil (todas as máquinas).
    pub async fn list_by_profile(&self, profile_id: &str) -> Result<Vec<ProfileEvidenceRecord>> {
        let rows = sqlx::query(
            "SELECT profile_id, fingerprint, executions, successful_executions, \
             average_gain, confidence, updated_at \
             FROM profile_evidence WHERE profile_id = ? \
             ORDER BY confidence DESC, successful_executions DESC",
        )
        .bind(profile_id)
        .fetch_all(&self.db)
        .await?;

        use sqlx::Row;
        Ok(rows
            .iter()
            .map(|r| ProfileEvidenceRecord {
                profile_id: r.get("profile_id"),
                fingerprint: r.get("fingerprint"),
                executions: r.get::<i64, _>("executions") as u32,
                successful_executions: r.get::<i64, _>("successful_executions") as u32,
                average_gain: r.get("average_gain"),
                confidence: r.get::<i64, _>("confidence") as u8,
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    /// Retorna todos os perfis com evidência para uma máquina específica.
    pub async fn list_by_fingerprint(
        &self,
        fingerprint: &str,
    ) -> Result<Vec<ProfileEvidenceRecord>> {
        let rows = sqlx::query(
            "SELECT profile_id, fingerprint, executions, successful_executions, \
             average_gain, confidence, updated_at \
             FROM profile_evidence WHERE fingerprint = ? \
             ORDER BY confidence DESC, successful_executions DESC",
        )
        .bind(fingerprint)
        .fetch_all(&self.db)
        .await?;

        use sqlx::Row;
        Ok(rows
            .iter()
            .map(|r| ProfileEvidenceRecord {
                profile_id: r.get("profile_id"),
                fingerprint: r.get("fingerprint"),
                executions: r.get::<i64, _>("executions") as u32,
                successful_executions: r.get::<i64, _>("successful_executions") as u32,
                average_gain: r.get("average_gain"),
                confidence: r.get::<i64, _>("confidence") as u8,
                updated_at: r.get("updated_at"),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn make_repo() -> (ProfileEvidenceRepo, Db, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pe.db");
        let db = tk_storage::open(path.to_str().unwrap()).await.unwrap();
        (ProfileEvidenceRepo::new(db.clone()), db, dir)
    }

    const FP: &str = "aabbcc112233";
    const FP2: &str = "ddeeff445566";

    // ── get ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_returns_none_when_no_record() {
        let (repo, _, _dir) = make_repo().await;
        let rec = repo.get("competitive", FP).await.unwrap();
        assert!(rec.is_none());
    }

    #[tokio::test]
    async fn get_returns_record_after_success() {
        let (repo, _, _dir) = make_repo().await;
        repo.record_success("competitive", FP, 5.0).await.unwrap();
        let rec = repo.get("competitive", FP).await.unwrap().unwrap();
        assert_eq!(rec.profile_id, "competitive");
        assert_eq!(rec.fingerprint, FP);
    }

    // ── record_success ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn record_success_creates_entry() {
        let (repo, _, _dir) = make_repo().await;
        repo.record_success("competitive", FP, 7.0).await.unwrap();
        assert!(repo.get("competitive", FP).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn record_success_sets_executions_and_successful_to_1_on_first() {
        let (repo, _, _dir) = make_repo().await;
        repo.record_success("competitive", FP, 5.0).await.unwrap();
        let rec = repo.get("competitive", FP).await.unwrap().unwrap();
        assert_eq!(rec.executions, 1);
        assert_eq!(rec.successful_executions, 1);
    }

    #[tokio::test]
    async fn record_success_stores_gain() {
        let (repo, _, _dir) = make_repo().await;
        repo.record_success("competitive", FP, 8.5).await.unwrap();
        let rec = repo.get("competitive", FP).await.unwrap().unwrap();
        assert!((rec.average_gain - 8.5).abs() < 0.001);
    }

    #[tokio::test]
    async fn record_success_accumulates_average() {
        let (repo, _, _dir) = make_repo().await;
        // avg de 6.0 e 8.0 = 7.0
        repo.record_success("competitive", FP, 6.0).await.unwrap();
        repo.record_success("competitive", FP, 8.0).await.unwrap();
        let rec = repo.get("competitive", FP).await.unwrap().unwrap();
        assert!((rec.average_gain - 7.0).abs() < 0.001);
        assert_eq!(rec.successful_executions, 2);
        assert_eq!(rec.executions, 2);
    }

    #[tokio::test]
    async fn record_success_three_times_correct_average() {
        let (repo, _, _dir) = make_repo().await;
        repo.record_success("power_saver", FP, 3.0).await.unwrap();
        repo.record_success("power_saver", FP, 6.0).await.unwrap();
        repo.record_success("power_saver", FP, 9.0).await.unwrap();
        let rec = repo.get("power_saver", FP).await.unwrap().unwrap();
        // avg(3, 6, 9) = 6.0
        assert!((rec.average_gain - 6.0).abs() < 0.001);
        assert_eq!(rec.successful_executions, 3);
    }

    // ── Confidence progression ───────────────────────────────────────────────

    #[tokio::test]
    async fn confidence_0_for_no_records() {
        let (repo, _, _dir) = make_repo().await;
        let rec = repo.get("competitive", FP).await.unwrap();
        assert!(rec.is_none()); // confidence seria 0
    }

    #[tokio::test]
    async fn confidence_25_after_1_success() {
        let (repo, _, _dir) = make_repo().await;
        repo.record_success("competitive", FP, 5.0).await.unwrap();
        let rec = repo.get("competitive", FP).await.unwrap().unwrap();
        assert_eq!(rec.confidence, 25);
    }

    #[tokio::test]
    async fn confidence_25_after_2_successes() {
        let (repo, _, _dir) = make_repo().await;
        repo.record_success("competitive", FP, 5.0).await.unwrap();
        repo.record_success("competitive", FP, 5.0).await.unwrap();
        let rec = repo.get("competitive", FP).await.unwrap().unwrap();
        assert_eq!(rec.confidence, 25);
    }

    #[tokio::test]
    async fn confidence_50_after_3_successes() {
        let (repo, _, _dir) = make_repo().await;
        for _ in 0..3 {
            repo.record_success("streaming", FP, 4.0).await.unwrap();
        }
        let rec = repo.get("streaming", FP).await.unwrap().unwrap();
        assert_eq!(rec.confidence, 50);
    }

    #[tokio::test]
    async fn confidence_75_after_5_successes() {
        let (repo, _, _dir) = make_repo().await;
        for _ in 0..5 {
            repo.record_success("balanced", FP, 3.0).await.unwrap();
        }
        let rec = repo.get("balanced", FP).await.unwrap().unwrap();
        assert_eq!(rec.confidence, 75);
    }

    #[tokio::test]
    async fn confidence_95_after_10_successes() {
        let (repo, _, _dir) = make_repo().await;
        for _ in 0..10 {
            repo.record_success("power_saver", FP, 2.0).await.unwrap();
        }
        let rec = repo.get("power_saver", FP).await.unwrap().unwrap();
        assert_eq!(rec.confidence, 95);
    }

    // ── record_failure ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn record_failure_creates_entry_with_zero_success() {
        let (repo, _, _dir) = make_repo().await;
        repo.record_failure("competitive", FP).await.unwrap();
        let rec = repo.get("competitive", FP).await.unwrap().unwrap();
        assert_eq!(rec.executions, 1);
        assert_eq!(rec.successful_executions, 0);
        assert_eq!(rec.confidence, 0);
    }

    #[tokio::test]
    async fn record_failure_increments_executions_not_successful() {
        let (repo, _, _dir) = make_repo().await;
        repo.record_success("competitive", FP, 5.0).await.unwrap(); // success: exec=1, succ=1
        repo.record_failure("competitive", FP).await.unwrap(); // failure: exec=2, succ=1
        let rec = repo.get("competitive", FP).await.unwrap().unwrap();
        assert_eq!(rec.executions, 2);
        assert_eq!(rec.successful_executions, 1);
    }

    #[tokio::test]
    async fn record_failure_does_not_change_confidence() {
        let (repo, _, _dir) = make_repo().await;
        repo.record_success("competitive", FP, 5.0).await.unwrap(); // conf → 25
        let conf_before = repo.get("competitive", FP).await.unwrap().unwrap().confidence;
        repo.record_failure("competitive", FP).await.unwrap();
        let conf_after = repo.get("competitive", FP).await.unwrap().unwrap().confidence;
        assert_eq!(conf_before, conf_after, "falha não deve alterar confidence");
    }

    #[tokio::test]
    async fn record_failure_does_not_change_average_gain() {
        let (repo, _, _dir) = make_repo().await;
        repo.record_success("competitive", FP, 7.4).await.unwrap();
        let avg_before = repo.get("competitive", FP).await.unwrap().unwrap().average_gain;
        repo.record_failure("competitive", FP).await.unwrap();
        let avg_after = repo.get("competitive", FP).await.unwrap().unwrap().average_gain;
        assert!((avg_before - avg_after).abs() < 0.001);
    }

    // ── Cenário completo: 14 execuções, 12 sucessos ──────────────────────────

    #[tokio::test]
    async fn scenario_14_executions_12_success_2_failure() {
        let (repo, _, _dir) = make_repo().await;
        // 12 sucessos com ganho médio de 7.4%
        for _ in 0..12 {
            repo.record_success("competitive", FP, 7.4).await.unwrap();
        }
        // 2 falhas
        repo.record_failure("competitive", FP).await.unwrap();
        repo.record_failure("competitive", FP).await.unwrap();

        let rec = repo.get("competitive", FP).await.unwrap().unwrap();
        assert_eq!(rec.executions, 14, "Executado: 14 vezes");
        assert_eq!(rec.successful_executions, 12, "Sucesso: 12");
        assert!((rec.average_gain - 7.4).abs() < 0.001, "Ganho médio: +7.4%");
        assert_eq!(rec.confidence, 95, "Confiança: 95%");
    }

    // ── Isolamento por fingerprint e profile ─────────────────────────────────

    #[tokio::test]
    async fn different_fingerprints_are_isolated() {
        let (repo, _, _dir) = make_repo().await;
        repo.record_success("competitive", FP, 8.0).await.unwrap();
        repo.record_success("competitive", FP2, 2.0).await.unwrap();

        let rec1 = repo.get("competitive", FP).await.unwrap().unwrap();
        let rec2 = repo.get("competitive", FP2).await.unwrap().unwrap();
        assert!((rec1.average_gain - 8.0).abs() < 0.001);
        assert!((rec2.average_gain - 2.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn different_profiles_are_isolated() {
        let (repo, _, _dir) = make_repo().await;
        repo.record_success("competitive", FP, 8.0).await.unwrap();
        repo.record_success("power_saver", FP, 2.0).await.unwrap();

        let rec1 = repo.get("competitive", FP).await.unwrap().unwrap();
        let rec2 = repo.get("power_saver", FP).await.unwrap().unwrap();
        assert!((rec1.average_gain - 8.0).abs() < 0.001);
        assert!((rec2.average_gain - 2.0).abs() < 0.001);
    }

    // ── Queries ──────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn list_by_profile_returns_matching_records() {
        let (repo, _, _dir) = make_repo().await;
        repo.record_success("competitive", FP, 8.0).await.unwrap();
        repo.record_success("competitive", FP2, 6.0).await.unwrap();
        repo.record_success("balanced", FP, 4.0).await.unwrap();

        let recs = repo.list_by_profile("competitive").await.unwrap();
        assert_eq!(recs.len(), 2, "deve retornar as 2 máquinas com competitive");
        assert!(recs.iter().all(|r| r.profile_id == "competitive"));
    }

    #[tokio::test]
    async fn list_by_profile_returns_empty_when_no_records() {
        let (repo, _, _dir) = make_repo().await;
        let recs = repo.list_by_profile("competitive").await.unwrap();
        assert!(recs.is_empty());
    }

    #[tokio::test]
    async fn list_by_fingerprint_returns_matching_records() {
        let (repo, _, _dir) = make_repo().await;
        repo.record_success("competitive", FP, 8.0).await.unwrap();
        repo.record_success("balanced", FP, 4.0).await.unwrap();
        repo.record_success("competitive", FP2, 6.0).await.unwrap(); // outra máquina

        let recs = repo.list_by_fingerprint(FP).await.unwrap();
        assert_eq!(recs.len(), 2, "deve retornar 2 perfis para FP");
        assert!(recs.iter().all(|r| r.fingerprint == FP));
    }

    #[tokio::test]
    async fn list_by_fingerprint_returns_empty_when_no_records() {
        let (repo, _, _dir) = make_repo().await;
        let recs = repo.list_by_fingerprint(FP).await.unwrap();
        assert!(recs.is_empty());
    }
}
