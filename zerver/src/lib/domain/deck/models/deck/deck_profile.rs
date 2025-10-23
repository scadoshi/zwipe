use crate::domain::deck::models::deck::deck_name::DeckName;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct DeckProfile {
    pub id: Uuid,
    pub name: DeckName,
    pub commander_id: Option<Uuid>,
    pub is_singleton: bool,
    pub user_id: Uuid,
}
