//! Deck rules the app keeps in its own crates: a deck row's count, price and pips here, and the command zone, list, form, pickers, export, buy, view and warning rules in the modules.

pub mod command_zone;
mod create_gate;
pub mod export;
pub mod form;
pub mod list;
pub mod pickers;
pub mod profile;
pub mod view;
pub mod warning;

pub use create_gate::{VERIFY_TO_CREATE, create_gate};
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::DeckProfile;

/// The deck's size as the app shows it: the cards in the deck, plus each command-zone card it has. The commander counts only in formats that have one.
#[must_use]
pub fn card_count(profile: &DeckProfile) -> i64 {
    let commander =
        profile.commander_id.is_some() && profile.format.is_some_and(|f| f.has_commander());
    let command_zone = [
        commander,
        profile.partner_commander_id.is_some(),
        profile.background_id.is_some(),
        profile.signature_spell_id.is_some(),
    ]
    .into_iter()
    .filter(|&has| has)
    .count();
    profile
        .card_count
        .saturating_add(i64::try_from(command_zone).unwrap_or_default())
}

/// `1 card`, `2 cards`.
#[must_use]
pub fn cards_label(count: i64) -> String {
    if count == 1 {
        "1 card".to_owned()
    } else {
        format!("{count} cards")
    }
}

/// The deck's total price for its list row, in the deck's own currency (its price target's, USD when it has none), formatted as the deck page shows it: `$27.80`, `€9.10`, `3.25 TIX`. `None` when no card is priced in that currency.
#[must_use]
pub fn price_label(profile: &DeckProfile) -> Option<String> {
    let currency = profile.price_target_currency.unwrap_or_default();
    let total = match currency {
        PriceCurrency::Usd => profile.total_price_usd,
        PriceCurrency::Eur => profile.total_price_eur,
        PriceCurrency::Tix => profile.total_price_tix,
    };
    total
        .filter(|&v| v > 0.0)
        .map(|v| currency.format_amount(v))
}

/// `1 deck`, `2 decks`.
#[must_use]
pub fn decks_label(count: u32) -> String {
    if count == 1 {
        "1 deck".to_owned()
    } else {
        format!("{count} decks")
    }
}

/// Whether `count` fits the format's minimum and maximum deck size. A deck with no format, or a format with no limit, always fits.
#[must_use]
pub fn count_fits(profile: &DeckProfile, count: i64) -> bool {
    let Some(format) = profile.format else {
        return true;
    };
    let above_min = format.min_cards().is_none_or(|min| count >= i64::from(min));
    let below_max = format.max_cards().is_none_or(|max| count <= i64::from(max));
    above_min && below_max
}

/// The deck's colors as places in WUBRG order, in that order.
#[must_use]
pub fn pips(profile: &DeckProfile) -> Vec<usize> {
    let mut pips: Vec<usize> = profile
        .color_identity
        .iter()
        .filter_map(|c| ["W", "U", "B", "R", "G"].iter().position(|l| l == c))
        .collect();
    pips.sort_unstable();
    pips.dedup();
    pips
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use zwipe_core::domain::deck::Format;

    use super::*;
    use crate::test_support::deck_profile;

    #[test]
    fn a_row_prices_the_deck_in_its_own_currency_and_only_when_priced() {
        let mut profile = deck_profile("Priced");
        profile.total_price_usd = Some(27.8);
        profile.total_price_eur = Some(25.1);
        profile.total_price_tix = Some(3.25);
        assert_eq!(
            price_label(&profile).as_deref(),
            Some("$27.80"),
            "USD with no target"
        );
        profile.price_target_currency = Some(PriceCurrency::Eur);
        assert_eq!(price_label(&profile).as_deref(), Some("€25.10"));
        profile.price_target_currency = Some(PriceCurrency::Tix);
        assert_eq!(price_label(&profile).as_deref(), Some("3.25 TIX"));

        profile.total_price_tix = None;
        assert_eq!(price_label(&profile), None, "nothing priced in TIX");
        profile.price_target_currency = None;
        profile.total_price_usd = Some(0.0);
        assert_eq!(
            price_label(&profile),
            None,
            "a zero total is nothing priced"
        );
    }

    #[test]
    fn the_command_zone_counts_toward_the_deck() {
        let mut profile = deck_profile("Commander");
        profile.card_count = 98;
        profile.format = Some(Format::Commander);
        profile.commander_id = Some(Uuid::new_v4());
        profile.partner_commander_id = Some(Uuid::new_v4());
        assert_eq!(card_count(&profile), 100);
        assert!(count_fits(&profile, 100));
        assert!(!count_fits(&profile, 101));
    }

    #[test]
    fn a_server_count_at_the_cap_stays_there() {
        let mut profile = deck_profile("Capped");
        profile.card_count = i64::MAX;
        profile.signature_spell_id = Some(Uuid::new_v4());
        assert_eq!(card_count(&profile), i64::MAX);
    }

    #[test]
    fn a_commander_counts_only_where_the_format_has_one() {
        let mut profile = deck_profile("Standard");
        profile.card_count = 60;
        profile.commander_id = Some(Uuid::new_v4());
        assert_eq!(card_count(&profile), 60);
        assert!(count_fits(&profile, 3), "no format, no limits");
    }

    #[test]
    fn one_card_is_singular() {
        assert_eq!(cards_label(1), "1 card");
        assert_eq!(cards_label(0), "0 cards");
        assert_eq!(cards_label(100), "100 cards");
    }

    #[test]
    fn pips_are_in_wubrg_order_without_repeats() {
        let mut profile = deck_profile("Colors");
        profile.color_identity = vec![
            "G".to_owned(),
            "W".to_owned(),
            "G".to_owned(),
            "x".to_owned(),
        ];
        assert_eq!(pips(&profile), [0, 4]);
    }
}
