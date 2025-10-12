use std::fmt::Debug;

use crate::domain::{
    card::{
        models::{card_profile::get_card_profile::GetCardProfiles, get_card::GetCards},
        ports::CardRepository,
    },
    deck::{
        models::{
            deck::{
                create_deck_profile::{CreateDeckProfile, CreateDeckProfileError},
                deck_profile::DeckProfile,
                delete_deck::{DeleteDeck, DeleteDeckError},
                get_deck::{GetDeck, GetDeckError, GetDeckProfileError},
                update_deck_profile::{UpdateDeckProfile, UpdateDeckProfileError},
                Deck,
            },
            deck_card::{
                create_deck_card::{CreateDeckCard, CreateDeckCardError},
                delete_deck_card::{DeleteDeckCard, DeleteDeckCardError},
                update_deck_card::{UpdateDeckCard, UpdateDeckCardError},
                DeckCard,
            },
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
    // ========
    //  create
    // ========
    async fn create_deck_profile(
        &self,
        request: &CreateDeckProfile,
    ) -> Result<DeckProfile, CreateDeckProfileError> {
        self.deck_repo.create_deck_profile(request).await
    }

    async fn create_deck_card(
        &self,
        request: &CreateDeckCard,
    ) -> Result<DeckCard, CreateDeckCardError> {
        let _deck_profile = self.get_deck_profile(&request.into()).await?;
        self.deck_repo.create_deck_card(request).await
    }

    // =====
    //  get
    // =====
    async fn get_deck_profile(
        &self,
        request: &GetDeck,
    ) -> Result<DeckProfile, GetDeckProfileError> {
        let deck_profile = self.deck_repo.get_deck_profile(request).await?;
        if request.user_id != deck_profile.user_id {
            return Err(GetDeckProfileError::Forbidden);
        }
        Ok(deck_profile)
    }

    async fn get_deck(&self, request: &GetDeck) -> Result<Deck, GetDeckError> {
        let deck_profile = self.deck_repo.get_deck_profile(request).await?;
        let gd = GetDeck::from(&deck_profile);
        let deck_cards = self.deck_repo.get_deck_cards(&gd).await?;

        let gcp = GetCardProfiles::from(deck_cards.as_slice());
        let card_profiles = self.card_repo.get_card_profiles(&gcp).await?;

        let gcs = GetCards::from(card_profiles.as_slice());
        let cards = self.card_repo.get_cards(&gcs).await?;

        let deck = Deck::new(deck_profile, cards);

        Ok(deck)
    }

    // ========
    //  update
    // ========
    async fn update_deck_profile(
        &self,
        request: &UpdateDeckProfile,
    ) -> Result<DeckProfile, UpdateDeckProfileError> {
        let get_deck = GetDeck::from(request);
        let _deck_profile = self.get_deck_profile(&get_deck).await?;
        self.deck_repo.update_deck_profile(request).await
    }

    async fn update_deck_card(
        &self,
        request: &UpdateDeckCard,
    ) -> Result<DeckCard, UpdateDeckCardError> {
        let _deck_profile = self.get_deck_profile(&request.into()).await?;
        self.deck_repo.update_deck_card(request).await
    }

    // ========
    //  delete
    // ========
    async fn delete_deck(&self, request: &DeleteDeck) -> Result<(), DeleteDeckError> {
        self.deck_repo.delete_deck(request).await
    }

    async fn delete_deck_card(&self, request: &DeleteDeckCard) -> Result<(), DeleteDeckCardError> {
        let _deck_profile = self.get_deck_profile(&request.into()).await?;
        self.deck_repo.delete_deck_card(request).await
    }
}
