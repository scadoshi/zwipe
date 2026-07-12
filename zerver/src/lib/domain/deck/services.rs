use std::{collections::HashMap, fmt::Debug};

use uuid::Uuid;

use crate::domain::{
    card::{
        models::synergy::SynergyPayload, ports::CardRepository,
        requests::get_scryfall_data::ScryfallDataIds,
    },
    deck::{
        models::{
            deck::{
                clear_deck_suppressions::ClearDeckSuppressionsError,
                clone_deck::CloneDeckError,
                create_deck_profile::CreateDeckProfileError,
                delete_deck::DeleteDeckError,
                get_deck::GetDeckError,
                get_deck_profile::GetDeckProfileError,
                get_deck_tokens::GetDeckTokensError,
                import_archidekt::ArchidektCard,
                search_deck_cards::SearchDeckCardsError,
                share_deck::{GetSharedDeckError, ShareDeckError, SharedDeck},
                skip_deck_card::SkipDeckCardError,
                update_deck_profile::UpdateDeckProfileError,
            },
            deck_card::{
                create_deck_card::CreateDeckCardError, delete_deck_card::DeleteDeckCardError,
                import_deck_cards::ImportDeckCardsError, update_deck_card::UpdateDeckCardError,
            },
        },
        ports::{DeckRepository, DeckService},
    },
};
use zwipe_core::domain::{
    card::{Card, search_card::card_filter::CardQuery},
    deck::{
        Board, Deck, DeckCard, DeckEntry, ImportMode,
        deck_profile::DeckProfile,
        requests::{
            clear_deck_suppressions::ClearDeckSuppressions,
            clone_deck::CloneDeck,
            create_deck_card::CreateDeckCard,
            create_deck_profile::CreateDeckProfile,
            delete_deck::DeleteDeck,
            delete_deck_card::DeleteDeckCard,
            get_deck_profile::GetDeckProfile,
            get_deck_profiles::GetDeckProfiles,
            import_deck_cards::{
                ImportDeckCards, ImportDeckCardsResult, ImportedCard, UnresolvedCard,
                dfc_front_face, entry_front_face,
            },
            skip_deck_card::SkipDeckCard,
            update_deck_card::UpdateDeckCard,
            update_deck_profile::UpdateDeckProfile,
        },
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
            return Err(if request.email_verified {
                CreateDeckProfileError::LimitReached
            } else {
                CreateDeckProfileError::UnverifiedLimitReached
            });
        }
        self.deck_repo.create_deck_profile(request).await
    }

    async fn create_deck_card(
        &self,
        request: &CreateDeckCard,
    ) -> Result<DeckCard, CreateDeckCardError> {
        let deck_profile = self.get_deck_profile(&request.into()).await?;

        // Resolve command zone scryfall_data_ids to oracle_ids for comparison
        let cz_ids: ScryfallDataIds = [
            deck_profile.commander_id,
            deck_profile.partner_commander_id,
            deck_profile.background_id,
            deck_profile.signature_spell_id,
        ]
        .into_iter()
        .flatten()
        .collect();
        if !cz_ids.is_empty() {
            let cz_oracle_ids: std::collections::HashSet<Uuid> = self
                .card_repo
                .get_multiple_scryfall_data(&cz_ids)
                .await
                .map_err(|e| CreateDeckCardError::Database(e.into()))?
                .into_iter()
                .filter_map(|sd| sd.oracle_id)
                .collect();
            if cz_oracle_ids.contains(&request.oracle_id) {
                return Err(CreateDeckCardError::IsCommander);
            }
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
            return Err(if request.email_verified {
                CreateDeckCardError::LimitReached
            } else {
                CreateDeckCardError::UnverifiedLimitReached
            });
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

        // Fetch all command zone cards (single batch query) so validate_deck
        // can compare by oracle_id rather than printing-specific scryfall_data_id.
        let cz_ids: ScryfallDataIds = [
            deck_profile.commander_id,
            deck_profile.partner_commander_id,
            deck_profile.background_id,
            deck_profile.signature_spell_id,
        ]
        .into_iter()
        .flatten()
        .collect();

        let cz_cards: HashMap<Uuid, Card> = if !cz_ids.is_empty() {
            self.card_repo
                .get_cards(&cz_ids)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|c| (c.scryfall_data.id, c))
                .collect()
        } else {
            HashMap::new()
        };

        let commander_card = deck_profile
            .commander_id
            .and_then(|id| cz_cards.get(&id).cloned());
        let partner_card = deck_profile
            .partner_commander_id
            .and_then(|id| cz_cards.get(&id).cloned());
        let background_card = deck_profile
            .background_id
            .and_then(|id| cz_cards.get(&id).cloned());
        let spell_card = deck_profile
            .signature_spell_id
            .and_then(|id| cz_cards.get(&id).cloned());

        let command_zone = zwipe_core::domain::deck::validate_deck::DeckCommandZone {
            commander: commander_card.as_ref(),
            partner_commander: partner_card.as_ref(),
            background: background_card.as_ref(),
            signature_spell: spell_card.as_ref(),
        };
        let warnings = zwipe_core::domain::deck::validate_deck::validate_deck(
            &deck_profile,
            &entries,
            &command_zone,
        );
        // Carry the command-zone cards to the client so it can fold them into
        // price and card-count calcs (they live on the profile, not in entries).
        let command_zone_cards = command_zone.cards();
        let deck =
            Deck::new(deck_profile, entries, warnings).with_command_zone_cards(command_zone_cards);
        Ok(deck)
    }

    async fn search_deck_cards(
        &self,
        request: &GetDeckProfile,
        filter: &CardQuery,
    ) -> Result<(Vec<Card>, bool), SearchDeckCardsError> {
        let deck_profile = self.deck_repo.get_deck_profile(request).await?;
        if request.user_id != deck_profile.user_id {
            return Err(GetDeckProfileError::Forbidden.into());
        }

        // Exclude everything already in the deck: all boards by oracle_id,
        // plus profile slots (printing ids resolved to oracle, mirroring the
        // importers' slot exclusion).
        let deck_cards = self
            .deck_repo
            .get_deck_cards(request)
            .await
            .map_err(|e| SearchDeckCardsError::Database(e.into()))?;
        let mut exclude_oracle_ids: std::collections::HashSet<Uuid> =
            deck_cards.iter().map(|dc| dc.oracle_id).collect();

        let slot_scryfall_ids: ScryfallDataIds = [
            deck_profile.commander_id,
            deck_profile.partner_commander_id,
            deck_profile.background_id,
            deck_profile.signature_spell_id,
        ]
        .into_iter()
        .flatten()
        .collect();
        if !slot_scryfall_ids.is_empty() {
            exclude_oracle_ids.extend(
                self.card_repo
                    .get_multiple_scryfall_data(&slot_scryfall_ids)
                    .await
                    .map_err(|e| SearchDeckCardsError::Database(e.into()))?
                    .into_iter()
                    .filter_map(|sd| sd.oracle_id),
            );
        }
        let exclude_oracle_ids: Vec<Uuid> = exclude_oracle_ids.into_iter().collect();

        // The synergy score map is needed in two cases: as the membership set
        // when Synergy is ON, or as the default ordering when no explicit sort is
        // set. A missing/unparseable signal degrades to the filter's own
        // semantics (full pool) — which is also the cold-cache fallback for ON.
        let synergy_only = filter.synergy();
        let synergy_scores: Option<serde_json::Value> = match deck_profile.commander_id {
            Some(commander_id) if synergy_only || filter.sort().is_none() => self
                .card_repo
                .commander_synergy_payload(commander_id)
                .await?
                .and_then(|payload| match serde_json::from_value::<SynergyPayload>(payload) {
                    Ok(parsed) => {
                        let scores = parsed.into_scores();
                        if scores.is_empty() {
                            None
                        } else {
                            serde_json::to_value(scores).ok()
                        }
                    }
                    Err(e) => {
                        tracing::warn!(%commander_id, "synergy payload failed to parse, serving unordered: {e}");
                        None
                    }
                }),
            _ => None,
        };

        // Oracle-tag-aware ordering (Phase 4): the deck's explicitly selected
        // otags lift matching cards within the synergy serve. Ladder v1 is just
        // the selected set — empty leaves ordering byte-identical (a future rung
        // could fall back to the commander's own otags, but commander decks are
        // already synergy-ordered, so it buys little).
        let deck_oracle_tags = &deck_profile.oracle_tags;

        let cards = self
            .card_repo
            .search_cards_deck_aware(
                filter,
                Some(deck_profile.id),
                &exclude_oracle_ids,
                synergy_scores.as_ref(),
                synergy_only,
                deck_oracle_tags,
            )
            .await?;
        // Synergy was requested but the commander's cache wasn't available (cold
        // / still computing), so the search fell back to the full pool. Surfaced
        // to the client as a header so it can show a "warming up" hint.
        let synergy_warming =
            synergy_only && deck_profile.commander_id.is_some() && synergy_scores.is_none();
        Ok((cards, synergy_warming))
    }

    async fn get_deck_tokens(
        &self,
        request: &GetDeckProfile,
    ) -> Result<Vec<Card>, GetDeckTokensError> {
        let deck = self.get_deck(request).await?;

        let token_ids: ScryfallDataIds = deck
            .entries
            .iter()
            .filter_map(|e| e.card.scryfall_data.all_parts.as_ref())
            .flat_map(|parts| parts.iter())
            .filter(|rc| rc.component == "token")
            .map(|rc| rc.id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if token_ids.is_empty() {
            return Ok(Vec::new());
        }

        let tokens = self.card_repo.get_cards(&token_ids).await?;
        Ok(tokens)
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

    async fn clear_deck_suppressions(
        &self,
        request: &ClearDeckSuppressions,
    ) -> Result<u64, ClearDeckSuppressionsError> {
        self.deck_repo.clear_deck_suppressions(request).await
    }

    async fn skip_deck_card(&self, request: &SkipDeckCard) -> Result<(), SkipDeckCardError> {
        self.deck_repo.skip_deck_card(request).await
    }

    async fn unskip_deck_card(&self, request: &SkipDeckCard) -> Result<(), SkipDeckCardError> {
        self.deck_repo.unskip_deck_card(request).await
    }

    async fn import_deck_cards(
        &self,
        request: &ImportDeckCards,
    ) -> Result<ImportDeckCardsResult, ImportDeckCardsError> {
        // Auth check
        let get_deck = GetDeckProfile::new(request.user_id, request.deck_id);
        let deck_profile = self.get_deck_profile(&get_deck).await?;
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

        // Build lookup map: lowercase name -> Card. Double-faced cards are also
        // aliased by their front face ("Boggart Trawler" -> "Boggart Trawler //
        // Boggart Bog") so entries that use just the front resolve; a real
        // full-name entry always wins over a front-face alias on collision.
        let mut card_map: HashMap<String, Card> = HashMap::new();
        for card in cards {
            let full = card.scryfall_data.name.to_lowercase();
            if let Some(front) = dfc_front_face(&full) {
                card_map
                    .entry(front.to_string())
                    .or_insert_with(|| card.clone());
            }
            card_map.insert(full, card);
        }

        // Resolve oracle_ids for additional cards (commander, partner, background, signature spell)
        // so we can skip them by oracle_id rather than scryfall_data_id (which is printing-specific).
        let additional_scryfall_ids: ScryfallDataIds = [
            deck_profile.commander_id,
            deck_profile.partner_commander_id,
            deck_profile.background_id,
            deck_profile.signature_spell_id,
        ]
        .into_iter()
        .flatten()
        .collect();
        let additional_oracle_ids: std::collections::HashSet<Uuid> =
            if additional_scryfall_ids.is_empty() {
                std::collections::HashSet::new()
            } else {
                self.card_repo
                    .get_multiple_scryfall_data(&additional_scryfall_ids)
                    .await
                    .map_err(|e| ImportDeckCardsError::Database(e.into()))?
                    .into_iter()
                    .filter_map(|sd| sd.oracle_id)
                    .collect()
            };

        // Classify lines, deduplicating by oracle_id (summing quantities).
        // Tuple: (scryfall_id, oracle_id, quantity, name, is_basic_land, board)
        let mut unresolved: Vec<UnresolvedCard> = Vec::new();
        let mut insert_map: HashMap<Uuid, (Uuid, Uuid, i32, String, bool, String)> = HashMap::new();

        for line in &request.lines {
            let key = line.card_name.to_lowercase();
            // Try the entry as written, then its front face (before a slash), so
            // "A // B", "A / B", and "A" all resolve to the same card.
            let matched = card_map.get(&key).or_else(|| {
                let front = entry_front_face(&key);
                if front.is_empty() || front == key {
                    None
                } else {
                    card_map.get(front)
                }
            });
            if let Some(card) = matched {
                let scryfall_id = card.scryfall_data.id;
                let oracle_id = match card.scryfall_data.oracle_id {
                    Some(id) => id,
                    None => {
                        unresolved.push(UnresolvedCard {
                            name: line.card_name.clone(),
                            reason: "missing oracle id".to_string(),
                        });
                        continue;
                    }
                };
                // Skip additional cards (commander, partner, background, signature spell)
                if additional_oracle_ids.contains(&oracle_id) {
                    continue;
                }
                let is_basic_land = card.scryfall_data.is_basic_land();
                let board = line.board.display_name().to_string();
                insert_map
                    .entry(oracle_id)
                    .and_modify(|(_, _, qty, _, _, _)| *qty += line.quantity)
                    .or_insert_with(|| {
                        (
                            scryfall_id,
                            oracle_id,
                            line.quantity,
                            card.scryfall_data.name.clone(),
                            is_basic_land,
                            board,
                        )
                    });
            } else {
                unresolved.push(UnresolvedCard {
                    name: line.card_name.clone(),
                    reason: "not found".to_string(),
                });
            }
        }

        // Build batch insert data: (scryfall_data_id, oracle_id, quantity, board)
        let batch: Vec<(Uuid, Uuid, i32, String)> = insert_map
            .values()
            .map(|(sid, oid, qty, _, _, board)| (*sid, *oid, *qty, board.clone()))
            .collect();

        // Check card limit before inserting. The limit counts the deck board
        // only. In replace mode a board present in the import becomes exactly
        // the imported list, so the post-import deck-board total is just the
        // imported deck-board quantity. In add mode the upsert replaces
        // quantities for existing oracle_ids, so subtract the overlap (existing
        // quantities of cards being reimported) to get the true total.
        let card_count = self
            .deck_repo
            .count_cards_in_deck(request.deck_id)
            .await
            .map_err(ImportDeckCardsError::Database)?;
        let import_total: i64 = batch.iter().map(|(_, _, qty, _)| i64::from(*qty)).sum();
        let post_import_total = if request.mode.is_replace() {
            if batch.iter().any(|(_, _, _, board)| board == "deck") {
                batch
                    .iter()
                    .filter(|(_, _, _, board)| board == "deck")
                    .map(|(_, _, qty, _)| i64::from(*qty))
                    .sum()
            } else {
                card_count
            }
        } else {
            let import_oracle_ids: Vec<Uuid> = batch.iter().map(|(_, oid, _, _)| *oid).collect();
            let overlap_qty = self
                .deck_repo
                .sum_quantities_for_oracle_ids(request.deck_id, &import_oracle_ids)
                .await
                .map_err(ImportDeckCardsError::Database)?;
            (card_count - overlap_qty) + import_total
        };
        let card_limit = if request.email_verified {
            MAX_CARDS_PER_DECK
        } else {
            UNVERIFIED_MAX_CARDS_PER_DECK
        };
        if post_import_total > card_limit {
            return Err(if request.email_verified {
                ImportDeckCardsError::LimitReached
            } else {
                ImportDeckCardsError::UnverifiedLimitReached
            });
        }

        // Bulk insert
        let _deck_cards = self
            .deck_repo
            .bulk_create_deck_cards(request, &batch)
            .await?;

        // Replace mode: make each board present in the import exactly match
        // the imported list by removing its other cards. Boards absent from
        // the import are left alone, so an empty paste never wipes anything.
        if request.mode.is_replace() {
            let boards: std::collections::HashSet<&str> = batch
                .iter()
                .map(|(_, _, _, board)| board.as_str())
                .collect();
            for board in boards {
                let keep: Vec<Uuid> = batch
                    .iter()
                    .filter(|(_, _, _, b)| b == board)
                    .map(|(_, oid, _, _)| *oid)
                    .collect();
                self.deck_repo
                    .delete_deck_cards_not_in(request.deck_id, board, &keep)
                    .await
                    .map_err(ImportDeckCardsError::Database)?;
            }
        }

        // Build imported list from insert_map
        let imported: Vec<ImportedCard> = insert_map
            .into_values()
            .map(|(_, _, qty, name, _, _)| ImportedCard {
                name,
                quantity: qty,
            })
            .collect();

        Ok(ImportDeckCardsResult {
            imported,
            unresolved,
        })
    }

    async fn import_archidekt_deck(
        &self,
        user_id: Uuid,
        deck_id: Uuid,
        cards: &[ArchidektCard],
        board: Board,
        email_verified: bool,
        mode: ImportMode,
    ) -> Result<ImportDeckCardsResult, ImportDeckCardsError> {
        use std::collections::HashSet;

        // Auth check
        let get_deck = GetDeckProfile::new(user_id, deck_id);
        let deck_profile = self.get_deck_profile(&get_deck).await?;

        // Resolve oracle_ids for additional cards (commander, partner,
        // background, signature spell) so imported copies of them are skipped,
        // matching import_deck_cards.
        let additional_scryfall_ids: ScryfallDataIds = [
            deck_profile.commander_id,
            deck_profile.partner_commander_id,
            deck_profile.background_id,
            deck_profile.signature_spell_id,
        ]
        .into_iter()
        .flatten()
        .collect();
        let additional_oracle_ids: HashSet<Uuid> = if additional_scryfall_ids.is_empty() {
            HashSet::new()
        } else {
            self.card_repo
                .get_multiple_scryfall_data(&additional_scryfall_ids)
                .await
                .map_err(|e| ImportDeckCardsError::Database(e.into()))?
                .into_iter()
                .filter_map(|sd| sd.oracle_id)
                .collect()
        };

        // Resolve every printing by Scryfall id in one batch. A row may exist
        // but carry a null oracle_id (reversible Secret Lair printings do); those
        // can't be deck cards (deck_cards.oracle_id is NOT NULL), so they fall
        // through to the name fallback below.
        let all_ids: ScryfallDataIds = cards.iter().map(|c| c.scryfall_id).collect();
        let resolved = self
            .card_repo
            .get_multiple_scryfall_data(&all_ids)
            .await
            .map_err(|e| ImportDeckCardsError::Database(e.into()))?;
        let oracle_by_id: HashMap<Uuid, Uuid> = resolved
            .iter()
            .filter_map(|sd| sd.oracle_id.map(|oracle| (sd.id, oracle)))
            .collect();

        // For anything the id lookup couldn't resolve, fall back to the card
        // name → latest printing (recovers reversible/Secret Lair printings,
        // swapping the fancy printing for a standard one with a valid oracle_id).
        let fallback_names: Vec<String> = cards
            .iter()
            .filter(|c| !oracle_by_id.contains_key(&c.scryfall_id) && !c.name.is_empty())
            .map(|c| c.name.to_lowercase())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let name_matches = if fallback_names.is_empty() {
            Vec::new()
        } else {
            self.card_repo
                .find_cards_by_exact_names(&fallback_names)
                .await
                .map_err(|e| ImportDeckCardsError::Database(e.into()))?
        };
        let by_name: HashMap<String, (Uuid, Uuid)> = name_matches
            .iter()
            .filter_map(|c| {
                c.scryfall_data.oracle_id.map(|oracle| {
                    (
                        c.scryfall_data.name.to_lowercase(),
                        (c.scryfall_data.id, oracle),
                    )
                })
            })
            .collect();

        // Classify each card, deduplicating by oracle_id (summing quantities).
        let mut unresolved: Vec<UnresolvedCard> = Vec::new();
        let mut insert_map: HashMap<Uuid, (Uuid, i32, String)> = HashMap::new();

        for card in cards {
            // Prefer the exact printing the deck used; fall back to name.
            let resolved = oracle_by_id
                .get(&card.scryfall_id)
                .map(|&oracle| (card.scryfall_id, oracle))
                .or_else(|| by_name.get(&card.name.to_lowercase()).copied());
            let Some((scryfall_id, oracle_id)) = resolved else {
                unresolved.push(UnresolvedCard {
                    name: if card.name.is_empty() {
                        card.scryfall_id.to_string()
                    } else {
                        card.name.clone()
                    },
                    reason: "card not found in card database".to_string(),
                });
                continue;
            };
            // Skip additional cards (commander, partner, background, signature spell)
            if additional_oracle_ids.contains(&oracle_id) {
                continue;
            }
            let entry = insert_map
                .entry(oracle_id)
                .or_insert_with(|| (scryfall_id, 0, card.name.clone()));
            entry.1 += card.quantity;
        }

        let board_name = board.display_name().to_string();
        let batch: Vec<(Uuid, Uuid, i32, String)> = insert_map
            .iter()
            .map(|(oracle_id, (sid, qty, _))| (*sid, *oracle_id, *qty, board_name.clone()))
            .collect();

        // Check card limit before inserting — same math as import_deck_cards.
        // The limit counts the deck board only.
        let card_count = self
            .deck_repo
            .count_cards_in_deck(deck_id)
            .await
            .map_err(ImportDeckCardsError::Database)?;
        let import_total: i64 = batch.iter().map(|(_, _, qty, _)| i64::from(*qty)).sum();
        let post_import_total = if mode.is_replace() {
            if board == Board::Deck {
                import_total
            } else {
                card_count
            }
        } else {
            let import_oracle_ids: Vec<Uuid> = insert_map.keys().copied().collect();
            let overlap_qty = self
                .deck_repo
                .sum_quantities_for_oracle_ids(deck_id, &import_oracle_ids)
                .await
                .map_err(ImportDeckCardsError::Database)?;
            (card_count - overlap_qty) + import_total
        };
        let card_limit = if email_verified {
            MAX_CARDS_PER_DECK
        } else {
            UNVERIFIED_MAX_CARDS_PER_DECK
        };
        if post_import_total > card_limit {
            return Err(if email_verified {
                ImportDeckCardsError::LimitReached
            } else {
                ImportDeckCardsError::UnverifiedLimitReached
            });
        }

        // Bulk upsert
        if !batch.is_empty() {
            let carrier = ImportDeckCards {
                user_id,
                deck_id,
                lines: Vec::new(),
                email_verified,
                mode,
            };
            self.deck_repo
                .bulk_create_deck_cards(&carrier, &batch)
                .await?;
        }

        // Replace mode: the board becomes exactly the imported list. Skipped
        // when nothing resolved, so a bad URL never wipes a board.
        if mode.is_replace() && !batch.is_empty() {
            let keep: Vec<Uuid> = insert_map.keys().copied().collect();
            self.deck_repo
                .delete_deck_cards_not_in(deck_id, &board_name, &keep)
                .await
                .map_err(ImportDeckCardsError::Database)?;
        }

        let imported: Vec<ImportedCard> = insert_map
            .into_values()
            .map(|(_, qty, name)| ImportedCard {
                name,
                quantity: qty,
            })
            .collect();

        Ok(ImportDeckCardsResult {
            imported,
            unresolved,
        })
    }

    // =======
    //  clone
    // =======
    async fn clone_deck(&self, request: &CloneDeck) -> Result<Uuid, CloneDeckError> {
        // 1. Verify the source exists and is owned by the caller. get_deck_profile
        //    on the repo already enforces ownership (returns Forbidden on mismatch)
        //    and existence (returns NotFound).
        let get_req = GetDeckProfile::new(request.user_id, request.source_deck_id);
        if let Err(e) = self.deck_repo.get_deck_profile(&get_req).await {
            return Err(match e {
                GetDeckProfileError::NotFound => CloneDeckError::SourceNotFound,
                GetDeckProfileError::Forbidden => CloneDeckError::Forbidden,
                other => CloneDeckError::GetSource(other),
            });
        }

        // 2. Enforce the same deck-count limit as create_deck_profile.
        let deck_count = self
            .deck_repo
            .count_decks_by_user(request.user_id)
            .await
            .map_err(CloneDeckError::Database)?;
        let deck_limit = if request.email_verified {
            MAX_DECKS_PER_USER
        } else {
            UNVERIFIED_MAX_DECKS_PER_USER
        };
        if deck_count >= deck_limit {
            return Err(if request.email_verified {
                CloneDeckError::LimitReached
            } else {
                CloneDeckError::UnverifiedLimitReached
            });
        }

        // 3. Delegate to the repo for the transactional copy.
        self.deck_repo
            .clone_deck(request.source_deck_id, &request.new_name, request.user_id)
            .await
    }

    // =======
    //  share
    // =======
    async fn share_deck(&self, request: &GetDeckProfile) -> Result<Uuid, ShareDeckError> {
        self.get_deck_profile(request).await.map_err(|e| match e {
            GetDeckProfileError::Forbidden => ShareDeckError::Forbidden,
            GetDeckProfileError::NotFound => ShareDeckError::NotFound,
            other => ShareDeckError::Database(other.into()),
        })?;
        self.deck_repo.set_share_token(request.deck_id).await
    }

    async fn unshare_deck(&self, request: &GetDeckProfile) -> Result<(), ShareDeckError> {
        self.get_deck_profile(request).await.map_err(|e| match e {
            GetDeckProfileError::Forbidden => ShareDeckError::Forbidden,
            GetDeckProfileError::NotFound => ShareDeckError::NotFound,
            other => ShareDeckError::Database(other.into()),
        })?;
        self.deck_repo.clear_share_token(request.deck_id).await
    }

    async fn get_shared_deck(&self, token: Uuid) -> Result<SharedDeck, GetSharedDeckError> {
        // The token is the capability: resolving it yields the owner's id,
        // which satisfies the ownership checks along the normal get_deck path.
        let (deck_id, owner_id) = self
            .deck_repo
            .get_deck_id_by_share_token(token)
            .await
            .map_err(GetSharedDeckError::Database)?
            .ok_or(GetSharedDeckError::NotFound)?;

        let request = GetDeckProfile::new(owner_id, deck_id);
        let deck = self.get_deck(&request).await?;

        let cz_ids: ScryfallDataIds = [
            deck.deck_profile.commander_id,
            deck.deck_profile.partner_commander_id,
            deck.deck_profile.background_id,
            deck.deck_profile.signature_spell_id,
        ]
        .into_iter()
        .flatten()
        .collect();
        let cz_cards: HashMap<Uuid, Card> = if cz_ids.is_empty() {
            HashMap::new()
        } else {
            self.card_repo
                .get_cards(&cz_ids)
                .await
                .map_err(|e| GetSharedDeckError::Database(e.into()))?
                .into_iter()
                .map(|c| (c.scryfall_data.id, c))
                .collect()
        };

        let commander = deck
            .deck_profile
            .commander_id
            .and_then(|id| cz_cards.get(&id).cloned());
        let partner_commander = deck
            .deck_profile
            .partner_commander_id
            .and_then(|id| cz_cards.get(&id).cloned());
        let background = deck
            .deck_profile
            .background_id
            .and_then(|id| cz_cards.get(&id).cloned());
        let signature_spell = deck
            .deck_profile
            .signature_spell_id
            .and_then(|id| cz_cards.get(&id).cloned());

        Ok(SharedDeck {
            deck,
            commander,
            partner_commander,
            background,
            signature_spell,
        })
    }
}
