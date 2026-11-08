//! Deck card import operation.
//!
//! The parse lives in `zwipe_core`; the service-layer error and the rows the
//! import writes live here.

use std::collections::HashMap;

use thiserror::Error;
use uuid::Uuid;
use zwipe_core::domain::deck::Board;
use zwipe_core::domain::deck::requests::import_deck_cards::NamedPrinting;

use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;

/// Errors that can occur during deck card import.
#[derive(Debug, Error)]
pub enum ImportDeckCardsError {
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to requesting user")]
    Forbidden,
    /// Deck not found or inaccessible.
    #[error(transparent)]
    DeckNotFound(#[from] GetDeckProfileError),
    /// Import would exceed the maximum number of cards per deck (verified user).
    #[error("card limit reached (mainboard, maybeboard, and sideboard all count toward it)")]
    LimitReached,
    /// Import would exceed the unverified card limit.
    #[error("card limit reached across all boards, verify your email to unlock more")]
    UnverifiedLimitReached,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

/// Where an import row's printing came from, which decides whether it replaces
/// the printing of a card the deck already holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowPrinting {
    /// The import named this printing (a decklist's `(SET) number`, an
    /// Archidekt card id). It replaces the deck's printing of the card.
    Named,
    /// The card's default printing, picked by name. A card the deck already
    /// holds keeps its printing.
    Default,
}

/// One deck card an import writes. Rows are unique by `oracle_id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRow {
    /// The printing to write.
    pub scryfall_data_id: Uuid,
    /// The card, across printings.
    pub oracle_id: Uuid,
    /// Copies, summed across the import's lines for this card.
    pub quantity: i32,
    /// The board the copies go on.
    pub board: Board,
    /// Whether the import named the printing.
    pub printing: RowPrinting,
}

/// The printings an import's `(SET) number`s can name, by card and set, each
/// list in the order the import prefers (English, paper, non-promo, then
/// collector number).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NamedPrintings {
    /// `(oracle_id, lowercase set code)` -> `(scryfall_data_id, collector number)`.
    by_card_and_set: HashMap<(Uuid, String), Vec<(Uuid, String)>>,
}

impl NamedPrintings {
    /// Adds one printing after the ones already added for its card and set.
    pub fn insert(
        &mut self,
        oracle_id: Uuid,
        set_code: &str,
        scryfall_data_id: Uuid,
        collector_number: String,
    ) {
        self.by_card_and_set
            .entry((oracle_id, set_code.to_ascii_lowercase()))
            .or_default()
            .push((scryfall_data_id, collector_number));
    }

    /// The printing of `oracle_id` that `named` picks: the one with its
    /// collector number, else the first in its set when the number is missing
    /// or matches none. `None` when the card has no printing in that set, so
    /// the import keeps the default printing.
    #[must_use]
    pub fn pick(&self, oracle_id: Uuid, named: &NamedPrinting) -> Option<Uuid> {
        let in_set = self
            .by_card_and_set
            .get(&(oracle_id, named.set_code.clone()))?;
        named
            .collector_number
            .as_deref()
            .and_then(|number| {
                in_set
                    .iter()
                    .find(|(_, collector)| collector.eq_ignore_ascii_case(number))
            })
            .or_else(|| in_set.first())
            .map(|(id, _)| *id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(set_code: &str, collector_number: Option<&str>) -> NamedPrinting {
        NamedPrinting {
            set_code: set_code.to_owned(),
            collector_number: collector_number.map(str::to_owned),
        }
    }

    #[test]
    fn a_named_printing_picks_its_set_and_number() {
        let card = Uuid::from_u128(1);
        let (isd_51, isd_51a, inr_60) = (
            Uuid::from_u128(10),
            Uuid::from_u128(11),
            Uuid::from_u128(12),
        );
        let mut printings = NamedPrintings::default();
        printings.insert(card, "ISD", isd_51, "51".to_owned());
        printings.insert(card, "isd", isd_51a, "51a".to_owned());
        printings.insert(card, "inr", inr_60, "60".to_owned());

        assert_eq!(
            printings.pick(card, &named("isd", Some("51A"))),
            Some(isd_51a)
        );
        assert_eq!(
            printings.pick(card, &named("inr", Some("60"))),
            Some(inr_60)
        );
        assert_eq!(
            printings.pick(card, &named("isd", None)),
            Some(isd_51),
            "no number takes the set's first printing"
        );
        assert_eq!(
            printings.pick(card, &named("isd", Some("999"))),
            Some(isd_51),
            "a number the set lacks takes the set's first printing"
        );
        assert_eq!(
            printings.pick(card, &named("m10", Some("51"))),
            None,
            "a set the card was never in falls back to the default printing"
        );
        assert_eq!(
            printings.pick(Uuid::from_u128(2), &named("isd", Some("51"))),
            None,
            "another card's printing never matches"
        );
    }
}
