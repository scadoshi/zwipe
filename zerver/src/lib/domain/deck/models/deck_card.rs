use uuid::Uuid;

#[derive(Debug)]
pub struct DeckCard {
    pub id: Uuid,
    pub deck_id: Uuid,
    pub card_id: Uuid,
    pub quantity: i32,
}
