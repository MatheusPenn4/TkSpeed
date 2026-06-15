//! Recommendation Engine Foundation — gera recomendações ordenadas por evidência real.
//!
//! Entrada: `RecommendationContext` (evidência, capacidades, histórico, perfil ativo).
//! Saída:   `Vec<Recommendation>` ordenado por score decrescente via `top_n(5)`.
//!
//! Sem IA, sem LLM, sem heurísticas inventadas — só dados reais + confidence engine.

pub mod engine;
pub mod model;
pub mod scoring;

pub use engine::RecommendationEngine;
pub use model::{Recommendation, RecommendationContext, RecommendationKind};
