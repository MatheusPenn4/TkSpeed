//! Profile Engine — agrupa configurações em perfis ativáveis com snapshot + rollback.

pub mod bundled;
pub mod engine;
pub mod evidence;
pub mod executor;
pub mod measurement;
pub mod model;
pub mod repo;

pub use engine::ProfileEngine;
pub use evidence::{ProfileEvidenceRecord, ProfileEvidenceRepo};
pub use measurement::MeasurementPipeline;
pub use model::{
    ActivationResult, ActivationSummary, EvidenceOutcome, ProfileConfigPreview, ProfileDefinition,
    ProfileMeasureResult, ProfilePreview, ProfileStateRow,
};
