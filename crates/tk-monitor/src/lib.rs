//! TkMonitor — coleta de telemetria em tempo real.
//!
//! MVP: coletor `sysinfo` (CPU/RAM/disco) + detecção de hardware básico.
//! O `SysinfoSampler` é a implementação concreta da estratégia de coleta;
//! adapters futuros (PDH, LibreHardwareMonitor, ETW/FPS) seguem o mesmo papel
//! e podem ser plugados sem alterar as camadas superiores.

use sysinfo::{DiskKind, Disks, System};
use tk_contracts::{DiskInfo, HardwareInfo, MetricSample, MetricSource, MetricsTick};

const BYTES_PER_GB: f64 = 1024.0 * 1024.0 * 1024.0;

/// Coletor de telemetria baseado em `sysinfo`. Mantém o `System` entre leituras
/// (necessário para o cálculo de uso de CPU por delta).
pub struct SysinfoSampler {
    sys: System,
}

impl SysinfoSampler {
    pub fn new() -> Self {
        // new_all popula CPUs/memória; primeira medida de CPU vira delta na 1ª leitura.
        let mut sys = System::new_all();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        Self { sys }
    }

    /// Lê uma rodada de telemetria (snapshot agregado para a UI).
    pub fn read(&mut self) -> MetricsTick {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();

        let cpu_usage = self.sys.global_cpu_usage() as f64;

        let total = self.sys.total_memory() as f64;
        let used = self.sys.used_memory() as f64;
        let ram_usage = if total > 0.0 { used / total * 100.0 } else { 0.0 };

        let (disk_usage, disk_label, disk_is_ssd) = system_disk_usage();

        MetricsTick {
            ts: now_ms(),
            cpu_usage,
            ram_usage,
            ram_used_gb: used / BYTES_PER_GB,
            ram_total_gb: total / BYTES_PER_GB,
            disk_usage,
            disk_label,
            disk_is_ssd,
        }
    }

    /// Detecta o hardware básico (executado uma vez no start).
    pub fn detect_hardware(&mut self) -> HardwareInfo {
        self.sys.refresh_memory();

        let cpu_name = self
            .sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "CPU desconhecida".into());
        let cpu_cores = self.sys.cpus().len() as u32;
        let ram_total_gb = self.sys.total_memory() as f64 / BYTES_PER_GB;
        let hostname = System::host_name().unwrap_or_else(|| "—".into());
        let os_name = System::long_os_version()
            .or_else(System::name)
            .unwrap_or_else(|| "Windows".into());

        let disks = Disks::new_with_refreshed_list()
            .list()
            .iter()
            .map(|d| DiskInfo {
                name: d.name().to_string_lossy().to_string(),
                mount: d.mount_point().to_string_lossy().to_string(),
                total_gb: d.total_space() as f64 / BYTES_PER_GB,
                available_gb: d.available_space() as f64 / BYTES_PER_GB,
                is_ssd: matches!(d.kind(), DiskKind::SSD),
            })
            .collect();

        HardwareInfo {
            hostname,
            os_name,
            cpu_name,
            cpu_cores,
            ram_total_gb,
            disks,
        }
    }
}

impl Default for SysinfoSampler {
    fn default() -> Self {
        Self::new()
    }
}

/// Uso (%), rótulo e tipo (SSD?) do volume do sistema (preferência por C:, depois
/// não-removível). `is_ssd` é CONSERVADOR: só `true` quando a API confirma SSD —
/// HDD/desconhecido viram `false` para não inflar o score.
fn system_disk_usage() -> (f64, String, bool) {
    let disks = Disks::new_with_refreshed_list();
    let pick = disks
        .list()
        .iter()
        .find(|d| d.mount_point().to_string_lossy().starts_with("C:"))
        .or_else(|| disks.list().iter().find(|d| !d.is_removable()))
        .or_else(|| disks.list().first());

    match pick {
        Some(d) => {
            let total = d.total_space() as f64;
            let avail = d.available_space() as f64;
            let usage = if total > 0.0 {
                (total - avail) / total * 100.0
            } else {
                0.0
            };
            let is_ssd = matches!(d.kind(), DiskKind::SSD);
            (usage, d.mount_point().to_string_lossy().to_string(), is_ssd)
        }
        None => (0.0, "—".into(), false),
    }
}

/// Converte um tick em amostras granulares para persistência (downsample).
pub fn to_samples(t: &MetricsTick) -> Vec<MetricSample> {
    let s = |source, metric: &str, value, unit: &str| MetricSample {
        ts: t.ts,
        source,
        metric: metric.into(),
        value,
        unit: unit.into(),
    };
    vec![
        s(MetricSource::Cpu, "usage", t.cpu_usage, "%"),
        s(MetricSource::Ram, "usage", t.ram_usage, "%"),
        s(MetricSource::Ram, "used", t.ram_used_gb, "GB"),
        s(MetricSource::Ssd, "usage", t.disk_usage, "%"),
    ]
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
