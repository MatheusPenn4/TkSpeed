//! TkOptimizationEngine — executa o pipeline e decide com evidência.
//! Aplica/verifica/reverte de forma GENÉRICA (ReversibleAction); cada otimização
//! só declara `plan()` e `validation()`.

use tk_contracts::{
    BenchmarkResult, OptDecision, OptimizationInfo, OptimizationRunInfo, PerfComparison, PerfVerdict,
    StartupItem,
};
use crate::configs::{ConfigMeta, ConfigRegistry, ConfigRisk};
use crate::evidence::{extract_primary_gain, EvidenceRepo};
use crate::machine;
use crate::profiles::executor::{build_action, ConfigAction};
use tk_perflab::{build_noise_profile, compare, run_complete, run_cpu, run_io, run_ram};
use tk_platform_win::registry;
use tk_rollback::{apply_actions, verify_actions, ProtectionService, ReversibleAction};
use tk_storage::{now_ms, AuditRepo, Db, OptRepo, PerfRepo};

use tk_storage::session_source;
use crate::catalog::{self, Validation};

/// Chave Run do usuário (HKCU) — inicialização reversível sem elevação.
const STARTUP_RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

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
    fn evidence(&self) -> EvidenceRepo {
        EvidenceRepo::new(self.db.clone())
    }

    pub fn catalog_info(&self) -> Vec<OptimizationInfo> {
        catalog::catalog_info()
    }

    pub fn startup_items(&self) -> Vec<StartupItem> {
        catalog::startup_items()
    }

    /// Executa o loop completo para uma otimização e devolve o registro com evidência.
    ///
    /// Resolve o `id` em dois lugares: primeiro no catálogo (catalog.rs) e, se não
    /// encontrar, no ConfigRegistry (configs/registry.rs). Sem esse fallback, configs
    /// recomendadas pelo advisor (ex.: "Plano de Energia — Balanceado") chegavam ao
    /// engine como id puro e falhavam com "Otimização não encontrada" — UX-004.
    pub async fn run(&self, id: &str) -> Result<OptimizationRunInfo, String> {
        if let Some(opt) = catalog::get(id) {
            return self.run_catalog(id, opt).await;
        }
        if let Some(cfg) = ConfigRegistry::new().find(id) {
            return self.run_config(id, cfg.meta()).await;
        }
        Err("Otimização não encontrada".to_string())
    }

    /// Caminho do catálogo (catalog.rs): plan() → snapshot → validação por suite.
    async fn run_catalog(
        &self,
        id: &str,
        opt: Box<dyn catalog::Optimization>,
    ) -> Result<OptimizationRunInfo, String> {
        let meta = opt.meta();

        // plan + snapshot (sem snapshot → não aplica)
        let actions = opt.plan()?;
        let fp = machine::fingerprint();
        let snapshot_id = self
            .protection()
            .create_snapshot(&format!("optimize:{id}"), &actions, Some(&fp))
            .await
            .map_err(|e| e.to_string())?;
        self.audit().log("user", "optimize.intent", &format!("{{\"id\":\"{id}\"}}")).await.ok();

        match opt.validation() {
            Validation::Benchmark(suite) => self.run_benchmarked(id, &meta, snapshot_id, &actions, suite, &fp).await,
            Validation::SpaceFreed => self.run_space_freed(id, &meta, snapshot_id, &actions, &fp).await,
            Validation::Manual => self.run_manual(id, &meta, snapshot_id, &actions, &fp).await,
        }
    }

    /// Caminho do ConfigRegistry: converte a config em ReversibleAction via o
    /// executor e aplica pelo mesmo pipeline reversível (snapshot → aplica → verifica).
    /// Configs ainda não suportadas pelo executor devolvem o MOTIVO real
    /// (ex.: "requer chave HKLM…") — nunca "Otimização não encontrada".
    async fn run_config(&self, id: &str, m: &ConfigMeta) -> Result<OptimizationRunInfo, String> {
        let action = match build_action(id)? {
            ConfigAction::Executable(a) => a,
            ConfigAction::Unsupported { reason } => return Err(reason.to_string()),
        };
        let risk = match m.risk {
            ConfigRisk::Safe => "Safe",
            ConfigRisk::Moderate => "Moderate",
            ConfigRisk::Advanced => "Advanced",
        };
        let meta = OptimizationInfo {
            id: m.id.into(),
            name: m.name.into(),
            description: m.description.into(),
            category: m.category.as_str().into(),
            risk: risk.into(),
            expected_impact: m.description.into(),
            requires_elevation: m.requires_elevation,
            requires_reboot: m.requires_reboot,
        };
        let actions = vec![action];
        let fp = machine::fingerprint();
        let snapshot_id = self
            .protection()
            .create_snapshot(&format!("optimize:{id}"), &actions, Some(&fp))
            .await
            .map_err(|e| e.to_string())?;
        self.audit().log("user", "optimize.intent", &format!("{{\"id\":\"{id}\"}}")).await.ok();
        self.run_manual(id, &meta, snapshot_id, &actions, &fp).await
    }

    /// Caminho com benchmark antes/depois + Confidence Engine.
    async fn run_benchmarked(
        &self,
        id: &str,
        meta: &OptimizationInfo,
        snapshot_id: i64,
        actions: &[ReversibleAction],
        suite: &str,
        fp: &str,
    ) -> Result<OptimizationRunInfo, String> {
        let before = run_suite(suite).await?;
        let before_id = self.perf().save_session(&format!("antes: {}", meta.name), &before, Some(fp), session_source::OPTIMIZATION_CATALOG).await.map_err(|e| e.to_string())?;

        if let Err(e) = apply_actions(actions) {
            let _ = self.protection().rollback(snapshot_id).await;
            return self.finish(id, meta, snapshot_id, Some(before_id), None, "failed", OptDecision::Inconclusive, None, 0, &format!("Falha ao aplicar: {e}"), fp).await;
        }
        if !verify_actions(actions).unwrap_or(false) {
            let _ = self.protection().rollback(snapshot_id).await;
            return self.finish(id, meta, snapshot_id, Some(before_id), None, "reverted", OptDecision::Revert, None, 0, "Pós-condição não confirmada — revertido.", fp).await;
        }

        let after = run_suite(suite).await?;
        let after_id = self.perf().save_session(&format!("depois: {}", meta.name), &after, Some(fp), session_source::OPTIMIZATION_CATALOG).await.map_err(|e| e.to_string())?;

        let hist = self.perf().metrics_by_suite(suite).await.map_err(|e| e.to_string())?;
        let noise = build_noise_profile(suite, &hist);
        let comp = compare(&before, &after, &noise);
        let decision = decide(&comp);
        let confidence = comp.confidence;
        let msg = comp.summary.clone();

        let (status, do_rollback) = benchmark_outcome(decision);
        if do_rollback {
            let _ = self.protection().rollback(snapshot_id).await;
        }
        self.finish(id, meta, snapshot_id, Some(before_id), Some(after_id), status, decision, Some(comp), confidence, &msg, fp).await
    }

    /// Caminho de limpeza: evidência = espaço liberado (sem benchmark).
    async fn run_space_freed(
        &self,
        id: &str,
        meta: &OptimizationInfo,
        snapshot_id: i64,
        actions: &[ReversibleAction],
        fp: &str,
    ) -> Result<OptimizationRunInfo, String> {
        let applied = apply_actions(actions);
        let (status, decision) = space_freed_decision(applied.is_ok());
        match applied {
            Ok(freed) => {
                let mb = freed as f64 / 1_000_000.0;
                self.finish(id, meta, snapshot_id, None, None, status, decision, None, 100, &format!("Liberados {mb:.1} MB para a quarentena (reversível)."), fp).await
            }
            Err(e) => {
                let _ = self.protection().rollback(snapshot_id).await;
                self.finish(id, meta, snapshot_id, None, None, status, decision, None, 0, &format!("Falha na limpeza: {e}"), fp).await
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
        fp: &str,
    ) -> Result<OptimizationRunInfo, String> {
        if let Err(e) = apply_actions(actions) {
            let _ = self.protection().rollback(snapshot_id).await;
            let (status, decision) = manual_decision(false);
            return self.finish(id, meta, snapshot_id, None, None, status, decision, None, 0, &format!("Falha ao aplicar: {e}"), fp).await;
        }
        let _ = verify_actions(actions);
        let (status, decision) = manual_decision(true);
        self.finish(
            id, meta, snapshot_id, None, None, status, decision, None, 0,
            "Aplicado e reversível. Comprove com uma captura de FPS no jogo (antes/depois) para manter com evidência.",
            fp,
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
        fp: &str,
    ) -> Result<OptimizationRunInfo, String> {
        // Extrair ganho ANTES de mover `comparison` para dentro de `info`.
        let evidence_gain = if matches!(decision, OptDecision::Keep) {
            comparison.as_ref().and_then(extract_primary_gain)
        } else {
            None
        };

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
        let run_id = self.opt().save_run(&info, snapshot_id, Some(fp), session_source::OPTIMIZATION_CATALOG).await.map_err(|e| e.to_string())?;
        self.audit().log("user", "optimize.run", &format!("{{\"id\":\"{id}\",\"status\":\"{status}\"}}")).await.ok();

        // Keep + comparação confiável → acumular evidência histórica.
        if let Some(gain) = evidence_gain {
            let relevance: Vec<String> = ConfigRegistry::new()
                .find(id)
                .map(|c| c.meta().benchmark_relevance.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default();
            self.evidence()
                .record_success(fp, id, session_source::OPTIMIZATION_CATALOG, &relevance, gain)
                .await
                .ok();
        }

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

    /// Desabilita um item de inicialização do usuário (HKCU\...\Run) de forma
    /// REVERSÍVEL, usando a mesma infra validada em A2: cria snapshot → remove o
    /// valor → verifica; falha reverte. Reverter = restaurar o snapshot pelo
    /// Rollback Center. HKLM exige admin e não é suportado aqui (honestidade).
    /// Retorna o id do snapshot criado.
    pub async fn disable_startup(&self, name: &str) -> Result<i64, String> {
        let current = registry::read_string(STARTUP_RUN_KEY, name)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| {
                "Item de inicialização não encontrado em HKCU (itens HKLM exigem administrador).".to_string()
            })?;

        // Desabilitar = remover o valor (new: None). O snapshot guarda o comando
        // original (old: Some), então o rollback o reescreve = reabilita.
        let action = ReversibleAction::RegistryHkcu {
            subkey: STARTUP_RUN_KEY.into(),
            name: name.into(),
            old: Some(current),
            new: None,
        };

        let fp = machine::fingerprint();
        let snap_id = self
            .protection()
            .create_snapshot(&format!("startup:disable:{name}"), std::slice::from_ref(&action), Some(&fp))
            .await
            .map_err(|e| e.to_string())?;

        if let Err(e) = apply_actions(std::slice::from_ref(&action)) {
            let _ = self.protection().rollback(snap_id).await;
            return Err(format!("Falha ao desabilitar: {e}"));
        }
        if !verify_actions(std::slice::from_ref(&action)).unwrap_or(false) {
            let _ = self.protection().rollback(snap_id).await;
            return Err("Não foi possível confirmar a desativação — revertido.".into());
        }

        self.audit()
            .log("user", "startup.disabled", &format!("{{\"name\":\"{name}\",\"snapshot\":{snap_id}}}"))
            .await
            .ok();
        Ok(snap_id)
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

/// Caminho benchmark: mapeia a decisão para (status persistido, se deve reverter).
/// Keep mantém; Revert e Inconclusive revertem o snapshot (sem evidência de ganho).
fn benchmark_outcome(decision: OptDecision) -> (&'static str, bool) {
    match decision {
        OptDecision::Keep => ("kept", false),
        OptDecision::Revert => ("reverted", true),
        OptDecision::Inconclusive => ("inconclusive", true),
    }
}

/// Política de limpeza (SpaceFreed): aplicação OK → mantém; falha → inconclusivo (revertido).
fn space_freed_decision(applied_ok: bool) -> (&'static str, OptDecision) {
    if applied_ok {
        ("kept", OptDecision::Keep)
    } else {
        ("failed", OptDecision::Inconclusive)
    }
}

/// Política do caminho manual: sempre inconclusivo (pende de evidência de FPS);
/// aplicado fica reversível, falha é revertida.
fn manual_decision(applied_ok: bool) -> (&'static str, OptDecision) {
    if applied_ok {
        ("applied", OptDecision::Inconclusive)
    } else {
        ("failed", OptDecision::Inconclusive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tk_contracts::ComparisonRow;

    fn row(metric: &str, verdict: PerfVerdict) -> ComparisonRow {
        ComparisonRow {
            metric: metric.into(),
            before: 100.0,
            after: 110.0,
            delta_pct: 10.0,
            margin_pct: 3.0,
            verdict,
            unit: "score".into(),
        }
    }

    fn comp(reliable: bool, rows: Vec<ComparisonRow>) -> PerfComparison {
        PerfComparison { before_id: 1, after_id: 2, rows, summary: String::new(), confidence: 80, reliable }
    }

    // ── decide(): mapeamento veredito → decisão (caminho benchmark) ──
    #[test]
    fn gain_keeps() {
        assert_eq!(decide(&comp(true, vec![row("cpu_multi", PerfVerdict::Gain)])), OptDecision::Keep);
    }

    #[test]
    fn loss_reverts() {
        assert_eq!(decide(&comp(true, vec![row("cpu_multi", PerfVerdict::Loss)])), OptDecision::Revert);
    }

    #[test]
    fn no_change_reverts() {
        assert_eq!(decide(&comp(true, vec![row("cpu_multi", PerfVerdict::NoChange)])), OptDecision::Revert);
    }

    #[test]
    fn unstable_is_inconclusive() {
        assert_eq!(decide(&comp(true, vec![row("cpu_multi", PerfVerdict::Unstable)])), OptDecision::Inconclusive);
    }

    #[test]
    fn unreliable_is_inconclusive_even_with_gain() {
        // Sessão não confiável NUNCA declara ganho (anti-alegação-falsa).
        assert_eq!(decide(&comp(false, vec![row("cpu_multi", PerfVerdict::Gain)])), OptDecision::Inconclusive);
    }

    #[test]
    fn prefers_cpu_multi_as_primary_metric() {
        let c = comp(true, vec![row("cpu_single", PerfVerdict::Loss), row("cpu_multi", PerfVerdict::Gain)]);
        assert_eq!(decide(&c), OptDecision::Keep);
    }

    #[test]
    fn empty_rows_is_inconclusive() {
        assert_eq!(decide(&comp(true, vec![])), OptDecision::Inconclusive);
    }

    // ── benchmark_outcome(): decisão → (status, reverter?) ──
    #[test]
    fn keep_does_not_rollback() {
        assert_eq!(benchmark_outcome(OptDecision::Keep), ("kept", false));
    }

    #[test]
    fn revert_decision_rolls_back() {
        assert_eq!(benchmark_outcome(OptDecision::Revert), ("reverted", true));
    }

    #[test]
    fn inconclusive_decision_rolls_back() {
        assert_eq!(benchmark_outcome(OptDecision::Inconclusive), ("inconclusive", true));
    }

    // ── SpaceFreed → Keep ── (e falha → Inconclusive)
    #[test]
    fn space_freed_ok_keeps() {
        assert_eq!(space_freed_decision(true), ("kept", OptDecision::Keep));
    }

    #[test]
    fn space_freed_err_is_inconclusive() {
        assert_eq!(space_freed_decision(false), ("failed", OptDecision::Inconclusive));
    }

    // ── Manual → Inconclusive ──
    #[test]
    fn manual_applied_is_inconclusive() {
        assert_eq!(manual_decision(true), ("applied", OptDecision::Inconclusive));
    }

    #[test]
    fn manual_failed_is_inconclusive() {
        assert_eq!(manual_decision(false), ("failed", OptDecision::Inconclusive));
    }
}
