use sqlx_macros::FromRow;
use uuid::Uuid;

use crate::{
    domain::deck::models::{
        deck::{deck_name::DeckName, deck_profile::DeckProfile},
        deck_card::{quantity::Quantity, DeckCard},
    },
    outbound::sqlx::deck::error::{IntoDeckCardError, IntoDeckProfileError},
};

/// raw database deck record
/// (unvalidated data from `PostgreSQL`)
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseDeckProfile {
    pub id: String,
    pub name: String,
    pub user_id: String,
}

/// converts database deck to validated domain deck
impl TryFrom<DatabaseDeckProfile> for DeckProfile {
    type Error = IntoDeckProfileError;
    fn try_from(value: DatabaseDeckProfile) -> Result<Self, Self::Error> {
        let id = Uuid::try_parse(&value.id).map_err(|e| IntoDeckProfileError::Id(e.into()))?;
        let name =
            DeckName::new(&value.name).map_err(|e| IntoDeckProfileError::DeckName(e.into()))?;
        let user_id =
            Uuid::try_parse(&value.user_id).map_err(|e| IntoDeckProfileError::UserId(e.into()))?;
        Ok(Self { id, name, user_id })
    }
}

/// raw database deck card record
/// (unvalidated data from `PostgreSQL`)
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseDeckCard {
    pub deck_id: String,
    pub card_profile_id: String,
    pub quantity: i32,
}

/// converts database deck card to validated domain deck card
impl TryFrom<DatabaseDeckCard> for DeckCard {
    type Error = IntoDeckCardError;
    fn try_from(value: DatabaseDeckCard) -> Result<Self, Self::Error> {
        let deck_id = Uuid::try_parse(&value.deck_id)
            .map_err(|e| IntoDeckCardError::InvalidDeckId(e.into()))?;
        let card_profile_id = Uuid::try_parse(&value.card_profile_id)
            .map_err(|e| IntoDeckCardError::InvalidCardId(e.into()))?;
        let quantity = Quantity::new(value.quantity)?;
        Ok(Self {
            deck_id,
            card_profile_id,
            quantity,
        })
    }
}
