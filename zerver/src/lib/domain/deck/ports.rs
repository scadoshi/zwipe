use std::future::Future;

use crate::domain::deck::models::{
    deck::{
        create_deck_profile::{CreateDeckProfile, CreateDeckProfileError},
        deck_profile::DeckProfile,
        delete_deck::{DeleteDeck, DeleteDeckError},
        get_deck::GetDeckError,
        get_deck_profile::{GetDeckProfile, GetDeckProfileError},
        get_deck_profiles::GetDeckProfiles,
        update_deck_profile::{UpdateDeckProfile, UpdateDeckProfileError},
        Deck,
    },
    deck_card::{
        create_deck_card::{CreateDeckCard, CreateDeckCardError},
        delete_deck_card::{DeleteDeckCard, DeleteDeckCardError},
        get_deck_card::GetDeckCardError,
        update_deck_card::{UpdateDeckCard, UpdateDeckCardError},
        DeckCard,
    },
};

/// enables deck related database operations
pub trait DeckRepository: Clone + Send + Sync + 'static {
    // ========
    //  create
    // ========
    fn create_deck_profile(
        &self,
        request: &CreateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, CreateDeckProfileError>> + Send;

    fn create_deck_card(
        &self,
        request: &CreateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, CreateDeckCardError>> + Send;

    // =====
    //  get
    // =====
    fn get_deck_profile(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, GetDeckProfileError>> + Send;

    fn get_deck_profiles(
        &self,
        request: &GetDeckProfiles,
    ) -> impl Future<Output = Result<Vec<DeckProfile>, GetDeckProfileError>> + Send;

    fn get_deck_cards(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<Vec<DeckCard>, GetDeckCardError>> + Send;

    // ========
    //  update
    // ========
    fn update_deck_profile(
        &self,
        request: &UpdateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, UpdateDeckProfileError>> + Send;

    fn update_deck_card(
        &self,
        request: &UpdateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, UpdateDeckCardError>> + Send;

    // ========
    //  delete
    // ========
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
    // ========
    //  create
    // ========
    fn create_deck_profile(
        &self,
        request: &CreateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, CreateDeckProfileError>> + Send;

    fn create_deck_card(
        &self,
        request: &CreateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, CreateDeckCardError>> + Send;

    // =====
    //  get
    // =====
    fn get_deck_profile(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, GetDeckProfileError>> + Send;

    fn get_deck_profiles(
        &self,
        request: &GetDeckProfiles,
    ) -> impl Future<Output = Result<Vec<DeckProfile>, GetDeckProfileError>> + Send;

    fn get_deck(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<Deck, GetDeckError>> + Send;

    // ========
    //  update
    // ========
    fn update_deck_profile(
        &self,
        request: &UpdateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, UpdateDeckProfileError>> + Send;

    fn update_deck_card(
        &self,
        request: &UpdateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, UpdateDeckCardError>> + Send;

    // ========
    //  delete
    // ========
    fn delete_deck(
        &self,
        request: &DeleteDeck,
    ) -> impl Future<Output = Result<(), DeleteDeckError>> + Send;

    fn delete_deck_card(
        &self,
        request: &DeleteDeckCard,
    ) -> impl Future<Output = Result<(), DeleteDeckCardError>> + Send;
}
