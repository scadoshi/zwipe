//! In-memory TTL cache decorating a [`CardRecommender`].
//!
//! `search_deck_cards` fires on **every page** of the add-card stack, but the
//! recommendation depends only on deck state — so paging, skips, and re-opening
//! the same deck all hit this cache instead of re-calling upstream. Adding a
//! card changes the key, so it's a miss and one fresh call, exactly when a new
//! recommendation is warranted.
//!
//! This matters because the public Recommander release has no API key: it
//! rate-limits by IP — our one server IP, a single bucket shared by every user
//! — so collapsing the redundant per-page calls is what keeps us under the
//! limit. See `plans/recommander-integration.md`.

use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use uuid::Uuid;

use crate::domain::recommendation::{
    models::{CardRecommendation, RecommendError, RecommendQuery},
    ports::CardRecommender,
};

/// Cache key: a deck state. Deck ids are sorted so card order never fragments
/// the key.
type Key = (Uuid, Option<Uuid>, Vec<Uuid>);

/// A cached upstream result plus when it was stored, for TTL expiry.
#[derive(Debug, Clone)]
struct Entry {
    stored_at: Instant,
    recommendations: Vec<CardRecommendation>,
}

/// Once the cache holds this many entries, an insert first sweeps expired rows
/// — a cheap memory bound that needs no background task (deck states churn, so
/// most rows are never re-queried and would otherwise linger until expiry).
const SWEEP_THRESHOLD: usize = 10_000;

/// TTL cache wrapping an inner [`CardRecommender`]. Cloneable and cheap to
/// clone — the map is shared via `Arc`.
#[derive(Debug, Clone)]
pub struct CachingRecommander<R> {
    inner: R,
    ttl: Duration,
    cache: Arc<DashMap<Key, Entry>>,
}

impl<R> CachingRecommander<R> {
    /// Wraps `inner` with an in-memory cache of the given TTL.
    pub fn new(inner: R, ttl: Duration) -> Self {
        Self {
            inner,
            ttl,
            cache: Arc::new(DashMap::new()),
        }
    }

    fn key(query: &RecommendQuery) -> Key {
        let mut deck = query.deck.clone();
        deck.sort();
        (query.commander, query.partner, deck)
    }
}

impl<R: CardRecommender> CardRecommender for CachingRecommander<R> {
    async fn recommend(
        &self,
        query: RecommendQuery,
    ) -> Result<Vec<CardRecommendation>, RecommendError> {
        let key = Self::key(&query);

        // Fresh hit? (The DashMap guard is scoped to this block and dropped
        // before the await below — never held across it.)
        if let Some(entry) = self.cache.get(&key)
            && entry.stored_at.elapsed() < self.ttl
        {
            return Ok(entry.recommendations.clone());
        }

        // Miss or stale: call upstream. Only successful, non-empty results are
        // cached — a transient blip or an empty result must not pin us off the
        // live signal for the whole TTL.
        let recommendations = self.inner.recommend(query).await?;
        if !recommendations.is_empty() {
            if self.cache.len() >= SWEEP_THRESHOLD {
                let ttl = self.ttl;
                self.cache.retain(|_, e| e.stored_at.elapsed() < ttl);
            }
            self.cache.insert(
                key,
                Entry {
                    stored_at: Instant::now(),
                    recommendations: recommendations.clone(),
                },
            );
        }
        Ok(recommendations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Clone)]
    struct CountingRecommender {
        calls: Arc<AtomicUsize>,
        result: Vec<CardRecommendation>,
    }

    impl CardRecommender for CountingRecommender {
        async fn recommend(
            &self,
            _query: RecommendQuery,
        ) -> Result<Vec<CardRecommendation>, RecommendError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.result.clone())
        }
    }

    fn rec(score: f64) -> CardRecommendation {
        CardRecommendation {
            oracle_id: Uuid::nil(),
            name: "X".to_string(),
            score,
        }
    }

    fn query() -> RecommendQuery {
        RecommendQuery {
            commander: Uuid::nil(),
            partner: None,
            deck: vec![],
        }
    }

    fn caching(
        result: Vec<CardRecommendation>,
        ttl: Duration,
    ) -> (CachingRecommander<CountingRecommender>, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        let inner = CountingRecommender {
            calls: Arc::clone(&calls),
            result,
        };
        (CachingRecommander::new(inner, ttl), calls)
    }

    #[tokio::test]
    async fn second_call_same_state_is_served_from_cache() {
        let (cache, calls) = caching(vec![rec(0.9)], Duration::from_secs(60));
        cache.recommend(query()).await.unwrap();
        cache.recommend(query()).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn expired_entry_refetches() {
        let (cache, calls) = caching(vec![rec(0.9)], Duration::from_millis(1));
        cache.recommend(query()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(5)).await;
        cache.recommend(query()).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn empty_result_is_not_cached() {
        let (cache, calls) = caching(vec![], Duration::from_secs(60));
        cache.recommend(query()).await.unwrap();
        cache.recommend(query()).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn different_deck_state_is_a_miss() {
        let (cache, calls) = caching(vec![rec(0.9)], Duration::from_secs(60));
        let mut other = query();
        other.deck = vec![Uuid::from_u128(1)];
        cache.recommend(query()).await.unwrap();
        cache.recommend(other).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn deck_order_does_not_fragment_the_key() {
        let (cache, calls) = caching(vec![rec(0.9)], Duration::from_secs(60));
        let a = Uuid::from_u128(1);
        let b = Uuid::from_u128(2);
        let mut q1 = query();
        q1.deck = vec![a, b];
        let mut q2 = query();
        q2.deck = vec![b, a];
        cache.recommend(q1).await.unwrap();
        cache.recommend(q2).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
