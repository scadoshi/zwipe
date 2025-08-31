use std::fmt::Debug;

use crate::domain::deck::{
    models::{
        deck::{
            CreateDeckError, CreateDeckRequest, Deck, DeleteDeckError, DeleteDeckRequest,
            GetDeckError, GetDeckRequest, UpdateDeckError, UpdateDeckRequest,
        },
        deck_card::{
            CreateDeckCardError, DeckCard, DeleteDeckCardError, DeleteDeckCardRequest,
            GetDeckCardError, GetDeckCardRequest, UpdateDeckCardError,
        },
        DeckWithCards,
    },
    ports::{DeckRepository, DeckService},
};

#[derive(Debug, Clone)]
pub struct Service<R>
where
    R: DeckRepository,
{
    repo: R,
}

impl<R> Service<R>
where
    R: DeckRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: DeckRepository> DeckService for Service<R> {
    async fn create_deck(&self, request: &CreateDeckRequest) -> Result<Deck, CreateDeckError> {
        todo!()
    }

    async fn get_deck(&self, request: &GetDeckRequest) -> Result<DeckWithCards, GetDeckError> {
        todo!()
    }

    async fn update_deck(
        &self,
        request: &UpdateDeckRequest,
    ) -> Result<DeckWithCards, UpdateDeckError> {
        todo!()
    }

    async fn delete_deck(&self, request: &DeleteDeckRequest) -> Result<(), DeleteDeckError> {
        todo!()
    }

    async fn create_deck_card(
        &self,
        request: &super::models::deck_card::CreateDeckCardRequest,
    ) -> Result<DeckCard, CreateDeckCardError> {
        todo!()
    }

    async fn get_deck_card(
        &self,
        request: &GetDeckCardRequest,
    ) -> Result<DeckCard, GetDeckCardError> {
        todo!()
    }

    async fn update_deck_card(
        &self,
        request: &super::models::deck_card::UpdateDeckCardRequest,
    ) -> Result<DeckCard, UpdateDeckCardError> {
        todo!()
    }

    async fn delete_deck_card(
        &self,
        request: &DeleteDeckCardRequest,
    ) -> Result<(), DeleteDeckCardError> {
        todo!()
    }
}
