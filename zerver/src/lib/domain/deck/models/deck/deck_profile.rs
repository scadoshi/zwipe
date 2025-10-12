use crate::domain::deck::models::deck::deck_name::DeckName;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DeckProfile {
    pub id: Uuid,
    pub name: DeckName,
    pub user_id: Uuid,
}
