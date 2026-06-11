//! TkOptimization Engine (Fase 2) — loop fechado de otimização baseada em evidência.
//!
//! Orquestra os serviços já validados:
//!   ProtectionService (snapshot/rollback) + tk-perflab (benchmark/Confidence/compare).
//! Pipeline obrigatório: snapshot → bench antes → aplicar → bench depois →
//! Confidence Engine → comparar → manter ou reverter. Sem evidência = inconclusivo.
//!
//! OE-1: piloto `energy.power_plan_high` (MODERATE, reversível, mensurável).

pub mod catalog;
pub mod engine;

pub use catalog::{catalog_info, Optimization, Preview};
pub use engine::Engine;
