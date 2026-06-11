//! PL-S0401 + PL-2d — Evidence Engine: comparação antes/depois com margem
//! DINÂMICA (intra-sessão + noise floor entre sessões) e confiança.
//!
//! REGRA DE OURO: só declara Ganho/Perda se a medição for estável E o delta
//! superar a margem efetiva (que inclui o ruído natural da máquina). Sessão
//! instável → "Medição instável" (nunca ganho/perda).

use tk_contracts::{BenchmarkResult, ComparisonRow, NoiseProfile, PerfComparison, PerfVerdict};

use crate::confidence::{noise_cv, NOISE_K};

const MARGIN_FLOOR_PCT: f64 = 0.5;
const K_INTRA: f64 = 2.0; // ~95% intra-sessão

fn higher_is_better(metric: &str) -> bool {
    matches!(
        metric,
        "cpu_single"
            | "cpu_multi"
            | "ram_bandwidth_gbs"
            | "io_seq_read_mbs"
            | "io_seq_write_mbs"
            | "io_rand_read_iops"
            | "fps_avg"
            | "fps_min"
            | "fps_max"
            | "fps_1pct_low"
            | "fps_01pct_low"
    )
    // frametime_* (avg/p95/p99) e *_latency_* são "menor é melhor" → ramo padrão.
}

fn is_informational(metric: &str) -> bool {
    matches!(metric, "cpu_load_pct")
}

fn classify(delta_pct: f64, margin_pct: f64, metric: &str) -> PerfVerdict {
    if is_informational(metric) {
        return PerfVerdict::NoChange;
    }
    if delta_pct.abs() <= margin_pct.max(MARGIN_FLOOR_PCT) {
        return PerfVerdict::NoChange;
    }
    let improved = if higher_is_better(metric) { delta_pct > 0.0 } else { delta_pct < 0.0 };
    if improved {
        PerfVerdict::Gain
    } else {
        PerfVerdict::Loss
    }
}

/// Compara duas sessões usando o perfil de ruído da máquina (margem dinâmica).
pub fn compare(
    before: &BenchmarkResult,
    after: &BenchmarkResult,
    noise: &NoiseProfile,
) -> PerfComparison {
    let reliable = before.stable && after.stable;
    let confidence = before.confidence.min(after.confidence);

    let mut rows = Vec::new();
    for b in &before.metrics {
        let Some(a) = after.metrics.iter().find(|m| m.metric == b.metric) else { continue };

        let delta_pct = if b.value != 0.0 { (a.value - b.value) / b.value * 100.0 } else { 0.0 };

        // Margem intra-sessão (erro padrão combinado das médias dos runs).
        let bn = before.runs.max(1) as f64;
        let an = after.runs.max(1) as f64;
        let se = ((b.stddev * b.stddev) / bn + (a.stddev * a.stddev) / an).sqrt();
        let intra_pct = if b.value != 0.0 { K_INTRA * se / b.value.abs() * 100.0 } else { 0.0 };

        // Margem de ruído ENTRE sessões (a máquina aprende sua variabilidade).
        let noise_pct = NOISE_K * noise_cv(noise, &b.metric);

        // Margem efetiva = a mais exigente das duas (mais o piso).
        let effective = intra_pct.max(noise_pct).max(MARGIN_FLOOR_PCT);

        let verdict = if reliable {
            classify(delta_pct, effective, &b.metric)
        } else {
            PerfVerdict::Unstable
        };

        rows.push(ComparisonRow {
            metric: b.metric.clone(),
            before: b.value,
            after: a.value,
            delta_pct,
            margin_pct: effective,
            verdict,
            unit: b.unit.clone(),
        });
    }

    let summary = build_summary(&rows, reliable, before, after, noise);
    PerfComparison { before_id: 0, after_id: 0, rows, summary, confidence, reliable }
}

