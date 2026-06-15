//! Recommendation Engine — gera recomendações ordenadas por score.
//!
//! Puramente data-driven: sem IA, sem LLM, sem heurísticas inventadas.
//! Só usa: evidência real acumulada + capacidades detectadas + confidence engine.

use crate::capabilities::status;
use crate::configs::ConfigRegistry;
use crate::profiles::bundled;

use super::model::{Recommendation, RecommendationContext, RecommendationKind};
use super::scoring;

/// Motor de recomendações — stateless, puramente funcional.
pub struct RecommendationEngine;

impl RecommendationEngine {
    /// Gera todas as recomendações aplicáveis para o contexto fornecido.
    /// Não ordena — use `top_n` para resultado final.
    pub fn generate(ctx: &RecommendationContext) -> Vec<Recommendation> {
        let mut recs = Vec::new();

        let fps_ready = ctx
            .capabilities
            .iter()
            .any(|c| c.id == "fps_measurement" && c.status == status::READY);

        // ── Config recommendations ────────────────────────────────────────
        let registry = ConfigRegistry::new();
        for config in registry.all() {
            let meta = config.meta();

            if ctx.active_config_ids.iter().any(|id| id == meta.id) {
                continue;
            }
            if !ctx.applicable_config_ids.iter().any(|id| id == meta.id) {
                continue;
            }

            let evidence = ctx.config_evidence.iter().find(|e| e.config_id == meta.id);
            let score = scoring::score_config(&meta.risk, meta.requires_reboot, evidence);
            if score <= 0.0 {
                continue;
            }

            recs.push(Recommendation {
                id: format!("config:{}", meta.id),
                title: meta.name.to_string(),
                description: meta.description.to_string(),
                kind: RecommendationKind::Config,
                score,
                confidence: evidence.map(|e| e.confidence).unwrap_or(0),
                estimated_gain: evidence.and_then(|e| {
                    if e.successful_executions > 0 {
                        Some(e.average_gain)
                    } else {
                        None
                    }
                }),
                risk: meta.risk.as_str().to_string(),
                requires_reboot: meta.requires_reboot,
                reason: scoring::reason_from_config_evidence(evidence),
            });
        }

        // ── Profile recommendations ───────────────────────────────────────
        for profile in bundled::all() {
            if ctx.active_profile.as_deref() == Some(profile.id.as_str()) {
                continue;
            }
            if profile.requires_fps && !fps_ready {
                continue;
            }

            let evidence = ctx
                .profile_evidence
                .iter()
                .find(|e| e.profile_id == profile.id);

            let score = scoring::score_profile(profile.requires_fps, fps_ready, evidence);
            if score <= 0.0 {
                continue;
            }

            recs.push(Recommendation {
                id: format!("profile:{}", profile.id),
                title: profile.name.clone(),
                description: profile.description.unwrap_or_default(),
                kind: RecommendationKind::Profile,
                score,
                confidence: evidence.map(|e| e.confidence).unwrap_or(0),
                estimated_gain: evidence.and_then(|e| {
                    if e.successful_executions > 0 {
                        Some(e.average_gain)
                    } else {
                        None
                    }
                }),
                risk: "safe".to_string(),
                requires_reboot: false,
                reason: scoring::reason_from_profile_evidence(evidence),
            });
        }

        // ── Benchmark recommendation ──────────────────────────────────────
        if ctx.recent_sessions.is_empty() {
            let benchmark_ready = ctx
                .capabilities
                .iter()
                .any(|c| c.id == "benchmark_engine" && c.status == status::READY);
            if benchmark_ready {
                recs.push(Recommendation {
                    id: "benchmark:initial".to_string(),
                    title: "Executar benchmark inicial".to_string(),
                    description:
                        "Nenhum benchmark foi executado ainda. Execute para estabelecer \
                         a linha de base de performance desta máquina."
                            .to_string(),
                    kind: RecommendationKind::Benchmark,
                    score: 5.0,
                    confidence: 0,
                    estimated_gain: None,
                    risk: "safe".to_string(),
                    requires_reboot: false,
                    reason: "Sem dados históricos de performance".to_string(),
                });
            }
        }

        recs
    }

