use std::fmt::Debug;

use crate::domain::{
    card::ports::CardRepository,
    deck::{
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
    },
};

#[derive(Debug, Clone)]
pub struct Service<DR, CR>
where
    DR: DeckRepository,
    CR: CardRepository,
{
    deck_repo: DR,
    card_repo: CR,
}

impl<DR, CR> Service<DR, CR>
where
    DR: DeckRepository,
    CR: CardRepository,
{
    pub fn new(deck_repo: DR, card_repo: CR) -> Self {
        Self {
            deck_repo,
            card_repo,
        }
    }
}

impl<DR, CR> DeckService for Service<DR, CR>
where
    DR: DeckRepository,
    CR: CardRepository,
{
    async fn create_deck(&self, request: &CreateDeckRequest) -> Result<Deck, CreateDeckError> {
        self.deck_repo.create_deck(request).await
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
