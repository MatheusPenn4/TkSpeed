//! PL-2d — Confidence Engine: qualidade da medição + perfil de ruído.
//!
//! - Confiança da sessão a partir do coeficiente de variação (CV) intra-sessão.
//! - Contaminação térmica via temperatura inicial/final.
//! - Perfil de ruído ENTRE sessões (a máquina aprende a própria variabilidade) —
//!   base da margem dinâmica usada pela comparação.

use std::collections::BTreeMap;

use tk_contracts::{BenchmarkMetric, NoiseEntry, NoiseProfile};

use crate::aggregate::aggregate;

// Confiança: confidence = 100 - CONF_CV_K * CV_médio(%). CV 5% → ~70.
const CONF_CV_K: f64 = 6.0;
/// Confiança mínima para considerar uma sessão "estável".
pub const STABLE_MIN: u8 = 70;
/// Aquecimento (°C) entre início/fim que sinaliza contaminação térmica.
const THERMAL_RISE_C: f64 = 15.0;
/// Temperatura final que sinaliza throttling provável.
const THERMAL_LIMIT_C: f64 = 90.0;

/// Mín. de sessões para aprender o ruído real; abaixo disso usa default conservador.
const MIN_SESSIONS_LEARNED: usize = 3;
/// k para a margem de ruído (~95% entre sessões). Usado pela comparação.
pub const NOISE_K: f64 = 2.0;

#[derive(Debug, Clone)]
pub struct SessionQuality {
    pub confidence: u8,
    pub stable: bool,
    pub contaminated: bool,
}

/// Métricas de qualidade/throughput que contam para o CV (exclui cargas/temperaturas).
fn is_quality_metric(metric: &str) -> bool {
    !matches!(metric, "cpu_load_pct")
}

/// Categoria → default de ruído (CV% típico entre sessões num laptop).
fn default_noise(metric: &str) -> f64 {
    if metric.starts_with("cpu") {
        8.0
    } else if metric.starts_with("ram") {
        8.0
    } else if metric.starts_with("io") {
        12.0
    } else if metric.starts_with("fps") || metric.starts_with("frametime") {
        6.0
    } else {
        8.0
    }
}

pub fn is_stable(confidence: u8, contaminated: bool) -> bool {
    confidence >= STABLE_MIN && !contaminated
}

/// Avalia a qualidade de uma sessão (confiança + estabilidade + contaminação).
pub fn assess(
    metrics: &[BenchmarkMetric],
    temp_start: Option<f64>,
    temp_end: Option<f64>,
) -> SessionQuality {
    let cvs: Vec<f64> = metrics
        .iter()
        .filter(|m| is_quality_metric(&m.metric) && m.value.abs() > f64::EPSILON)
        .map(|m| m.stddev / m.value.abs() * 100.0)
        .collect();
    let cv_avg = if cvs.is_empty() {
        0.0
    } else {
        cvs.iter().sum::<f64>() / cvs.len() as f64
    };

    let contaminated = match (temp_start, temp_end) {
        (Some(a), Some(b)) => (b - a) >= THERMAL_RISE_C || b >= THERMAL_LIMIT_C,
        _ => false,
    };

    let mut confidence = (100.0 - CONF_CV_K * cv_avg).clamp(0.0, 100.0);
    if contaminated {
        confidence = confidence.min(40.0);
    }
    let confidence = confidence.round() as u8;

    SessionQuality {
        confidence,
        stable: is_stable(confidence, contaminated),
        contaminated,
    }
}

/// Constrói o perfil de ruído a partir do histórico (médias por sessão da mesma suite).
/// Com < 3 sessões usa defaults conservadores (source = "default").
pub fn build_noise_profile(suite: &str, sessions: &[Vec<BenchmarkMetric>]) -> NoiseProfile {
    let learned = sessions.len() >= MIN_SESSIONS_LEARNED;

    // Agrupa as médias de cada métrica entre as sessões.
    let mut by_metric: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for s in sessions {
        for m in s {
            by_metric.entry(m.metric.clone()).or_default().push(m.value);
        }
    }

    let entries = by_metric
        .into_iter()
        .map(|(metric, vals)| {
            let def = default_noise(&metric);
            let cv = if learned && vals.len() >= 2 {
                let a = aggregate(&vals);
                if a.mean.abs() > f64::EPSILON {
                    (a.stddev / a.mean.abs() * 100.0).max(def * 0.5)
                } else {
                    def
                }
            } else {
                def
            };
            NoiseEntry { metric, cv_pct: cv }
        })
        .collect();

    NoiseProfile {
        suite: suite.into(),
        sessions: sessions.len() as u32,
        source: if learned { "learned" } else { "default" }.into(),
        entries,
    }
}

/// CV de ruído para uma métrica (cai no default se não houver no perfil).
pub fn noise_cv(profile: &NoiseProfile, metric: &str) -> f64 {
    profile
        .entries
        .iter()
        .find(|e| e.metric == metric)
        .map(|e| e.cv_pct)
        .unwrap_or_else(|| default_noise(metric))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(metric: &str, value: f64, stddev: f64) -> BenchmarkMetric {
        BenchmarkMetric { metric: metric.into(), value, unit: "x".into(), stddev, samples: 5 }
    }

    #[test]
    fn low_variance_is_high_confidence_and_stable() {
        let q = assess(&[m("cpu_multi", 10000.0, 50.0)], Some(60.0), Some(62.0));
        assert!(q.confidence >= 90, "conf={}", q.confidence);
        assert!(q.stable);
        assert!(!q.contaminated);
    }

    #[test]
    fn high_variance_is_low_confidence() {
        let q = assess(&[m("cpu_multi", 10000.0, 1500.0)], None, None); // CV 15%
        assert!(q.confidence < STABLE_MIN as u8, "conf={}", q.confidence);
        assert!(!q.stable);
    }

    #[test]
    fn thermal_rise_contaminates() {
        let q = assess(&[m("cpu_multi", 10000.0, 20.0)], Some(60.0), Some(85.0));
        assert!(q.contaminated);
        assert!(!q.stable);
    }

    #[test]
    fn few_sessions_use_default_noise() {
        let p = build_noise_profile("cpu-1.0.0", &[vec![m("cpu_multi", 10000.0, 0.0)]]);
        assert_eq!(p.source, "default");
        assert!((noise_cv(&p, "cpu_multi") - 8.0).abs() < 1e-9);
    }

    #[test]
    fn enough_sessions_learn_noise() {
        let sessions = vec![
            vec![m("cpu_multi", 10000.0, 0.0)],
            vec![m("cpu_multi", 11000.0, 0.0)],
            vec![m("cpu_multi", 9000.0, 0.0)],
        ];
        let p = build_noise_profile("cpu-1.0.0", &sessions);
        assert_eq!(p.source, "learned");
        assert!(noise_cv(&p, "cpu_multi") > 0.0);
    }
}