    /// Retorna as `n` melhores recomendações, ordenadas por score decrescente.
    pub fn top_n(ctx: &RecommendationContext, n: usize) -> Vec<Recommendation> {
        let mut all = Self::generate(ctx);
        all.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all.truncate(n);
        all
    }
}

#[cfg(test)]
mod tests {
    use tk_contracts::BenchmarkSessionInfo;

    use crate::capabilities::Capability;
    use crate::configs::ConfigRisk;
    use crate::evidence::EvidenceRecord;
    use crate::profiles::evidence::ProfileEvidenceRecord;

    use super::*;

    // ── helpers ──────────────────────────────────────────────────────────────

    fn cap(id: &str, st: &str) -> Capability {
        Capability { id: id.into(), status: st.into(), detail: String::new() }
    }

    fn base_caps() -> Vec<Capability> {
        vec![
            cap("benchmark_engine", "ready"),
            cap("fps_measurement", "missing"),
            cap("gpu_monitoring", "unavailable"),
            cap("cpu_monitoring", "ready"),
            cap("admin_privileges", "missing"),
        ]
    }

    fn base_ctx() -> RecommendationContext {
        RecommendationContext {
            machine_fingerprint: "fp-test".into(),
            capabilities: base_caps(),
            recent_sessions: vec![],
            active_profile: None,
            active_config_ids: vec![],
            config_evidence: vec![],
            profile_evidence: vec![],
            applicable_config_ids: vec![
                "power_plan_balanced".into(),
                "visual_effects_gaming".into(),
            ],
        }
    }

    fn make_session() -> BenchmarkSessionInfo {
        BenchmarkSessionInfo {
            id: 1,
            ts: 0,
            kind: "manual".into(),
            suite_version: "complete-1.0.0".into(),
            label: "test".into(),
            metrics: vec![],
            confidence: 80,
            stable: true,
            contaminated: false,
        }
    }

    fn make_config_ev(config_id: &str, confidence: u8, gain: f64, succ: u32, total: u32) -> EvidenceRecord {
        EvidenceRecord {
            fingerprint: "fp-test".into(),
            config_id: config_id.into(),
            source: "test".into(),
            benchmark_relevance: vec![],
            executions: total,
            successful_executions: succ,
            average_gain: gain,
            confidence,
            updated_at: 0,
        }
    }

    fn make_profile_ev(profile_id: &str, confidence: u8, gain: f64, succ: u32, total: u32) -> ProfileEvidenceRecord {
        ProfileEvidenceRecord {
            profile_id: profile_id.into(),
            fingerprint: "fp-test".into(),
            executions: total,
            successful_executions: succ,
            average_gain: gain,
            confidence,
            updated_at: 0,
        }
    }

    // ── generate ─────────────────────────────────────────────────────────────

