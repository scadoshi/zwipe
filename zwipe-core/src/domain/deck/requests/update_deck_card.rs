//! Update deck card operation (idempotent PATCH semantics).
//!
//! One constructor, one wire shape: [`UpdateDeckCard::patch`] with quantity
//! as an **absolute** value to set, so replaying a request is harmless. The
//! legacy delta (PUT) form was removed at the end of the migration in
//! `context/plans/patch_idempotent_updates.md`.

use crate::domain::deck::{Board, InvalidQuantity, Quantity};
use thiserror::Error;
use uuid::Uuid;

/// Errors that occur while constructing an [`UpdateDeckCard`] request.
#[derive(Debug, Error)]
pub enum InvalidUpdateDeckCard {
    /// Invalid deck ID format.
    #[error(transparent)]
    DeckId(uuid::Error),
    /// Invalid card ID format.
    #[error(transparent)]
    ScryfallDataId(uuid::Error),
    /// Invalid new printing ID format.
    #[error(transparent)]
    NewScryfallDataId(uuid::Error),
    /// Absolute quantity below 1 (removal is an explicit DELETE, not qty 0).
    #[error(transparent)]
    Quantity(InvalidQuantity),
    /// No fields provided to update.
    #[error("at least one of quantity, board, scryfall_data_id, or mvp must be provided")]
    NothingToUpdate,
}

impl From<InvalidQuantity> for InvalidUpdateDeckCard {
    fn from(value: InvalidQuantity) -> Self {
        Self::Quantity(value)
    }
}

/// Request to update a card in a deck.
///
/// Supports setting quantity (absolute), board, printing, and/or MVP star.
/// At least one field must be provided.
#[derive(Debug, Clone)]
pub struct UpdateDeckCard {
    /// Requesting user (for authorization).
    pub user_id: Uuid,
    /// Deck containing the card.
    pub deck_id: Uuid,
    /// Card to update (Scryfall data ID).
    pub scryfall_data_id: Uuid,
    /// Absolute quantity to set. `None` = no quantity change.
    pub set_quantity: Option<Quantity>,
    /// Move card to this board. `None` = no change.
    pub board: Option<Board>,
    /// Change the selected printing to this Scryfall data ID. `None` = no change.
    pub new_scryfall_data_id: Option<Uuid>,
    /// Star (true) or unstar (false) the card as a deck MVP. `None` = no change.
    pub mvp: Option<bool>,
}

impl UpdateDeckCard {
    /// Creates an idempotent (PATCH) update request with validation:
    /// `quantity`, when present, is an **absolute** value to set (≥ 1), not
    /// a delta — replaying the request is harmless.
    ///
    /// At least one of `quantity`, `board`, `new_scryfall_data_id`, or `mvp`
    /// must be `Some`.
    pub fn patch(
        user_id: Uuid,
        deck_id: &str,
        scryfall_data_id: &str,
        quantity: Option<i32>,
        board: Option<Board>,
        new_scryfall_data_id: Option<&str>,
        mvp: Option<bool>,
    ) -> Result<Self, InvalidUpdateDeckCard> {
        let deck_id = Uuid::try_parse(deck_id).map_err(InvalidUpdateDeckCard::DeckId)?;
        let scryfall_data_id =
            Uuid::try_parse(scryfall_data_id).map_err(InvalidUpdateDeckCard::ScryfallDataId)?;

        let set_quantity = quantity.map(Quantity::new).transpose()?;

        let new_scryfall_data_id = new_scryfall_data_id
            .map(|s| Uuid::try_parse(s).map_err(InvalidUpdateDeckCard::NewScryfallDataId))
            .transpose()?;

        if set_quantity.is_none()
            && board.is_none()
            && new_scryfall_data_id.is_none()
            && mvp.is_none()
        {
            return Err(InvalidUpdateDeckCard::NothingToUpdate);
        }

        Ok(Self {
            user_id,
            deck_id,
            scryfall_data_id,
            set_quantity,
            board,
            new_scryfall_data_id,
            mvp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids() -> (Uuid, String, String) {
        (
            Uuid::new_v4(),
            Uuid::new_v4().to_string(),
            Uuid::new_v4().to_string(),
        )
    }

    #[test]
    fn patch_accepts_absolute_quantity() {
        let (user_id, deck_id, card_id) = ids();
        let request =
            UpdateDeckCard::patch(user_id, &deck_id, &card_id, Some(3), None, None, None).unwrap();
        assert_eq!(request.set_quantity, Some(Quantity::new(3).unwrap()));
    }

    #[test]
    fn patch_rejects_quantity_below_one() {
        let (user_id, deck_id, card_id) = ids();
        assert!(matches!(
            UpdateDeckCard::patch(user_id, &deck_id, &card_id, Some(0), None, None, None),
            Err(InvalidUpdateDeckCard::Quantity(_))
        ));
    }

    #[test]
    fn patch_rejects_nothing_to_update() {
        let (user_id, deck_id, card_id) = ids();
        assert!(matches!(
            UpdateDeckCard::patch(user_id, &deck_id, &card_id, None, None, None, None),
            Err(InvalidUpdateDeckCard::NothingToUpdate)
        ));
    }
}
