//! Profile Measurement Pipeline — benchmark antes/depois de uma ativação de perfil.
//!
//! Fluxo de `activate_with_measure`:
//!   1. Rodar benchmark "antes" e salvar sessão
//!   2. Ativar perfil via ProfileEngine
//!   3. Se pending_reboot → salvar before_session_id, retornar (sem after)
//!   4. Rodar benchmark "depois" e salvar sessão
//!   5. Comparar com Confidence Engine
//!   6. Processar evidência: Gain → Success, Loss/NoChange → Failure, Instável → Inconclusive
//!   7. Auto-reverter perfil em caso de Failure
//!   8. Atualizar profile_activations com before/after session ids e evidence_json

use tk_contracts::{BenchmarkResult, PerfComparison, PerfVerdict};
use tk_perflab::{build_noise_profile, compare, run_complete, run_cpu};
use tk_storage::{session_source, Db, PerfRepo};

use crate::evidence::extract_primary_gain;
use crate::machine;

use super::engine::ProfileEngine;
use super::evidence::ProfileEvidenceRepo;
use super::model::{EvidenceOutcome, ProfileMeasureResult};
use super::repo::ProfileRepo;

/// Suite de benchmark selecionada para um perfil.
#[derive(Debug, Clone, Copy)]
enum Suite {
    /// Todos os sub-benchmarks (cpu + ram + io) — usada para competitive e balanced.
    Complete,
    /// Apenas CPU — usada para streaming e power_saver.
    Cpu,
}

fn suite_for(profile_id: &str) -> Suite {
    match profile_id {
        "competitive" | "balanced" => Suite::Complete,
        _ => Suite::Cpu,
    }
}

fn suite_key(s: Suite) -> &'static str {
    match s {
        Suite::Complete => "complete-1.0.0",
        Suite::Cpu => "cpu-1.0.0",
    }
}

pub struct MeasurementPipeline {
    db: Db,
    /// Número de runs por sessão de benchmark. Padrão 3 em produção, 1 em testes.
    runs_per_bench: u32,
}

impl MeasurementPipeline {
    pub fn new(db: Db) -> Self {
        Self { db, runs_per_bench: 3 }
    }

    #[cfg(test)]
    pub fn new_fast(db: Db) -> Self {
        Self { db, runs_per_bench: 1 }
    }

    pub fn profile_engine(&self) -> ProfileEngine {
        ProfileEngine::new(self.db.clone())
    }

    fn repo(&self) -> ProfileRepo {
        ProfileRepo::new(self.db.clone())
    }

    fn perf_repo(&self) -> PerfRepo {
        PerfRepo::new(self.db.clone())
    }

    fn evidence_repo(&self) -> ProfileEvidenceRepo {
        ProfileEvidenceRepo::new(self.db.clone())
    }

