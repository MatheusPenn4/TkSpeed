//! TkAnalyzer — motor de análise e detecção de gargalo.
//! Produz `Diagnosis` (findings + TkSpeed Score) a partir de uma janela real
//! de telemetria (médias + última leitura) coletada pelo TkMonitor.

use tk_contracts::{Diagnosis, Finding, MetricsTick, ScoreBreakdown, TkSpeedScore};

pub mod bottleneck;
pub mod score;

/// Entrada agregada de uma janela de telemetria, base da análise e do score.
#[derive(Debug, Clone, Default)]
pub struct AnalysisInput {
    pub cpu_avg: f64,      // % uso médio na janela
    pub cpu_peak: f64,     // % pico na janela
    pub ram_used_pct: f64, // % uso (última leitura)
    pub ram_total_gb: f64,
    pub ram_avail_gb: f64,
    pub disk_free_pct: f64,
    pub disk_is_ssd: bool,
    pub samples: usize, // nº de amostras analisadas
}

impl AnalysisInput {
    /// Constrói o input a partir da janela de ticks.
    /// Usa médias para "utilização contínua" e a última leitura para o estado atual.
    /// O tipo do disco (SSD?) vem direto do tick (conservador), sem matching de rótulo.
    pub fn from_window(window: &[MetricsTick]) -> Self {
        if window.is_empty() {
            // Sem dados: assume HDD (conservador) para não inflar o score.
            return Self::default();
        }

        let n = window.len() as f64;
        let cpu_avg = window.iter().map(|t| t.cpu_usage).sum::<f64>() / n;
        let cpu_peak = window.iter().map(|t| t.cpu_usage).fold(0.0_f64, f64::max);

        let last = window.last().expect("janela não-vazia");
        let ram_used_pct = last.ram_usage;
        let ram_total_gb = last.ram_total_gb;
        let ram_avail_gb = (last.ram_total_gb - last.ram_used_gb).max(0.0);
        let disk_free_pct = (100.0 - last.disk_usage).clamp(0.0, 100.0);
        let disk_is_ssd = last.disk_is_ssd;

        Self {
            cpu_avg,
            cpu_peak,
            ram_used_pct,
            ram_total_gb,
            ram_avail_gb,
            disk_free_pct,
            disk_is_ssd,
            samples: window.len(),
        }
    }
}

/// Orquestra os detectores de gargalo e compõe o diagnóstico (findings + score).
pub struct AnalyzerEngine;

impl AnalyzerEngine {
    pub fn new() -> Self {
        Self
    }

    /// Análise completa: detecta gargalos (findings reais) e calcula o TkSpeed Score.
    pub fn run_full(&self, run_id: i64, input: &AnalysisInput) -> Diagnosis {
        let mut findings: Vec<Finding> = Vec::new();
        let mut push = |f: Option<Finding>| {
            if let Some(f) = f {
                findings.push(f);
            }
        };

        push(bottleneck::detect_cpu_sustained(input.cpu_avg));
        push(bottleneck::detect_cpu_spikes(input.cpu_avg, input.cpu_peak));
        push(bottleneck::detect_ram_pressure(input.ram_used_pct));
        push(bottleneck::detect_low_memory(input.ram_avail_gb));
        push(bottleneck::detect_storage_space(input.disk_free_pct));
        push(bottleneck::detect_hdd_system(input.disk_is_ssd));
        push(bottleneck::detect_gaming_impact(input.cpu_avg, input.ram_used_pct));

        let score = score::compute_mvp(input, &findings);
        Diagnosis {
            run_id,
            findings,
            score,
        }
    }
}

impl Default for AnalyzerEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Diagnóstico vazio (estado inicial / testes).
pub fn empty_diagnosis(run_id: i64) -> Diagnosis {
    Diagnosis {
        run_id,
        findings: Vec::new(),
        score: TkSpeedScore {
            total: 0,
            classification: tk_contracts::Classification::Critico,
            breakdown: ScoreBreakdown {
                cpu: 0,
                gpu: 0,
                ram: 0,
                storage: 0,
                windows: 0,
                network: 0,
                temperature: 0,
                games: 0,
                stability: 0,
            },
            score_version: score::SCORE_VERSION.to_string(),
        },
    }
}
