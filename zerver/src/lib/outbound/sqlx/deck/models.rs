use sqlx_macros::FromRow;
use uuid::Uuid;

use crate::outbound::sqlx::deck::error::{IntoDeckCardError, IntoDeckProfileError};
use zwipe_core::domain::deck::{
    DeckCard,
    deck_name::DeckName,
    deck_profile::DeckProfile,
    format::Format,
    quantity::Quantity,
};

/// raw database deck record
/// (unvalidated data from `PostgreSQL`)
#[allow(missing_docs)]
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseDeckProfile {
    pub id: Uuid,
    pub name: String,
    pub commander_id: Option<Uuid>,
    pub format: Option<String>,
    pub user_id: Uuid,
    pub card_count: Option<i64>,
    pub commander_name: Option<String>,
}

/// converts database deck to validated domain deck
impl TryFrom<DatabaseDeckProfile> for DeckProfile {
    type Error = IntoDeckProfileError;
    fn try_from(value: DatabaseDeckProfile) -> Result<Self, Self::Error> {
        let name = DeckName::new(value.name)?;
        let format = value.format.map(Format::try_from).transpose()?;

        Ok(Self {
            id: value.id,
            name,
            commander_id: value.commander_id,
            format,
            user_id: value.user_id,
            card_count: value.card_count.unwrap_or(0),
            commander_name: value.commander_name,
        })
    }
}

/// raw database deck card record
/// (unvalidated data from `PostgreSQL`)
#[allow(missing_docs)]
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseDeckCard {
    pub deck_id: String,
    pub scryfall_data_id: String,
    pub quantity: i32,
}

/// converts database deck card to validated domain deck card
impl TryFrom<DatabaseDeckCard> for DeckCard {
    type Error = IntoDeckCardError;
    fn try_from(value: DatabaseDeckCard) -> Result<Self, Self::Error> {
        let deck_id = Uuid::try_parse(&value.deck_id).map_err(IntoDeckCardError::InvalidDeckId)?;
        let scryfall_data_id =
            Uuid::try_parse(&value.scryfall_data_id).map_err(IntoDeckCardError::InvalidCardId)?;
        let quantity = Quantity::new(value.quantity)?;
        Ok(Self {
            deck_id,
            scryfall_data_id,
            quantity,
        })
    }
}
