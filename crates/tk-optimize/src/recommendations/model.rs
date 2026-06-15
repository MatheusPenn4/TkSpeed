//! Tipos de entrada e saída do Recommendation Engine.

use serde::Serialize;
use tk_contracts::BenchmarkSessionInfo;

use crate::capabilities::Capability;
use crate::evidence::EvidenceRecord;
use crate::profiles::evidence::ProfileEvidenceRecord;

/// Classificação funcional de uma recomendação.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum RecommendationKind {
    /// Recomendação de configuração individual (ex.: power plan).
    Config,
    /// Recomendação de perfil completo (ex.: Competitive FPS).
    Profile,
    /// Recomendação de executar benchmark para estabelecer baseline.
    Benchmark,
    /// Recomendação de manutenção (ex.: limpar snapshots antigos).
    Maintenance,
}

/// Recomendação gerada pelo engine, ordenada por score.
#[derive(Debug, Clone, Serialize)]
pub struct Recommendation {
    /// Identificador único (ex.: "config:visual_effects_gaming", "profile:competitive").
    pub id: String,
    pub title: String,
    pub description: String,
    pub kind: RecommendationKind,
    /// Score interno para ordenação — não expor diretamente ao usuário.
    pub score: f64,
    /// Confiança histórica (0–95), derivada da evidência acumulada.
    pub confidence: u8,
    /// Ganho médio observado (%), `None` se sem evidência de sucesso.
    pub estimated_gain: Option<f64>,
    /// "safe" | "moderate" | "advanced"
    pub risk: String,
    pub requires_reboot: bool,
    /// Explicação legível do motivo da recomendação.
    pub reason: String,
}

/// Contexto completo fornecido ao engine para gerar recomendações.
///
/// O chamador é responsável por construir o contexto com dados reais:
/// evidência filtrada por fingerprint, capacidades do sistema, histórico
/// de sessões e IDs de configurações confirmadas como executáveis nesta máquina.
pub struct RecommendationContext {
    pub machine_fingerprint: String,
    /// Capacidades detectadas em runtime (matrix::build()).
    pub capabilities: Vec<Capability>,
    /// Sessões de benchmark recentes; vazio se nenhum benchmark foi executado.
    pub recent_sessions: Vec<BenchmarkSessionInfo>,
    /// Perfil atualmente ativo ("competitive", "balanced", …), ou None.
    pub active_profile: Option<String>,
    /// Config IDs que já estão aplicadas via perfil ativo — excluídas das recomendações individuais.
    pub active_config_ids: Vec<String>,
    /// Evidência acumulada por config, filtrada para este fingerprint.
    pub config_evidence: Vec<EvidenceRecord>,
    /// Evidência acumulada por perfil, filtrada para este fingerprint.
    pub profile_evidence: Vec<ProfileEvidenceRecord>,
    /// Config IDs que executor::build_action confirmou como Executable nesta máquina.
    pub applicable_config_ids: Vec<String>,
}