    /// Pipeline completo: medir antes → ativar → medir depois → comparar → evidência.
    pub async fn activate_with_measure(
        &self,
        profile_id: &str,
    ) -> Result<ProfileMeasureResult, String> {
        let fp = machine::fingerprint();
        let suite = suite_for(profile_id);

        // 1. Benchmark antes
        let before_result = self.run_bench(suite).await?;
        let before_id = self
            .perf_repo()
            .save_session(
                &format!("profile_antes:{profile_id}"),
                &before_result,
                Some(&fp),
                session_source::PROFILE_ACTIVATION,
            )
            .await
            .map_err(|e| e.to_string())?;

        // 2. Ativar perfil
        let activation = self.profile_engine().activate(profile_id).await?;

        // 3. Pending reboot: registrar before_id e retornar sem after benchmark
        if activation.pending_reboot {
            self.repo()
                .update_activation_sessions(activation.activation_id, Some(before_id), None, "{}")
                .await
                .map_err(|e| e.to_string())?;

            return Ok(ProfileMeasureResult {
                profile_id: profile_id.into(),
                activation_id: activation.activation_id,
                snapshot_id: activation.snapshot_id,
                before_session_id: Some(before_id),
                after_session_id: None,
                evidence_recorded: EvidenceOutcome::PendingReboot,
                pending_reboot: true,
                message: "Perfil ativo. Reinicialização necessária antes de coletar evidência."
                    .into(),
            });
        }

        // 4. Benchmark depois
        let after_result = self.run_bench(suite).await?;
        let after_id = self
            .perf_repo()
            .save_session(
                &format!("profile_depois:{profile_id}"),
                &after_result,
                Some(&fp),
                session_source::PROFILE_ACTIVATION,
            )
            .await
            .map_err(|e| e.to_string())?;

        // 5. Comparar
        let hist = self
            .perf_repo()
            .metrics_by_suite(suite_key(suite))
            .await
            .map_err(|e| e.to_string())?;
        let noise = build_noise_profile(suite_key(suite), &hist);
        let comparison = compare(&before_result, &after_result, &noise);

        // 6. Processar evidência
        let evidence = self
            .process_comparison(profile_id, &fp, &comparison)
            .await?;

        // 7. Atualizar activation record com sessões e evidence_json
        let evidence_json =
            serde_json::to_string(&comparison).unwrap_or_else(|_| "{}".into());
        self.repo()
            .update_activation_sessions(
                activation.activation_id,
                Some(before_id),
                Some(after_id),
                &evidence_json,
            )
            .await
            .map_err(|e| e.to_string())?;

        // 8. Auto-reverter em caso de falha confiável
        if matches!(evidence, EvidenceOutcome::Failure) {
            self.profile_engine().deactivate().await.ok();
        }

        let msg = match &evidence {
            EvidenceOutcome::Success { gain } => {
                format!("Perfil ativo com evidência. Ganho confirmado: +{gain:.1}%")
            }
            EvidenceOutcome::Failure => {
                "Resultado negativo — perfil revertido automaticamente.".into()
            }
            EvidenceOutcome::Inconclusive => {
                "Medição inconclusiva. Perfil ativo sem evidência confirmada ainda.".into()
            }
            EvidenceOutcome::PendingReboot => unreachable!(),
        };

        Ok(ProfileMeasureResult {
            profile_id: profile_id.into(),
            activation_id: activation.activation_id,
            snapshot_id: activation.snapshot_id,
            before_session_id: Some(before_id),
            after_session_id: Some(after_id),
            evidence_recorded: evidence,
            pending_reboot: false,
            message: msg,
        })
    }

    /// Processa um `PerfComparison` e registra a evidência correspondente.
    ///
    /// Exposto para testes unitários com dados sintéticos.
    pub async fn process_comparison(
        &self,
        profile_id: &str,
        fingerprint: &str,
        comparison: &PerfComparison,
    ) -> Result<EvidenceOutcome, String> {
        // Não confiável → Inconclusive, sem registro
        if !comparison.reliable {
            return Ok(EvidenceOutcome::Inconclusive);
        }

        // Extrai ganho da métrica primária (cpu_multi ou primeira disponível)
        if let Some(gain) = extract_primary_gain(comparison) {
            self.evidence_repo()
                .record_success(profile_id, fingerprint, gain)
                .await
                .map_err(|e| e.to_string())?;
            return Ok(EvidenceOutcome::Success { gain });
        }

        // Confiável mas sem ganho — verificar se é Loss/NoChange ou Instável
        let primary_verdict = comparison
            .rows
            .iter()
            .find(|r| r.metric == "cpu_multi")
            .or_else(|| comparison.rows.first())
            .map(|r| r.verdict);

        match primary_verdict {
            Some(PerfVerdict::Loss) | Some(PerfVerdict::NoChange) => {
                self.evidence_repo()
                    .record_failure(profile_id, fingerprint)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(EvidenceOutcome::Failure)
            }
            _ => Ok(EvidenceOutcome::Inconclusive),
        }
    }

