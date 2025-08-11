use uuid::Uuid;

/// Card profile data linked to ScryfallCard
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CardProfile {
    pub id: i32,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub scryfall_card_id: Uuid,
}
