//! Which one-tap fix a deck warning offers, sorted out the way the app's `deck_warnings.rs` does it.

use uuid::Uuid;
use zwipe_core::domain::deck::{DeckWarning, WarningAction};

/// A warning's fix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Fix {
    /// Too few cards: "Add cards" or "Import".
    BelowMinimum,
    /// Sets the card's quantity to this many: "Fix to {n}".
    Quantity { card: Uuid, quantity: i32 },
    /// Clears the commander: "Clear".
    ClearCommander,
    /// Takes the card out of the deck: "Remove".
    Remove { card: Uuid },
}

/// The fix `warning` offers, or `None` for a deck-level warning with nothing to tap.
///
/// The below-minimum warning is matched on its text, as in the app, so the wire format stays as it is.
pub(crate) fn fix(warning: &DeckWarning) -> Option<Fix> {
    let Some(card) = warning.scryfall_data_id() else {
        return warning
            .contains("requires at least")
            .then_some(Fix::BelowMinimum);
    };
    Some(match warning.action() {
        Some(WarningAction::FixQuantity(quantity)) => Fix::Quantity {
            card,
            quantity: *quantity,
        },
        Some(WarningAction::ClearCommander) => Fix::ClearCommander,
        Some(WarningAction::Remove) | None => Fix::Remove { card },
    })
}

/// The labels of a fix's buttons, in order.
pub(crate) fn labels(fix: Fix) -> Vec<String> {
    match fix {
        Fix::BelowMinimum => vec!["Add cards".to_owned(), "Import".to_owned()],
        Fix::Quantity { quantity, .. } => vec![format!("Fix to {quantity}")],
        Fix::ClearCommander => vec!["Clear".to_owned()],
        Fix::Remove { .. } => vec!["Remove".to_owned()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_warning_gets_the_apps_fix() {
        let card = Uuid::new_v4();
        let short = DeckWarning::new("deck has 60 cards, commander requires at least 100");
        assert_eq!(fix(&short), Some(Fix::BelowMinimum));
        assert_eq!(labels(Fix::BelowMinimum), ["Add cards", "Import"]);
        assert_eq!(
            fix(&DeckWarning::new(
                "deck has 40 lands, below the land target of 41"
            )),
            None
        );
        let copies = DeckWarning::with_action(
            "Sol Ring exceeds copy limit (2/1)",
            card,
            WarningAction::FixQuantity(1),
        );
        assert_eq!(fix(&copies), Some(Fix::Quantity { card, quantity: 1 }));
        assert_eq!(labels(Fix::Quantity { card, quantity: 1 }), ["Fix to 1"]);
        let commander = DeckWarning::with_action(
            "Opt is not a valid commander for commander",
            card,
            WarningAction::ClearCommander,
        );
        assert_eq!(fix(&commander), Some(Fix::ClearCommander));
        let banned = DeckWarning::with_card("Mana Crypt is banned in commander", card);
        assert_eq!(fix(&banned), Some(Fix::Remove { card }));
    }
}
