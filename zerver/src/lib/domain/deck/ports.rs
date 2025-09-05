use std::future::Future;

use crate::domain::deck::models::{
    deck::{
        CreateDeckProfile, CreateDeckProfileError, Deck, DeckProfile, DeleteDeck, DeleteDeckError,
        GetDeck, GetDeckError, GetDeckProfileError, UpdateDeckProfile, UpdateDeckProfileError,
    },
    deck_card::{
        CreateDeckCard, CreateDeckCardError, DeckCard, DeleteDeckCard, DeleteDeckCardError,
        GetDeckCard, GetDeckCardError, UpdateDeckCard, UpdateDeckCardError,
    },
};

/// enables deck related database operations
pub trait DeckRepository: Clone + Send + Sync + 'static {
    fn create_deck_profile(
        &self,
        request: &CreateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, CreateDeckProfileError>> + Send;

    fn create_deck_card(
        &self,
        request: &CreateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, CreateDeckCardError>> + Send;

    fn get_deck_profile(
        &self,
        request: &GetDeck,
    ) -> impl Future<Output = Result<DeckProfile, GetDeckProfileError>> + Send;

    fn get_deck_card(
        &self,
        request: &GetDeckCard,
    ) -> impl Future<Output = Result<DeckCard, GetDeckCardError>> + Send;

    fn get_deck_cards(
        &self,
        request: &GetDeckCard,
    ) -> impl Future<Output = Result<Vec<DeckCard>, GetDeckCardError>> + Send;

    fn update_deck_profile(
        &self,
        request: &UpdateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, UpdateDeckProfileError>> + Send;

    fn update_deck_card(
        &self,
        request: &UpdateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, UpdateDeckCardError>> + Send;

    fn delete_deck(
        &self,
        request: &DeleteDeck,
    ) -> impl Future<Output = Result<(), DeleteDeckError>> + Send;

    fn delete_deck_card(
        &self,
        request: &DeleteDeckCard,
    ) -> impl Future<Output = Result<(), DeleteDeckCardError>> + Send;
}

/// orchestrates deck related operations
pub trait DeckService: Clone + Send + Sync + 'static {
    fn create_deck_profile(
        &self,
        request: &CreateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, CreateDeckProfileError>> + Send;

    fn create_deck_card(
        &self,
        request: &CreateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, CreateDeckCardError>> + Send;

    fn get_deck_profile(
        &self,
        request: &GetDeck,
    ) -> impl Future<Output = Result<DeckProfile, GetDeckProfileError>> + Send;

    fn get_deck(
        &self,
        request: &GetDeck,
    ) -> impl Future<Output = Result<Deck, GetDeckError>> + Send;

    fn get_deck_card(
        &self,
        request: &GetDeckCard,
    ) -> impl Future<Output = Result<DeckCard, GetDeckCardError>> + Send;

    fn update_deck_profile(
        &self,
        request: &UpdateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, UpdateDeckProfileError>> + Send;

    fn update_deck_card(
        &self,
        request: &UpdateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, UpdateDeckCardError>> + Send;

    fn delete_deck(
        &self,
        request: &DeleteDeck,
    ) -> impl Future<Output = Result<(), DeleteDeckError>> + Send;

    fn delete_deck_card(
        &self,
        request: &DeleteDeckCard,
    ) -> impl Future<Output = Result<(), DeleteDeckCardError>> + Send;
}
