use std::collections::HashMap;
use std::fmt::Debug;

use uuid::Uuid;

use crate::domain::{
    card::{models::scryfall_data::get_scryfall_data::ScryfallDataIds, ports::CardRepository},
    deck::{
        models::{
            deck::{
                Deck, DeckEntry,
                create_deck_profile::{CreateDeckProfile, CreateDeckProfileError},
                deck_profile::DeckProfile,
                delete_deck::{DeleteDeck, DeleteDeckError},
                get_deck::GetDeckError,
                get_deck_profile::{GetDeckProfile, GetDeckProfileError},
                get_deck_profiles::GetDeckProfiles,
                update_deck_profile::{UpdateDeckProfile, UpdateDeckProfileError},
            },
            deck_card::{
                DeckCard,
                create_deck_card::{CreateDeckCard, CreateDeckCardError},
                delete_deck_card::{DeleteDeckCard, DeleteDeckCardError},
                import_deck_cards::{
                    ImportDeckCards, ImportDeckCardsError, ImportDeckCardsResult, ImportedCard,
                    UnresolvedCard,
                },
                update_deck_card::{UpdateDeckCard, UpdateDeckCardError},
            },
        },
        ports::{DeckRepository, DeckService},
    },
};

use crate::domain::deck::{
    MAX_CARDS_PER_DECK, MAX_DECKS_PER_USER, UNVERIFIED_MAX_CARDS_PER_DECK,
    UNVERIFIED_MAX_DECKS_PER_USER,
};

