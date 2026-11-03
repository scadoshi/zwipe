//! One opening of a screen, which its requests carry out and its replies carry back.
//!
//! A zwipe-ui model's tickets start over each time its screen opens, and a request's task outlives the screen that sent it. A screen that can close and open again while its replies are out stamps each request with its [`Visit`] and lands a reply only in the view of the same visit.

use std::sync::atomic::{AtomicU64, Ordering};

/// One opening of a screen, unlike every other opening in the process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Visit(u64);

/// The number the next [`Visit`] takes.
static VISITS: AtomicU64 = AtomicU64::new(0);

impl Visit {
    /// A visit no earlier one shares.
    pub(crate) fn next() -> Self {
        Self(VISITS.fetch_add(1, Ordering::Relaxed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_visit_is_new() {
        assert_ne!(Visit::next(), Visit::next());
    }
}
