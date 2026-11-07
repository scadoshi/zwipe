//! The refresh reuse window: a rotation's successor, remembered for a short
//! while so a racing second presentation of the same token gets it back.
//!
//! Two tabs, or one page firing two refreshes at once, present the same
//! refresh token. The first rotates it; the second, inside the window, gets
//! that same successor instead of a replay's revocation. Both then hold one
//! token, so a later replay of it is still caught. See
//! `context/plans/refresh_reuse_window.md`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use chrono::{DateTime, Duration, Utc};
use zwipe_core::domain::auth::models::refresh_token::Sha256Hash;

use crate::domain::auth::requests::refresh_session::RotatedRefreshToken;

/// Default window: room for two tabs or two calls racing on one token.
pub const REFRESH_REUSE_WINDOW_DEFAULT: Duration = Duration::seconds(30);

/// Links in a successor chain followed at most, a guard against a cycle.
const MAX_CHAIN: usize = 8;

/// A successor and when it was minted.
#[derive(Clone)]
struct Remembered {
    rotated: RotatedRefreshToken,
    at: DateTime<Utc>,
}

/// The successors of the rotations inside the window, by the consumed
/// token's hash. Shared by every clone of the repository.
#[derive(Clone)]
pub struct RefreshReuse {
    window: Duration,
    successors: Arc<Mutex<HashMap<String, Remembered>>>,
}

impl RefreshReuse {
    /// An empty map with a `window`; a zero window remembers nothing.
    #[must_use]
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            successors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// The window a second presentation must land in.
    #[must_use]
    pub fn window(&self) -> Duration {
        self.window
    }

    /// Whether `replaced_at` is recent enough at `now` for a second
    /// presentation to count as a race.
    #[must_use]
    pub fn within(&self, replaced_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
        self.window > Duration::zero() && now.signed_duration_since(replaced_at) <= self.window
    }

    /// Remembers that the token hashed `consumed` was rotated into `rotated`
    /// at `now`, and drops entries past the window.
    pub fn remember(&self, consumed: String, rotated: RotatedRefreshToken, now: DateTime<Utc>) {
        if self.window <= Duration::zero() {
            return;
        }
        let mut successors = self
            .successors
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        successors.retain(|_, r| now.signed_duration_since(r.at) <= self.window);
        successors.insert(consumed, Remembered { rotated, at: now });
    }

    /// The newest successor of the token hashed `consumed` inside the window,
    /// following the chain when that successor was itself rotated, or `None`.
    #[must_use]
    pub fn recall(&self, consumed: &str, now: DateTime<Utc>) -> Option<RotatedRefreshToken> {
        let successors = self
            .successors
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let live = |hash: &str| {
            successors
                .get(hash)
                .filter(|r| now.signed_duration_since(r.at) <= self.window)
        };
        let mut newest = live(consumed)?;
        for _ in 0..MAX_CHAIN {
            match live(&newest.rotated.refresh_token.sha256_hash()) {
                Some(next) => newest = next,
                None => break,
            }
        }
        Some(newest.rotated.clone())
    }
}

impl Default for RefreshReuse {
    fn default() -> Self {
        Self::new(REFRESH_REUSE_WINDOW_DEFAULT)
    }
}

/// Holds credentials: only the window shows.
impl std::fmt::Debug for RefreshReuse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefreshReuse")
            .field("window", &self.window)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::auth::models::refresh_token::RefreshToken;

    use super::*;

    fn rotated(value: &str) -> RotatedRefreshToken {
        RotatedRefreshToken {
            refresh_token: RefreshToken {
                value: value.to_string(),
                expires_at: Utc::now(),
            },
            platform: None,
        }
    }

    /// The stored hash of a token valued `value`.
    fn h(value: &str) -> String {
        value.to_string().sha256_hash()
    }

    fn value(r: Option<RotatedRefreshToken>) -> Option<String> {
        r.map(|r| r.refresh_token.value)
    }

    #[test]
    fn a_second_presentation_inside_the_window_gets_the_same_successor() {
        let reuse = RefreshReuse::default();
        let now = Utc::now();
        reuse.remember(h("old"), rotated("next"), now);
        let later = now + Duration::seconds(29);
        assert_eq!(
            value(reuse.recall(&h("old"), later)),
            Some("next".to_string())
        );
    }

    #[test]
    fn past_the_window_nothing_is_recalled() {
        let reuse = RefreshReuse::default();
        let now = Utc::now();
        reuse.remember(h("old"), rotated("next"), now);
        assert_eq!(reuse.recall(&h("old"), now + Duration::seconds(31)), None);
        assert!(!reuse.within(now, now + Duration::seconds(31)));
        assert!(reuse.within(now, now + Duration::seconds(30)));
    }

    #[test]
    fn a_successor_rotated_again_recalls_the_newest() {
        let reuse = RefreshReuse::default();
        let now = Utc::now();
        reuse.remember(h("a"), rotated("b"), now);
        reuse.remember(h("b"), rotated("c"), now + Duration::seconds(5));
        assert_eq!(
            value(reuse.recall(&h("a"), now + Duration::seconds(10))),
            Some("c".to_string())
        );
    }

    #[test]
    fn old_entries_are_pruned_and_a_zero_window_remembers_nothing() {
        let reuse = RefreshReuse::default();
        let now = Utc::now();
        reuse.remember(h("a"), rotated("b"), now);
        reuse.remember(h("x"), rotated("y"), now + Duration::seconds(60));
        let held = reuse.successors.lock().unwrap().len();
        assert_eq!(held, 1, "the entry past the window was pruned");

        let off = RefreshReuse::new(Duration::zero());
        off.remember(h("a"), rotated("b"), now);
        assert_eq!(off.recall(&h("a"), now), None);
        assert!(!off.within(now, now));
    }
}
