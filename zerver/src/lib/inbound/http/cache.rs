//! Serving-layer caches.
//!
//! [`TtlSlot`] is a cached *variable*, not a cache store: one global value
//! that knows how to refresh itself. There are no keys and no lookups; the
//! slot holds THE current value (the featured flavor card, a metrics blob)
//! plus the deadline after which it must be rebuilt. Overwrite-on-expiry is
//! the entire eviction policy.
//!
//! Deadlines are pinned instants, not durations, so each use case aligns
//! expiry to whatever boundary it wants (top of the UTC hour, +5 minutes,
//! next sync). `AppState` carries one typed slot per use case; the "registry"
//! is the struct itself, so every cached type is known at compile time and
//! there are no string keys to collide.

use std::time::{Duration, Instant};

use tokio::sync::RwLock;

/// How long a stale value is served after a failed refresh before the next
/// attempt, so a database outage costs one refresh per window rather than one
/// per request.
const REFRESH_RETRY_BACKOFF: Duration = Duration::from_secs(30);

/// One-slot, deadline-based cache. See the module docs for the model.
#[derive(Debug)]
pub struct TtlSlot<V> {
    slot: RwLock<Option<(Instant, V)>>,
}

impl<V> Default for TtlSlot<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> TtlSlot<V> {
    /// An empty slot; the first `get_or_refresh` populates it.
    pub fn new() -> Self {
        Self {
            slot: RwLock::new(None),
        }
    }
}

impl<V: Clone> TtlSlot<V> {
    /// Serves the stored value while its deadline is in the future; otherwise
    /// rebuilds via `refresh`, which returns the new value AND its deadline.
    ///
    /// Single-flight: an expired slot funnels concurrent callers through the
    /// write lock, the first rebuilds, the rest re-check and serve the fresh
    /// value; one refresh per expiry no matter the request load.
    ///
    /// Stale grace: if the refresh fails but an expired value exists, the
    /// stale value is served, the error logged, and the deadline pushed out by
    /// `REFRESH_RETRY_BACKOFF` so callers don't queue on the write lock to
    /// retry a failing refresh one after another; for content like the
    /// featured flavor, an hour-stale card beats a 5xx.
    ///
    /// # Errors
    ///
    /// Returns `refresh`'s error only when the slot is empty; with a stale
    /// value on hand, the error is logged and the stale value served.
    pub async fn get_or_refresh<F, Fut, E>(&self, refresh: F) -> Result<V, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<(V, Instant), E>>,
        E: std::fmt::Display,
    {
        if let Some(value) = self.fresh().await {
            return Ok(value);
        }
        self.refresh_locked(refresh).await
    }

    /// The stored value while its deadline is in the future, read under the read lock.
    async fn fresh(&self) -> Option<V> {
        let read = self.slot.read().await;
        read.as_ref()
            .filter(|(expires_at, _)| *expires_at > Instant::now())
            .map(|(_, value)| value.clone())
    }

