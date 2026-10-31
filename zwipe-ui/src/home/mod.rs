//! The home screen's figures, scope and flavor card expiry. The buy links
//! live in [`crate::buy`] and the color map test in
//! [`crate::playstyle::color_map_has_data`].

use chrono::{DateTime, Timelike, Utc};
use zwipe_core::http::contracts::metrics::{HttpLifetimeCounters, HttpPublicMetrics};

/// The three counters under the logo, each `None` while it loads or when
/// it is no number to show. Negative counts read as zero.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Figures {
    /// Decks created.
    pub decks_created: Option<u64>,
    /// Searches run.
    pub searches: Option<u64>,
    /// Cards swiped, every direction together.
    pub cards_swiped: Option<u64>,
}

/// A count the server keeps signed, never below zero.
fn count(n: i64) -> u64 {
    u64::try_from(n).unwrap_or(0)
}

impl From<&HttpPublicMetrics> for Figures {
    fn from(metrics: &HttpPublicMetrics) -> Self {
        Self {
            decks_created: Some(count(metrics.decks_created)),
            searches: Some(count(metrics.searches)),
            cards_swiped: Some(count(metrics.cards_swiped)),
        }
    }
}

impl From<&HttpLifetimeCounters> for Figures {
    /// The user's own counters. Every swipe direction counts as a card
    /// swiped, and a sum past `i64` is no number to show, so it waits like a
    /// loading one, as in the app.
    fn from(counters: &HttpLifetimeCounters) -> Self {
        let swiped = counters
            .swipes_right
            .checked_add(counters.swipes_left)
            .and_then(|n| n.checked_add(counters.swipes_up))
            .and_then(|n| n.checked_add(counters.swipes_down));
        Self {
            decks_created: Some(count(i64::from(counters.decks_created))),
            searches: Some(count(counters.searches)),
            cards_swiped: swiped.map(count),
        }
    }
}

impl Figures {
    /// The labels under the three figures, in order.
    pub const LABELS: [&'static str; 3] = ["Decks created", "Searches run", "Cards swiped"];

    /// The three values in the order of [`Figures::LABELS`].
    #[must_use]
    pub fn values(self) -> [Option<u64>; 3] {
        [self.decks_created, self.searches, self.cards_swiped]
    }
}

/// Whose counters the home screen shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Scope {
    /// Everyone's counters, the default.
    #[default]
    Everybody,
    /// The signed-in user's own counters.
    OnlyMe,
}

impl Scope {
    /// The scope chip's label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Everybody => "Everybody",
            Self::OnlyMe => "Only me",
        }
    }

    /// The other scope, what a tap on the chip switches to.
    #[must_use]
    pub fn toggled(self) -> Self {
        match self {
            Self::Everybody => Self::OnlyMe,
            Self::OnlyMe => Self::Everybody,
        }
    }

    /// The bytes the app stores the scope as: `all` or `mine`.
    #[must_use]
    pub fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::Everybody => b"all",
            Self::OnlyMe => b"mine",
        }
    }

    /// The scope stored as `bytes`, reading anything but `mine` as everybody.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes == b"mine" {
            Self::OnlyMe
        } else {
            Self::Everybody
        }
    }
}

/// The latest point into an hour [`next_hour`] counts, so the gap it adds
/// is always at least a second.
const LAST_SECOND_OF_HOUR: chrono::Duration = chrono::Duration::seconds(3599);

/// The top of the UTC hour after `now`, when the server picks a new featured
/// card. It keeps `now`'s fraction of a second, so the gap is whole seconds
/// and at least one, and past the end of time it is the end of time.
#[must_use]
pub fn next_hour(now: DateTime<Utc>) -> DateTime<Utc> {
    let hour_start = now
        .with_minute(0)
        .and_then(|t| t.with_second(0))
        .unwrap_or(now);
    let into_hour = now
        .signed_duration_since(hour_start)
        .min(LAST_SECOND_OF_HOUR);
    chrono::Duration::hours(1)
        .checked_sub(&into_hour)
        .and_then(|gap| now.checked_add_signed(gap))
        .unwrap_or(DateTime::<Utc>::MAX_UTC)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use uuid::Uuid;

    use super::*;

    fn mine(swipes_right: i64) -> HttpLifetimeCounters {
        HttpLifetimeCounters {
            user_id: Uuid::nil(),
            swipes_right,
            swipes_left: 1,
            swipes_up: 0,
            swipes_down: 0,
            searches: -9,
            decks_created: 6,
            decks_completed: 0,
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn my_figures_count_every_swipe_direction_and_clamp_at_zero() {
        let mut counters = mine(5);
        counters.swipes_left = 7;
        counters.swipes_up = 1;
        counters.swipes_down = 2;
        assert_eq!(
            Figures::from(&counters).values(),
            [Some(6), Some(0), Some(15)]
        );
    }

    #[test]
    fn my_cards_swiped_past_i64_waits_instead_of_saturating() {
        assert_eq!(
            Figures::from(&mine(i64::MAX)).values(),
            [Some(6), Some(0), None]
        );
        assert_eq!(Figures::default().values(), [None; 3], "loading");
    }

    #[test]
    fn scope_toggles_names_itself_and_round_trips_the_app_bytes() {
        assert_eq!(Scope::Everybody.toggled(), Scope::OnlyMe);
        assert_eq!(Scope::OnlyMe.toggled().label(), "Everybody");
        assert_eq!(Scope::from_bytes(Scope::OnlyMe.as_bytes()), Scope::OnlyMe);
        assert_eq!(Scope::from_bytes(b"all"), Scope::Everybody);
        assert_eq!(Scope::from_bytes(b"junk"), Scope::Everybody);
    }

    #[test]
    fn a_flavor_card_expires_at_the_top_of_the_next_hour() {
        let at = |h, m, s| {
            Utc.with_ymd_and_hms(2026, 10, 30, h, m, s)
                .unwrap()
                .with_nanosecond(789_000_000)
                .unwrap()
        };
        assert_eq!(next_hour(at(12, 34, 56)), at(13, 0, 0));
        assert_eq!(next_hour(at(12, 0, 0)), at(13, 0, 0));
        assert_eq!(next_hour(at(12, 59, 59)), at(13, 0, 0));
    }

    #[test]
    fn a_flavor_card_fetched_at_the_end_of_time_never_expires() {
        assert_eq!(
            next_hour(DateTime::<Utc>::MAX_UTC),
            DateTime::<Utc>::MAX_UTC
        );
    }
}
