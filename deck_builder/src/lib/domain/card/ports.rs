use std::future::Future;

use uuid::Uuid;

use crate::domain::card::models::{
    scryfall_card::ScryfallCard, CardNotFound, CardSearchParameters, InvalidUuid,
};

pub trait CardRepository: Clone + Send + Sync + 'static {
    fn insert(
        &self,
        card: ScryfallCard,
    ) -> impl Future<Output = Result<(), CreateCardError>> + Send;

    fn bulk_insert(
        &self,
        cards: Vec<ScryfallCard>,
    ) -> impl Future<Output = Result<(), CreateCardError>>;

    fn batch_insert(
        &self,
        cards: Vec<ScryfallCard>,
        batch_size: usize,
    ) -> impl Future<Output = Result<(), CreateCardError>>;

    fn smart_insert(
        &self,
        cards: Vec<ScryfallCard>,
        batch_size: usize,
    ) -> impl Future<Output = Result<(), CreateCardError>>;

    fn get_card(&self, id: &Uuid)
        -> impl Future<Output = Result<ScryfallCard, InvalidUuid>> + Send;

    fn search_cards(
        &self,
        params: CardSearchParameters,
    ) -> impl Future<Output = Result<Vec<ScryfallCard>, CardNotFound>> + Send;

    fn delete_all(&self) -> impl Future<Output = Result<(), anyhow::Error>> + Send;
}

pub trait CardService {
    fn get_card(&self, id: &Uuid)
        -> impl Future<Output = Result<ScryfallCard, InvalidUuid>> + Send;

    fn search_cards(
        &self,
        params: CardSearchParameters,
    ) -> impl Future<Output = Result<Vec<ScryfallCard>, CardNotFound>> + Send;
}
