use serde_json::Value;

use crate::configs::types::{ConfigCategory, ConfigError, ConfigMeta, ConfigRisk};

/// Interface de uma configuração de sistema gerenciada pelo TkSpeed.
///
/// V4.1-B: metadata completa + stubs de I/O. As implementações reais de
/// `read_current`, `apply` e `revert` chegam nas slices seguintes, quando
/// os módulos de plataforma Windows correspondentes estiverem prontos.
pub trait ConfigDefinition: Send + Sync {
    fn meta(&self) -> &ConfigMeta;

    /// Lê o estado atual desta configuração no sistema.
    /// Retorna um Value que pode ser usado como argumento `previous` em `revert`.
    fn read_current(&self) -> Result<Value, ConfigError> {
        Err(ConfigError::NotImplemented(self.meta().id))
    }

    /// Aplica um novo valor a esta configuração.
    /// `value` tem o mesmo formato retornado por `read_current`.
    fn apply(&self, _value: &Value) -> Result<(), ConfigError> {
        Err(ConfigError::NotImplemented(self.meta().id))
    }

    /// Reverte para um estado anterior (previamente capturado por `read_current`).
    fn revert(&self, _previous: &Value) -> Result<(), ConfigError> {
        Err(ConfigError::NotImplemented(self.meta().id))
    }
}

// ──────────────────────── Implementações — metadata-only (V4.1-B) ────────────────────────

macro_rules! config_def {
    ($name:ident, $meta:expr) => {
        pub struct $name;

        impl ConfigDefinition for $name {
            fn meta(&self) -> &ConfigMeta {
                static M: ConfigMeta = $meta;
                &M
            }
        }
    };
}

config_def!(
    GpuHardwareScheduling,
    ConfigMeta {
        id: "gpu_hardware_scheduling",
        name: "Agendamento de GPU por Hardware",
        description: "Ativa o agendamento de GPU via hardware (WDDM 2.7+). \
                      Reduz latência de renderização em DirectX 12 e Vulkan. \
                      Requer reinicialização para efeito completo.",
        category: ConfigCategory::Gpu,
        risk: ConfigRisk::Safe,
        reversible: true,
        requires_reboot: true,
        requires_elevation: true,
        benchmark_relevance: &["fps", "frame_time"],
    }
);

// NOTA: `memory_standby_flush` foi REMOVIDO do Config Registry — é uma ação
// one-shot IRREVERSÍVEL (libera a standby list), não uma config persistente
// reversível. Continua disponível como comando avulso `ram_flush_standby`
// (Memory Manager). Manter aqui violaria o contrato `reversible` das configs.

config_def!(
    VisualEffectsGaming,
    ConfigMeta {
        id: "visual_effects_gaming",
        name: "Efeitos Visuais — Modo Gaming",
        description: "Desativa animações, transparências e sombras do Windows para \
                      liberar ciclos de CPU e GPU para o jogo em execução.",
        category: ConfigCategory::Display,
        risk: ConfigRisk::Safe,
        reversible: true,
        requires_reboot: false,
        requires_elevation: false,
        benchmark_relevance: &["fps", "cpu_multi"],
    }
);

config_def!(
    CpuCoreParking,
    ConfigMeta {
        id: "cpu_core_parking",
        name: "Estacionamento de Núcleos de CPU",
        description: "Desativa o estacionamento de núcleos (core parking) para garantir \
                      que todos os núcleos estejam disponíveis imediatamente sob carga. \
                      Requer reinicialização.",
        category: ConfigCategory::Cpu,
        risk: ConfigRisk::Safe,
        reversible: true,
        requires_reboot: true,
        requires_elevation: true,
        benchmark_relevance: &["cpu_multi", "cpu_single"],
    }
);

config_def!(
    TimerResolution,
    ConfigMeta {
        id: "timer_resolution",
        name: "Resolução do Timer do Sistema",
        description: "Reduz a resolução do timer do sistema para 0.5ms via \
                      NtSetTimerResolution. Melhora consistência de frame time \
                      e agendamento de threads.",
        category: ConfigCategory::Timer,
        risk: ConfigRisk::Moderate,
        reversible: true,
        requires_reboot: false,
        requires_elevation: false,
        benchmark_relevance: &["fps", "frame_time", "cpu_single"],
    }
);

