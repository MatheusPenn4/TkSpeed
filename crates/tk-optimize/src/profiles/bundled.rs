//! Perfis bundled — definições estáticas dos perfis oficiais do TkSpeed.
//! Modificar aqui → incrementar `bundle_version` do perfil afetado.

use super::model::{CompositionEntry, ProfileDefinition};

fn entry(config_id: &str) -> CompositionEntry {
    CompositionEntry { config_id: config_id.into(), value: None }
}

/// Competitive FPS — frame time, FPS, latência de input.
pub fn competitive() -> ProfileDefinition {
    ProfileDefinition {
        id: "competitive".into(),
        name: "Competitive FPS".into(),
        description: Some("Maximiza frame time consistente, FPS e latência de input.".into()),
        icon: Some("zap".into()),
        is_custom: false,
        compositions: vec![
            entry("gpu_hardware_scheduling"),
            entry("timer_resolution"),
            entry("power_plan_high_performance"),
        ],
        suite_id: "complete".into(),
        requires_fps: true,
        bundle_version: 1,
    }
}

/// Balanced Gaming — FPS estável com temperatura controlada.
pub fn balanced() -> ProfileDefinition {
    ProfileDefinition {
        id: "balanced".into(),
        name: "Balanced Gaming".into(),
        description: Some("Equilibra desempenho, estabilidade e temperatura.".into()),
        icon: Some("layers".into()),
        is_custom: false,
        compositions: vec![
            entry("gpu_hardware_scheduling"),
            entry("power_plan_balanced"),
        ],
        suite_id: "complete".into(),
        requires_fps: false,
        bundle_version: 1,
    }
}

/// Streaming — multitarefa, CPU multi-thread, memória.
pub fn streaming() -> ProfileDefinition {
    ProfileDefinition {
        id: "streaming".into(),
        name: "Streaming".into(),
        description: Some("Otimizado para multitarefa, encoding e gerenciamento de RAM.".into()),
        icon: Some("radio".into()),
        is_custom: false,
        compositions: vec![
            entry("memory_standby_flush"),
            entry("power_plan_balanced"),
        ],
        suite_id: "complete".into(),
        requires_fps: false,
        bundle_version: 1,
    }
}

/// Power Saver — temperatura e consumo de energia.
pub fn power_saver() -> ProfileDefinition {
    ProfileDefinition {
        id: "power_saver".into(),
        name: "Power Saver".into(),
        description: Some("Prioriza bateria e temperatura baixa.".into()),
        icon: Some("battery".into()),
        is_custom: false,
        compositions: vec![
            entry("power_plan_power_saver"),
        ],
        suite_id: "complete".into(),
        requires_fps: false,
        bundle_version: 1,
    }
}

/// Retorna todos os perfis bundled.
pub fn all() -> Vec<ProfileDefinition> {
    vec![competitive(), balanced(), streaming(), power_saver()]
}
