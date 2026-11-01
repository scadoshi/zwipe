//! What the card screens tell the user as a deck changes: the main board's land count against its land target, and its price against its budget.

use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::DeckEntry;
use zwipe_core::domain::deck::deck_metrics::{budget_tier, card_price};

/// How loud a signal is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Good news, such as a target reached.
    Info,
    /// Worth a look, such as going over budget.
    Warning,
}

/// Lands on the main board, counting copies.
#[must_use]
pub fn land_count(entries: &[DeckEntry]) -> i32 {
    entries
        .iter()
        .filter(|e| e.deck_card.board.is_active() && e.card.scryfall_data.is_land())
        .fold(0, |total, e| total.saturating_add(*e.deck_card.quantity))
}

/// The land toast when the count crosses an explicit target: up to it, "Land target reached", and when `both_ways`, down below it, "Below land target".
#[must_use]
pub fn land_crossing(
    target: Option<i32>,
    before: i32,
    after: i32,
    direction: Crossing,
) -> Option<(Level, String)> {
    let target = target?;
    let up = before < target && after >= target;
    let down = before >= target && after < target;
    match direction {
        Crossing::Up | Crossing::Both if up => {
            Some((Level::Info, format!("Land target reached ({target})")))
        }
        Crossing::Down | Crossing::Both if down => {
            Some((Level::Warning, format!("Below land target ({target})")))
        }
        _ => None,
    }
}

/// Which land crossings a screen announces: Add only up, Remove only down, the deck cards view both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crossing {
    /// Only reaching the target.
    Up,
    /// Only dropping below it.
    Down,
    /// Both ways.
    Both,
}

/// The budget toast when the total moves into a higher band (50, 75, 100 percent): a warning at or over the budget, else info.
#[must_use]
pub fn budget_crossing(
    budget: Option<f64>,
    currency: PriceCurrency,
    before: f64,
    after: f64,
) -> Option<(Level, String)> {
    let budget = budget.filter(|b| *b > 0.0)?;
    if budget_tier(budget, after) <= budget_tier(budget, before) {
        return None;
    }
    let pct = after / budget * 100.0;
    let message = format!(
        "Deck at {pct:.2}% of your {} budget",
        currency.format_amount(budget)
    );
    let level = if after >= budget {
        Level::Warning
    } else {
        Level::Info
    };
    Some((level, message))
}

/// One copy's price toward the budget: its price in the currency, or nothing.
#[must_use]
pub fn price_of(card: &Card, currency: PriceCurrency) -> f64 {
    card_price(&card.scryfall_data, currency).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::deck::{Board, Quantity};

    use super::*;
    use crate::test_support::{card, deck, deck_profile};

    #[test]
    fn lands_count_copies_on_the_main_board_only() {
        let mut forest = card("Forest");
        forest.scryfall_data.type_line = Some("Basic Land — Forest".to_owned());
        let mut d = deck(
            deck_profile("Lands"),
            vec![forest.clone(), forest, card("Elf")],
        );
        d.entries[0].deck_card.quantity = Quantity::new(3).unwrap();
        d.entries[1].deck_card.board = Board::Maybeboard;
        assert_eq!(land_count(&d.entries), 3);
    }

    #[test]
    fn land_crossings_fire_once_per_direction() {
        assert_eq!(land_crossing(None, 9, 10, Crossing::Both), None);
        assert_eq!(
            land_crossing(Some(10), 9, 10, Crossing::Up),
            Some((Level::Info, "Land target reached (10)".to_owned()))
        );
        assert_eq!(land_crossing(Some(10), 10, 11, Crossing::Both), None);
        assert_eq!(land_crossing(Some(10), 10, 9, Crossing::Up), None);
        assert_eq!(
            land_crossing(Some(10), 10, 9, Crossing::Down),
            Some((Level::Warning, "Below land target (10)".to_owned()))
        );
        assert_eq!(land_crossing(Some(10), 9, 10, Crossing::Down), None);
    }

    #[test]
    fn budget_toasts_only_when_the_band_rises() {
        assert_eq!(budget_crossing(None, PriceCurrency::Usd, 0.0, 99.0), None);
        assert_eq!(
            budget_crossing(Some(100.0), PriceCurrency::Usd, 40.0, 55.5),
            Some((
                Level::Info,
                "Deck at 55.50% of your $100.00 budget".to_owned()
            ))
        );
        assert_eq!(
            budget_crossing(Some(100.0), PriceCurrency::Usd, 55.0, 60.0),
            None
        );
        assert_eq!(
            budget_crossing(Some(100.0), PriceCurrency::Eur, 80.0, 100.0).map(|c| c.0),
            Some(Level::Warning)
        );
        assert_eq!(
            budget_crossing(Some(100.0), PriceCurrency::Usd, 90.0, 40.0),
            None
        );
    }

    #[test]
    fn a_card_without_a_price_adds_nothing() {
        let mut c = card("Bolt");
        c.scryfall_data.prices.usd = None;
        c.scryfall_data.prices.usd_foil = None;
        c.scryfall_data.prices.usd_etched = None;
        assert!(price_of(&c, PriceCurrency::Usd).abs() < f64::EPSILON);
    }
}
