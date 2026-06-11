//! PL-S0101 / PL-S0102 — abstração de coleta (Strategy) + coletor `sysinfo`.
//!
//! O trait padroniza fontes de métrica (hoje: sysinfo; futuro: PDH-GPU, ETW-FPS,
//! LHM) sem que as camadas superiores conheçam o detalhe. Sessão explícita
//! (`start`/`poll`/`stop`) — base também da coleta de alta fidelidade futura.

use sysinfo::System;

/// Ponto de métrica instantâneo coletado de uma fonte.
#[derive(Debug, Clone)]
pub struct MetricPoint {
    pub metric: String,
    pub value: f64,
    pub unit: String,
}

/// Estratégia de coleta. Cada adapter implementa este contrato.
pub trait MetricsCollector {
    fn source(&self) -> &'static str;
    /// Abre a sessão de coleta (ex.: warmup de contadores).
    fn start(&mut self) -> Result<(), String>;
    /// Amostra atual (barato, síncrono).
    fn poll(&mut self) -> Vec<MetricPoint>;
    /// Encerra a sessão.
    fn stop(&mut self);
}

/// Coletor baseado em `sysinfo` (CPU/RAM). Sem elevação.
pub struct SysinfoCollector {
    sys: System,
}

impl SysinfoCollector {
    pub fn new() -> Self {
        Self { sys: System::new() }
    }
}

impl Default for SysinfoCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector for SysinfoCollector {
    fn source(&self) -> &'static str {
        "sysinfo"
    }

    fn start(&mut self) -> Result<(), String> {
        // Primeira leitura serve de base para o delta de CPU.
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        Ok(())
    }

    fn poll(&mut self) -> Vec<MetricPoint> {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();

        let cpu = self.sys.global_cpu_usage() as f64;
        let total = self.sys.total_memory() as f64;
        let used = self.sys.used_memory() as f64;
        let ram = if total > 0.0 { used / total * 100.0 } else { 0.0 };

        vec![
            MetricPoint { metric: "cpu_usage".into(), value: cpu, unit: "%".into() },
            MetricPoint { metric: "ram_usage".into(), value: ram, unit: "%".into() },
        ]
    }

    fn stop(&mut self) {}
}
