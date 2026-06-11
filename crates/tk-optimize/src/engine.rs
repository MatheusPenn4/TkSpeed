//! TkOptimizationEngine — executa o pipeline e decide com evidência.
//! Aplica/verifica/reverte de forma GENÉRICA (ReversibleAction); cada otimização
//! só declara `plan()` e `validation()`.

use tk_contracts::{
    BenchmarkResult, OptDecision, OptimizationInfo, OptimizationRunInfo, PerfComparison, PerfVerdict,
    StartupItem,
};
use tk_perflab::{build_noise_profile, compare, run_complete, run_cpu, run_io, run_ram};
use tk_rollback::{apply_actions, verify_actions, ProtectionService, ReversibleAction};
use tk_storage::{now_ms, AuditRepo, Db, OptRepo, PerfRepo};

use crate::catalog::{self, Validation};

pub struct Engine {
    db: Db,
}

impl Engine {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    fn protection(&self) -> ProtectionService {
        ProtectionService::new(self.db.clone())
    }
    fn perf(&self) -> PerfRepo {
        PerfRepo::new(self.db.clone())
    }
    fn opt(&self) -> OptRepo {
        OptRepo::new(self.db.clone())
    }
    fn audit(&self) -> AuditRepo {
        AuditRepo::new(self.db.clone())
    }

    pub fn catalog_info(&self) -> Vec<OptimizationInfo> {
        catalog::catalog_info()
    }

    pub fn startup_items(&self) -> Vec<StartupItem> {
        catalog::startup_items()
    }

    /// Executa o loop completo para uma otimização e devolve o registro com evidência.
    pub async fn run(&self, id: &str) -> Result<OptimizationRunInfo, String> {
        let opt = catalog::get(id).ok_or_else(|| "Otimização não encontrada".to_string())?;
        let meta = opt.meta();

        // plan + snapshot (sem snapshot → não aplica)
        let actions = opt.plan()?;
        let snapshot_id = self
            .protection()
            .create_snapshot(&format!("optimize:{id}"), &actions)
            .await
            .map_err(|e| e.to_string())?;
        self.audit().log("user", "optimize.intent", &format!("{{\"id\":\"{id}\"}}")).await.ok();

        match opt.validation() {
            Validation::Benchmark(suite) => self.run_benchmarked(id, &meta, snapshot_id, &actions, suite).await,
            Validation::SpaceFreed => self.run_space_freed(id, &meta, snapshot_id, &actions).await,
            Validation::Manual => self.run_manual(id, &meta, snapshot_id, &actions).await,
        }
    }

    /// Caminho com benchmark antes/depois + Confidence Engine.
    async fn run_benchmarked(
        &self,
        id: &str,
        meta: &OptimizationInfo,
        snapshot_id: i64,
        actions: &[ReversibleAction],
        suite: &str,
    ) -> Result<OptimizationRunInfo, String> {
        let before = run_suite(suite).await?;
        let before_id = self.perf().save_session(&format!("antes: {}", meta.name), &before).await.map_err(|e| e.to_string())?;

        if let Err(e) = apply_actions(actions) {
            let _ = self.protection().rollback(snapshot_id).await;
            return self.finish(id, meta, snapshot_id, Some(before_id), None, "failed", OptDecision::Inconclusive, None, 0, &format!("Falha ao aplicar: {e}")).await;
        }
        if !verify_actions(actions).unwrap_or(false) {
            let _ = self.protection().rollback(snapshot_id).await;
            return self.finish(id, meta, snapshot_id, Some(before_id), None, "reverted", OptDecision::Revert, None, 0, "Pós-condição não confirmada — revertido.").await;
        }

        let after = run_suite(suite).await?;
        let after_id = self.perf().save_session(&format!("depois: {}", meta.name), &after).await.map_err(|e| e.to_string())?;

        let hist = self.perf().metrics_by_suite(suite).await.map_err(|e| e.to_string())?;
        let noise = build_noise_profile(suite, &hist);
        let comp = compare(&before, &after, &noise);
        let decision = decide(&comp);
        let confidence = comp.confidence;
        let msg = comp.summary.clone();

        let status = match decision {
            OptDecision::Keep => "kept",
            OptDecision::Revert => {
                let _ = self.protection().rollback(snapshot_id).await;
                "reverted"
            }
            OptDecision::Inconclusive => {
                let _ = self.protection().rollback(snapshot_id).await;
                "inconclusive"
            }
        };
        self.finish(id, meta, snapshot_id, Some(before_id), Some(after_id), status, decision, Some(comp), confidence, &msg).await
    }

