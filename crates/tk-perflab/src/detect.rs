//! PL-2 — detectores de gargalo. Classificação pura (testável) + amostragem de
//! janela curta. Sem carga real (jogo) o veredito tende a "Balanceado/Ocioso".
//! Storage bound exige captura durante IO → marcado como futuro.

use std::time::Duration;

use tk_contracts::{BottleneckKind, BottleneckReport};

use crate::collector::{MetricsCollector, SysinfoCollector};
use crate::hardware::GpuCollector;

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        0.0
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

/// Classificação pura a partir das médias da janela (função testável).
pub fn classify(
    cpu_avg: f64,
    gpu_avg: Option<f64>,
    ram_avg: f64,
    gpu_temp: Option<f64>,
) -> (BottleneckKind, String) {
    if let Some(t) = gpu_temp {
        if t >= 85.0 {
            return (BottleneckKind::Thermal, format!("GPU a {t:.0}°C — possível throttling térmico."));
        }
    }
    match gpu_avg {
        Some(g) if g >= 90.0 && cpu_avg < 85.0 => (
            BottleneckKind::Gpu,
            format!("GPU saturada ({g:.0}%) com CPU folgada — limitado pela GPU."),
        ),
        _ if cpu_avg >= 85.0 && gpu_avg.is_none_or(|g| g < 80.0) => (
            BottleneckKind::Cpu,
            format!("CPU saturada ({cpu_avg:.0}%) — limitado pela CPU."),
        ),
        _ if ram_avg >= 90.0 => {
            (BottleneckKind::Ram, format!("Memória sob pressão ({ram_avg:.0}%)."))
        }
        _ if cpu_avg < 25.0 && gpu_avg.is_none_or(|g| g < 25.0) => (
            BottleneckKind::Balanced,
            "Sistema ocioso — execute sob carga (jogo/render) para um diagnóstico real.".into(),
        ),
        _ => (BottleneckKind::Inconclusive, "Sem gargalo dominante na amostra atual.".into()),
    }
}

/// Amostra ~2s (CPU/RAM via sysinfo, GPU via NVML) e classifica.
pub fn detect_bottleneck() -> BottleneckReport {
    let mut sys = SysinfoCollector::new();
    let _ = sys.start();
    let gpu_c = GpuCollector::new();
    let gpu_available = gpu_c.available();

    let (mut cpu, mut ram, mut gpu) = (vec![], vec![], vec![]);
    let mut gpu_temp_last = None;

    for _ in 0..8 {
        std::thread::sleep(Duration::from_millis(250));
        for p in sys.poll() {
            match p.metric.as_str() {
                "cpu_usage" => cpu.push(p.value),
                "ram_usage" => ram.push(p.value),
                _ => {}
            }
        }
        if let Some(g) = gpu_c.snapshot() {
            gpu.push(g.usage_pct);
            gpu_temp_last = g.temp_c;
        }
    }

    let cpu_avg = mean(&cpu);
    let ram_avg = mean(&ram);
    let gpu_avg = if gpu.is_empty() { None } else { Some(mean(&gpu)) };
    let thermal_available = gpu_temp_last.is_some();
    let (primary, detail) = classify(cpu_avg, gpu_avg, ram_avg, gpu_temp_last);

    BottleneckReport {
        primary,
        detail,
        cpu_avg,
        gpu_avg,
        ram_avg,
        gpu_available,
        thermal_available,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_bound_when_gpu_saturated_cpu_free() {
        let (k, _) = classify(40.0, Some(98.0), 50.0, Some(70.0));
        assert_eq!(k, BottleneckKind::Gpu);
    }

    #[test]
    fn cpu_bound_when_cpu_saturated() {
        let (k, _) = classify(95.0, Some(50.0), 50.0, Some(60.0));
        assert_eq!(k, BottleneckKind::Cpu);
    }

    #[test]
    fn thermal_takes_priority() {
        let (k, _) = classify(95.0, Some(99.0), 50.0, Some(90.0));
        assert_eq!(k, BottleneckKind::Thermal);
    }

    #[test]
    fn idle_is_balanced() {
        let (k, _) = classify(5.0, Some(3.0), 30.0, Some(45.0));
        assert_eq!(k, BottleneckKind::Balanced);
    }
}
