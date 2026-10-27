//! The Activity screen's data: daily swipes and searches, what the
//! swipes came to, the keep rate per card role and the cards passed on most.

use chrono::NaiveDate;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::suggestions::{
    ACTIVITY_MIN_SEEN, ACTIVITY_ROLES_SENT, ACTIVITY_TAGS_SENT,
};

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

impl SwipeOutcomes {
    /// Cards dealt and acted on: added, skipped and saved together.
    #[must_use]
    pub fn seen(&self) -> u64 {
        self.added
            .saturating_add(self.skipped)
            .saturating_add(self.maybed)
    }
}

/// What one card role's dealt cards came to, in decks with a commander.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleRates {
    /// The card role slug.
    pub role: String,
    /// Its cards' outcomes.
    pub outcomes: SwipeOutcomes,
}

/// What one oracle tag's dealt cards came to, for the user and every other
/// player, in decks with a commander.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagRates {
    /// The oracle tag slug.
    pub tag: String,
    /// The card role it falls under, the first by name when several.
    pub role: String,
    /// The user's outcomes on cards with the tag.
    pub outcomes: SwipeOutcomes,
    /// Every other player's outcomes on cards with the tag.
    pub everyone: SwipeOutcomes,
}

/// A card skipped and never kept or saved, before its card is fetched. The
/// repository ranks them by lift, the user's skips times the log of how much
/// more often they skip it than other players do, so a card everyone passes
/// on makes the list only after many skips.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassCount {
    /// The card's oracle id.
    pub oracle_id: Uuid,
    /// Times skipped, across every commander.
    pub skipped: u32,
    /// The names of the commanders it was skipped under, by name.
    pub commanders: Vec<String>,
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
    /// Every role with a dealt card, in any order.
    pub roles: Vec<RoleRates>,
    /// Every tag under a role with a dealt card, in any order.
    pub tags: Vec<TagRates>,
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
    /// The names of the commanders it was skipped under, by name.
    pub commanders: Vec<String>,
}

/// The Activity screen's data.
#[derive(Debug, Clone, PartialEq)]
pub struct Activity {
    /// The database's UTC date, the last day of the heatmap.
    pub through: NaiveDate,
    /// Days with any swipe or search, oldest first.
    pub days: Vec<ActivityDay>,
    /// Every card swiped on.
    pub outcomes: SwipeOutcomes,
    /// The roles [`rank_roles`] keeps.
    pub roles: Vec<RoleRates>,
    /// The tags [`rank_tags`] keeps.
    pub tags: Vec<TagRates>,
    /// The hard passes, the most skipped first.
    pub passes: Vec<HardPass>,
}

/// The roles worth a rate: each dealt at least [`ACTIVITY_MIN_SEEN`] times,
/// the [`ACTIVITY_ROLES_SENT`] most seen, most seen first and then by slug.
#[must_use]
pub fn rank_roles(mut roles: Vec<RoleRates>) -> Vec<RoleRates> {
    roles.retain(|r| r.outcomes.seen() >= ACTIVITY_MIN_SEEN);
    roles.sort_by(|a, b| {
        b.outcomes
            .seen()
            .cmp(&a.outcomes.seen())
            .then_with(|| a.role.cmp(&b.role))
    });
    roles.truncate(ACTIVITY_ROLES_SENT);
    roles
}

/// The tags worth a rate: each dealt at least [`ACTIVITY_MIN_SEEN`] times,
/// the [`ACTIVITY_TAGS_SENT`] most seen, most seen first and then by slug.
#[must_use]
pub fn rank_tags(mut tags: Vec<TagRates>) -> Vec<TagRates> {
    tags.retain(|t| t.outcomes.seen() >= ACTIVITY_MIN_SEEN);
    tags.sort_by(|a, b| {
        b.outcomes
            .seen()
            .cmp(&a.outcomes.seen())
            .then_with(|| a.tag.cmp(&b.tag))
    });
    tags.truncate(ACTIVITY_TAGS_SENT);
    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcomes(added: u64, skipped: u64) -> SwipeOutcomes {
        SwipeOutcomes {
            added,
            skipped,
            ..SwipeOutcomes::default()
        }
    }

    fn role(role: &str, added: u64, skipped: u64) -> RoleRates {
        RoleRates {
            role: role.to_string(),
            outcomes: outcomes(added, skipped),
        }
    }

    fn tag(tag: &str, added: u64, skipped: u64) -> TagRates {
        TagRates {
            tag: tag.to_string(),
            role: "ramp".to_string(),
            outcomes: outcomes(added, skipped),
            everyone: SwipeOutcomes::default(),
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

    #[test]
    fn tags_rank_like_roles_and_stop_at_the_sent_number() {
        let ranked = rank_tags(vec![
            tag("mana-rock", 6, 6),
            tag("rare", 1, 0),
            tag("ramp-land", 9, 9),
        ]);
        let order: Vec<&str> = ranked.iter().map(|t| t.tag.as_str()).collect();
        assert_eq!(order, ["ramp-land", "mana-rock"]);
        let many = (0..150).map(|i| tag(&format!("t{i:03}"), 10, 0)).collect();
        assert_eq!(rank_tags(many).len(), ACTIVITY_TAGS_SENT);
    }

    #[test]
    fn a_saved_card_counts_as_seen() {
        let saved = SwipeOutcomes {
            maybed: 3,
            removed: 2,
            ..SwipeOutcomes::default()
        };
        assert_eq!(
            saved.seen(),
            3,
            "removed cards were already counted as added"
        );
    }
}