    /// Caminho de limpeza: evidência = espaço liberado (sem benchmark).
    async fn run_space_freed(
        &self,
        id: &str,
        meta: &OptimizationInfo,
        snapshot_id: i64,
        actions: &[ReversibleAction],
    ) -> Result<OptimizationRunInfo, String> {
        match apply_actions(actions) {
            Ok(freed) => {
                let mb = freed as f64 / 1_000_000.0;
                self.finish(id, meta, snapshot_id, None, None, "kept", OptDecision::Keep, None, 100, &format!("Liberados {mb:.1} MB para a quarentena (reversível).")).await
            }
            Err(e) => {
                let _ = self.protection().rollback(snapshot_id).await;
                self.finish(id, meta, snapshot_id, None, None, "failed", OptDecision::Inconclusive, None, 0, &format!("Falha na limpeza: {e}")).await
            }
        }
    }

    /// Caminho manual: aplica (reversível) e fica pendente de evidência (ex.: FPS em jogo).
    async fn run_manual(
        &self,
        id: &str,
        meta: &OptimizationInfo,
        snapshot_id: i64,
        actions: &[ReversibleAction],
    ) -> Result<OptimizationRunInfo, String> {
        if let Err(e) = apply_actions(actions) {
            let _ = self.protection().rollback(snapshot_id).await;
            return self.finish(id, meta, snapshot_id, None, None, "failed", OptDecision::Inconclusive, None, 0, &format!("Falha ao aplicar: {e}")).await;
        }
        let _ = verify_actions(actions);
        self.finish(
            id, meta, snapshot_id, None, None, "applied", OptDecision::Inconclusive, None, 0,
            "Aplicado e reversível. Comprove com uma captura de FPS no jogo (antes/depois) para manter com evidência.",
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn finish(
        &self,
        id: &str,
        meta: &OptimizationInfo,
        snapshot_id: i64,
        before: Option<i64>,
        after: Option<i64>,
        status: &str,
        decision: OptDecision,
        comparison: Option<PerfComparison>,
        confidence: u8,
        msg: &str,
    ) -> Result<OptimizationRunInfo, String> {
        let info = OptimizationRunInfo {
            id: 0,
            ts: now_ms(),
            optimization_id: id.into(),
            name: meta.name.clone(),
            status: status.into(),
            decision,
            confidence,
            before_session: before,
            after_session: after,
            comparison,
            message: msg.into(),
        };
        let run_id = self.opt().save_run(&info, snapshot_id).await.map_err(|e| e.to_string())?;
        self.audit().log("user", "optimize.run", &format!("{{\"id\":\"{id}\",\"status\":\"{status}\"}}")).await.ok();
        Ok(OptimizationRunInfo { id: run_id, ..info })
    }

    pub async fn history(&self, limit: i64) -> Result<Vec<OptimizationRunInfo>, String> {
        self.opt().list_runs(limit).await.map_err(|e| e.to_string())
    }

    pub async fn rollback_run(&self, run_id: i64) -> Result<(), String> {
        let snap = self
            .opt()
            .run_snapshot(run_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Execução não encontrada".to_string())?;
        self.protection().rollback(snap).await.map_err(|e| e.to_string())?;
        self.opt().set_status(run_id, "reverted").await.map_err(|e| e.to_string())?;
        Ok(())
    }
}

async fn run_suite(suite: &str) -> Result<BenchmarkResult, String> {
    let s = suite.to_string();
    tokio::task::spawn_blocking(move || match s.as_str() {
        "ram-1.0.0" => run_ram(5),
        "io-1.0.0" => run_io(3),
        "complete-1.0.0" => run_complete(3),
        _ => run_cpu(5),
    })
    .await
    .map_err(|e| format!("benchmark falhou: {e}"))
}

fn decide(comp: &PerfComparison) -> OptDecision {
    if !comp.reliable {
        return OptDecision::Inconclusive;
    }
    let primary = comp.rows.iter().find(|r| r.metric == "cpu_multi").or_else(|| comp.rows.first());
    match primary.map(|r| r.verdict) {
        Some(PerfVerdict::Gain) => OptDecision::Keep,
        Some(PerfVerdict::Loss) => OptDecision::Revert,
        Some(PerfVerdict::NoChange) => OptDecision::Revert,
        _ => OptDecision::Inconclusive,
    }
}
