//! One card's public numbers, served by `GET /api/card/{oracle_id}/stats`.

use zwipe_core::http::contracts::card::{HttpCardStats, HttpCardSwipes, HttpCommanderPicks};

/// One card's numbers pooled across every user, read live.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CardStats {
    /// Decks holding the card on the mainboard.
    pub in_decks: u64,
    /// Mainboard decks starring it as an MVP for at least [`MVP_VESTING_DAYS`].
    pub mvp_decks: u64,
    /// Its row in `card_swipe_signal`, `None` when it has none.
    pub swipes: Option<HttpCardSwipes>,
    /// Decks leading with it, as commander or partner, by any printing.
    pub decks_led: u64,
    /// Its row in `commander_select_signal`, `None` when it has none.
    pub commander_picks: Option<HttpCommanderPicks>,
}

/// Days a star has to stand before it counts as an MVP outside its own deck, so a star cut
/// within days never reaches a public number.
pub const MVP_VESTING_DAYS: i32 = 3;

impl From<CardStats> for HttpCardStats {
    fn from(value: CardStats) -> Self {
        Self {
            in_decks: Some(value.in_decks),
            mvp_decks: Some(value.mvp_decks),
            swipes: value.swipes,
            decks_led: Some(value.decks_led),
            commander_picks: value.commander_picks,
        }
    }
}
