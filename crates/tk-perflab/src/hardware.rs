//! PL-2 — leitura de hardware ao vivo: GPU (NVML) + temperaturas (NVML/sysinfo).
//! Tudo graciosamente degradável: sem NVIDIA → GPU `None`; sem sensor → `None`.

use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};
use nvml_wrapper::Nvml;
use sysinfo::{Components, System};
use tk_contracts::{GpuInfo, HardwareSnapshot};

const BYTES_PER_GB: f64 = 1024.0 * 1024.0 * 1024.0;
const BYTES_PER_MB: f64 = 1024.0 * 1024.0;

/// Coletor de GPU via NVML (NVIDIA). `available()` indica se há GPU NVIDIA.
pub struct GpuCollector {
    nvml: Option<Nvml>,
}

impl GpuCollector {
    pub fn new() -> Self {
        // Err se não houver driver NVIDIA / nvml.dll — tratado como indisponível.
        Self { nvml: Nvml::init().ok() }
    }

    pub fn available(&self) -> bool {
        self.nvml.is_some()
    }

    pub fn snapshot(&self) -> Option<GpuInfo> {
        let nvml = self.nvml.as_ref()?;
        let dev = nvml.device_by_index(0).ok()?;

        let name = dev.name().unwrap_or_else(|_| "GPU".into());
        let util = dev.utilization_rates().ok();
        let mem = dev.memory_info().ok();
        let clock = dev.clock_info(Clock::Graphics).ok();
        let temp = dev.temperature(TemperatureSensor::Gpu).ok();

        Some(GpuInfo {
            name,
            usage_pct: util.map(|u| u.gpu as f64).unwrap_or(0.0),
            vram_used_mb: mem.as_ref().map(|m| m.used as f64 / BYTES_PER_MB).unwrap_or(0.0),
            vram_total_mb: mem.as_ref().map(|m| m.total as f64 / BYTES_PER_MB).unwrap_or(0.0),
            clock_mhz: clock.map(|c| c as f64),
            temp_c: temp.map(|t| t as f64),
        })
    }
}

impl Default for GpuCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Temperatura de CPU via `sysinfo::Components` (best-effort no Windows).
/// Retorna `None` quando o SO não expõe o sensor (comum sem driver/admin).
/// Suporta Intel (CPU Package, Core) e AMD (Tctl/Tdie, k10temp, Tccd).
pub fn cpu_temp_c() -> Option<f64> {
    let comps = Components::new_with_refreshed_list();

    // Prioridade: Intel "Package" > AMD "Tdie/Tctl" > qualquer "core 0" > primeiro
    let pick = comps
        .list()
        .iter()
        .find(|c| {
            let l = c.label().to_lowercase();
            l.contains("package") || l.contains("cpu package")
        })
        .or_else(|| {
            comps.list().iter().find(|c| {
                let l = c.label().to_lowercase();
                l.contains("tdie") || l.contains("tctl") || l.contains("k10temp")
                    || l.contains("cpu die")
            })
        })
        .or_else(|| {
            comps.list().iter().find(|c| {
                let l = c.label().to_lowercase();
                l.contains("cpu") || l.contains("core 0") || l.contains("core#0")
            })
        })
        .or_else(|| comps.list().first());

    pick.map(|c| c.temperature() as f64).filter(|t| *t > 0.0 && *t < 150.0)
}

/// Fotografia ao vivo do hardware para o dashboard do Performance Lab.
pub fn hardware_snapshot() -> HardwareSnapshot {
    let gpu = GpuCollector::new().snapshot();
    let cpu_temp_c = cpu_temp_c();

    let mut sys = System::new();
    sys.refresh_memory();
    let total = sys.total_memory() as f64;
    let used = sys.used_memory() as f64;

    HardwareSnapshot {
        gpu,
        cpu_temp_c,
        ram_usage_pct: if total > 0.0 { used / total * 100.0 } else { 0.0 },
        ram_used_gb: used / BYTES_PER_GB,
        ram_total_gb: total / BYTES_PER_GB,
    }
}
