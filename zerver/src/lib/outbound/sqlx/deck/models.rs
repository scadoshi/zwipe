use sqlx_macros::FromRow;
use uuid::Uuid;

use crate::{
    domain::deck::models::{
        deck::{copy_max::CopyMax, deck_name::DeckName, deck_profile::DeckProfile},
        deck_card::{quantity::Quantity, DeckCard},
    },
    outbound::sqlx::deck::error::{IntoDeckCardError, IntoDeckProfileError},
};

/// raw database deck record
/// (unvalidated data from `PostgreSQL`)
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseDeckProfile {
    pub id: Uuid,
    pub name: String,
    pub commander_id: Option<Uuid>,
    pub copy_max: Option<i32>,
    pub user_id: Uuid,
}

/// converts database deck to validated domain deck
impl TryFrom<DatabaseDeckProfile> for DeckProfile {
    type Error = IntoDeckProfileError;
    fn try_from(value: DatabaseDeckProfile) -> Result<Self, Self::Error> {
        let name = DeckName::new(&value.name)?;
        let copy_max = value.copy_max.map(|max| CopyMax::new(max)).transpose()?;

        Ok(Self {
            id: value.id,
            name,
            commander_id: value.commander_id,
            copy_max,
            user_id: value.user_id,
        })
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