/// Deck service implementation handling deck building and card management operations.
///
/// This service coordinates:
/// - **Deck profiles**: Create, read, update, delete deck metadata (name, format, etc.)
/// - **Deck cards**: Add, update, remove cards from decks with quantity tracking
/// - **Authorization**: Enforces deck ownership checks (user can only access their own decks)
/// - **Card data**: Fetches Scryfall card data for complete deck views
///
/// # Authorization Pattern
/// All deck operations verify that the requesting user owns the deck before
/// allowing modifications. `GetDeckProfileError::Forbidden` is returned if
/// the user ID doesn't match the deck's owner.
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
    /// Creates a new deck service with the provided repositories.
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
        let deck_count = self
            .deck_repo
            .count_decks_by_user(request.user_id)
            .await
            .map_err(CreateDeckProfileError::Database)?;
        let deck_limit = if request.email_verified {
            MAX_DECKS_PER_USER
        } else {
            UNVERIFIED_MAX_DECKS_PER_USER
        };
        if deck_count >= deck_limit {
            return Err(CreateDeckProfileError::LimitReached);
        }
        self.deck_repo.create_deck_profile(request).await
    }

    async fn create_deck_card(
        &self,
        request: &CreateDeckCard,
    ) -> Result<DeckCard, CreateDeckCardError> {
        let deck_profile = self.get_deck_profile(&request.into()).await?;
        if deck_profile.commander_id == Some(request.scryfall_data_id) {
            return Err(CreateDeckCardError::IsCommander);
        }
        let card_count = self
            .deck_repo
            .count_cards_in_deck(request.deck_id)
            .await
            .map_err(CreateDeckCardError::Database)?;
        let card_limit = if request.email_verified {
            MAX_CARDS_PER_DECK
        } else {
            UNVERIFIED_MAX_CARDS_PER_DECK
        };
        if card_count + i64::from(*request.quantity) > card_limit {
            return Err(CreateDeckCardError::LimitReached);
        }
        self.deck_repo.create_deck_card(request).await
    }

    // =====
    //  get
    // =====
    async fn get_deck_profile(
        &self,
        request: &GetDeckProfile,
    ) -> Result<DeckProfile, GetDeckProfileError> {
        let deck_profile = self.deck_repo.get_deck_profile(request).await?;
        if request.user_id != deck_profile.user_id {
            return Err(GetDeckProfileError::Forbidden);
        }
        Ok(deck_profile)
    }

    async fn get_deck_profiles(
        &self,
        request: &GetDeckProfiles,
    ) -> Result<Vec<DeckProfile>, GetDeckProfileError> {
        self.deck_repo.get_deck_profiles(request).await
    }

    async fn get_deck(&self, request: &GetDeckProfile) -> Result<Deck, GetDeckError> {
        let deck_profile = self.deck_repo.get_deck_profile(request).await?;
        let deck_cards = self.deck_repo.get_deck_cards(request).await?;
        let scryfall_data_ids = ScryfallDataIds::from(deck_cards.as_slice());
        let cards = self.card_repo.get_cards(&scryfall_data_ids).await?;

        let mut deck_card_map: HashMap<Uuid, DeckCard> = deck_cards
            .into_iter()
            .map(|dc| (dc.scryfall_data_id, dc))
            .collect();

        let entries: Vec<DeckEntry> = cards
            .into_iter()
            .filter_map(|card| {
                deck_card_map
                    .remove(&card.scryfall_data.id)
                    .map(|deck_card| DeckEntry { card, deck_card })
            })
            .collect();

        let deck = Deck::new(deck_profile, entries);
        Ok(deck)
    }

    // ========
    //  update
    // ========
    async fn update_deck_profile(
        &self,
        request: &UpdateDeckProfile,
    ) -> Result<DeckProfile, UpdateDeckProfileError> {
        let get_deck = GetDeckProfile::from(request);
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

    async fn import_deck_cards(
        &self,
        request: &ImportDeckCards,
    ) -> Result<ImportDeckCardsResult, ImportDeckCardsError> {
        // Auth check
        let get_deck = GetDeckProfile::new(request.user_id, request.deck_id);
        let deck_profile = self.get_deck_profile(&get_deck).await?;
        let copy_max = deck_profile.copy_max.as_ref().map(|cm| **cm);

        // Collect unique lowercased card names
        let names: Vec<String> = request
            .lines
            .iter()
            .map(|l| l.card_name.to_lowercase())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Resolve names (single batch query)
        let cards = self
            .card_repo
            .find_cards_by_exact_names(&names)
            .await
            .map_err(|e| ImportDeckCardsError::Database(e.into()))?;

        // Build lookup map: lowercase name -> Card
        let card_map: HashMap<String, crate::domain::card::models::Card> = cards
            .into_iter()
            .map(|c| (c.scryfall_data.name.to_lowercase(), c))
            .collect();

        // Classify lines, deduplicating by summing quantities
        // Tuple: (scryfall_id, quantity, name, is_basic_land)
        let commander_id = deck_profile.commander_id;
        let mut unresolved: Vec<UnresolvedCard> = Vec::new();
        let mut insert_map: HashMap<Uuid, (Uuid, i32, String, bool)> = HashMap::new();

        for line in &request.lines {
            let key = line.card_name.to_lowercase();
            if let Some(card) = card_map.get(&key) {
                let scryfall_id = card.scryfall_data.id;
                // Skip the commander — it's deck metadata, not a regular card
                if commander_id == Some(scryfall_id) {
                    continue;
                }
                let is_basic_land = card.scryfall_data.is_basic_land();
                insert_map
                    .entry(scryfall_id)
                    .and_modify(|(_, qty, _, _)| *qty += line.quantity)
                    .or_insert_with(|| {
                        (scryfall_id, line.quantity, card.scryfall_data.name.clone(), is_basic_land)
                    });
            } else {
                unresolved.push(UnresolvedCard {
                    name: line.card_name.clone(),
                    reason: "not found".to_string(),
                });
            }
        }

        // Clamp quantities to copy_max (basic lands exempt)
        if let Some(max) = copy_max {
            for (_, qty, _, is_basic_land) in insert_map.values_mut() {
                if !*is_basic_land && *qty > max {
                    *qty = max;
                }
            }
        }

        // Build batch insert data
        let batch: Vec<(Uuid, i32)> =
            insert_map.values().map(|(id, qty, _, _)| (*id, *qty)).collect();

        // Check card limit before inserting
        let card_count = self
            .deck_repo
            .count_cards_in_deck(request.deck_id)
            .await
            .map_err(|e| ImportDeckCardsError::Database(e))?;
        let import_total: i64 = batch.iter().map(|(_, qty)| i64::from(*qty)).sum();
        let card_limit = if request.email_verified {
            MAX_CARDS_PER_DECK
        } else {
            UNVERIFIED_MAX_CARDS_PER_DECK
        };
        if card_count + import_total > card_limit {
            return Err(ImportDeckCardsError::LimitReached);
        }

        // Bulk insert
        let _deck_cards = self.deck_repo.bulk_create_deck_cards(request, &batch).await?;

        // Build imported list from insert_map
        let imported: Vec<ImportedCard> = insert_map
            .into_values()
            .map(|(_, qty, name, _)| ImportedCard { name, quantity: qty })
            .collect();

        Ok(ImportDeckCardsResult {
            imported,
            unresolved,
        })
    }
}
