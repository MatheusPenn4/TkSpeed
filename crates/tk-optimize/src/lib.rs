//! TkOptimization Engine (Fase 2) — loop fechado de otimização baseada em evidência.

pub mod capabilities;
pub mod catalog;
pub mod configs;
pub mod engine;
pub mod evidence;
pub mod machine;
pub mod profiles;
pub mod recommendations;

pub use catalog::{catalog_info, Optimization, Preview};
pub use configs::{ConfigCategory, ConfigDefinition, ConfigError, ConfigMeta, ConfigRegistry, ConfigRisk};
pub use engine::Engine;
pub use evidence::{confidence_for_executions, extract_primary_gain, EvidenceRecord, EvidenceRepo};
pub use profiles::{
    ActivationResult, EvidenceOutcome, MeasurementPipeline, ProfileDefinition, ProfileEngine,
    ProfileEvidenceRecord, ProfileEvidenceRepo, ProfileMeasureResult, ProfilePreview,
};
pub use recommendations::{Recommendation, RecommendationContext, RecommendationEngine, RecommendationKind};
