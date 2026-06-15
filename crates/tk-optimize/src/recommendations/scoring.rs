//! Funções puras de scoring para o Recommendation Engine.
//!
//! Ordem de prioridade:
//!   1. Evidência histórica desta máquina (fingerprint) — maior peso
//!   2. Sem evidência → base por capability readiness
//!   3. Penalidade de risco (Safe=1.0, Moderate=0.85, Advanced=0.60)
//!   4. Penalidade de reboot (requires_reboot=true → ×0.90)
//!
//! Configs com evidência de falha (0 execuções bem-sucedidas) retornam 0.0
//! e são excluídas das recomendações.

use crate::configs::ConfigRisk;
use crate::evidence::EvidenceRecord;
use crate::profiles::evidence::ProfileEvidenceRecord;

/// Score base para configs sem evidência histórica.
pub fn capability_base(risk: &ConfigRisk) -> f64 {
    match risk {
        ConfigRisk::Safe => 10.0,
        ConfigRisk::Moderate => 8.0,
        ConfigRisk::Advanced => 4.0,
    }
}

/// Penalidade multiplicativa pelo nível de risco.
pub fn risk_penalty(risk: &ConfigRisk) -> f64 {
    match risk {
        ConfigRisk::Safe => 1.0,
        ConfigRisk::Moderate => 0.85,
        ConfigRisk::Advanced => 0.60,
    }
}

/// Penalidade multiplicativa para mudanças que exigem reinicialização.
pub fn reboot_penalty(requires_reboot: bool) -> f64 {
    if requires_reboot { 0.90 } else { 1.0 }
}

/// Calcula o score de uma recomendação de configuração individual.
///
/// Quando há evidência: `confidence × avg_gain × success_rate × risk × reboot`
/// Sem evidência: `capability_base(risk) × risk × reboot`
/// Com evidência de falha (0 sucessos): retorna 0.0
pub fn score_config(
    risk: &ConfigRisk,
    requires_reboot: bool,
    evidence: Option<&EvidenceRecord>,
) -> f64 {
    let base = match evidence {
        Some(e) => {
            let success_rate = e.successful_executions as f64 / e.executions.max(1) as f64;
            e.confidence as f64 * e.average_gain.max(0.0) * success_rate
        }
        None => capability_base(risk),
    };
    base * risk_penalty(risk) * reboot_penalty(requires_reboot)
}

/// Calcula o score de uma recomendação de perfil.
///
/// Se o perfil requer FPS e fps não está ready → 0.0 (excluído).
/// Com evidência: `confidence × avg_gain × success_rate`
/// Sem evidência: 8.0 (base de perfil — acima do benchmark rec de 5.0, abaixo de configs com evidência)
pub fn score_profile(
    requires_fps: bool,
    fps_ready: bool,
    evidence: Option<&ProfileEvidenceRecord>,
) -> f64 {
    if requires_fps && !fps_ready {
        return 0.0;
    }
    match evidence {
        Some(e) => {
            let success_rate = e.successful_executions as f64 / e.executions.max(1) as f64;
            e.confidence as f64 * e.average_gain.max(0.0) * success_rate
        }
        None => 8.0,
    }
}

/// Constrói string legível de motivo a partir de evidência de config.
pub fn reason_from_config_evidence(evidence: Option<&EvidenceRecord>) -> String {
    match evidence {
        Some(e) if e.executions > 0 => format!(
            "{} execuções, {} sucessos, ganho médio +{:.1}%",
            e.executions, e.successful_executions, e.average_gain
        ),
        _ => "Nenhuma execução anterior nesta máquina".to_string(),
    }
}

