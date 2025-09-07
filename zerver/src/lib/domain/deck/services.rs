use std::fmt::Debug;

use crate::domain::{
    card::{
        models::{card_profile::GetCardProfiles, GetCards},
        ports::CardRepository,
    },
    deck::{
        models::{
            deck::{
                CreateDeckProfile, CreateDeckProfileError, Deck, DeckProfile, DeleteDeck,
                DeleteDeckError, GetDeck, GetDeckError, GetDeckProfileError, UpdateDeckProfile,
                UpdateDeckProfileError,
            },
            deck_card::{
                CreateDeckCard, CreateDeckCardError, DeckCard, DeleteDeckCard, DeleteDeckCardError,
                GetDeckCard, GetDeckCardError, UpdateDeckCardError,
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
    async fn create_deck_profile(
        &self,
        request: &CreateDeckProfile,
    ) -> Result<DeckProfile, CreateDeckProfileError> {
        self.deck_repo.create_deck_profile(request).await
    }

    async fn get_deck_profile(
        &self,
        request: &GetDeck,
    ) -> Result<DeckProfile, GetDeckProfileError> {
        self.deck_repo.get_deck_profile(request).await
    }

    async fn get_deck(&self, request: &GetDeck) -> Result<Deck, GetDeckError> {
        let deck_profile = self.deck_repo.get_deck_profile(request).await?;

        if deck_profile.user_id != request.user_id {
            return Err(GetDeckError::DeckNotOwnedByUser);
        }

        let gdc = GetDeckCard::from(&deck_profile);
        let deck_cards = self.deck_repo.get_deck_cards(&gdc).await?;

        let gcp = GetCardProfiles::from(deck_cards.as_slice());
        let card_profiles = self.card_repo.get_card_profiles(&gcp).await?;

        let gcs = GetCards::from(card_profiles.as_slice());
        let cards = self.card_repo.get_cards(&gcs).await?;

        let deck = Deck::new(deck_profile, cards);

        Ok(deck)
    }

    async fn update_deck_profile(
        &self,
        request: &UpdateDeckProfile,
    ) -> Result<DeckProfile, UpdateDeckProfileError> {
        let get_deck = GetDeck::from(request);
        let _deck_profile = self.get_deck_profile(&get_deck).await?;
        self.deck_repo.update_deck_profile(request).await
    }

    async fn delete_deck(&self, request: &DeleteDeck) -> Result<(), DeleteDeckError> {
        self.deck_repo.delete_deck(request).await
    }

    async fn create_deck_card(
        &self,
        request: &CreateDeckCard,
    ) -> Result<DeckCard, CreateDeckCardError> {
        self.deck_repo.create_deck_card(request).await
    }

    async fn get_deck_card(&self, request: &GetDeckCard) -> Result<DeckCard, GetDeckCardError> {
        self.deck_repo.get_deck_card(request).await
    }

    async fn update_deck_card(
        &self,
        request: &super::models::deck_card::UpdateDeckCard,
    ) -> Result<DeckCard, UpdateDeckCardError> {
        self.deck_repo.update_deck_card(request).await
    }

    async fn delete_deck_card(&self, request: &DeleteDeckCard) -> Result<(), DeleteDeckCardError> {
        self.deck_repo.delete_deck_card(request).await
    }
}
