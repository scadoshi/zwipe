//! Deck management repository implementation.

/// SQLx error-to-domain error mappings and intermediate conversion errors.
pub mod error;
/// Query-based deck ownership verification.
pub mod helper;
/// Database-to-domain deck model conversions.
pub mod models;

use crate::{
    domain::deck::{
        models::{
            deck::{
                clear_deck_suppressions::ClearDeckSuppressionsError, clone_deck::CloneDeckError,
                commander_maybeboard::CommanderMaybeboardError,
                create_deck_profile::CreateDeckProfileError, delete_deck::DeleteDeckError,
                get_deck_profile::GetDeckProfileError, share_deck::ShareDeckError,
                skip_deck_card::SkipDeckCardError, update_deck_profile::UpdateDeckProfileError,
            },
            deck_card::{
                create_deck_card::CreateDeckCardError, delete_deck_card::DeleteDeckCardError,
                get_deck_card::GetDeckCardError, import_deck_cards::ImportDeckCardsError,
                update_deck_card::UpdateDeckCardError,
            },
        },
        ports::DeckRepository,
    },
    outbound::sqlx::{
        deck::{
            error::{IntoDeckCardError, IntoDeckProfileError},
            helper::OwnsDeck,
            models::{DatabaseDeckCard, DatabaseDeckProfile},
        },
        postgres::Postgres,
    },
};
use sqlx::{QueryBuilder, query, query_as, query_scalar};
use zwipe_core::domain::deck::{
    Board, DeckCard, DeckName, DeckOtherTag, DeckTag,
    deck_profile::DeckProfile,
    requests::{
        clear_deck_suppressions::ClearDeckSuppressions,
        commander_maybeboard::CommanderMaybeboardCard, create_deck_card::CreateDeckCard,
        create_deck_profile::CreateDeckProfile, delete_deck::DeleteDeck,
        delete_deck_card::DeleteDeckCard, get_deck_profile::GetDeckProfile,
        get_deck_profiles::GetDeckProfiles, skip_deck_card::SkipDeckCard,
        update_deck_card::UpdateDeckCard, update_deck_profile::UpdateDeckProfile,
    },
};

/// Per-deck ceiling on suppression rows, enforced at ingest by evicting the
/// oldest `suppressed_at` beyond it.
pub(crate) const MAX_SUPPRESSIONS_PER_DECK: i64 = 5_000;

/// Serializes deck tags to a JSONB array of snake_case strings for storage.
fn deck_tags_to_json(tags: &[DeckTag]) -> serde_json::Value {
    serde_json::Value::Array(
        tags.iter()
            .map(|t| serde_json::Value::String(t.to_string()))
            .collect(),
    )
}

/// Serializes deck other-tags to a JSONB array of snake_case strings for storage.
fn deck_other_tags_to_json(tags: &[DeckOtherTag]) -> serde_json::Value {
    serde_json::Value::Array(
        tags.iter()
            .map(|t| serde_json::Value::String(t.to_string()))
            .collect(),
    )
}

/// Serializes deck oracle-tag slugs to a JSONB array of strings for storage.
fn deck_oracle_tags_to_json(tags: &[String]) -> serde_json::Value {
    serde_json::Value::Array(
        tags.iter()
            .map(|t| serde_json::Value::String(t.clone()))
            .collect(),
    )
}

