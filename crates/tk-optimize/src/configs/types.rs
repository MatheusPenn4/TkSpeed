use thiserror::Error;

/// Categoria funcional de uma configuração de sistema.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfigCategory {
    Gpu,
    Memory,
    Cpu,
    Timer,
    Power,
    Display,
}

impl ConfigCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigCategory::Gpu => "gpu",
            ConfigCategory::Memory => "memory",
            ConfigCategory::Cpu => "cpu",
            ConfigCategory::Timer => "timer",
            ConfigCategory::Power => "power",
            ConfigCategory::Display => "display",
        }
    }
}

/// Nível de risco de uma configuração de sistema.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigRisk {
    Safe,
    Moderate,
    Advanced,
}

impl ConfigRisk {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigRisk::Safe => "safe",
            ConfigRisk::Moderate => "moderate",
            ConfigRisk::Advanced => "advanced",
        }
    }
}

/// Metadados completos de uma configuração de sistema.
/// Imutável — descreve o que a configuração é e como se comporta.
#[derive(Debug, Clone)]
pub struct ConfigMeta {
    /// Identificador único estável. Ex.: "gpu_hardware_scheduling".
    pub id: &'static str,
    /// Nome legível (PT-BR). Ex.: "Agendamento de GPU por Hardware".
    pub name: &'static str,
    /// Descrição funcional para o usuário.
    pub description: &'static str,
    pub category: ConfigCategory,
    pub risk: ConfigRisk,
    /// Se a configuração pode ser completamente revertida via snapshot.
    pub reversible: bool,
    /// Se o efeito completo requer reinicialização do sistema.
    /// Configurações com requires_reboot = true ficam FORA do pipeline automático
    /// de evidência — são aplicadas e marcadas como pending_reboot.
    pub requires_reboot: bool,
    /// Se a operação requer privilégios de administrador.
    pub requires_elevation: bool,
    /// IDs das suites de benchmark relevantes para medir o efeito desta config.
    /// Slice estática aberta: novas suites em V4.3/V4.4 não forçam refatoração.
    pub benchmark_relevance: &'static [&'static str],
}

impl ConfigMeta {
    /// Retorna true se esta config pode participar do pipeline de evidência
    /// automático (reversível e sem reboot necessário).
    pub fn eligible_for_auto_evidence(&self) -> bool {
        self.reversible && !self.requires_reboot
    }

    /// Retorna true se ao menos um benchmark_relevance está em `capabilities`.
    pub fn has_measurable_evidence(&self, capabilities: &[&str]) -> bool {
        self.benchmark_relevance
            .iter()
            .any(|r| capabilities.contains(r))
    }
}

/// Erro de operação nas configurações de sistema.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("implementação pendente: {0}")]
    NotImplemented(&'static str),
    #[error("requer privilégios de administrador")]
    ElevationRequired,
    #[error("configuração não encontrada: {0}")]
    NotFound(String),
    #[error("falha ao ler estado atual: {0}")]
    ReadFailed(String),
    #[error("falha ao aplicar: {0}")]
    ApplyFailed(String),
    #[error("falha ao reverter: {0}")]
    RevertFailed(String),
    #[error("reboot necessário para efeito completo")]
    RebootRequired,
}
