use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// To be stored against card
/// against the "all_parts" field
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RelatedCard {
    pub id: Uuid,
    pub object: String,
    pub component: String,
    pub name: String,
    pub type_line: String,
    pub uri: String,
}
