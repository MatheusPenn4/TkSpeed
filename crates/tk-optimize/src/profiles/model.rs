//! Structs do Profile Engine — dados, preview e resultado de ativação.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Uma configuração dentro de um perfil, com valor opcional.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionEntry {
    pub config_id: String,
    /// Valor a aplicar; `None` = usar padrão da config.
    pub value: Option<Value>,
}

/// Definição de um perfil (bundled ou customizado).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileDefinition {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub is_custom: bool,
    pub compositions: Vec<CompositionEntry>,
    pub suite_id: String,
    pub requires_fps: bool,
    pub bundle_version: u32,
}

/// Informação de uma config individual dentro do preview.
#[derive(Debug, Clone, Serialize)]
pub struct ProfileConfigPreview {
    pub config_id: String,
    pub name: String,
    pub risk: String,
    pub reversible: bool,
    pub requires_reboot: bool,
    pub benchmark_relevance: Vec<String>,
}

/// Resultado do preview de um perfil (sem aplicar nada).
#[derive(Debug, Clone, Serialize)]
pub struct ProfilePreview {
    pub profile_id: String,
    pub name: String,
    pub description: Option<String>,
    pub configs: Vec<ProfileConfigPreview>,
    pub requires_reboot: bool,
    pub max_risk: String,
    pub benchmark_relevance: Vec<String>,
}

/// Resultado de uma ativação de perfil.
#[derive(Debug, Clone, Serialize)]
pub struct ActivationResult {
    pub activation_id: i64,
    pub profile_id: String,
    pub snapshot_id: i64,
    /// Configs que foram aplicadas com sucesso.
    pub applied_configs: Vec<String>,
    /// Configs puladas (NotImplemented nesta versão).
    pub skipped_configs: Vec<String>,
    pub pending_reboot: bool,
    pub message: String,
}

/// Linha lida do profile_state.
#[derive(Debug, Clone)]
pub struct ProfileStateRow {
    pub user_context: String,
    pub profile_id: Option<String>,
    pub activated_at: Option<i64>,
    pub snapshot_id: Option<i64>,
    pub pending_reboot: bool,
}

/// Dados para INSERT em profile_activations.
pub struct ActivationRow {
    pub ts: i64,
    pub user_context: String,
    pub profile_id: String,
    pub from_profile_id: Option<String>,
    pub snapshot_id: i64,
    pub machine_fingerprint: Option<String>,
    pub pending_reboot: bool,
    pub source: String,
}

/// Resumo de uma ativação passada.
#[derive(Debug, Clone, Serialize)]
pub struct ActivationSummary {
    pub id: i64,
    pub ts: i64,
    pub profile_id: String,
    pub from_profile_id: Option<String>,
    pub pending_reboot: bool,
    pub source: String,
}

/// Desfecho do processamento de evidência após comparação de benchmark.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum EvidenceOutcome {
    /// Comparação confiável mostrou ganho real → record_success chamado.
    Success { gain: f64 },
    /// Comparação confiável sem ganho (perda ou sem mudança) → record_failure chamado.
    Failure,
    /// Sessões instáveis ou comparação não confiável → sem evidência registrada.
    Inconclusive,
    /// Perfil com pending_reboot → after benchmark bloqueado; evidência após reboot.
    PendingReboot,
}

/// Resultado completo de `activate_with_measure`.
#[derive(Debug, Clone, Serialize)]
pub struct ProfileMeasureResult {
    pub profile_id: String,
    pub activation_id: i64,
    pub snapshot_id: i64,
    pub before_session_id: Option<i64>,
    pub after_session_id: Option<i64>,
    pub evidence_recorded: EvidenceOutcome,
    pub pending_reboot: bool,
    pub message: String,
}
