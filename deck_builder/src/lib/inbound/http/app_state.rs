// TODO: Import service traits once they exist
// use crate::domain::ports::services::{UserService, CardService, DeckService};

#[derive(Debug, Clone)]
pub struct AppState {
    // TODO: Add services back once traits are defined
    // pub user_service: Arc<dyn UserService + Send + Sync>,
    // pub card_service: Arc<dyn CardService + Send + Sync>,
    // pub deck_service: Arc<dyn DeckService + Send + Sync>,
}

impl AppState {
    pub async fn initialize() -> Result<Self, Box<dyn std::error::Error>> {
        todo!("Need to implement after creating services");
        Ok(Self {
            // Empty for now
        })
    }
}
