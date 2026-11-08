//! Delete card from deck operation.

use thiserror::Error;
use uuid::Uuid;

/// Errors that occur while constructing a [`DeleteDeckCard`] request.
#[derive(Debug, Error)]
pub enum InvalidDeleteDeckCard {
    /// Invalid deck ID format.
    #[error(transparent)]
    DeckId(uuid::Error),
    /// Invalid card ID format.
    #[error(transparent)]
    ScryfallDataId(uuid::Error),
}

/// Why a card leaves the deck, which decides whether it is suppressed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum DeleteReason {
    /// A deliberate removal: the card is suppressed from the deck's swipe
    /// pile for a while, as a "doesn't fit" signal.
    #[default]
    Removal,
    /// The undo of an add: the card goes back to how it was before, with no
    /// suppression.
    Undo,
}

/// Request to delete a card from a deck (removes all copies).
#[derive(Debug, Clone)]
pub struct DeleteDeckCard {
    /// Requesting user (for authorization).
    pub user_id: Uuid,
    /// Deck containing the card.
    pub deck_id: Uuid,
    /// Card to remove (Scryfall data ID).
    pub scryfall_data_id: Uuid,
    /// A removal or an undo; [`DeleteReason::Removal`] unless
    /// [`undone`](Self::undone) says otherwise.
    pub reason: DeleteReason,
}

impl DeleteDeckCard {
    /// Creates a new deck card deletion request with validation.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidDeleteDeckCard`] when either id is not a UUID.
    pub fn new(
        user_id: Uuid,
        deck_id: &str,
        scryfall_data_id: &str,
    ) -> Result<Self, InvalidDeleteDeckCard> {
        let deck_id = Uuid::try_parse(deck_id).map_err(InvalidDeleteDeckCard::DeckId)?;
        let scryfall_data_id =
            Uuid::try_parse(scryfall_data_id).map_err(InvalidDeleteDeckCard::ScryfallDataId)?;
        Ok(Self {
            user_id,
            deck_id,
            scryfall_data_id,
            reason: DeleteReason::Removal,
        })
    }

    /// This request as the undo of an add when `undo`, else unchanged.
    #[must_use]
    pub fn undone(mut self, undo: bool) -> Self {
        if undo {
            self.reason = DeleteReason::Undo;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_delete_is_a_removal_unless_undone() {
        let id = Uuid::nil().to_string();
        let request = DeleteDeckCard::new(Uuid::nil(), &id, &id).unwrap();
        assert_eq!(request.reason, DeleteReason::Removal);
        assert_eq!(request.clone().undone(false).reason, DeleteReason::Removal);
        assert_eq!(request.undone(true).reason, DeleteReason::Undo);
    }
}
