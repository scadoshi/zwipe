use sqlx_macros::FromRow;
use uuid::Uuid;

use crate::outbound::sqlx::deck::error::{IntoDeckCardError, IntoDeckProfileError};
use zwipe_core::domain::deck::{
    Board,
    DeckCard,
    DeckTag,
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
    pub partner_commander_id: Option<Uuid>,
    pub background_id: Option<Uuid>,
    pub signature_spell_id: Option<Uuid>,
    pub format: Option<String>,
    pub tags: Option<serde_json::Value>,
    pub land_target: Option<i32>,
    pub user_id: Uuid,
    pub card_count: Option<i64>,
    pub commander_name: Option<String>,
    pub partner_commander_name: Option<String>,
    pub background_name: Option<String>,
    pub signature_spell_name: Option<String>,
}

/// converts database deck to validated domain deck
impl TryFrom<DatabaseDeckProfile> for DeckProfile {
    type Error = IntoDeckProfileError;
    fn try_from(value: DatabaseDeckProfile) -> Result<Self, Self::Error> {
        let name = DeckName::new(value.name)?;
        let format = value.format.map(Format::try_from).transpose()?;
        // Unrecognized tag strings are dropped (forward-compatible), like card
        // mechanical_categories.
        let tags = value
            .tags
            .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
            .map(|strings| {
                strings
                    .iter()
                    .filter_map(|s| DeckTag::try_from(s.as_str()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            id: value.id,
            name,
            commander_id: value.commander_id,
            partner_commander_id: value.partner_commander_id,
            background_id: value.background_id,
            signature_spell_id: value.signature_spell_id,
            format,
            tags,
            land_target: value.land_target,
            user_id: value.user_id,
            card_count: value.card_count.unwrap_or(0),
            commander_name: value.commander_name,
            partner_commander_name: value.partner_commander_name,
            background_name: value.background_name,
            signature_spell_name: value.signature_spell_name,
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
    pub oracle_id: String,
    pub quantity: i32,
    pub board: String,
}

/// converts database deck card to validated domain deck card
impl TryFrom<DatabaseDeckCard> for DeckCard {
    type Error = IntoDeckCardError;
    fn try_from(value: DatabaseDeckCard) -> Result<Self, Self::Error> {
        let deck_id = Uuid::try_parse(&value.deck_id).map_err(IntoDeckCardError::InvalidDeckId)?;
        let scryfall_data_id =
            Uuid::try_parse(&value.scryfall_data_id).map_err(IntoDeckCardError::InvalidCardId)?;
        let oracle_id =
            Uuid::try_parse(&value.oracle_id).map_err(IntoDeckCardError::InvalidOracleId)?;
        let quantity = Quantity::new(value.quantity)?;
        let board = Board::try_from(value.board.as_str())?;
        Ok(Self {
            deck_id,
            scryfall_data_id,
            oracle_id,
            quantity,
            board,
        })
    }
}
