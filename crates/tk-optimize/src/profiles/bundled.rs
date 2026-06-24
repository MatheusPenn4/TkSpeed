//! Perfis bundled — definições estáticas dos perfis oficiais do TkSpeed.
//! Modificar aqui → incrementar `bundle_version` do perfil afetado.

use super::model::{CompositionEntry, ProfileDefinition};

fn entry(config_id: &str) -> CompositionEntry {
    CompositionEntry { config_id: config_id.into(), value: None }
}

/// Competitive FPS — frame time, FPS, latência de input.
/// `timer_resolution` removido (P2.4): era Unsupported no executor → nunca aplicava,
/// só poluía os configs "skipped" e criava expectativa falsa. Volta quando implementado.
pub fn competitive() -> ProfileDefinition {
    ProfileDefinition {
        id: "competitive".into(),
        name: "Competitive FPS".into(),
        description: Some("Prioriza desempenho de GPU e CPU para FPS e frame time consistentes.".into()),
        icon: Some("zap".into()),
        is_custom: false,
        compositions: vec![
            entry("gpu_hardware_scheduling"),
            entry("power_plan_high_performance"),
        ],
        suite_id: "complete".into(),
        requires_fps: true,
        bundle_version: 2,
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

/// Energia Equilibrada (id histórico: "streaming").
/// P2.4: renomeado para refletir o que realmente faz. Após a remoção do
/// `memory_standby_flush` (irreversível), este perfil aplica APENAS o plano de
/// energia Balanceado do Windows — não altera GPU/CPU nem "otimiza encoding".
/// Descrição honesta para Beta Fechado (Opção B/C). `id` preservado para não
/// quebrar evidência/histórico já gravados.
pub fn streaming() -> ProfileDefinition {
    ProfileDefinition {
        id: "streaming".into(),
        name: "Energia Equilibrada".into(),
        description: Some(
            "Mantém o plano de energia Balanceado do Windows. Não altera GPU nem CPU."
                .into(),
        ),
        icon: Some("radio".into()),
        is_custom: false,
        compositions: vec![
            entry("power_plan_balanced"),
        ],
        suite_id: "complete".into(),
        requires_fps: false,
        bundle_version: 3,
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
