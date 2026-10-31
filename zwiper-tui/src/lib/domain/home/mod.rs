//! The home screen's figures, scope, color map test, flavor card expiry and buy links.

use chrono::{DateTime, Duration as ChronoDuration, DurationRound, Utc};
use zwipe_core::http::contracts::metrics::{HttpLifetimeCounters, HttpPublicMetrics};
use zwipe_core::http::contracts::suggestions::HttpColorMap;

/// The three counters under the logo, negative counts read as zero.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Figures {
    pub(crate) decks_created: u64,
    pub(crate) searches: u64,
    pub(crate) cards_swiped: u64,
}

/// A count the server keeps signed, never below zero.
fn count(n: i64) -> u64 {
    u64::try_from(n).unwrap_or(0)
}

impl From<&HttpPublicMetrics> for Figures {
    fn from(metrics: &HttpPublicMetrics) -> Self {
        Self {
            decks_created: count(metrics.decks_created),
            searches: count(metrics.searches),
            cards_swiped: count(metrics.cards_swiped),
        }
    }
}

impl From<&HttpLifetimeCounters> for Figures {
    /// The user's own counters. Every swipe direction counts as a card swiped, as in the app.
    fn from(counters: &HttpLifetimeCounters) -> Self {
        Self {
            decks_created: count(i64::from(counters.decks_created)),
            searches: count(counters.searches),
            cards_swiped: count(
                [
                    counters.swipes_right,
                    counters.swipes_left,
                    counters.swipes_up,
                    counters.swipes_down,
                ]
                .into_iter()
                .fold(0, i64::saturating_add),
            ),
        }
    }
}

impl Figures {
    /// The labels under the three figures, in order.
    pub(crate) const LABELS: [&'static str; 3] = ["Decks created", "Searches run", "Cards swiped"];

    /// The three values in the order of [`Figures::LABELS`].
    pub(crate) fn values(self) -> [u64; 3] {
        [self.decks_created, self.searches, self.cards_swiped]
    }
}

/// Whose counters the home screen shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Scope {
    #[default]
    Everybody,
    OnlyMe,
}

impl Scope {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Everybody => "Everybody",
            Self::OnlyMe => "Only me",
        }
    }

    pub(crate) fn toggled(self) -> Self {
        match self {
            Self::Everybody => Self::OnlyMe,
            Self::OnlyMe => Self::Everybody,
        }
    }

    /// The bytes the app stores the scope as: `all` or `mine`.
    pub(crate) fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::Everybody => b"all",
            Self::OnlyMe => b"mine",
        }
    }

    /// The scope stored as `bytes`, reading anything but `mine` as everybody.
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        if bytes == b"mine" {
            Self::OnlyMe
        } else {
            Self::Everybody
        }
    }
}

/// Whether the map has any cards to show, colored or colorless.
pub(crate) fn has_data(map: &HttpColorMap) -> bool {
    map.colorless_cards > 0 || map.colors.iter().any(|c| c.cards > 0)
}

/// The top of the UTC hour after `now`, when the server picks a new featured card.
pub(crate) fn next_hour(now: DateTime<Utc>) -> DateTime<Utc> {
    let hour = ChronoDuration::hours(1);
    now.duration_trunc(hour)
        .ok()
        .and_then(|top| top.checked_add_signed(hour))
        .unwrap_or(now)
}

/// The front face of a card name, before any ` // `.
fn front_face(name: &str) -> &str {
    name.split(" // ").next().unwrap_or(name)
}

/// A card name as the retailers' mass entry reads it: spaces as `+`, commas and apostrophes escaped, nothing else.
fn encode_name(name: &str) -> String {
    front_face(name)
        .replace(' ', "+")
        .replace(',', "%2C")
        .replace('\'', "%27")
}

/// The card's `TCGplayer` page when its product id is known, else a mass entry of one copy.
pub(crate) fn tcgplayer_card_url(name: &str, tcgplayer_id: Option<i32>) -> String {
    match tcgplayer_id {
        Some(id) => format!("https://www.tcgplayer.com/product/{id}"),
        None => format!(
            "https://www.tcgplayer.com/massentry?c=1+{}",
            encode_name(name)
        ),
    }
}

/// A Card Kingdom deck builder entry of one copy.
pub(crate) fn cardkingdom_card_url(name: &str) -> String {
    format!(
        "https://www.cardkingdom.com/builder?c=1+{}",
        encode_name(name)
    )
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use zwipe_core::http::contracts::suggestions::HttpColorCards;

    use super::*;

    #[test]
    fn my_figures_count_every_swipe_direction_and_clamp_at_zero() {
        let counters: HttpLifetimeCounters = serde_json::from_value(serde_json::json!({
            "user_id": "00000000-0000-0000-0000-000000000000",
            "swipes_right": 5, "swipes_left": 7, "swipes_up": 1, "swipes_down": 2,
            "searches": -9, "decks_created": 3, "decks_completed": 1,
            "updated_at": "2026-01-01T00:00:00Z",
        }))
        .unwrap();
        assert_eq!(
            Figures::from(&counters),
            Figures {
                decks_created: 3,
                searches: 0,
                cards_swiped: 15
            }
        );
    }

    #[test]
    fn an_empty_map_has_no_data() {
        let map = |colors: Vec<HttpColorCards>, colorless| HttpColorMap {
            colors,
            colorless_cards: colorless,
            pairs: Vec::new(),
        };
        let white = HttpColorCards {
            color: "W".to_owned(),
            cards: 0,
        };
        assert!(!has_data(&map(vec![white], 0)));
        assert!(has_data(&map(Vec::new(), 1)));
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
    fn the_flavor_card_expires_at_the_next_utc_hour() {
        let now = Utc.with_ymd_and_hms(2026, 10, 30, 14, 59, 59).unwrap();
        assert_eq!(
            next_hour(now),
            Utc.with_ymd_and_hms(2026, 10, 30, 15, 0, 0).unwrap()
        );
        let top = Utc.with_ymd_and_hms(2026, 10, 30, 15, 0, 0).unwrap();
        assert_eq!(
            next_hour(top),
            Utc.with_ymd_and_hms(2026, 10, 30, 16, 0, 0).unwrap()
        );
    }

    #[test]
    fn buy_links_encode_the_front_face_only() {
        assert_eq!(
            tcgplayer_card_url("Fire // Ice", None),
            "https://www.tcgplayer.com/massentry?c=1+Fire"
        );
        assert_eq!(
            tcgplayer_card_url("Sol Ring", Some(42)),
            "https://www.tcgplayer.com/product/42"
        );
        assert_eq!(
            cardkingdom_card_url("Urza's Saga, Again"),
            "https://www.cardkingdom.com/builder?c=1+Urza%27s+Saga%2C+Again"
        );
    }
}
