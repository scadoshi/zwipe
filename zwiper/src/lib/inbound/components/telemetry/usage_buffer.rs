//! In-memory swipe/search counters with snapshot-and-zero semantics.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use zwipe_core::http::contracts::metrics::HttpUsageBatch;

use crate::inbound::components::interactions::swipe::direction::Direction;

/// Atomic buffer of pending usage counters. Cheap clone (Arc inner).
#[derive(Debug, Clone, Default)]
pub struct UsageBuffer {
    inner: Arc<UsageBufferInner>,
}

#[derive(Debug, Default)]
struct UsageBufferInner {
    swipes_right: AtomicU32,
    swipes_left: AtomicU32,
    swipes_up: AtomicU32,
    swipes_down: AtomicU32,
    searches: AtomicU32,
}

impl UsageBuffer {
    /// Creates an empty buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records one swipe in the given direction.
    pub fn record_swipe(&self, direction: Direction) {
        let counter = match direction {
            Direction::Right => &self.inner.swipes_right,
            Direction::Left => &self.inner.swipes_left,
            Direction::Up => &self.inner.swipes_up,
            Direction::Down => &self.inner.swipes_down,
        };
        counter.fetch_add(1, Ordering::Relaxed);
    }

    /// Records one card search.
    pub fn record_search(&self) {
        self.inner.searches.fetch_add(1, Ordering::Relaxed);
    }

    /// Snapshots all counters into a batch and resets them to zero.
    ///
    /// Returns `None` when every counter is zero (nothing to flush).
    pub fn snapshot_and_zero(&self) -> Option<HttpUsageBatch> {
        let right = self.inner.swipes_right.swap(0, Ordering::Relaxed);
        let left = self.inner.swipes_left.swap(0, Ordering::Relaxed);
        let up = self.inner.swipes_up.swap(0, Ordering::Relaxed);
        let down = self.inner.swipes_down.swap(0, Ordering::Relaxed);
        let searches = self.inner.searches.swap(0, Ordering::Relaxed);

        if right == 0 && left == 0 && up == 0 && down == 0 && searches == 0 {
            return None;
        }

        Some(HttpUsageBatch {
            swipes_right: right,
            swipes_left: left,
            swipes_up: up,
            swipes_down: down,
            searches,
        })
    }
}
