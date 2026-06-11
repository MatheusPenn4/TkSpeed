//! TkSpeed Score — modelo de pontuação 0–1000 (ver docs/13-TKSPEED-SCORE.md).

use crate::AnalysisInput;
use tk_contracts::{Classification, Finding, ScoreBreakdown, Severity, TkSpeedScore};

pub const SCORE_VERSION: &str = "1.0.0";

/// Pesos do score MVP — usa apenas dimensões REAIS medidas (CPU/RAM/Storage)
/// + uma "saúde" derivada dos gargalos detectados. Somam 1.0.
const W_CPU_MVP: f64 = 0.35;
const W_RAM_MVP: f64 = 0.30;
const W_STORAGE_MVP: f64 = 0.20;
const W_HEALTH_MVP: f64 = 0.15;

fn pct(v: f64) -> u16 {
    v.clamp(0.0, 100.0).round() as u16
}

/// Calcula o TkSpeed Score (0–1000) a partir de telemetria real + findings.
///
/// Cálculo (MVP):
/// - CPU      = folga de processamento (100 − uso médio).
/// - RAM      = folga de memória (100 − uso %), com teto se a RAM livre for crítica.
/// - Storage  = base por tipo (SSD 85 / HDD 55) menos penalidade por pouco espaço.
/// - Saúde    = 100 menos penalidade por severidade de cada gargalo detectado.
/// - Total    = média ponderada (35/30/20/15) × 10  → faixa 0–1000.
///
/// Dimensões ainda não medidas no MVP (GPU/rede/temperatura/jogos) NÃO entram no
/// total; no breakdown recebem um valor neutro derivado das medidas (não exibido).
pub fn compute_mvp(input: &AnalysisInput, findings: &[Finding]) -> TkSpeedScore {
    let cpu = pct(100.0 - input.cpu_avg);

    let mut ram_v = 100.0 - input.ram_used_pct;
    if input.ram_avail_gb < 1.0 {
        ram_v = ram_v.min(20.0);
    } else if input.ram_avail_gb < 2.0 {
        ram_v = ram_v.min(40.0);
    }
    let ram = pct(ram_v);

    let mut storage_v = if input.disk_is_ssd { 85.0 } else { 55.0 };
    if input.disk_free_pct < 5.0 {
        storage_v -= 50.0;
    } else if input.disk_free_pct < 10.0 {
        storage_v -= 30.0;
    }
    let storage = pct(storage_v);

    // Saúde a partir dos gargalos detectados (gargalos derrubam o score).
    let mut health: i32 = 100;
    for f in findings {
        health -= match f.severity {
            Severity::Critical => 35,
            Severity::High => 20,
            Severity::Medium => 10,
            Severity::Low => 4,
            Severity::Info => 0,
        };
    }
    let stability = health.clamp(0, 100) as u16;

    let total_f = (cpu as f64) * W_CPU_MVP
        + (ram as f64) * W_RAM_MVP
        + (storage as f64) * W_STORAGE_MVP
        + (stability as f64) * W_HEALTH_MVP;
    let total = (total_f * 10.0).round().clamp(0.0, 1000.0) as u16;

    // Neutro para dimensões não medidas (não exibido na UI do MVP).
    let neutral = ((cpu as u32 + ram as u32 + storage as u32) / 3) as u16;

    TkSpeedScore {
        total,
        classification: classify(total),
        breakdown: ScoreBreakdown {
            cpu,
            gpu: neutral,
            ram,
            storage,
            windows: neutral,
            network: neutral,
            temperature: neutral,
            games: neutral,
            stability,
        },
        score_version: SCORE_VERSION.to_string(),
    }
}

/// Pesos por categoria (somam 1.0).
const W: ScoreWeights = ScoreWeights {
    cpu: 0.16, gpu: 0.16, ram: 0.12, storage: 0.12, windows: 0.12,
    network: 0.08, temperature: 0.12, games: 0.06, stability: 0.06,
};