/// Constrói string legível de motivo a partir de evidência de perfil.
pub fn reason_from_profile_evidence(evidence: Option<&ProfileEvidenceRecord>) -> String {
    match evidence {
        Some(e) if e.executions > 0 => format!(
            "{} ativações, {} sucessos, ganho médio +{:.1}%",
            e.executions, e.successful_executions, e.average_gain
        ),
        _ => "Nenhuma ativação anterior deste perfil nesta máquina".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── capability_base ──────────────────────────────────────────────────────

    #[test]
    fn capability_base_safe_is_10() {
        assert_eq!(capability_base(&ConfigRisk::Safe), 10.0);
    }

    #[test]
    fn capability_base_moderate_is_8() {
        assert_eq!(capability_base(&ConfigRisk::Moderate), 8.0);
    }

    #[test]
    fn capability_base_advanced_is_4() {
        assert_eq!(capability_base(&ConfigRisk::Advanced), 4.0);
    }

    // ── risk_penalty ─────────────────────────────────────────────────────────

    #[test]
    fn risk_penalty_safe_is_1() {
        assert_eq!(risk_penalty(&ConfigRisk::Safe), 1.0);
    }

    #[test]
    fn risk_penalty_moderate_is_0_85() {
        assert_eq!(risk_penalty(&ConfigRisk::Moderate), 0.85);
    }

    #[test]
    fn risk_penalty_advanced_is_0_60() {
        assert_eq!(risk_penalty(&ConfigRisk::Advanced), 0.60);
    }

    // ── reboot_penalty ───────────────────────────────────────────────────────

    #[test]
    fn reboot_penalty_false_is_1() {
        assert_eq!(reboot_penalty(false), 1.0);
    }

    #[test]
    fn reboot_penalty_true_is_0_90() {
        assert_eq!(reboot_penalty(true), 0.90);
    }

    // ── score_config ─────────────────────────────────────────────────────────

    #[test]
    fn score_config_no_evidence_safe_no_reboot() {
        let s = score_config(&ConfigRisk::Safe, false, None);
        assert_eq!(s, 10.0);
    }

    #[test]
    fn score_config_no_evidence_advanced_with_reboot() {
        let s = score_config(&ConfigRisk::Advanced, true, None);
        let expected = 4.0 * 0.60 * 0.90;
        assert!((s - expected).abs() < 1e-9);
    }

    #[test]
    fn score_config_no_evidence_moderate_no_reboot() {
        let s = score_config(&ConfigRisk::Moderate, false, None);
        assert_eq!(s, 8.0 * 0.85);
    }

    #[test]
    fn score_config_evidence_high_confidence_high_gain() {
        let ev = make_config_evidence(95, 7.4, 12, 14);
        let s = score_config(&ConfigRisk::Safe, false, Some(&ev));
        // 95 × 7.4 × (12/14) × 1.0 × 1.0
        let expected = 95.0 * 7.4 * (12.0 / 14.0);
        assert!((s - expected).abs() < 0.01);
    }

    #[test]
    fn score_config_evidence_zero_successful_returns_zero() {
        let ev = make_config_evidence(0, 0.0, 0, 5);
        let s = score_config(&ConfigRisk::Safe, false, Some(&ev));
        assert_eq!(s, 0.0);
    }

    #[test]
    fn score_config_evidence_beats_no_evidence() {
        let ev = make_config_evidence(25, 3.0, 1, 2);
        let with_ev = score_config(&ConfigRisk::Safe, false, Some(&ev));
        let without_ev = score_config(&ConfigRisk::Safe, false, None);
        // 25 × 3.0 × 0.5 = 37.5 > 10.0
        assert!(with_ev > without_ev);
    }

    #[test]
    fn score_config_requires_reboot_reduces_score() {
        let with_reboot = score_config(&ConfigRisk::Safe, true, None);
        let no_reboot = score_config(&ConfigRisk::Safe, false, None);
        assert!(with_reboot < no_reboot);
    }

    // ── score_profile ────────────────────────────────────────────────────────

    #[test]
    fn score_profile_fps_required_but_not_ready_returns_zero() {
        let s = score_profile(true, false, None);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn score_profile_fps_required_and_ready_returns_base() {
        let s = score_profile(true, true, None);
        assert_eq!(s, 8.0);
    }

    #[test]
    fn score_profile_no_fps_required_returns_base() {
        let s = score_profile(false, false, None);
        assert_eq!(s, 8.0);
    }

    #[test]
    fn score_profile_with_evidence_scales() {
        let ev = make_profile_evidence(75, 5.0, 7, 9);
        let s = score_profile(false, false, Some(&ev));
        let expected = 75.0 * 5.0 * (7.0 / 9.0);
        assert!((s - expected).abs() < 0.01);
    }

    #[test]
    fn score_profile_evidence_beats_base() {
        let ev = make_profile_evidence(50, 4.0, 3, 4);
        let with_ev = score_profile(false, false, Some(&ev));
        let without_ev = score_profile(false, false, None);
        // 50 × 4.0 × 0.75 = 150.0 > 8.0
        assert!(with_ev > without_ev);
    }

    // ── reason strings ───────────────────────────────────────────────────────

    #[test]
    fn reason_config_no_evidence() {
        let r = reason_from_config_evidence(None);
        assert!(r.contains("Nenhuma execução"));
    }

    #[test]
    fn reason_config_with_evidence() {
        let ev = make_config_evidence(75, 4.5, 7, 9);
        let r = reason_from_config_evidence(Some(&ev));
        assert!(r.contains("9 execuções"));
        assert!(r.contains("7 sucessos"));
        assert!(r.contains("+4.5%"));
    }

    #[test]
    fn reason_profile_no_evidence() {
        let r = reason_from_profile_evidence(None);
        assert!(r.contains("Nenhuma ativação"));
    }

    #[test]
    fn reason_profile_with_evidence() {
        let ev = make_profile_evidence(50, 3.2, 3, 4);
        let r = reason_from_profile_evidence(Some(&ev));
        assert!(r.contains("4 ativações"));
        assert!(r.contains("3 sucessos"));
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    fn make_config_evidence(
        confidence: u8,
        average_gain: f64,
        successful_executions: u32,
        executions: u32,
    ) -> EvidenceRecord {
        EvidenceRecord {
            fingerprint: "test-fp".into(),
            config_id: "test-config".into(),
            source: "test".into(),
            benchmark_relevance: vec![],
            executions,
            successful_executions,
            average_gain,
            confidence,
            updated_at: 0,
        }
    }

    fn make_profile_evidence(
        confidence: u8,
        average_gain: f64,
        successful_executions: u32,
        executions: u32,
    ) -> ProfileEvidenceRecord {
        ProfileEvidenceRecord {
            profile_id: "test-profile".into(),
            fingerprint: "test-fp".into(),
            executions,
            successful_executions,
            average_gain,
            confidence,
            updated_at: 0,
        }
    }
}