impl DeckRepository for Postgres {
    // ========
    //  create
    // ========
    async fn create_deck_profile(
        &self,
        request: &CreateDeckProfile,
    ) -> Result<DeckProfile, CreateDeckProfileError> {
        let mut tx = self.pool.begin().await?;
        let tags_json = deck_tags_to_json(&request.tags);
        let other_tags_json = deck_other_tags_to_json(&request.other_tags);
        let oracle_tags_json = deck_oracle_tags_to_json(&request.oracle_tags);
        let database_deck_profile = query_as!(
            DatabaseDeckProfile,
            r#"INSERT INTO decks (name, commander_id, partner_commander_id, background_id, signature_spell_id, format, tags, power_level, other_tags, oracle_tags, land_target, price_target, price_target_currency, user_id)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING id, name, commander_id, partner_commander_id, background_id, signature_spell_id, format, tags as "tags?", power_level, other_tags as "other_tags?", oracle_tags as "oracle_tags?", land_target, price_target, price_target_currency, share_token, user_id,
                         0::bigint as "card_count",
                         (SELECT sd.name FROM scryfall_data sd WHERE sd.id = commander_id) as "commander_name?",
                         (SELECT sd.name FROM scryfall_data sd WHERE sd.id = partner_commander_id) as "partner_commander_name?",
                         (SELECT sd.name FROM scryfall_data sd WHERE sd.id = background_id) as "background_name?",
                         (SELECT sd.name FROM scryfall_data sd WHERE sd.id = signature_spell_id) as "signature_spell_name?",
                         (SELECT COALESCE(sd.image_uris->>'art_crop', sd.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data sd WHERE sd.id = commander_id) as "commander_art_url?",
                         (SELECT COALESCE(sd.image_uris->>'art_crop', sd.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data sd WHERE sd.id = partner_commander_id) as "partner_commander_art_url?",
                         (SELECT COALESCE(sd.image_uris->>'art_crop', sd.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data sd WHERE sd.id = background_id) as "background_art_url?",
                         (SELECT COALESCE(sd.image_uris->>'art_crop', sd.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data sd WHERE sd.id = signature_spell_id) as "signature_spell_art_url?",
                         (SELECT array_agg(DISTINCT ci)
                            FROM scryfall_data sci, unnest(sci.color_identity) AS ci
                           WHERE sci.id = ANY(ARRAY[commander_id, partner_commander_id, background_id, signature_spell_id]
                                              || ARRAY(SELECT dc2.scryfall_data_id FROM deck_cards dc2 WHERE dc2.deck_id = id AND dc2.board = 'deck'))) as "color_identity?: Vec<String>""#,
            request.name.to_string(),
            request.commander_id,
            request.partner_commander_id,
            request.background_id,
            request.signature_spell_id,
            request.format.map(|f| f.to_legality_key().to_string()) as Option<String>,
            tags_json,
            request.power_level.map(|p| p.to_string()) as Option<String>,
            other_tags_json,
            oracle_tags_json,
            request.land_target,
            request.price_target,
            request.price_target_currency.map(|c| c.json_key().to_string()) as Option<String>,
            request.user_id
        )
        .fetch_one(&mut *tx)
        .await?;
        let deck_profile: DeckProfile = database_deck_profile.try_into()?;
        tx.commit().await?;
        Ok(deck_profile)
    }

    async fn create_deck_card(
        &self,
        request: &CreateDeckCard,
        card_limit: i64,
    ) -> Result<DeckCard, CreateDeckCardError> {
        let mut tx = self.pool.begin().await?;
        // Ownership + row lock in one shot; FOR UPDATE serializes concurrent
        // adds (and imports) on this deck, closing the count-then-insert TOCTOU.
        let owned = sqlx::query_scalar!(
            "SELECT id FROM decks WHERE id = $1 AND user_id = $2 FOR UPDATE",
            request.deck_id,
            request.user_id
        )
        .fetch_optional(&mut *tx)
        .await?;
        if owned.is_none() {
            return Err(CreateDeckCardError::Forbidden);
        }
        // Card-limit check under the lock (all boards count toward the cap).
        let card_count = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(quantity), 0) FROM deck_cards WHERE deck_id = $1",
            request.deck_id
        )
        .fetch_one(&mut *tx)
        .await?
        .unwrap_or(0);
        if card_count + i64::from(*request.quantity) > card_limit {
            return Err(if request.email_verified {
                CreateDeckCardError::LimitReached
            } else {
                CreateDeckCardError::UnverifiedLimitReached
            });
        }
        let database_deck_card = query_as!(
            DatabaseDeckCard,
            "INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, board) VALUES ($1, $2, $3, $4, $5) RETURNING deck_id, scryfall_data_id, oracle_id, quantity, board, mvp_at",
            request.deck_id,
            request.scryfall_data_id,
            request.oracle_id,
            *request.quantity,
            request.board.display_name()
        )
        .fetch_one(&mut *tx)
        .await?;
        // Adding a card cancels any suppression on it (e.g. undoing a removal
        // re-adds the card — the "doesn't fit" signal no longer holds).
        query!(
            "DELETE FROM deck_card_suppressions WHERE deck_id = $1 AND oracle_id = $2",
            request.deck_id,
            request.oracle_id,
        )
        .execute(&mut *tx)
        .await?;
        let deck_card: DeckCard = database_deck_card.try_into()?;
        tx.commit().await?;
        Ok(deck_card)
    }

    // =======
    //  count
    // =======
    async fn count_decks_by_user(&self, user_id: uuid::Uuid) -> Result<i64, anyhow::Error> {
        let count = sqlx::query_scalar!("SELECT COUNT(*) FROM decks WHERE user_id = $1", user_id)
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);
        Ok(count)
    }

    async fn count_cards_in_deck(&self, deck_id: uuid::Uuid) -> Result<i64, anyhow::Error> {
        // Counts ALL boards (mainboard + maybeboard + sideboard) — this feeds the
        // per-deck card cap, which applies across every board.
        let count = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(quantity), 0) FROM deck_cards WHERE deck_id = $1",
            deck_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);
        Ok(count)
    }

    // =====
    //  get
    // =====
    async fn get_deck_profile(
        &self,
        request: &GetDeckProfile,
    ) -> Result<DeckProfile, GetDeckProfileError> {
        let database_deck_profile = query_as!(
            DatabaseDeckProfile,
            r#"SELECT d.id, d.name, d.commander_id, d.partner_commander_id, d.background_id, d.signature_spell_id,
                      d.format, d.tags as "tags?", d.power_level, d.other_tags as "other_tags?", d.oracle_tags as "oracle_tags?", d.land_target, d.price_target, d.price_target_currency, d.share_token, d.user_id,
                      COALESCE(SUM(dc.quantity) FILTER (WHERE dc.board = 'deck'), 0) as "card_count",
                      sd.name as "commander_name?",
                      (SELECT s2.name FROM scryfall_data s2 WHERE s2.id = d.partner_commander_id) as "partner_commander_name?",
                      (SELECT s3.name FROM scryfall_data s3 WHERE s3.id = d.background_id) as "background_name?",
                      (SELECT s4.name FROM scryfall_data s4 WHERE s4.id = d.signature_spell_id) as "signature_spell_name?",
                      -- Command-zone art, as correlated subqueries rather than off the
                      -- joined `sd` so the GROUP BY below stays as it is. COALESCE is
                      -- the front-face fallback double-faced cards need.
                      (SELECT COALESCE(s5.image_uris->>'art_crop', s5.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data s5 WHERE s5.id = d.commander_id) as "commander_art_url?",
                      (SELECT COALESCE(s6.image_uris->>'art_crop', s6.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data s6 WHERE s6.id = d.partner_commander_id) as "partner_commander_art_url?",
                      (SELECT COALESCE(s7.image_uris->>'art_crop', s7.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data s7 WHERE s7.id = d.background_id) as "background_art_url?",
                      (SELECT COALESCE(s8.image_uris->>'art_crop', s8.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data s8 WHERE s8.id = d.signature_spell_id) as "signature_spell_art_url?",
                      (SELECT array_agg(DISTINCT ci)
                         FROM scryfall_data sci, unnest(sci.color_identity) AS ci
                        WHERE sci.id = ANY(ARRAY[d.commander_id, d.partner_commander_id, d.background_id, d.signature_spell_id]
                                           || ARRAY(SELECT dc2.scryfall_data_id FROM deck_cards dc2 WHERE dc2.deck_id = d.id AND dc2.board = 'deck'))) as "color_identity?: Vec<String>"
               FROM decks d
               LEFT JOIN deck_cards dc ON d.id = dc.deck_id
               LEFT JOIN scryfall_data sd ON d.commander_id = sd.id
               WHERE d.id = $1
               GROUP BY d.id, d.name, d.commander_id, d.partner_commander_id, d.background_id, d.signature_spell_id,
                        d.format, d.land_target, d.price_target, d.price_target_currency, d.share_token, d.user_id, sd.name"#,
            request.deck_id
        )
        .fetch_one(&self.pool)
        .await?;
        if database_deck_profile.user_id != request.user_id {
            return Err(GetDeckProfileError::Forbidden);
        }
        let deck_profile: DeckProfile = database_deck_profile.try_into()?;
        Ok(deck_profile)
    }

    async fn get_deck_profiles(
        &self,
        request: &GetDeckProfiles,
    ) -> Result<Vec<DeckProfile>, GetDeckProfileError> {
        let database_deck_profiles = query_as!(
            DatabaseDeckProfile,
            r#"SELECT d.id, d.name, d.commander_id, d.partner_commander_id, d.background_id, d.signature_spell_id,
                      d.format, d.tags as "tags?", d.power_level, d.other_tags as "other_tags?", d.oracle_tags as "oracle_tags?", d.land_target, d.price_target, d.price_target_currency, d.share_token, d.user_id,
                      COALESCE(SUM(dc.quantity) FILTER (WHERE dc.board = 'deck'), 0) as "card_count",
                      sd.name as "commander_name?",
                      (SELECT s2.name FROM scryfall_data s2 WHERE s2.id = d.partner_commander_id) as "partner_commander_name?",
                      (SELECT s3.name FROM scryfall_data s3 WHERE s3.id = d.background_id) as "background_name?",
                      (SELECT s4.name FROM scryfall_data s4 WHERE s4.id = d.signature_spell_id) as "signature_spell_name?",
                      -- Command-zone art, as correlated subqueries rather than off the
                      -- joined `sd` so the GROUP BY below stays as it is. COALESCE is
                      -- the front-face fallback double-faced cards need.
                      (SELECT COALESCE(s5.image_uris->>'art_crop', s5.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data s5 WHERE s5.id = d.commander_id) as "commander_art_url?",
                      (SELECT COALESCE(s6.image_uris->>'art_crop', s6.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data s6 WHERE s6.id = d.partner_commander_id) as "partner_commander_art_url?",
                      (SELECT COALESCE(s7.image_uris->>'art_crop', s7.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data s7 WHERE s7.id = d.background_id) as "background_art_url?",
                      (SELECT COALESCE(s8.image_uris->>'art_crop', s8.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data s8 WHERE s8.id = d.signature_spell_id) as "signature_spell_art_url?",
                      (SELECT array_agg(DISTINCT ci)
                         FROM scryfall_data sci, unnest(sci.color_identity) AS ci
                        WHERE sci.id = ANY(ARRAY[d.commander_id, d.partner_commander_id, d.background_id, d.signature_spell_id]
                                           || ARRAY(SELECT dc2.scryfall_data_id FROM deck_cards dc2 WHERE dc2.deck_id = d.id AND dc2.board = 'deck'))) as "color_identity?: Vec<String>"
               FROM decks d
               LEFT JOIN deck_cards dc ON d.id = dc.deck_id
               LEFT JOIN scryfall_data sd ON d.commander_id = sd.id
               WHERE d.user_id = $1
               GROUP BY d.id, d.name, d.commander_id, d.partner_commander_id, d.background_id, d.signature_spell_id,
                        d.format, d.land_target, d.price_target, d.price_target_currency, d.share_token, d.user_id, sd.name"#,
            request.user_id
        )
        .fetch_all(&self.pool)
        .await?;
        let deck_profiles: Vec<DeckProfile> = database_deck_profiles
            .into_iter()
            .map(|x| x.try_into())
            .collect::<Result<Vec<DeckProfile>, IntoDeckProfileError>>()?;
        Ok(deck_profiles)
    }

    async fn get_deck_cards(
        &self,
        request: &GetDeckProfile,
    ) -> Result<Vec<DeckCard>, GetDeckCardError> {
        if !request
            .user_id
            .owns_deck(request.deck_id, &self.pool)
            .await?
        {
            return Err(GetDeckCardError::Forbidden);
        }
        let database_deck_cards = query_as!(
            DatabaseDeckCard,
            "SELECT deck_id, scryfall_data_id, oracle_id, quantity, board, mvp_at FROM deck_cards WHERE deck_id = $1",
            request.deck_id
        )
        .fetch_all(&self.pool)
        .await?;
        let deck_cards: Vec<DeckCard> = database_deck_cards
            .into_iter()
            .map(|x| x.try_into())
            .collect::<Result<Vec<DeckCard>, IntoDeckCardError>>()?;
        Ok(deck_cards)
    }

    // ========
    //  update
    // ========
    /// Dynamically builds an `UPDATE` query for only the provided fields.
    ///
    /// Always sets `updated_at` to the current timestamp regardless of which
    /// fields are being updated.
    async fn update_deck_profile(
        &self,
        request: &UpdateDeckProfile,
    ) -> Result<DeckProfile, UpdateDeckProfileError> {
        if !request
            .user_id
            .owns_deck(request.deck_id, &self.pool)
            .await?
        {
            return Err(UpdateDeckProfileError::Forbidden);
        }
        let mut tx = self.pool.begin().await?;
        let mut qb = QueryBuilder::new("UPDATE decks SET ");
        let mut sep = qb.separated(", ");
        if let Some(name) = &request.name {
            sep.push("name = ").push_bind_unseparated(name.to_string());
        }

        if let Some(commander_id) = &request.commander_id {
            sep.push("commander_id = ")
                .push_bind_unseparated(commander_id);
        }
        if let Some(partner_commander_id) = &request.partner_commander_id {
            sep.push("partner_commander_id = ")
                .push_bind_unseparated(partner_commander_id);
        }
        if let Some(background_id) = &request.background_id {
            sep.push("background_id = ")
                .push_bind_unseparated(background_id);
        }
        if let Some(signature_spell_id) = &request.signature_spell_id {
            sep.push("signature_spell_id = ")
                .push_bind_unseparated(signature_spell_id);
        }
        if let Some(format) = &request.format {
            sep.push("format = ")
                .push_bind_unseparated(format.map(|f| f.to_legality_key().to_string()));
        }
        if let Some(tags) = &request.tags {
            sep.push("tags = ")
                .push_bind_unseparated(deck_tags_to_json(tags));
        }
        if let Some(power_level) = &request.power_level {
            sep.push("power_level = ")
                .push_bind_unseparated(power_level.map(|p| p.to_string()));
        }
        if let Some(other_tags) = &request.other_tags {
            sep.push("other_tags = ")
                .push_bind_unseparated(deck_other_tags_to_json(other_tags));
        }
        if let Some(oracle_tags) = &request.oracle_tags {
            sep.push("oracle_tags = ")
                .push_bind_unseparated(deck_oracle_tags_to_json(oracle_tags));
        }
        if let Some(land_target) = &request.land_target {
            sep.push("land_target = ")
                .push_bind_unseparated(*land_target);
        }
        if let Some(price_target) = &request.price_target {
            sep.push("price_target = ")
                .push_bind_unseparated(*price_target);
        }
        if let Some(price_target_currency) = &request.price_target_currency {
            sep.push("price_target_currency = ")
                .push_bind_unseparated(price_target_currency.map(|c| c.json_key().to_string()));
        }
        let now = chrono::Utc::now();
        sep.push("updated_at = ").push_bind_unseparated(now);

        qb.push(" WHERE id = ")
            .push_bind(request.deck_id)
            .push(r#" RETURNING id, name, commander_id, partner_commander_id, background_id, signature_spell_id, format, tags, power_level, other_tags, oracle_tags, land_target, price_target, price_target_currency, share_token, user_id,
                       (SELECT COALESCE(SUM(dc.quantity) FILTER (WHERE dc.board = 'deck'), 0) FROM deck_cards dc WHERE dc.deck_id = decks.id) as card_count,
                       (SELECT sd.name FROM scryfall_data sd WHERE sd.id = decks.commander_id) as commander_name,
                       (SELECT sd.name FROM scryfall_data sd WHERE sd.id = decks.partner_commander_id) as partner_commander_name,
                       (SELECT sd.name FROM scryfall_data sd WHERE sd.id = decks.background_id) as background_name,
                       (SELECT sd.name FROM scryfall_data sd WHERE sd.id = decks.signature_spell_id) as signature_spell_name,
                       (SELECT COALESCE(sd.image_uris->>'art_crop', sd.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data sd WHERE sd.id = decks.commander_id) as commander_art_url,
                       (SELECT COALESCE(sd.image_uris->>'art_crop', sd.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data sd WHERE sd.id = decks.partner_commander_id) as partner_commander_art_url,
                       (SELECT COALESCE(sd.image_uris->>'art_crop', sd.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data sd WHERE sd.id = decks.background_id) as background_art_url,
                       (SELECT COALESCE(sd.image_uris->>'art_crop', sd.card_faces->0->'image_uris'->>'art_crop') FROM scryfall_data sd WHERE sd.id = decks.signature_spell_id) as signature_spell_art_url,
                       (SELECT array_agg(DISTINCT ci)
                          FROM scryfall_data sci, unnest(sci.color_identity) AS ci
                         WHERE sci.id = ANY(ARRAY[decks.commander_id, decks.partner_commander_id, decks.background_id, decks.signature_spell_id]
                                            || ARRAY(SELECT dc2.scryfall_data_id FROM deck_cards dc2 WHERE dc2.deck_id = decks.id AND dc2.board = 'deck'))) as color_identity"#);
        let database_deck: DatabaseDeckProfile = qb.build_query_as().fetch_one(&mut *tx).await?;
        let deck_profile: DeckProfile = database_deck.try_into()?;

        tx.commit().await?;
        Ok(deck_profile)
    }

    /// Dynamically builds an `UPDATE` query for the provided fields.
    ///
    /// Supports setting quantity (absolute), board, printing, and/or
    /// MVP star. The database's check constraint on `quantity` stays as the
    /// backstop, surfacing as `QuantityUnderflow` if it ever trips. MVP rules (context/plans/deck_mvps/): mainboard
    /// only, at most 3 per deck (checked in the tx), and moving a card off
    /// the mainboard clears its star in the same UPDATE.
    async fn update_deck_card(
        &self,
        request: &UpdateDeckCard,
    ) -> Result<DeckCard, UpdateDeckCardError> {
        if !request
            .user_id
            .owns_deck(request.deck_id, &self.pool)
            .await?
        {
            return Err(UpdateDeckCardError::Forbidden);
        }
        let mut tx = self.pool.begin().await?;
        if request.mvp == Some(true) {
            // Board rule: the star lands on the mainboard — either the board
            // this request sets, or the row's current board when untouched.
            let effective_board = match &request.board {
                Some(board) => *board,
                None => {
                    let current: String = query_scalar!(
                        "SELECT board FROM deck_cards WHERE deck_id = $1 AND scryfall_data_id = $2",
                        request.deck_id,
                        request.scryfall_data_id
                    )
                    .fetch_optional(&mut *tx)
                    .await?
                    .ok_or(UpdateDeckCardError::NotFound)?;
                    Board::try_from(current.as_str())
                        .map_err(|e| UpdateDeckCardError::Database(anyhow::anyhow!(e)))?
                }
            };
            if effective_board != Board::Deck {
                return Err(UpdateDeckCardError::MvpNotMainboard);
            }
            // Podium cap: at most 3 mainboard MVPs per deck, not counting
            // this row (re-starring an existing MVP stays legal).
            let starred = query_scalar!(
                r#"SELECT count(*) AS "count!" FROM deck_cards
                   WHERE deck_id = $1 AND board = 'deck' AND mvp_at IS NOT NULL
                     AND scryfall_data_id != $2"#,
                request.deck_id,
                request.scryfall_data_id
            )
            .fetch_one(&mut *tx)
            .await?;
            if starred >= 3 {
                return Err(UpdateDeckCardError::MvpCapReached);
            }
        }
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new("UPDATE deck_cards SET ");
        let mut sep = qb.separated(", ");
        if let Some(set_quantity) = &request.set_quantity {
            // Absolute set — idempotent, can't underflow (>= 1 by
            // construction). The legacy delta arm was removed with the PUT
            // route at the end of the PATCH migration.
            sep.push("quantity = ")
                .push_bind_unseparated(**set_quantity);
        }
        if let Some(board) = &request.board {
            sep.push("board = ")
                .push_bind_unseparated(board.display_name().to_string());
        }
        if let Some(new_id) = request.new_scryfall_data_id {
            sep.push("scryfall_data_id = ")
                .push_bind_unseparated(new_id);
        }
        if let Some(mvp) = request.mvp {
            // COALESCE keeps the original vesting clock if a client re-sends
            // true; false clears the star.
            sep.push("mvp_at = CASE WHEN ")
                .push_bind_unseparated(mvp)
                .push_unseparated(" THEN COALESCE(mvp_at, now()) ELSE NULL END");
        } else if request.board.as_ref().is_some_and(|b| *b != Board::Deck) {
            // MVPs are mainboard-only: leaving the mainboard drops the star.
            sep.push("mvp_at = NULL");
        }
        let now = chrono::Utc::now();
        sep.push("updated_at = ").push_bind_unseparated(now);
        qb.push(" WHERE deck_id = ")
            .push_bind(request.deck_id)
            .push(" AND scryfall_data_id = ")
            .push_bind(request.scryfall_data_id)
            .push(" RETURNING deck_id::TEXT, scryfall_data_id::TEXT, oracle_id::TEXT, quantity, board, mvp_at");
        let database_deck_card: DatabaseDeckCard = qb.build_query_as().fetch_one(&mut *tx).await?;
        let deck_card: DeckCard = database_deck_card.try_into()?;
        tx.commit().await?;
        Ok(deck_card)
    }

    // ========
    //  delete
    // ========
    async fn delete_deck(&self, request: &DeleteDeck) -> Result<(), DeleteDeckError> {
        if !request
            .user_id
            .owns_deck(request.deck_id, &self.pool)
            .await?
        {
            return Err(DeleteDeckError::Forbidden);
        }
        let mut tx = self.pool.begin().await?;
        let result = query!("DELETE FROM decks WHERE id = $1", request.deck_id)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DeleteDeckError::NotFound);
        }
        tx.commit().await?;
        Ok(())
    }

    async fn clear_deck_suppressions(
        &self,
        request: &ClearDeckSuppressions,
    ) -> Result<u64, ClearDeckSuppressionsError> {
        if !request
            .user_id
            .owns_deck(request.deck_id, &self.pool)
            .await
            .map_err(|e| ClearDeckSuppressionsError::Database(e.into()))?
        {
            return Err(ClearDeckSuppressionsError::Forbidden);
        }
        let result = query!(
            "DELETE FROM deck_card_suppressions WHERE deck_id = $1",
            request.deck_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ClearDeckSuppressionsError::Database(e.into()))?;
        Ok(result.rows_affected())
    }

    async fn skip_deck_card(&self, request: &SkipDeckCard) -> Result<(), SkipDeckCardError> {
        if !request
            .user_id
            .owns_deck(request.deck_id, &self.pool)
            .await
            .map_err(|e| SkipDeckCardError::Database(e.into()))?
        {
            return Err(SkipDeckCardError::Forbidden);
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| SkipDeckCardError::Database(e.into()))?;
        query!(
            r#"INSERT INTO deck_card_suppressions (deck_id, oracle_id, source)
               VALUES ($1, $2, 'skip')
               ON CONFLICT (deck_id, oracle_id) DO UPDATE SET
                   source = EXCLUDED.source,
                   suppressed_at = now()"#,
            request.deck_id,
            request.oracle_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| SkipDeckCardError::Database(e.into()))?;
        query!(
            r#"DELETE FROM deck_card_suppressions
               WHERE deck_id = $1 AND oracle_id IN (
                   SELECT oracle_id FROM deck_card_suppressions
                   WHERE deck_id = $1
                   ORDER BY suppressed_at DESC
                   OFFSET $2
               )"#,
            request.deck_id,
            MAX_SUPPRESSIONS_PER_DECK,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| SkipDeckCardError::Database(e.into()))?;
        tx.commit()
            .await
            .map_err(|e| SkipDeckCardError::Database(e.into()))?;
        Ok(())
    }

    async fn unskip_deck_card(&self, request: &SkipDeckCard) -> Result<(), SkipDeckCardError> {
        if !request
            .user_id
            .owns_deck(request.deck_id, &self.pool)
            .await
            .map_err(|e| SkipDeckCardError::Database(e.into()))?
        {
            return Err(SkipDeckCardError::Forbidden);
        }
        // Only skip-sourced rows: an undo must not erase a removal suppression.
        query!(
            r#"DELETE FROM deck_card_suppressions
               WHERE deck_id = $1 AND oracle_id = $2 AND source = 'skip'"#,
            request.deck_id,
            request.oracle_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SkipDeckCardError::Database(e.into()))?;
        Ok(())
    }

    async fn get_commander_maybeboard_oracle_ids(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<uuid::Uuid>, CommanderMaybeboardError> {
        let oracle_ids = query_scalar!(
            r#"SELECT oracle_id FROM commander_maybeboard
               WHERE user_id = $1
               ORDER BY created_at DESC"#,
            user_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CommanderMaybeboardError::Database(e.into()))?;
        Ok(oracle_ids)
    }

    async fn add_commander_maybeboard_card(
        &self,
        request: &CommanderMaybeboardCard,
        cap: i64,
    ) -> Result<(), CommanderMaybeboardError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CommanderMaybeboardError::Database(e.into()))?;
        // The user row lock serializes concurrent adds for one user — there
        // is no parent row like a deck to lock, so the count-then-insert
        // TOCTOU closes here instead.
        query!(
            "SELECT 1 as one FROM users WHERE id = $1 FOR UPDATE",
            request.user_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| CommanderMaybeboardError::Database(e.into()))?;
        let known = query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM latest_cards WHERE oracle_id = $1) as "exists!""#,
            request.oracle_id,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| CommanderMaybeboardError::Database(e.into()))?;
        if !known {
            return Err(CommanderMaybeboardError::UnknownCard);
        }
        let inserted = query!(
            r#"INSERT INTO commander_maybeboard (user_id, oracle_id)
               VALUES ($1, $2)
               ON CONFLICT (user_id, oracle_id) DO NOTHING"#,
            request.user_id,
            request.oracle_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| CommanderMaybeboardError::Database(e.into()))?
        .rows_affected();
        // A duplicate add is a no-op success even at cap; only a NEW entry
        // can tip the count over.
        if inserted > 0 {
            let count = query_scalar!(
                r#"SELECT COUNT(*) as "count!" FROM commander_maybeboard WHERE user_id = $1"#,
                request.user_id,
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| CommanderMaybeboardError::Database(e.into()))?;
            if count > cap {
                return Err(CommanderMaybeboardError::LimitReached);
            }
        }
        tx.commit()
            .await
            .map_err(|e| CommanderMaybeboardError::Database(e.into()))?;
        Ok(())
    }

    async fn remove_commander_maybeboard_card(
        &self,
        request: &CommanderMaybeboardCard,
    ) -> Result<(), CommanderMaybeboardError> {
        query!(
            "DELETE FROM commander_maybeboard WHERE user_id = $1 AND oracle_id = $2",
            request.user_id,
            request.oracle_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CommanderMaybeboardError::Database(e.into()))?;
        Ok(())
    }

    async fn clear_commander_maybeboard(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<u64, CommanderMaybeboardError> {
        let result = query!(
            "DELETE FROM commander_maybeboard WHERE user_id = $1",
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CommanderMaybeboardError::Database(e.into()))?;
        Ok(result.rows_affected())
    }

    async fn delete_deck_card(&self, request: &DeleteDeckCard) -> Result<(), DeleteDeckCardError> {
        if !request
            .user_id
            .owns_deck(request.deck_id, &self.pool)
            .await?
        {
            return Err(DeleteDeckCardError::Forbidden);
        }
        let mut tx = self.pool.begin().await?;
        let deleted = query!(
            "DELETE FROM deck_cards WHERE deck_id = $1 AND scryfall_data_id = $2
             RETURNING oracle_id",
            request.deck_id,
            request.scryfall_data_id
        )
        .fetch_optional(&mut *tx)
        .await?;
        let Some(deleted) = deleted else {
            return Err(DeleteDeckCardError::NotFound);
        };
        // A deliberate single-card removal is a "doesn't fit" signal: suppress
        // the card so the deck-aware search stops re-serving it. Bulk deletes
        // (replace-mode imports) intentionally don't do this.
        query!(
            r#"INSERT INTO deck_card_suppressions (deck_id, oracle_id, source)
               VALUES ($1, $2, 'removal')
               ON CONFLICT (deck_id, oracle_id) DO UPDATE SET
                   source = EXCLUDED.source,
                   suppressed_at = now()"#,
            request.deck_id,
            deleted.oracle_id,
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn apply_import_batch(
        &self,
        user_id: uuid::Uuid,
        deck_id: uuid::Uuid,
        mode: zwipe_core::domain::deck::ImportMode,
        batch: &[(uuid::Uuid, uuid::Uuid, i32, String)],
        card_limit: i64,
        email_verified: bool,
    ) -> Result<Vec<DeckCard>, ImportDeckCardsError> {
        let db = |e: sqlx::Error| ImportDeckCardsError::Database(e.into());
        let mut tx = self.pool.begin().await.map_err(db)?;

        // Ownership + row lock in one shot. FOR UPDATE serializes concurrent
        // imports (and card adds) on this deck, closing the limit TOCTOU.
        // Runs before the empty-batch return so a foreign deck is Forbidden
        // regardless of what resolved.
        let owned = sqlx::query_scalar!(
            "SELECT id FROM decks WHERE id = $1 AND user_id = $2 FOR UPDATE",
            deck_id,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(db)?;
        if owned.is_none() {
            return Err(ImportDeckCardsError::Forbidden);
        }

        // An import where nothing resolved never wipes a board.
        if batch.is_empty() {
            return Ok(vec![]);
        }

        // Current all-boards quantity, read under the lock.
        let card_count = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(quantity), 0) FROM deck_cards WHERE deck_id = $1",
            deck_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(db)?
        .unwrap_or(0);

        // Post-import total: in replace mode a deck board present in the import
        // becomes exactly the imported list; in add mode the upsert replaces
        // quantities for re-imported oracle_ids, so subtract that overlap.
        let import_total: i64 = batch.iter().map(|(_, _, qty, _)| i64::from(*qty)).sum();
        let post_import_total = if mode.is_replace() {
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
            let import_oracle_ids: Vec<uuid::Uuid> =
                batch.iter().map(|(_, oid, _, _)| *oid).collect();
            let overlap_qty = sqlx::query_scalar!(
                "SELECT COALESCE(SUM(quantity), 0) FROM deck_cards WHERE deck_id = $1 AND oracle_id = ANY($2) AND board = 'deck'",
                deck_id,
                &import_oracle_ids
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(db)?
            .unwrap_or(0);
            (card_count - overlap_qty) + import_total
        };
        if post_import_total > card_limit {
            return Err(if email_verified {
                ImportDeckCardsError::LimitReached
            } else {
                ImportDeckCardsError::UnverifiedLimitReached
            });
        }

        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, board) ",
        );
        qb.push_values(
            batch,
            |mut b, (scryfall_data_id, oracle_id, quantity, board)| {
                b.push_bind(deck_id)
                    .push_bind(scryfall_data_id)
                    .push_bind(oracle_id)
                    .push_bind(quantity)
                    .push_bind(board);
            },
        );
        qb.push(
            " ON CONFLICT (deck_id, oracle_id) DO UPDATE SET quantity = EXCLUDED.quantity, board = EXCLUDED.board RETURNING deck_id::TEXT, scryfall_data_id::TEXT, oracle_id::TEXT, quantity, board, mvp_at",
        );
        let rows: Vec<DatabaseDeckCard> =
            qb.build_query_as().fetch_all(&mut *tx).await.map_err(db)?;
        let deck_cards: Vec<DeckCard> = rows
            .into_iter()
            .map(|r| {
                r.try_into()
                    .map_err(|e: IntoDeckCardError| ImportDeckCardsError::Database(e.into()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Replace mode: each board present in the import becomes exactly the
        // imported list. Boards absent from the import are untouched. Bulk
        // deletes intentionally don't suppress.
        if mode.is_replace() {
            let boards: std::collections::HashSet<&str> = batch
                .iter()
                .map(|(_, _, _, board)| board.as_str())
                .collect();
            for board in boards {
                let keep: Vec<uuid::Uuid> = batch
                    .iter()
                    .filter(|(_, _, _, b)| b == board)
                    .map(|(_, oid, _, _)| *oid)
                    .collect();
                sqlx::query(
                    "DELETE FROM deck_cards WHERE deck_id = $1 AND board = $2 AND NOT (oracle_id = ANY($3))",
                )
                .bind(deck_id)
                .bind(board)
                .bind(&keep)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            }
        }

        tx.commit().await.map_err(db)?;
        Ok(deck_cards)
    }

    // =======
    //  clone
    // =======
    async fn clone_deck(
        &self,
        source_deck_id: uuid::Uuid,
        new_name: &DeckName,
        owner_id: uuid::Uuid,
    ) -> Result<uuid::Uuid, CloneDeckError> {
        let mut tx = self.pool.begin().await?;

        // 1. Insert the new deck row by SELECT-ing from the source. Name and
        //    owner are caller-supplied; commander / partner / background /
        //    signature_spell / format copy column-to-column, sidestepping
        //    any Rust-side serialization. Returns only the new id.
        //
        //    A unique violation on unique_deck_name_per_user is converted to
        //    CloneDeckError::Duplicate via the From<sqlx::Error> impl.
        let new_deck_id = sqlx::query_scalar!(
            r#"
            INSERT INTO decks (
                name, commander_id, partner_commander_id, background_id,
                signature_spell_id, format, tags, power_level, other_tags, oracle_tags,
                land_target, price_target, price_target_currency, user_id
            )
            SELECT
                $1, commander_id, partner_commander_id, background_id,
                signature_spell_id, format, tags, power_level, other_tags, oracle_tags,
                land_target, price_target, price_target_currency, $2
            FROM decks
            WHERE id = $3
            RETURNING id
            "#,
            new_name.to_string(),
            owner_id,
            source_deck_id,
        )
        .fetch_one(&mut *tx)
        .await?;

        // 2. Bulk copy every deck_cards row from source to new deck in a
        //    single SQL statement. Preserves board / quantity /
        //    scryfall_data_id / oracle_id verbatim. No Rust-side iteration.
        sqlx::query!(
            r#"
            INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, board, mvp_at)
            SELECT $1, scryfall_data_id, oracle_id, quantity, board, mvp_at
            FROM deck_cards
            WHERE deck_id = $2
            "#,
            new_deck_id,
            source_deck_id,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(new_deck_id)
    }

    // =======
    //  share
    // =======
    async fn set_share_token(&self, deck_id: uuid::Uuid) -> Result<uuid::Uuid, ShareDeckError> {
        // Always regenerates: re-sharing rotates the token so old links die.
        let token = query_scalar!(
            r#"UPDATE decks SET share_token = gen_random_uuid(), updated_at = now()
               WHERE id = $1
               RETURNING share_token AS "share_token!""#,
            deck_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ShareDeckError::Database(e.into()))?
        .ok_or(ShareDeckError::NotFound)?;
        Ok(token)
    }

    async fn clear_share_token(&self, deck_id: uuid::Uuid) -> Result<(), ShareDeckError> {
        let result = query!(
            "UPDATE decks SET share_token = NULL, updated_at = now() WHERE id = $1",
            deck_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ShareDeckError::Database(e.into()))?;
        if result.rows_affected() == 0 {
            return Err(ShareDeckError::NotFound);
        }
        Ok(())
    }

    async fn get_deck_id_by_share_token(
        &self,
        token: uuid::Uuid,
    ) -> Result<Option<(uuid::Uuid, uuid::Uuid)>, anyhow::Error> {
        let row = query!(
            "SELECT id, user_id FROM decks WHERE share_token = $1",
            token
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| (r.id, r.user_id)))
    }
}