fn build_summary(
    rows: &[ComparisonRow],
    reliable: bool,
    before: &BenchmarkResult,
    after: &BenchmarkResult,
    noise: &NoiseProfile,
) -> String {
    if !reliable {
        let why = if before.contaminated || after.contaminated {
            "throttling/temperatura comprometeu a medição"
        } else {
            "variância alta entre as execuções"
        };
        return format!(
            "Medição instável ({why}) — não é possível afirmar ganho ou perda. \
             Repita o benchmark com a máquina em repouso."
        );
    }

    // Métrica principal por contexto.
    let primary = rows
        .iter()
        .find(|r| r.metric == "fps_1pct_low") // fluidez é o que mais importa em jogos
        .or_else(|| rows.iter().find(|r| r.metric == "fps_avg"))
        .or_else(|| rows.iter().find(|r| r.metric == "cpu_multi"))
        .or_else(|| rows.iter().find(|r| r.metric == "ram_bandwidth_gbs"))
        .or_else(|| rows.iter().find(|r| r.metric == "io_seq_read_mbs"))
        .or_else(|| rows.first());

    let base = match primary {
        Some(m) => match m.verdict {
            PerfVerdict::Gain => format!("Ganho de {:+.2}% em {} (±{:.2}%, evidência válida).", m.delta_pct, m.metric, m.margin_pct),
            PerfVerdict::Loss => format!("Perda de {:+.2}% em {} (±{:.2}%).", m.delta_pct, m.metric, m.margin_pct),
            PerfVerdict::NoChange => format!("Sem alteração significativa em {} ({:+.2}%, dentro da margem de ±{:.2}%).", m.metric, m.delta_pct, m.margin_pct),
            PerfVerdict::Unstable => "Medição instável.".into(),
        },
        None => "Sem métricas comparáveis.".into(),
    };
    let src = if noise.source == "learned" {
        format!(" Margem calibrada com {} sessões da máquina.", noise.sessions)
    } else {
        " Margem em modo conservador (rode 3+ benchmarks p/ calibrar o ruído).".into()
    };
    base + &src
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::confidence::build_noise_profile;
    use tk_contracts::BenchmarkMetric;

    fn res(multi: f64, sd: f64, conf: u8, stable: bool, contaminated: bool) -> BenchmarkResult {
        BenchmarkResult {
            kind: "synthetic_cpu".into(),
            suite_version: "cpu-1.0.0".into(),
            runs: 5,
            duration_ms: 0,
            metrics: vec![BenchmarkMetric {
                metric: "cpu_multi".into(),
                value: multi,
                unit: "score".into(),
                stddev: sd,
                samples: 5,
            }],
            confidence: conf,
            stable,
            contaminated,
            temp_start_c: None,
            temp_end_c: None,
        }
    }

    fn noise_low() -> NoiseProfile {
        // 3 sessões quase idênticas → ruído aprendido baixo (piso 4% CV → margem 8%).
        build_noise_profile(
            "cpu-1.0.0",
            &[
                vec![BenchmarkMetric { metric: "cpu_multi".into(), value: 8000.0, unit: "score".into(), stddev: 0.0, samples: 5 }],
                vec![BenchmarkMetric { metric: "cpu_multi".into(), value: 8010.0, unit: "score".into(), stddev: 0.0, samples: 5 }],
                vec![BenchmarkMetric { metric: "cpu_multi".into(), value: 7990.0, unit: "score".into(), stddev: 0.0, samples: 5 }],
            ],
        )
    }
    fn noise_default() -> NoiseProfile {
        build_noise_profile("cpu-1.0.0", &[]) // sem histórico → default conservador (CV 8% → margem 16%)
    }

    #[test]
    fn tiny_delta_is_no_change() {
        let c = compare(&res(8420.0, 40.0, 95, true, false), &res(8435.0, 40.0, 95, true, false), &noise_low());
        assert_eq!(c.rows[0].verdict, PerfVerdict::NoChange);
        assert!(c.reliable);
    }

    #[test]
    fn real_gain_passes_with_low_noise() {
        // +12% supera a margem de 8% (ruído baixo calibrado) → Ganho.
        let c = compare(&res(8000.0, 20.0, 95, true, false), &res(8960.0, 20.0, 95, true, false), &noise_low());
        assert_eq!(c.rows[0].verdict, PerfVerdict::Gain);
    }

    #[test]
    fn gain_within_noise_floor_is_rejected() {
        // +10% mas máquina não calibrada (margem 16%) → não declara ganho.
        let c = compare(&res(8000.0, 20.0, 95, true, false), &res(8800.0, 20.0, 95, true, false), &noise_default());
        assert_eq!(c.rows[0].verdict, PerfVerdict::NoChange);
    }

    #[test]
    fn unstable_session_blocks_verdict() {
        // Antes instável (confiança baixa) → Instável, mesmo com delta grande.
        let c = compare(&res(8000.0, 1500.0, 40, false, false), &res(9600.0, 20.0, 95, true, false), &noise_low());
        assert_eq!(c.rows[0].verdict, PerfVerdict::Unstable);
        assert!(!c.reliable);
    }

    #[test]
    fn regression_passes_with_low_noise() {
        let c = compare(&res(9000.0, 20.0, 95, true, false), &res(8100.0, 20.0, 95, true, false), &noise_low());
        assert_eq!(c.rows[0].verdict, PerfVerdict::Loss);
    }
}