    #[test]
    fn generate_returns_applicable_configs() {
        let ctx = base_ctx();
        let recs = RecommendationEngine::generate(&ctx);
        let ids: Vec<_> = recs.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"config:power_plan_balanced"));
        assert!(ids.contains(&"config:visual_effects_gaming"));
    }

    #[test]
    fn generate_excludes_non_applicable_configs() {
        let ctx = base_ctx();
        let recs = RecommendationEngine::generate(&ctx);
        let ids: Vec<_> = recs.iter().map(|r| r.id.as_str()).collect();
        assert!(!ids.contains(&"config:gpu_hardware_scheduling"));
        assert!(!ids.contains(&"config:power_plan_high_performance"));
    }

    #[test]
    fn generate_excludes_active_config_ids() {
        let mut ctx = base_ctx();
        ctx.active_config_ids = vec!["power_plan_balanced".into()];
        let recs = RecommendationEngine::generate(&ctx);
        let ids: Vec<_> = recs.iter().map(|r| r.id.as_str()).collect();
        assert!(!ids.contains(&"config:power_plan_balanced"));
        assert!(ids.contains(&"config:visual_effects_gaming"));
    }

    #[test]
    fn generate_benchmark_rec_when_no_sessions() {
        let ctx = base_ctx();
        let recs = RecommendationEngine::generate(&ctx);
        let has_bench = recs.iter().any(|r| r.id == "benchmark:initial");
        assert!(has_bench);
    }

    #[test]
    fn generate_no_benchmark_rec_when_sessions_exist() {
        let mut ctx = base_ctx();
        ctx.recent_sessions = vec![make_session()];
        let recs = RecommendationEngine::generate(&ctx);
        let has_bench = recs.iter().any(|r| r.id == "benchmark:initial");
        assert!(!has_bench);
    }

    #[test]
    fn generate_benchmark_rec_absent_when_engine_not_ready() {
        let mut ctx = base_ctx();
        ctx.capabilities = vec![cap("benchmark_engine", "missing")];
        let recs = RecommendationEngine::generate(&ctx);
        let has_bench = recs.iter().any(|r| r.id == "benchmark:initial");
        assert!(!has_bench);
    }

    #[test]
    fn generate_competitive_excluded_when_fps_not_ready() {
        let ctx = base_ctx(); // fps_measurement = missing
        let recs = RecommendationEngine::generate(&ctx);
        let has_competitive = recs.iter().any(|r| r.id == "profile:competitive");
        assert!(!has_competitive);
    }

    #[test]
    fn generate_competitive_included_when_fps_ready() {
        let mut ctx = base_ctx();
        ctx.capabilities = vec![
            cap("benchmark_engine", "ready"),
            cap("fps_measurement", "ready"),
        ];
        let recs = RecommendationEngine::generate(&ctx);
        let has_competitive = recs.iter().any(|r| r.id == "profile:competitive");
        assert!(has_competitive);
    }

    #[test]
    fn generate_active_profile_excluded_from_profile_recs() {
        let mut ctx = base_ctx();
        ctx.active_profile = Some("balanced".into());
        let recs = RecommendationEngine::generate(&ctx);
        let has_balanced = recs.iter().any(|r| r.id == "profile:balanced");
        assert!(!has_balanced);
    }

    #[test]
    fn generate_config_with_failure_evidence_excluded() {
        let mut ctx = base_ctx();
        // 0 successful executions → confidence=0 → score=0 → excluded
        ctx.config_evidence = vec![make_config_ev("power_plan_balanced", 0, 0.0, 0, 5)];
        let recs = RecommendationEngine::generate(&ctx);
        let has = recs.iter().any(|r| r.id == "config:power_plan_balanced");
        assert!(!has);
    }

    #[test]
    fn generate_config_with_success_evidence_uses_evidence_score() {
        let mut ctx = base_ctx();
        ctx.config_evidence = vec![make_config_ev("visual_effects_gaming", 75, 4.5, 7, 9)];
        let recs = RecommendationEngine::generate(&ctx);
        let rec = recs.iter().find(|r| r.id == "config:visual_effects_gaming").unwrap();
        assert_eq!(rec.confidence, 75);
        assert_eq!(rec.estimated_gain, Some(4.5));
    }

    #[test]
    fn generate_profile_with_success_evidence_uses_evidence_score() {
        let mut ctx = base_ctx();
        ctx.profile_evidence = vec![make_profile_ev("balanced", 50, 3.0, 3, 4)];
        let recs = RecommendationEngine::generate(&ctx);
        let rec = recs.iter().find(|r| r.id == "profile:balanced").unwrap();
        assert_eq!(rec.confidence, 50);
        assert_eq!(rec.estimated_gain, Some(3.0));
    }

    #[test]
    fn generate_no_duplicate_ids() {
        let ctx = base_ctx();
        let recs = RecommendationEngine::generate(&ctx);
        let ids: Vec<_> = recs.iter().map(|r| r.id.clone()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len());
    }

    // ── top_n ─────────────────────────────────────────────────────────────────

    #[test]
    fn top_n_truncates_to_n() {
        let ctx = base_ctx();
        let result = RecommendationEngine::top_n(&ctx, 1);
        assert!(result.len() <= 1);
    }

    #[test]
    fn top_n_returns_at_most_5() {
        let ctx = base_ctx();
        let result = RecommendationEngine::top_n(&ctx, 5);
        assert!(result.len() <= 5);
    }

    #[test]
    fn top_n_sorted_descending_by_score() {
        let mut ctx = base_ctx();
        ctx.config_evidence = vec![
            make_config_ev("visual_effects_gaming", 95, 7.0, 10, 12),
            make_config_ev("power_plan_balanced", 25, 1.0, 1, 4),
        ];
        let result = RecommendationEngine::top_n(&ctx, 5);
        for w in result.windows(2) {
            assert!(w[0].score >= w[1].score, "ordering violated: {} < {}", w[0].score, w[1].score);
        }
    }

    #[test]
    fn top_n_high_evidence_ranked_above_no_evidence() {
        let mut ctx = base_ctx();
        ctx.config_evidence = vec![make_config_ev("visual_effects_gaming", 95, 7.0, 10, 12)];
        let result = RecommendationEngine::top_n(&ctx, 5);
        let visual_pos = result.iter().position(|r| r.id == "config:visual_effects_gaming");
        let balanced_pos = result.iter().position(|r| r.id == "config:power_plan_balanced");
        if let (Some(vp), Some(bp)) = (visual_pos, balanced_pos) {
            assert!(vp < bp, "high-evidence config should rank above no-evidence config");
        }
    }

    #[test]
    fn top_n_zero_returns_empty() {
        let ctx = base_ctx();
        let result = RecommendationEngine::top_n(&ctx, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn top_n_empty_applicable_returns_only_benchmark_or_profiles() {
        let mut ctx = base_ctx();
        ctx.applicable_config_ids = vec![];
        let result = RecommendationEngine::top_n(&ctx, 5);
        for r in &result {
            assert_ne!(r.kind, RecommendationKind::Config);
        }
    }

    #[test]
    fn top_5_benchmark_present_when_only_rec_available() {
        // Only benchmark recommendation available (no applicable configs, no profiles)
        let mut ctx = base_ctx();
        ctx.applicable_config_ids = vec![];
        ctx.capabilities = vec![
            cap("benchmark_engine", "ready"),
            // fps_measurement missing → competitive excluded
            cap("fps_measurement", "missing"),
        ];
        ctx.active_profile = Some("balanced".into());
        // Exclude all other profiles too — only benchmark should remain
        // Can't exclude profiles directly; just verify benchmark is present in generate
        let all = RecommendationEngine::generate(&ctx);
        let has_bench = all.iter().any(|r| r.kind == RecommendationKind::Benchmark);
        assert!(has_bench, "benchmark rec should always be generated when no sessions and engine ready");
    }

    #[test]
    fn recommendation_kind_config_has_correct_risk_field() {
        let ctx = base_ctx();
        let recs = RecommendationEngine::generate(&ctx);
        for r in recs.iter().filter(|r| r.kind == RecommendationKind::Config) {
            let valid_risks = ["safe", "moderate", "advanced"];
            assert!(
                valid_risks.contains(&r.risk.as_str()),
                "unexpected risk: {}",
                r.risk
            );
        }
    }

    #[test]
    fn config_evidence_no_successful_has_none_estimated_gain() {
        let mut ctx = base_ctx();
        ctx.config_evidence = vec![make_config_ev("visual_effects_gaming", 25, 2.0, 0, 3)];
        let recs = RecommendationEngine::generate(&ctx);
        // score = 0 → excluded entirely (0 successful means 0 confidence → 0 score)
        let rec = recs.iter().find(|r| r.id == "config:visual_effects_gaming");
        assert!(rec.is_none(), "config with 0 successful execs should be excluded");
    }

    #[test]
    fn all_profile_recs_have_safe_risk() {
        let mut ctx = base_ctx();
        ctx.capabilities = vec![
            cap("benchmark_engine", "ready"),
            cap("fps_measurement", "ready"),
        ];
        let recs = RecommendationEngine::generate(&ctx);
        for r in recs.iter().filter(|r| r.kind == RecommendationKind::Profile) {
            assert_eq!(r.risk, "safe");
        }
    }

    #[test]
    fn config_risk_advanced_has_lower_score_than_safe_same_no_evidence() {
        // power_plan_balanced is Safe (in base_ctx applicable list)
        // We can compare indirectly via scoring
        let safe_score = scoring::score_config(&ConfigRisk::Safe, false, None);
        let advanced_score = scoring::score_config(&ConfigRisk::Advanced, false, None);
        assert!(safe_score > advanced_score);
    }
}
