//! [`CardRecommender`] outbound port for live, deck-aware card recommendations.

use crate::domain::recommendation::models::{CardRecommendation, RecommendError, RecommendQuery};
use std::future::Future;

/// Outbound port for live, deck-aware card recommendations.
///
/// Implemented by the `Recommander` adapter in `outbound/recommander`. Callers
/// (the deck service) depend on this trait, not the concrete adapter — keeping
/// the recommendation source swappable and the request path resilient: any
/// `Err` degrades to the cached synergy signal rather than failing the search.
pub trait CardRecommender: Clone + Send + Sync + 'static {
    /// Fetch recommended cards for the given deck state.
    fn recommend(
        &self,
        query: RecommendQuery,
    ) -> impl Future<Output = Result<Vec<CardRecommendation>, RecommendError>> + Send;
}