config_def!(
    Hpet,
    ConfigMeta {
        id: "hpet",
        name: "Timer de Alta Precisão (HPET)",
        description: "Desativa o HPET via bcdedit para reduzir overhead de interrupção \
                      em sistemas onde o TSC oferece precisão superior. \
                      Requer reinicialização.",
        category: ConfigCategory::Timer,
        risk: ConfigRisk::Moderate,
        reversible: true,
        requires_reboot: true,
        requires_elevation: true,
        benchmark_relevance: &["frame_time", "cpu_single"],
    }
);

config_def!(
    PowerPlanHighPerformance,
    ConfigMeta {
        id: "power_plan_high_performance",
        name: "Plano de Energia — Alto Desempenho",
        description: "Ativa o plano de energia de Alto Desempenho do Windows. \
                      Elimina throttling de CPU por economia de energia.",
        category: ConfigCategory::Power,
        risk: ConfigRisk::Safe,
        reversible: true,
        requires_reboot: false,
        // powercfg /setactive para um esquema existente NÃO exige admin (P2.4).
        requires_elevation: false,
        benchmark_relevance: &["cpu_multi", "cpu_single", "fps"],
    }
);

config_def!(
    PowerPlanBalanced,
    ConfigMeta {
        id: "power_plan_balanced",
        name: "Plano de Energia — Balanceado",
        description: "Restaura o plano de energia Balanceado padrão do Windows. \
                      Equilibra desempenho e consumo de energia.",
        category: ConfigCategory::Power,
        risk: ConfigRisk::Safe,
        reversible: true,
        requires_reboot: false,
        // powercfg /setactive para um esquema existente NÃO exige admin (P2.4).
        requires_elevation: false,
        benchmark_relevance: &["cpu_multi", "cpu_single"],
    }
);

config_def!(
    PowerPlanPowerSaver,
    ConfigMeta {
        id: "power_plan_power_saver",
        name: "Plano de Energia — Economia",
        description: "Ativa o plano de energia de Economia do Windows. \
                      Reduz frequência de CPU e consumo total para menor ruído e calor.",
        category: ConfigCategory::Power,
        risk: ConfigRisk::Safe,
        reversible: true,
        requires_reboot: false,
        // powercfg /setactive para um esquema existente NÃO exige admin (P2.4).
        requires_elevation: false,
        benchmark_relevance: &["cpu_single"],
    }
);

// ──────────────────────────────────── ConfigRegistry ────────────────────────────────────

/// Registro central de todas as configurações de sistema gerenciadas pelo TkSpeed.
pub struct ConfigRegistry {
    entries: Vec<Box<dyn ConfigDefinition>>,
}

impl ConfigRegistry {
    /// Constrói o registro com todas as configurações disponíveis.
    pub fn new() -> Self {
        Self {
            entries: vec![
                Box::new(GpuHardwareScheduling),
                Box::new(VisualEffectsGaming),
                Box::new(CpuCoreParking),
                Box::new(TimerResolution),
                Box::new(Hpet),
                Box::new(PowerPlanHighPerformance),
                Box::new(PowerPlanBalanced),
                Box::new(PowerPlanPowerSaver),
            ],
        }
    }

    /// Todas as configurações registradas.
    pub fn all(&self) -> &[Box<dyn ConfigDefinition>] {
        &self.entries
    }

    /// Busca uma configuração pelo seu id.
    pub fn find(&self, id: &str) -> Option<&dyn ConfigDefinition> {
        self.entries
            .iter()
            .find(|e| e.meta().id == id)
            .map(|e| e.as_ref())
    }

    /// Metadata de todas as configurações.
    pub fn all_meta(&self) -> Vec<&ConfigMeta> {
        self.entries.iter().map(|e| e.meta()).collect()
    }

    /// Filtra por categoria.
    pub fn by_category(&self, category: &ConfigCategory) -> Vec<&dyn ConfigDefinition> {
        self.entries
            .iter()
            .filter(|e| &e.meta().category == category)
            .map(|e| e.as_ref())
            .collect()
    }

    /// Filtra por risco máximo (inclusive).
    pub fn by_max_risk(&self, max_risk: &ConfigRisk) -> Vec<&dyn ConfigDefinition> {
        self.entries
            .iter()
            .filter(|e| &e.meta().risk <= max_risk)
            .map(|e| e.as_ref())
            .collect()
    }

