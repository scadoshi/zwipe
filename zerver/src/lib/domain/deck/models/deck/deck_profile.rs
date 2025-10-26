use crate::domain::deck::models::deck::{copy_max::CopyMax, deck_name::DeckName};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct DeckProfile {
    pub id: Uuid,
    pub name: DeckName,
    pub commander_id: Option<Uuid>,
    pub copy_max: Option<CopyMax>,
    pub user_id: Uuid,
}