struct ScoreWeights {
    cpu: f64, gpu: f64, ram: f64, storage: f64, windows: f64,
    network: f64, temperature: f64, games: f64, stability: f64,
}

/// Recebe subscores 0..=100 por categoria e produz o score final 0..=1000.
pub fn compute(b: &ScoreBreakdown) -> TkSpeedScore {
    let total = (b.cpu as f64 / 100.0 * W.cpu
        + b.gpu as f64 / 100.0 * W.gpu
        + b.ram as f64 / 100.0 * W.ram
        + b.storage as f64 / 100.0 * W.storage
        + b.windows as f64 / 100.0 * W.windows
        + b.network as f64 / 100.0 * W.network
        + b.temperature as f64 / 100.0 * W.temperature
        + b.games as f64 / 100.0 * W.games
        + b.stability as f64 / 100.0 * W.stability)
        * 1000.0;
    let total = total.round() as u16;

    TkSpeedScore {
        total,
        classification: classify(total),
        breakdown: b.clone(),
        score_version: SCORE_VERSION.into(),
    }
}

fn classify(total: u16) -> Classification {
    match total {
        0..=199 => Classification::Critico,
        200..=449 => Classification::Regular,
        450..=699 => Classification::Bom,
        700..=899 => Classification::Excelente,
        _ => Classification::Elite,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn perfect_machine_is_elite() {
        let b = ScoreBreakdown {
            cpu: 100, gpu: 100, ram: 100, storage: 100, windows: 100,
            network: 100, temperature: 100, games: 100, stability: 100,
        };
        let s = compute(&b);
        assert_eq!(s.total, 1000);
        assert!(matches!(s.classification, Classification::Elite));
    }

    #[test]
    fn idle_ssd_machine_scores_high() {
        let input = AnalysisInput {
            cpu_avg: 8.0,
            cpu_peak: 25.0,
            ram_used_pct: 35.0,
            ram_total_gb: 32.0,
            ram_avail_gb: 20.0,
            disk_free_pct: 60.0,
            disk_is_ssd: true,
            samples: 60,
        };
        let s = compute_mvp(&input, &[]);
        // Máquina ociosa/SSD sem gargalos deve pontuar alto (faixa Excelente+).
        // Valor determinístico ≈ 837 com o modelo de "folga" atual.
        assert!(s.total >= 800, "esperado alto, veio {}", s.total);
        assert!(matches!(s.classification, Classification::Excelente | Classification::Elite));
    }

    #[test]
    fn high_cpu_load_lowers_score() {
        let calm = AnalysisInput { cpu_avg: 10.0, ram_used_pct: 30.0, ram_avail_gb: 20.0, ram_total_gb: 32.0, disk_free_pct: 60.0, disk_is_ssd: true, cpu_peak: 20.0, samples: 60 };
        let busy = AnalysisInput { cpu_avg: 96.0, ram_used_pct: 30.0, ram_avail_gb: 20.0, ram_total_gb: 32.0, disk_free_pct: 60.0, disk_is_ssd: true, cpu_peak: 99.0, samples: 60 };
        let calm_score = compute_mvp(&calm, &[]).total;
        let busy_score = compute_mvp(&busy, &[]).total;
        assert!(busy_score < calm_score, "{busy_score} deveria ser < {calm_score}");
    }

    #[test]
    fn full_disk_hurts_storage_subscore() {
        let input = AnalysisInput { cpu_avg: 10.0, ram_used_pct: 30.0, ram_avail_gb: 20.0, ram_total_gb: 32.0, disk_free_pct: 3.0, disk_is_ssd: true, cpu_peak: 20.0, samples: 60 };
        let s = compute_mvp(&input, &[]);
        assert!(s.breakdown.storage <= 40, "storage={} deveria ser baixo", s.breakdown.storage);
    }
}