    /// Configurações elegíveis para inclusão em um perfil automático:
    /// - reversible = true
    /// - ao menos uma benchmark_relevance presente em `available_capabilities`
    ///
    /// Nota: configs com requires_reboot = true são incluídas; o Profile Engine
    /// decide se usa o pipeline automático (sem reboot) ou apply_only (com reboot).
    pub fn eligible_for_profiles<'a>(
        &'a self,
        available_capabilities: &[&str],
    ) -> Vec<&'a dyn ConfigDefinition> {
        self.entries
            .iter()
            .filter(|e| {
                let m = e.meta();
                m.reversible && m.has_measurable_evidence(available_capabilities)
            })
            .map(|e| e.as_ref())
            .collect()
    }

    /// Quantidade de configs que requerem reboot (fora do pipeline automático de evidência).
    pub fn reboot_required_count(&self) -> usize {
        self.entries.iter().filter(|e| e.meta().requires_reboot).count()
    }
}

impl Default for ConfigRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ──────────────────────────────────── Testes unitários ────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configs::types::{ConfigCategory, ConfigRisk};

    fn reg() -> ConfigRegistry {
        ConfigRegistry::new()
    }

    // ── Contagem e cobertura ────────────────────────────────────────────────────────────

    #[test]
    fn registry_has_eight_configs() {
        // 9 originais − memory_standby_flush (removido: ação irreversível).
        assert_eq!(reg().all().len(), 8);
    }

    #[test]
    fn memory_standby_flush_not_in_registry() {
        assert!(
            reg().find("memory_standby_flush").is_none(),
            "memory_standby_flush é one-shot irreversível — não pode ser config de perfil"
        );
    }

    #[test]
    fn all_ids_are_unique() {
        let r = reg();
        let mut ids: Vec<&str> = r.all_meta().iter().map(|m| m.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 8, "IDs duplicados detectados");
    }

    #[test]
    fn all_configs_have_non_empty_benchmark_relevance() {
        for m in reg().all_meta() {
            assert!(
                !m.benchmark_relevance.is_empty(),
                "config '{}' sem benchmark_relevance — viola regra central",
                m.id
            );
        }
    }

    #[test]
    fn all_configs_are_reversible() {
        for m in reg().all_meta() {
            assert!(m.reversible, "config '{}' não é reversível", m.id);
        }
    }

    // ── Find ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn find_existing_config() {
        let r = reg();
        let c = r.find("gpu_hardware_scheduling");
        assert!(c.is_some());
        assert_eq!(c.unwrap().meta().id, "gpu_hardware_scheduling");
    }

    #[test]
    fn find_nonexistent_returns_none() {
        assert!(reg().find("does_not_exist").is_none());
    }

    #[test]
    fn network_stack_not_in_registry() {
        assert!(
            reg().find("network_stack").is_none(),
            "network_stack não tem benchmark de rede — não pode estar em perfis"
        );
    }

    // ── Categoria ────────────────────────────────────────────────────────────────────

    #[test]
    fn power_category_has_three_entries() {
        assert_eq!(
            reg().by_category(&ConfigCategory::Power).len(),
            3,
            "Power deve ter: high_performance, balanced, power_saver"
        );
    }

    #[test]
    fn timer_category_has_two_entries() {
        assert_eq!(
            reg().by_category(&ConfigCategory::Timer).len(),
            2,
            "Timer deve ter: timer_resolution, hpet"
        );
    }

    #[test]
    fn gpu_category_has_one_entry() {
        let r = reg();
        let gpu = r.by_category(&ConfigCategory::Gpu);
        assert_eq!(gpu.len(), 1);
        assert_eq!(gpu[0].meta().id, "gpu_hardware_scheduling");
    }

    #[test]
    fn cpu_category_has_one_entry() {
        let r = reg();
        let cpu = r.by_category(&ConfigCategory::Cpu);
        assert_eq!(cpu.len(), 1);
        assert_eq!(cpu[0].meta().id, "cpu_core_parking");
    }

    // ── Risco ────────────────────────────────────────────────────────────────────────

    #[test]
    fn only_timer_configs_are_moderate() {
        let r = reg();
        let moderate: Vec<_> = r
            .all_meta()
            .into_iter()
            .filter(|m| m.risk == ConfigRisk::Moderate)
            .collect();
        assert_eq!(moderate.len(), 2);
        let ids: Vec<&str> = moderate.iter().map(|m| m.id).collect();
        assert!(ids.contains(&"timer_resolution"));
        assert!(ids.contains(&"hpet"));
    }

    #[test]
    fn by_max_risk_safe_returns_six() {
        let r = reg();
        let safe = r.by_max_risk(&ConfigRisk::Safe);
        // 8 total − 2 moderate (timer_resolution, hpet) = 6
        assert_eq!(safe.len(), 6);
        for c in &safe {
            assert_ne!(
                c.meta().risk,
                ConfigRisk::Moderate,
                "config '{}' não deveria ser Moderate",
                c.meta().id
            );
        }
    }

    #[test]
    fn by_max_risk_moderate_includes_all() {
        // Nenhuma config é Advanced — Moderate inclui todas as 8.
        assert_eq!(reg().by_max_risk(&ConfigRisk::Moderate).len(), 8);
    }

    // ── Reboot ───────────────────────────────────────────────────────────────────────

    #[test]
    fn reboot_required_configs_are_exactly_three() {
        let reboot: Vec<&str> = reg()
            .all_meta()
            .into_iter()
            .filter(|m| m.requires_reboot)
            .map(|m| m.id)
            .collect();
        assert!(reboot.contains(&"gpu_hardware_scheduling"));
        assert!(reboot.contains(&"cpu_core_parking"));
        assert!(reboot.contains(&"hpet"));
        assert_eq!(reboot.len(), 3);
    }

    #[test]
    fn reboot_count_is_three() {
        assert_eq!(reg().reboot_required_count(), 3);
    }

    // ── Elevação (P2.4) ──────────────────────────────────────────────────────

    #[test]
    fn power_plans_do_not_require_elevation() {
        let r = reg();
        for id in [
            "power_plan_high_performance",
            "power_plan_balanced",
            "power_plan_power_saver",
        ] {
            let c = r.find(id).expect("power plan deve existir");
            assert!(
                !c.meta().requires_elevation,
                "{id} não deve exigir admin — powercfg /setactive para esquema existente dispensa elevação"
            );
        }
    }

    // ── eligible_for_auto_evidence ───────────────────────────────────────────────────

    #[test]
    fn reboot_configs_are_not_eligible_for_auto_evidence() {
        for m in reg().all_meta() {
            if m.requires_reboot {
                assert!(
                    !m.eligible_for_auto_evidence(),
                    "config '{}' requer reboot mas está marcada como auto-evidence",
                    m.id
                );
            }
        }
    }

    #[test]
    fn non_reboot_reversible_configs_are_eligible_for_auto_evidence() {
        for m in reg().all_meta() {
            if !m.requires_reboot && m.reversible {
                assert!(
                    m.eligible_for_auto_evidence(),
                    "config '{}' deveria ser elegível para auto evidence",
                    m.id
                );
            }
        }
    }

    // ── eligible_for_profiles ────────────────────────────────────────────────────────

    #[test]
    fn eligible_with_all_capabilities_returns_all_eight() {
        let caps = &["fps", "frame_time", "cpu_multi", "cpu_single"];
        assert_eq!(reg().eligible_for_profiles(caps).len(), 8);
    }

    #[test]
    fn eligible_with_only_cpu_multi_excludes_gpu() {
        let r = reg();
        let eligible = r.eligible_for_profiles(&["cpu_multi"]);
        let ids: Vec<&str> = eligible.iter().map(|c| c.meta().id).collect();
        // gpu_hardware_scheduling: fps + frame_time — sem cpu_multi
        assert!(!ids.contains(&"gpu_hardware_scheduling"));
        // visual_effects_gaming: fps + cpu_multi — tem cpu_multi
        assert!(ids.contains(&"visual_effects_gaming"));
    }

    #[test]
    fn eligible_with_no_capabilities_returns_empty() {
        assert!(
            reg().eligible_for_profiles(&[]).is_empty(),
            "Sem capabilities não há config elegível"
        );
    }

    // ── ConfigDefinition trait stubs ─────────────────────────────────────────────────

    #[test]
    fn read_current_returns_not_implemented() {
        assert!(matches!(
            GpuHardwareScheduling.read_current(),
            Err(ConfigError::NotImplemented(_))
        ));
    }

    #[test]
    fn apply_returns_not_implemented() {
        assert!(matches!(
            CpuCoreParking.apply(&Value::Null),
            Err(ConfigError::NotImplemented(_))
        ));
    }

    #[test]
    fn revert_returns_not_implemented() {
        assert!(matches!(
            VisualEffectsGaming.revert(&Value::Null),
            Err(ConfigError::NotImplemented(_))
        ));
    }
}
