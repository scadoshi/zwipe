use std::fmt::Debug;

use crate::domain::{
    card::{
        models::{card_profile::GetCardProfiles, Card, GetCards, Sleeve},
        ports::CardRepository,
    },
    deck::{
        models::{
            deck::{
                CreateDeckError, CreateDeckProfile, Deck, DeckProfile, DeleteDeckError, GetDeck,
                GetDeckError, UpdateDeckProfileError,
            },
            deck_card::{
                CreateDeckCardError, DeckCard, DeleteDeckCardError, GetDeckCardError,
                UpdateDeckCardError,
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
    async fn create_deck(
        &self,
        request: &CreateDeckProfile,
    ) -> Result<DeckProfile, CreateDeckError> {
        self.deck_repo.create_deck(request).await
    }

    async fn get_deck_profile(&self, request: &GetDeck) -> Result<DeckProfile, GetDeckError> {
        self.deck_repo.get_deck_profile(request).await
    }

    async fn get_deck(&self, request: &GetDeck) -> Result<Deck, GetDeckError> {
        let deck_profile = self.deck_repo.get_deck_profile(request).await?;

        let gdcr = GetDeckCard::from(&deck_profile);
        let deck_cards = self.deck_repo.get_deck_cards(&gdcr).await?;

        let gcpr: GetCardProfiles = deck_cards.into();
        let card_profiles = self.card_repo.get_card_profiles(&gcpr).await?;

        let gmsfdr: GetCards = card_profiles.clone().into();
        let scryfall_data = self.card_repo.get_multiple_scryfall_data(&gmsfdr).await?;

        let cards: Vec<Card> = scryfall_data.sleeve(card_profiles);

        let deck = Deck::new(deck_profile, cards);

        Ok(deck)
    }

    async fn update_deck_profile(
        &self,
        request: &UpdateDeckProfile,
    ) -> Result<DeckProfile, UpdateDeckProfileError> {
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
