use serde::{Deserialize, Serialize};
// use serde_json::Value;
use uuid::Uuid;

/// To be stored against card
/// against the "all_parts" field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedCard {
    pub id: Uuid,
    pub object: String,
    pub component: String,
    pub name: String,
    pub type_line: String,
    pub uri: String,
}

#[derive(Debug, Clone)]
pub struct AllParts(Vec<RelatedCard>);

impl Serialize for AllParts {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AllParts {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Vec::<RelatedCard>::deserialize(deserializer).map(AllParts)
    }
}
