//! Config Execution Adapter — converte config_id em ReversibleAction.
//!
//! Configs que requerem HKLM (elevação) ou chamadas de kernel são marcadas como
//! Unsupported nesta versão. Configs HKCU e power-plan são totalmente suportadas.

use tk_platform_win::power;
use tk_rollback::ReversibleAction;

// GUIDs padrão dos planos de energia do Windows (imutáveis em todas as versões).
pub const GUID_HIGH_PERF: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
pub const GUID_BALANCED: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";
pub const GUID_POWER_SAVER: &str = "a1841308-3541-4fab-bc81-f71556f20b4a";

/// Resultado da tentativa de construir uma ação para um config_id.
pub enum ConfigAction {
    /// Ação reversível pronta para aplicar.
    Executable(ReversibleAction),
    /// Config não suportada nesta versão.
    Unsupported { reason: &'static str },
}

/// Constrói a ação reversível para um config_id.
///
/// Lê o estado atual do sistema para preencher o valor "antes" da ação.
/// Retorna `Err` somente se houver falha ao ler o estado atual (e.g., OS error).
pub fn build_action(config_id: &str) -> Result<ConfigAction, String> {
    match config_id {
        "power_plan_high_performance" => power_plan(GUID_HIGH_PERF),
        "power_plan_balanced" => power_plan(GUID_BALANCED),
        "power_plan_power_saver" => power_plan(GUID_POWER_SAVER),
        "gpu_hardware_scheduling" => Ok(ConfigAction::Unsupported {
            reason: "requer chave HKLM — suporte de elevação pendente",
        }),
        "timer_resolution" => Ok(ConfigAction::Unsupported {
            reason: "requer chave HKLM — suporte de elevação pendente",
        }),
        "cpu_core_parking" => Ok(ConfigAction::Unsupported {
            reason: "requer chave HKLM — suporte de elevação pendente",
        }),
        "hpet" => Ok(ConfigAction::Unsupported {
            reason: "requer bcdedit — suporte de elevação pendente",
        }),
        "memory_standby_flush" => Ok(ConfigAction::Unsupported {
            reason: "requer chamada de kernel não disponível nesta versão",
        }),
        "visual_effects_gaming" => visual_effects(),
        _ => Ok(ConfigAction::Unsupported {
            reason: "config não reconhecida pelo executor",
        }),
    }
}

fn power_plan(target_guid: &'static str) -> Result<ConfigAction, String> {
    if !power::scheme_exists(target_guid) {
        return Ok(ConfigAction::Unsupported {
            reason: "plano de energia não disponível nesta instalação (powercfg /list)",
        });
    }
    let old = power::get_active_scheme()
        .map_err(|e| format!("falha ao ler plano de energia ativo: {e}"))?;
    Ok(ConfigAction::Executable(ReversibleAction::PowerPlan {
        old_guid: old,
        new_guid: target_guid.into(),
    }))
}

fn visual_effects() -> Result<ConfigAction, String> {
    const SUBKEY: &str =
        "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VisualEffects";
    const NAME: &str = "VisualFXSetting";
    let old = tk_platform_win::registry::read_u32(SUBKEY, NAME)
        .map_err(|e| format!("falha ao ler VisualFXSetting: {e}"))?;
    Ok(ConfigAction::Executable(ReversibleAction::RegistryHkcuDword {
        subkey: SUBKEY.into(),
        name: NAME.into(),
        old,
        new: Some(2),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tk_platform_win::power;
    use tk_rollback::ReversibleAction;

    // ── Power plan configs — resultado depende do sistema ───────────────────
    // scheme_exists() é a fonte de verdade: se listado pelo OS → Executable,
    // senão → Unsupported. Testes usam guards para adaptar a cada máquina.

    #[test]
    fn high_performance_result_matches_scheme_existence() {
        let exists = power::scheme_exists(GUID_HIGH_PERF);
        let action = build_action("power_plan_high_performance").unwrap();
        if exists {
            assert!(matches!(action, ConfigAction::Executable(_)), "plano existe mas retornou Unsupported");
        } else {
            assert!(matches!(action, ConfigAction::Unsupported { .. }), "plano ausente mas retornou Executable");
        }
    }

    #[test]
    fn balanced_is_always_available_and_executable() {
        // Balanced (381b4222-...) é o plano padrão do Windows, sempre presente.
        assert!(power::scheme_exists(GUID_BALANCED), "Balanced deve existir em toda instalação Windows");
        let action = build_action("power_plan_balanced").unwrap();
        assert!(matches!(action, ConfigAction::Executable(_)));
    }

    #[test]
    fn power_saver_result_matches_scheme_existence() {
        let exists = power::scheme_exists(GUID_POWER_SAVER);
        let action = build_action("power_plan_power_saver").unwrap();
        if exists {
            assert!(matches!(action, ConfigAction::Executable(_)));
        } else {
            assert!(matches!(action, ConfigAction::Unsupported { .. }));
        }
    }

    #[test]
    fn balanced_action_has_correct_new_guid() {
        // Balanced sempre existe — podemos testar o GUID mapeado diretamente.
        let action = build_action("power_plan_balanced").unwrap();
        if let ConfigAction::Executable(ReversibleAction::PowerPlan { new_guid, .. }) = action {
            assert_eq!(new_guid, GUID_BALANCED);
        } else {
            panic!("power_plan_balanced deve ser Executable");
        }
    }

    #[test]
    fn high_perf_action_has_correct_new_guid_when_available() {
        if !power::scheme_exists(GUID_HIGH_PERF) { return; }
        let action = build_action("power_plan_high_performance").unwrap();
        if let ConfigAction::Executable(ReversibleAction::PowerPlan { new_guid, .. }) = action {
            assert_eq!(new_guid, GUID_HIGH_PERF);
        } else {
            panic!("high_performance deve ser Executable quando disponível");
        }
    }

    #[test]
    fn power_saver_action_has_correct_new_guid_when_available() {
        if !power::scheme_exists(GUID_POWER_SAVER) { return; }
        let action = build_action("power_plan_power_saver").unwrap();
        if let ConfigAction::Executable(ReversibleAction::PowerPlan { new_guid, .. }) = action {
            assert_eq!(new_guid, GUID_POWER_SAVER);
        } else {
            panic!("power_saver deve ser Executable quando disponível");
        }
    }

    #[test]
    fn power_plan_action_captures_old_guid() {
        // Balanced sempre existe — testa captura do estado atual.
        let action = build_action("power_plan_balanced").unwrap();
        if let ConfigAction::Executable(ReversibleAction::PowerPlan { old_guid, .. }) = action {
            assert_eq!(old_guid.len(), 36, "GUID deve ter 36 caracteres sem braces");
            assert!(old_guid.contains('-'), "GUID deve conter hífens");
        } else {
            panic!("power_plan_balanced deve ser Executable");
        }
    }

    // ── Configs HKLM / kernel — sempre Unsupported ──────────────────────────

    #[test]
    fn gpu_hardware_scheduling_is_unsupported() {
        let action = build_action("gpu_hardware_scheduling").unwrap();
        assert!(matches!(action, ConfigAction::Unsupported { .. }));
    }

    #[test]
    fn timer_resolution_is_unsupported() {
        let action = build_action("timer_resolution").unwrap();
        assert!(matches!(action, ConfigAction::Unsupported { .. }));
    }

    #[test]
    fn cpu_core_parking_is_unsupported() {
        let action = build_action("cpu_core_parking").unwrap();
        assert!(matches!(action, ConfigAction::Unsupported { .. }));
    }

    #[test]
    fn memory_standby_flush_is_unsupported() {
        let action = build_action("memory_standby_flush").unwrap();
        assert!(matches!(action, ConfigAction::Unsupported { .. }));
    }

    #[test]
    fn hpet_is_unsupported() {
        let action = build_action("hpet").unwrap();
        assert!(matches!(action, ConfigAction::Unsupported { .. }));
    }

    #[test]
    fn unknown_config_is_unsupported() {
        let action = build_action("totally_unknown_config_xyz").unwrap();
        assert!(matches!(action, ConfigAction::Unsupported { .. }));
    }

    // ── build_action nunca retorna Err para configs conhecidas ───────────────

    #[test]
    fn no_known_config_returns_err() {
        let configs = [
            "power_plan_high_performance",
            "power_plan_balanced",
            "power_plan_power_saver",
            "gpu_hardware_scheduling",
            "timer_resolution",
            "cpu_core_parking",
            "memory_standby_flush",
            "hpet",
            "visual_effects_gaming",
        ];
        for id in configs {
            assert!(build_action(id).is_ok(), "config '{id}' não deve retornar Err");
        }
    }
}