    /// The rebuild under the write lock, for a caller that found the slot expired: serves the
    /// value another caller rebuilt while this one waited for the lock, else runs `refresh`
    /// with [`get_or_refresh`](Self::get_or_refresh)'s stale grace.
    async fn refresh_locked<F, Fut, E>(&self, refresh: F) -> Result<V, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<(V, Instant), E>>,
        E: std::fmt::Display,
    {
        let mut write = self.slot.write().await;
        // Double-check: another caller may have refreshed while we waited.
        if let Some((expires_at, value)) = write.as_ref()
            && *expires_at > Instant::now()
        {
            return Ok(value.clone());
        }

        match refresh().await {
            Ok((value, expires_at)) => {
                *write = Some((expires_at, value.clone()));
                Ok(value)
            }
            Err(error) => match write.as_mut() {
                Some((expires_at, stale)) => {
                    tracing::warn!("TtlSlot refresh failed, serving stale value: {error}");
                    let now = Instant::now();
                    *expires_at = now.checked_add(REFRESH_RETRY_BACKOFF).unwrap_or(now);
                    Ok(stale.clone())
                }
                None => Err(error),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    use super::*;

    fn far() -> Instant {
        Instant::now()
            .checked_add(Duration::from_secs(3600))
            .unwrap()
    }

    #[tokio::test]
    async fn serves_fresh_without_refreshing() {
        let slot = TtlSlot::new();
        let calls = AtomicU32::new(0);
        for _ in 0..3 {
            let value: Result<i32, String> = slot
                .get_or_refresh(|| async {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok((7, far()))
                })
                .await;
            assert_eq!(value.unwrap(), 7);
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn rebuilds_after_deadline() {
        let slot = TtlSlot::new();
        let _: Result<i32, String> = slot
            .get_or_refresh(|| async { Ok((1, Instant::now())) })
            .await;
        // Deadline == now counts as expired (freshness requires strictly future).
        let value: Result<i32, String> = slot.get_or_refresh(|| async { Ok((2, far())) }).await;
        assert_eq!(value.unwrap(), 2);
    }

    #[tokio::test]
    async fn stale_grace_on_refresh_failure() {
        let slot = TtlSlot::new();
        let _: Result<i32, String> = slot
            .get_or_refresh(|| async { Ok((1, Instant::now())) })
            .await;
        // Expired + failing refresh -> the stale value, not the error.
        let value = slot
            .get_or_refresh(|| async { Err::<(i32, Instant), _>("db down".to_string()) })
            .await;
        assert_eq!(value.unwrap(), 1);
        // Empty slot + failing refresh -> the error.
        let empty: TtlSlot<i32> = TtlSlot::new();
        let error = empty
            .get_or_refresh(|| async { Err::<(i32, Instant), _>("db down".to_string()) })
            .await;
        assert_eq!(error.unwrap_err(), "db down");
    }

    #[tokio::test]
    async fn single_flight_on_concurrent_expiry() {
        let slot = std::sync::Arc::new(TtlSlot::new());
        let calls = std::sync::Arc::new(AtomicU32::new(0));
        let tasks: Vec<_> = (0..8)
            .map(|_| {
                let slot = std::sync::Arc::clone(&slot);
                let calls = std::sync::Arc::clone(&calls);
                tokio::spawn(async move {
                    let value: Result<i32, String> = slot
                        .get_or_refresh(|| async {
                            calls.fetch_add(1, Ordering::SeqCst);
                            // Hold the rebuild briefly so the herd piles up.
                            tokio::time::sleep(Duration::from_millis(20)).await;
                            Ok((42, far()))
                        })
                        .await;
                    value.unwrap()
                })
            })
            .collect();
        for task in tasks {
            assert_eq!(task.await.unwrap(), 42);
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    /// Two callers both find the slot expired before either takes the write lock; the first
    /// rebuilds, and the second, taking the lock after it, serves that value without
    /// rebuilding again. Each step runs in turn, so the interleaving is the same on any runtime.
    #[tokio::test]
    async fn a_caller_that_saw_the_slot_expired_serves_the_value_rebuilt_while_it_waited() {
        let slot = TtlSlot::new();
        let _: Result<i32, String> = slot
            .get_or_refresh(|| async { Ok((1, Instant::now())) })
            .await;
        assert_eq!(
            slot.fresh().await,
            None,
            "the first caller finds it expired"
        );
        assert_eq!(slot.fresh().await, None, "and so does the second");
        let calls = AtomicU32::new(0);
        let rebuild = || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok::<_, String>((42, far()))
        };
        assert_eq!(slot.refresh_locked(rebuild).await.unwrap(), 42);
        assert_eq!(slot.refresh_locked(rebuild).await.unwrap(), 42);
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "the second caller rebuilt a value already fresh"
        );
    }

    #[tokio::test]
    async fn failed_refresh_backs_off_before_retrying() {
        let slot = TtlSlot::new();
        let _: Result<i32, String> = slot
            .get_or_refresh(|| async { Ok((1, Instant::now())) })
            .await;
        let calls = AtomicU32::new(0);
        for _ in 0..2 {
            let value = slot
                .get_or_refresh(|| async {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Err::<(i32, Instant), _>("db down".to_string())
                })
                .await;
            assert_eq!(value.unwrap(), 1);
        }
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "the second call retried at once"
        );
    }
}