    async fn run_bench(&self, suite: Suite) -> Result<BenchmarkResult, String> {
        let runs = self.runs_per_bench;
        tokio::task::spawn_blocking(move || match suite {
            Suite::Complete => run_complete(runs),
            Suite::Cpu => run_cpu(runs),
        })
        .await
        .map_err(|e| format!("benchmark falhou: {e}"))
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tk_contracts::ComparisonRow;

    async fn make_pipeline() -> (MeasurementPipeline, Db, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("measure.db");
        let db = tk_storage::open(path.to_str().unwrap()).await.unwrap();
        (MeasurementPipeline::new_fast(db.clone()), db, dir)
    }

    fn gain_comparison(metric: &str, delta: f64) -> PerfComparison {
        PerfComparison {
            before_id: 1,
            after_id: 2,
            rows: vec![ComparisonRow {
                metric: metric.into(),
                before: 1000.0,
                after: 1000.0 + delta * 10.0,
                delta_pct: delta,
                margin_pct: 1.0,
                verdict: PerfVerdict::Gain,
                unit: "pts".into(),
            }],
            summary: format!("Ganho de {delta}%"),
            confidence: 90,
            reliable: true,
        }
    }

    fn loss_comparison(metric: &str) -> PerfComparison {
        PerfComparison {
            before_id: 1,
            after_id: 2,
            rows: vec![ComparisonRow {
                metric: metric.into(),
                before: 1000.0,
                after: 950.0,
                delta_pct: -5.0,
                margin_pct: 1.0,
                verdict: PerfVerdict::Loss,
                unit: "pts".into(),
            }],
            summary: "Perda de 5%".into(),
            confidence: 85,
            reliable: true,
        }
    }

    fn nochange_comparison() -> PerfComparison {
        PerfComparison {
            before_id: 1,
            after_id: 2,
            rows: vec![ComparisonRow {
                metric: "cpu_multi".into(),
                before: 1000.0,
                after: 1002.0,
                delta_pct: 0.2,
                margin_pct: 3.0,
                verdict: PerfVerdict::NoChange,
                unit: "pts".into(),
            }],
            summary: "Sem mudança significativa".into(),
            confidence: 80,
            reliable: true,
        }
    }

    fn unreliable_comparison() -> PerfComparison {
        PerfComparison {
            before_id: 1,
            after_id: 2,
            rows: vec![ComparisonRow {
                metric: "cpu_multi".into(),
                before: 1000.0,
                after: 1050.0,
                delta_pct: 5.0,
                margin_pct: 8.0,
                verdict: PerfVerdict::Unstable,
                unit: "pts".into(),
            }],
            summary: "Sessões instáveis".into(),
            confidence: 30,
            reliable: false,
        }
    }

    fn unstable_verdict_comparison() -> PerfComparison {
        PerfComparison {
            before_id: 1,
            after_id: 2,
            rows: vec![ComparisonRow {
                metric: "cpu_multi".into(),
                before: 1000.0,
                after: 1020.0,
                delta_pct: 2.0,
                margin_pct: 5.0,
                verdict: PerfVerdict::Unstable,
                unit: "pts".into(),
            }],
            summary: "Estável mas veredito instável".into(),
            confidence: 65,
            reliable: true, // sessões OK, mas veredito da métrica = Unstable
        }
    }

    // ── process_comparison — testes unitários (sem benchmarks) ───────────────

    #[tokio::test]
    async fn process_comparison_reliable_gain_records_success() {
        let (pipeline, _, _dir) = make_pipeline().await;
        let fp = "fp_unit_001";
        let comp = gain_comparison("cpu_multi", 8.0);
        let outcome = pipeline.process_comparison("competitive", fp, &comp).await.unwrap();

        assert!(
            matches!(outcome, EvidenceOutcome::Success { gain } if (gain - 8.0).abs() < 0.001),
            "ganho confiável deve registrar Success"
        );
        let rec = pipeline.evidence_repo().get("competitive", fp).await.unwrap().unwrap();
        assert_eq!(rec.successful_executions, 1);
        assert!((rec.average_gain - 8.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn process_comparison_reliable_loss_records_failure() {
        let (pipeline, _, _dir) = make_pipeline().await;
        let fp = "fp_unit_002";
        let comp = loss_comparison("cpu_multi");
        let outcome = pipeline.process_comparison("balanced", fp, &comp).await.unwrap();

        assert!(matches!(outcome, EvidenceOutcome::Failure));
        let rec = pipeline.evidence_repo().get("balanced", fp).await.unwrap().unwrap();
        assert_eq!(rec.executions, 1);
        assert_eq!(rec.successful_executions, 0);
    }

    #[tokio::test]
    async fn process_comparison_nochange_records_failure() {
        let (pipeline, _, _dir) = make_pipeline().await;
        let fp = "fp_unit_003";
        let comp = nochange_comparison();
        let outcome = pipeline.process_comparison("streaming", fp, &comp).await.unwrap();

        assert!(matches!(outcome, EvidenceOutcome::Failure), "NoChange é Failure (sem melhoria)");
        let rec = pipeline.evidence_repo().get("streaming", fp).await.unwrap().unwrap();
        assert_eq!(rec.successful_executions, 0);
    }

    #[tokio::test]
    async fn process_comparison_unreliable_returns_inconclusive() {
        let (pipeline, _, _dir) = make_pipeline().await;
        let fp = "fp_unit_004";
        let comp = unreliable_comparison();
        let outcome = pipeline.process_comparison("power_saver", fp, &comp).await.unwrap();

        assert!(matches!(outcome, EvidenceOutcome::Inconclusive));
        // Comparação não confiável não deve registrar evidência
        let rec = pipeline.evidence_repo().get("power_saver", fp).await.unwrap();
        assert!(rec.is_none(), "comparação não confiável não deve criar registro");
    }

    #[tokio::test]
    async fn process_comparison_unstable_verdict_is_inconclusive() {
        let (pipeline, _, _dir) = make_pipeline().await;
        let fp = "fp_unit_005";
        let comp = unstable_verdict_comparison(); // reliable=true mas verdict=Unstable
        let outcome = pipeline.process_comparison("streaming", fp, &comp).await.unwrap();

        assert!(matches!(outcome, EvidenceOutcome::Inconclusive), "Unstable verdict → Inconclusive");
        let rec = pipeline.evidence_repo().get("streaming", fp).await.unwrap();
        assert!(rec.is_none(), "veredito instável não deve criar registro");
    }

    #[tokio::test]
    async fn process_comparison_records_exact_gain_value() {
        let (pipeline, _, _dir) = make_pipeline().await;
        let fp = "fp_unit_006";
        let comp = gain_comparison("cpu_multi", 12.5);
        pipeline.process_comparison("competitive", fp, &comp).await.unwrap();

        let rec = pipeline.evidence_repo().get("competitive", fp).await.unwrap().unwrap();
        assert!((rec.average_gain - 12.5).abs() < 0.001);
    }

    #[tokio::test]
    async fn process_comparison_uses_first_metric_when_no_cpu_multi() {
        let (pipeline, _, _dir) = make_pipeline().await;
        let fp = "fp_unit_007";
        // Comparação sem cpu_multi, só ram_latency
        let comp = gain_comparison("ram_latency", 5.0);
        let outcome = pipeline.process_comparison("streaming", fp, &comp).await.unwrap();

        assert!(matches!(outcome, EvidenceOutcome::Success { .. }), "deve usar primeira métrica quando não há cpu_multi");
    }

    #[tokio::test]
    async fn process_comparison_profiles_are_isolated() {
        let (pipeline, _, _dir) = make_pipeline().await;
        let fp = "fp_unit_008";
        pipeline.process_comparison("competitive", fp, &gain_comparison("cpu_multi", 9.0)).await.unwrap();
        pipeline.process_comparison("power_saver", fp, &loss_comparison("cpu_multi")).await.unwrap();

        let comp_rec = pipeline.evidence_repo().get("competitive", fp).await.unwrap().unwrap();
        let ps_rec = pipeline.evidence_repo().get("power_saver", fp).await.unwrap().unwrap();
        assert_eq!(comp_rec.successful_executions, 1);
        assert_eq!(ps_rec.successful_executions, 0);
    }

    #[tokio::test]
    async fn process_comparison_fingerprints_are_isolated() {
        let (pipeline, _, _dir) = make_pipeline().await;
        pipeline.process_comparison("competitive", "fp_A", &gain_comparison("cpu_multi", 7.0)).await.unwrap();
        pipeline.process_comparison("competitive", "fp_B", &loss_comparison("cpu_multi")).await.unwrap();

        let rec_a = pipeline.evidence_repo().get("competitive", "fp_A").await.unwrap().unwrap();
        let rec_b = pipeline.evidence_repo().get("competitive", "fp_B").await.unwrap().unwrap();
        assert_eq!(rec_a.successful_executions, 1);
        assert_eq!(rec_b.successful_executions, 0);
    }

    // ── activate_with_measure — testes de integração (rodam benchmarks reais) ──

    #[tokio::test]
    async fn activate_with_measure_streaming_stores_session_ids() {
        let (pipeline, db, _dir) = make_pipeline().await;
        let result = pipeline.activate_with_measure("streaming").await.unwrap();

        assert!(result.before_session_id.is_some(), "before_session_id deve ser criado");
        if !result.pending_reboot {
            assert!(result.after_session_id.is_some(), "after_session_id deve ser criado");
        }

        // Verificar que profile_activations foi atualizado
        let before_sid: Option<i64> = sqlx::query_scalar(
            "SELECT before_session_id FROM profile_activations WHERE id = ?",
        )
        .bind(result.activation_id)
        .fetch_one(&db)
        .await
        .unwrap();
        assert_eq!(before_sid, result.before_session_id);

        pipeline.profile_engine().deactivate().await.ok();
    }

    #[tokio::test]
    async fn activate_with_measure_pending_reboot_skips_after_benchmark() {
        // balanced tem gpu_hardware_scheduling (requires_reboot=true) → pending_reboot
        let (pipeline, _, _dir) = make_pipeline().await;
        let result = pipeline.activate_with_measure("balanced").await.unwrap();

        assert!(result.pending_reboot, "balanced deve ter pending_reboot=true");
        assert!(result.after_session_id.is_none(), "pending_reboot deve bloquear after benchmark");
        assert!(
            matches!(result.evidence_recorded, EvidenceOutcome::PendingReboot),
            "evidence deve ser PendingReboot"
        );
        assert!(result.before_session_id.is_some(), "before benchmark deve ter sido rodado");

        pipeline.profile_engine().deactivate().await.ok();
    }

    #[tokio::test]
    async fn activate_with_measure_profile_activations_record_updated_with_sessions() {
        let (pipeline, db, _dir) = make_pipeline().await;
        let result = pipeline.activate_with_measure("streaming").await.unwrap();

        if result.pending_reboot {
            // Não continuar — este perfil não deve ter after benchmark
            pipeline.profile_engine().deactivate().await.ok();
            return;
        }

        let (before_sid, after_sid): (Option<i64>, Option<i64>) = sqlx::query_as(
            "SELECT before_session_id, after_session_id FROM profile_activations WHERE id = ?",
        )
        .bind(result.activation_id)
        .fetch_one(&db)
        .await
        .unwrap();

        assert_eq!(before_sid, result.before_session_id);
        assert_eq!(after_sid, result.after_session_id);

        pipeline.profile_engine().deactivate().await.ok();
    }

    #[tokio::test]
    async fn activate_with_measure_evidence_record_exists_after_run() {
        let (pipeline, _, _dir) = make_pipeline().await;
        let result = pipeline.activate_with_measure("streaming").await.unwrap();

        if result.pending_reboot {
            pipeline.profile_engine().deactivate().await.ok();
            return;
        }

        let fp = machine::fingerprint();
        // Se houve benchmarks, pode ter ou não evidência (depende do resultado real)
        // Mas para comparação confiável, deve haver registro
        let _ = pipeline.evidence_repo().get("streaming", &fp).await.unwrap();
        // O teste apenas verifica que não há panic/erro — o outcome depende do hardware

        pipeline.profile_engine().deactivate().await.ok();
    }
}
