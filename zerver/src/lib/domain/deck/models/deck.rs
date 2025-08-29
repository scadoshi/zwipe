use thiserror::Error;
use uuid::Uuid;

// ========
//  errors
// ========

#[derive(Debug, Error)]
pub enum DeckNameError {
    #[error("name must be present")]
    NotFound,
}

// ==========
//  newtypes
// ==========

#[derive(Debug)]
pub struct DeckName(String);

impl DeckName {
    pub fn new(name: &str) -> Result<Self, DeckNameError> {
        if name.is_empty() {
            return Err(DeckNameError::NotFound);
        }
        Ok(Self(name.to_string()))
    }

    pub fn name(self) -> String {
        self.0
    }
}

// ======
//  main
// ======

#[derive(Debug)]
pub struct Deck {
    pub id: Uuid,
    pub name: DeckName,
    pub user_id: Uuid,
}
