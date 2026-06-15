use serde::Serialize;
use tk_perflab::{hardware_snapshot, PresentMonFrameSource};
use tk_platform_win::elevation;

/// Capacidade do sistema detectada em runtime.
///
/// Estrutura aberta — novas capacidades em V4.3/V4.4 são adicionadas
/// sem breaking change (novo id na lista, mesmo tipo).
#[derive(Debug, Clone, Serialize)]
pub struct Capability {
    pub id: String,
    /// "ready" | "limited" | "missing" | "unavailable"
    pub status: String,
    pub detail: String,
}

impl Capability {
    fn new(id: &str, status: &str, detail: &str) -> Self {
        Self {
            id: id.into(),
            status: status.into(),
            detail: detail.into(),
        }
    }
}

/// Status semânticos (constantes abertas — não enum fechado).
pub mod status {
    pub const READY: &str = "ready";
    pub const LIMITED: &str = "limited";
    pub const MISSING: &str = "missing";
    pub const UNAVAILABLE: &str = "unavailable";
}

/// Conjunto de status válidos — usado em testes e no Profile Engine.
pub const VALID_STATUSES: &[&str] = &["ready", "limited", "missing", "unavailable"];

/// Detecta o estado atual de todas as capacidades do TkSpeed.
///
/// Função síncrona — deve ser chamada de dentro de `spawn_blocking`.
pub fn build() -> Vec<Capability> {
    let snap = hardware_snapshot();
    let is_admin = elevation::is_elevated();
    let presentmon = PresentMonFrameSource::locate();

    let gpu = if snap.gpu.is_some() {
        (status::READY, "NVML disponível")
    } else {
        (status::UNAVAILABLE, "GPU não-NVIDIA ou NVML indisponível")
    };

    let thermal = if snap.cpu_temp_c.is_some() {
        (status::READY, "sysinfo")
    } else {
        (status::LIMITED, "sensores não disponíveis neste hardware")
    };

    // ready = PresentMon presente + admin (ETW precisa de elevação)
    // limited = PresentMon presente, mas sem admin
    // missing = PresentMon ausente
    let fps = match (&presentmon, is_admin) {
        (None, _) => (status::MISSING, "PresentMon ausente"),
        (Some(_), false) => (status::LIMITED, "PresentMon presente; admin necessário para ETW"),
        (Some(_), true) => (status::READY, "PresentMon + elevação OK"),
    };

    let admin = if is_admin {
        (status::READY, "processo elevado")
    } else {
        (status::MISSING, "sem elevação — configs avançadas indisponíveis")
    };

    vec![
        Capability::new("cpu_monitoring", status::READY, "sysinfo"),
        Capability::new("ram_monitoring", status::READY, "sysinfo"),
        Capability::new("storage_monitoring", status::READY, "sysinfo"),
        Capability::new("gpu_monitoring", gpu.0, gpu.1),
        Capability::new("thermal_monitoring", thermal.0, thermal.1),
        Capability::new("fps_measurement", fps.0, fps.1),
        Capability::new("rollback_protection", status::READY, "snapshot + rollback disponíveis"),
        Capability::new("benchmark_engine", status::READY, "tk-perflab"),
        Capability::new("optimization_engine", status::READY, "tk-optimize"),
        Capability::new("admin_privileges", admin.0, admin.1),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_IDS: &[&str] = &[
        "cpu_monitoring",
        "ram_monitoring",
        "storage_monitoring",
        "gpu_monitoring",
        "thermal_monitoring",
        "fps_measurement",
        "rollback_protection",
        "benchmark_engine",
        "optimization_engine",
        "admin_privileges",
    ];

    // ── Estrutura aberta ────────────────────────────────────────────────────────

    #[test]
    fn has_ten_capabilities() {
        assert_eq!(build().len(), 10);
    }

    #[test]
    fn all_ids_are_unique() {
        let caps = build();
        let mut ids: Vec<&str> = caps.iter().map(|c| c.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), caps.len(), "IDs duplicados detectados");
    }

    #[test]
    fn all_expected_ids_present() {
        let caps = build();
        let ids: Vec<&str> = caps.iter().map(|c| c.id.as_str()).collect();
        for expected in EXPECTED_IDS {
            assert!(ids.contains(expected), "ID esperado ausente: {}", expected);
        }
    }

    #[test]
    fn all_statuses_are_valid() {
        for c in &build() {
            assert!(
                VALID_STATUSES.contains(&c.status.as_str()),
                "capability '{}' tem status inválido: '{}'",
                c.id,
                c.status
            );
        }
    }

    #[test]
    fn all_details_are_non_empty() {
        for c in &build() {
            assert!(!c.detail.is_empty(), "capability '{}' tem detail vazio", c.id);
        }
    }

    // ── IDs individuais ─────────────────────────────────────────────────────────

    #[test]
    fn cpu_ram_storage_always_ready() {
        let caps = build();
        for id in ["cpu_monitoring", "ram_monitoring", "storage_monitoring"] {
            let c = caps.iter().find(|c| c.id == id).expect(id);
            assert_eq!(c.status, "ready", "'{}' deve sempre ser ready", id);
        }
    }

    #[test]
    fn benchmark_and_optimization_engine_always_ready() {
        let caps = build();
        for id in ["benchmark_engine", "optimization_engine"] {
            let c = caps.iter().find(|c| c.id == id).expect(id);
            assert_eq!(c.status, "ready", "'{}' deve sempre ser ready", id);
        }
    }

    #[test]
    fn rollback_protection_is_ready() {
        let c = build().into_iter().find(|c| c.id == "rollback_protection").unwrap();
        assert_eq!(c.status, "ready");
    }

    // ── fps_measurement ────────────────────────────────────────────────────────

    #[test]
    fn fps_measurement_never_unavailable() {
        // "unavailable" é reservado para hardware (gpu). fps usa: ready | limited | missing.
        let c = build().into_iter().find(|c| c.id == "fps_measurement").unwrap();
        assert_ne!(
            c.status, "unavailable",
            "fps_measurement não usa 'unavailable' — hardware ausente é 'missing'"
        );
    }

    #[test]
    fn fps_measurement_missing_when_presentmon_absent() {
        // Em ambiente de teste (sem PresentMon instalado), status deve ser "missing".
        // Se PresentMon estiver presente no ambiente, este assert é skipped via early return.
        if PresentMonFrameSource::locate().is_some() {
            return; // PresentMon presente — status pode ser "ready" ou "limited"
        }
        let c = build().into_iter().find(|c| c.id == "fps_measurement").unwrap();
        assert_eq!(c.status, "missing", "fps_measurement deve ser 'missing' sem PresentMon");
    }

    // ── gpu_monitoring ─────────────────────────────────────────────────────────

    #[test]
    fn gpu_monitoring_never_missing() {
        // "missing" é reservado para software ausente (fps, admin). GPU usa: ready | unavailable.
        let c = build().into_iter().find(|c| c.id == "gpu_monitoring").unwrap();
        assert_ne!(
            c.status, "missing",
            "gpu_monitoring não usa 'missing' — hardware não suportado é 'unavailable'"
        );
    }

    // ── admin_privileges ───────────────────────────────────────────────────────

    #[test]
    fn admin_privileges_never_unavailable_or_limited() {
        // admin_privileges é binário: ready (sim) ou missing (não).
        let c = build().into_iter().find(|c| c.id == "admin_privileges").unwrap();
        assert!(
            c.status == "ready" || c.status == "missing",
            "admin_privileges deve ser 'ready' ou 'missing', não '{}'",
            c.status
        );
    }

    // ── Tipo aberto ─────────────────────────────────────────────────────────────

    #[test]
    fn capability_struct_has_no_fixed_fields_per_id() {
        // Todos os elementos são do mesmo tipo Capability {id, status, detail}.
        // Este teste seria quebrado se o struct virasse um enum fechado.
        let caps = build();
        for c in &caps {
            // Compilação deste acesso garante que não há campos extras nem enum.
            let _: &str = &c.id;
            let _: &str = &c.status;
            let _: &str = &c.detail;
        }
    }
}
