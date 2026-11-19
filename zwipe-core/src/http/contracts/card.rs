//! A card's public numbers: the response of `GET /api/card/{oracle_id}/stats`.

use serde::{Deserialize, Serialize};

/// One card's public numbers, pooled across every user and read live.
///
/// Counts only: each client turns them into rates and copy, and no number has a floor, so a card
/// swiped once carries its one swipe. Every field is `#[serde(default)]`, so a field the server
/// doesn't send reads as `None` and a later field is additive. The server sends `in_decks`,
/// `mvp_decks` and `decks_led` on every answer, zero when there are none, and `swipes` and
/// `commander_picks` only once their pool holds a row for the card. Whether the card can lead a
/// deck isn't decided here: the client holding the card decides whether to show `decks_led`
/// and `commander_picks`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HttpCardStats {
    /// Decks holding the card on the mainboard, in every format.
    #[serde(default)]
    pub in_decks: Option<u64>,
    /// Decks starring the card as an MVP for at least 3 days, the same vesting the global
    /// signal uses.
    #[serde(default)]
    pub mvp_decks: Option<u64>,
    /// Its Add-stack swipes and removals, from every deck in every format.
    #[serde(default)]
    pub swipes: Option<HttpCardSwipes>,
    /// Decks leading with the card, as commander or partner, by any printing.
    #[serde(default)]
    pub decks_led: Option<u64>,
    /// How often Swipe select's commander pick showed the card and how often it was picked.
    #[serde(default)]
    pub commander_picks: Option<HttpCommanderPicks>,
}

/// A card's pooled Add-stack counts. `added`, `skipped` and `maybed` count the swipes of each
/// kind, `shown` counts every time the card was dealt, and `removed` counts the times it was
/// cut from a deck after being added.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HttpCardSwipes {
    /// Times the card was dealt on an Add stack.
    pub shown: u64,
    /// Right swipes: added to the deck.
    pub added: u64,
    /// Left swipes: skipped.
    pub skipped: u64,
    /// Up swipes: put on the maybeboard.
    pub maybed: u64,
    /// Times it was removed from a deck.
    pub removed: u64,
}

/// A commander's pooled Swipe select counts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HttpCommanderPicks {
    /// Times Swipe select dealt it for a commander slot.
    pub shown: u64,
    /// Times it was picked.
    pub selected: u64,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn missing_fields_read_as_none_so_new_ones_stay_additive() {
        let stats: HttpCardStats = serde_json::from_value(json!({ "in_decks": 3 })).unwrap();
        assert_eq!(
            stats,
            HttpCardStats {
                in_decks: Some(3),
                ..HttpCardStats::default()
            }
        );
        let unknown: HttpCardStats =
            serde_json::from_value(json!({ "in_decks": 1, "a_later_field": 9 })).unwrap();
        assert_eq!(unknown.in_decks, Some(1), "an unknown field is ignored");
    }

    #[test]
    fn stats_round_trip() {
        let stats = HttpCardStats {
            in_decks: Some(2),
            mvp_decks: Some(1),
            swipes: Some(HttpCardSwipes {
                shown: 50,
                added: 30,
                skipped: 15,
                maybed: 5,
                removed: 3,
            }),
            decks_led: Some(0),
            commander_picks: None,
        };
        let back: HttpCardStats =
            serde_json::from_value(serde_json::to_value(stats).unwrap()).unwrap();
        assert_eq!(back, stats);
    }
}
