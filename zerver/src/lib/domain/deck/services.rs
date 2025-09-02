use std::{collections::HashMap, fmt::Debug};

use uuid::Uuid;

use crate::domain::{
    card::{
        models::{
            card_profile::GetCardProfilesRequest,
            scryfall_data::{GetScryfallDatasRequest, ScryfallData},
        },
        ports::CardRepository,
    },
    deck::{
        models::{
            deck::{
                CreateDeckError, CreateDeckRequest, Deck, DeleteDeckError, DeleteDeckRequest,
                GetDeckError, GetDeckRequest, UpdateDeckError, UpdateDeckRequest,
            },
            deck_card::{
                CreateDeckCardError, CreateDeckCardRequest, DeckCard, DeleteDeckCardError,
                DeleteDeckCardRequest, GetDeckCardError, GetDeckCardRequest, UpdateDeckCardError,
            },
            DeckWithCards, FullCard,
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
        let deck = self.deck_repo.get_deck(request).await?;
        let get_deck_card_request = GetDeckCardRequest::from(&deck);
        let deck_cards = self
            .deck_repo
            .get_deck_cards(&get_deck_card_request)
            .await?;
        let get_card_profile_request: GetCardProfilesRequest = deck_cards.into();
        let card_profiles = self
            .card_repo
            .get_card_profiles(&get_card_profile_request)
            .await?;
        let get_cards_request: GetScryfallDatasRequest = card_profiles.clone().into();
        let scryfall_data = self.card_repo.get_cards(&get_cards_request).await?;
        let scryfall_data_map: HashMap<Uuid, ScryfallData> = scryfall_data
            .into_iter()
            .map(|scryfall_data| (scryfall_data.id.to_owned(), scryfall_data))
            .collect();
        let cards: Vec<FullCard> = card_profiles
            .into_iter()
            .filter_map(|card_profile| {
                scryfall_data_map
                    .get(&card_profile.scryfall_data_id)
                    .map(|scryfall_data| FullCard::new(card_profile, scryfall_data.clone()))
            })
            .collect();
        let deck_with_cards = DeckWithCards::new(deck, cards);
        Ok(deck_with_cards)
    }

    async fn update_deck(&self, request: &UpdateDeckRequest) -> Result<Deck, UpdateDeckError> {
        self.deck_repo.update_deck(request).await
    }

    async fn delete_deck(&self, request: &DeleteDeckRequest) -> Result<(), DeleteDeckError> {
        self.deck_repo.delete_deck(request).await
    }

    async fn create_deck_card(
        &self,
        request: &CreateDeckCardRequest,
    ) -> Result<DeckCard, CreateDeckCardError> {
        self.deck_repo.create_deck_card(request).await
    }

    async fn get_deck_card(
        &self,
        request: &GetDeckCardRequest,
    ) -> Result<DeckCard, GetDeckCardError> {
        self.deck_repo.get_deck_card(request).await
    }

    async fn update_deck_card(
        &self,
        request: &super::models::deck_card::UpdateDeckCardRequest,
    ) -> Result<DeckCard, UpdateDeckCardError> {
        self.deck_repo.update_deck_card(request).await
    }

    async fn delete_deck_card(
        &self,
        request: &DeleteDeckCardRequest,
    ) -> Result<(), DeleteDeckCardError> {
        self.deck_repo.delete_deck_card(request).await
    }
}
