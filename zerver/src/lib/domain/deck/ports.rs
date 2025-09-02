use std::future::Future;

use crate::domain::deck::models::{
    deck::{
        CreateDeckError, CreateDeckRequest, Deck, DeleteDeckError, DeleteDeckRequest, GetDeckError,
        GetDeckRequest, UpdateDeckError, UpdateDeckRequest,
    },
    deck_card::{
        CreateDeckCardError, CreateDeckCardRequest, DeckCard, DeleteDeckCardError,
        DeleteDeckCardRequest, GetDeckCardError, GetDeckCardRequest, UpdateDeckCardError,
        UpdateDeckCardRequest,
    },
    DeckWithCards,
};

/// enables deck related database operations
pub trait DeckRepository: Clone + Send + Sync + 'static {
    fn create_deck(
        &self,
        request: &CreateDeckRequest,
    ) -> impl Future<Output = Result<Deck, CreateDeckError>> + Send;

    fn create_deck_card(
        &self,
        request: &CreateDeckCardRequest,
    ) -> impl Future<Output = Result<DeckCard, CreateDeckCardError>> + Send;

    fn get_deck(
        &self,
        request: &GetDeckRequest,
    ) -> impl Future<Output = Result<Deck, GetDeckError>> + Send;

    fn get_deck_card(
        &self,
        request: &GetDeckCardRequest,
    ) -> impl Future<Output = Result<DeckCard, GetDeckCardError>> + Send;

    fn get_deck_cards(
        &self,
        request: &GetDeckCardRequest,
    ) -> impl Future<Output = Result<Vec<DeckCard>, GetDeckCardError>> + Send;

    fn update_deck(
        &self,
        request: &UpdateDeckRequest,
    ) -> impl Future<Output = Result<Deck, UpdateDeckError>> + Send;

    fn update_deck_card(
        &self,
        request: &UpdateDeckCardRequest,
    ) -> impl Future<Output = Result<DeckCard, UpdateDeckCardError>> + Send;

    fn delete_deck(
        &self,
        request: &DeleteDeckRequest,
    ) -> impl Future<Output = Result<(), DeleteDeckError>> + Send;

    fn delete_deck_card(
        &self,
        request: &DeleteDeckCardRequest,
    ) -> impl Future<Output = Result<(), DeleteDeckCardError>> + Send;
}

/// orchestrates deck related operations
pub trait DeckService: Clone + Send + Sync + 'static {
    fn create_deck(
        &self,
        request: &CreateDeckRequest,
    ) -> impl Future<Output = Result<Deck, CreateDeckError>> + Send;

    fn get_deck(
        &self,
        request: &GetDeckRequest,
    ) -> impl Future<Output = Result<DeckWithCards, GetDeckError>> + Send;

    fn update_deck(
        &self,
        request: &UpdateDeckRequest,
    ) -> impl Future<Output = Result<Deck, UpdateDeckError>> + Send;

    fn delete_deck(
        &self,
        request: &DeleteDeckRequest,
    ) -> impl Future<Output = Result<(), DeleteDeckError>> + Send;

    fn create_deck_card(
        &self,
        request: &CreateDeckCardRequest,
    ) -> impl Future<Output = Result<DeckCard, CreateDeckCardError>> + Send;

    fn get_deck_card(
        &self,
        request: &GetDeckCardRequest,
    ) -> impl Future<Output = Result<DeckCard, GetDeckCardError>> + Send;

    fn update_deck_card(
        &self,
        request: &UpdateDeckCardRequest,
    ) -> impl Future<Output = Result<DeckCard, UpdateDeckCardError>> + Send;

    fn delete_deck_card(
        &self,
        request: &DeleteDeckCardRequest,
    ) -> impl Future<Output = Result<(), DeleteDeckCardError>> + Send;
}
