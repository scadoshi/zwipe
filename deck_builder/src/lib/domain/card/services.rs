use crate::domain::card::{
    models::{scryfall_card::ScryfallCard, GetCardError, SearchCardError},
    ports::{CardRepository, CardService},
};

#[derive(Debug, Clone)]
pub struct Service<R>
where
    R: CardRepository,
{
    repo: R,
}

impl<R> Service<R>
where
    R: CardRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: CardRepository> CardService for Service<R> {
    async fn get_card(&self, id: &uuid::Uuid) -> Result<ScryfallCard, GetCardError> {
        todo!()
    }

    async fn search_cards(
        &self,
        params: super::models::CardSearchParameters,
    ) -> Result<Vec<ScryfallCard>, SearchCardError> {
        todo!()
    }

    async fn scryfall_sync(
        &self,
        ignore_if_exists: bool,
    ) -> Result<super::models::SyncResult, anyhow::Error> {
        todo!()
    }
}
