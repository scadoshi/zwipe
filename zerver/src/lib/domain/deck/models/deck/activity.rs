//! The Playstyle screen's activity: daily swipes and searches, what the
//! swipes came to, the keep rate per card role and the cards passed on most.

use chrono::NaiveDate;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::suggestions::{ACTIVITY_ROLE_MIN_SEEN, ACTIVITY_ROLES_SENT};

/// How many dealt cards' worth of the overall skip rate a card's baseline
/// starts from before other players' swipes on it count. A card few others
/// have been dealt sits near the overall rate instead of swinging to 0 or 1.
pub const PASS_BASELINE_WEIGHT: f64 = 10.0;

/// The lift a hard pass needs: its skips times the log of how much more often
/// the user skips it than other players do. A card others usually keep gets
/// there in two skips; one they skip almost as often takes dozens.
pub const PASS_MIN_SCORE: f64 = 1.0;

/// One UTC day's swipes and searches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivityDay {
    /// The UTC date.
    pub day: NaiveDate,
    /// Swipes in any direction on any screen.
    pub swipes: u32,
    /// Searches run.
    pub searches: u32,
}

/// What the user's swipes on dealt cards came to, across every deck.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SwipeOutcomes {
    /// Added to a deck.
    pub added: u64,
    /// Skipped.
    pub skipped: u64,
    /// Saved to a maybeboard.
    pub maybed: u64,
    /// Taken back out of a deck.
    pub removed: u64,
}

/// One card role's kept and passed cards, in decks with a commander.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleKeep {
    /// The card role slug.
    pub role: String,
    /// Cards with the role added to a deck.
    pub kept: u64,
    /// Cards with the role skipped.
    pub passed: u64,
}

impl RoleKeep {
    /// Kept and passed together.
    fn seen(&self) -> u64 {
        self.kept.saturating_add(self.passed)
    }
}

/// A card skipped and never kept or saved, before its card is fetched. The
/// repository ranks them by lift, the user's skips times the log of how much
/// more often they skip it than other players do, so a card everyone passes
/// on makes the list only after many skips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassCount {
    /// The card's oracle id.
    pub oracle_id: Uuid,
    /// Times skipped, across every commander.
    pub skipped: u32,
    /// Commanders it was skipped under.
    pub commanders: u32,
}

/// What the repository reads for [`Activity`], with the hard passes as
/// oracle ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityCounts {
    /// The database's UTC date.
    pub through: NaiveDate,
    /// Days with any swipe or search, oldest first.
    pub days: Vec<ActivityDay>,
    /// Every card swiped on.
    pub outcomes: SwipeOutcomes,
    /// Every role with a kept or passed card, in any order.
    pub roles: Vec<RoleKeep>,
    /// The most skipped cards never kept or saved, the most skipped first.
    pub passes: Vec<PassCount>,
}

/// A hard pass with its card.
#[derive(Debug, Clone, PartialEq)]
pub struct HardPass {
    /// The card's most recent normal printing.
    pub card: Card,
    /// Times skipped, across every commander.
    pub skipped: u32,
    /// Commanders it was skipped under.
    pub commanders: u32,
}

/// The Playstyle screen's activity.
#[derive(Debug, Clone, PartialEq)]
pub struct Activity {
    /// The database's UTC date, the last day of the heatmap.
    pub through: NaiveDate,
    /// Days with any swipe or search, oldest first.
    pub days: Vec<ActivityDay>,
    /// Every card swiped on.
    pub outcomes: SwipeOutcomes,
    /// The roles [`rank_roles`] keeps.
    pub roles: Vec<RoleKeep>,
    /// The hard passes, the most skipped first.
    pub passes: Vec<HardPass>,
}

/// The roles worth a keep rate: each seen at least [`ACTIVITY_ROLE_MIN_SEEN`]
/// times, the [`ACTIVITY_ROLES_SENT`] most seen, most seen first and then by
/// slug.
#[must_use]
pub fn rank_roles(mut roles: Vec<RoleKeep>) -> Vec<RoleKeep> {
    roles.retain(|r| r.seen() >= ACTIVITY_ROLE_MIN_SEEN);
    roles.sort_by(|a, b| b.seen().cmp(&a.seen()).then_with(|| a.role.cmp(&b.role)));
    roles.truncate(ACTIVITY_ROLES_SENT);
    roles
}

#[cfg(test)]
mod tests {
    use super::*;

    fn role(role: &str, kept: u64, passed: u64) -> RoleKeep {
        RoleKeep {
            role: role.to_string(),
            kept,
            passed,
        }
    }

    #[test]
    fn rarely_seen_roles_are_dropped_and_the_rest_rank_by_how_often_seen() {
        let ranked = rank_roles(vec![
            role("ramp", 4, 8),
            role("draw", 1, 1),
            role("removal", 20, 30),
            role("counter", 0, 12),
        ]);
        let order: Vec<&str> = ranked.iter().map(|r| r.role.as_str()).collect();
        assert_eq!(order, ["removal", "counter", "ramp"]);
    }

    #[test]
    fn at_most_the_sent_number_of_roles_go() {
        let roles = (0..20).map(|i| role(&format!("r{i:02}"), 10, i)).collect();
        assert_eq!(rank_roles(roles).len(), ACTIVITY_ROLES_SENT);
    }
}
