//! Deck management repository implementation.

pub mod error;
pub mod helper;
pub mod models;

use sqlx::{QueryBuilder, query, query_as, query_scalar};
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::domain::deck::requests::clear_deck_suppressions::ClearDeckSuppressions;
use zwipe_core::domain::deck::requests::commander_maybeboard::CommanderMaybeboardCard;
use zwipe_core::domain::deck::requests::create_deck_card::CreateDeckCard;
use zwipe_core::domain::deck::requests::create_deck_profile::CreateDeckProfile;
use zwipe_core::domain::deck::requests::delete_deck::DeleteDeck;
use zwipe_core::domain::deck::requests::delete_deck_card::DeleteDeckCard;
use zwipe_core::domain::deck::requests::get_deck_profile::GetDeckProfile;
use zwipe_core::domain::deck::requests::get_deck_profiles::GetDeckProfiles;
use zwipe_core::domain::deck::requests::skip_deck_card::SkipDeckCard;
use zwipe_core::domain::deck::requests::update_deck_card::UpdateDeckCard;
use zwipe_core::domain::deck::requests::update_deck_profile::UpdateDeckProfile;
use zwipe_core::domain::deck::tag_profile::PROFILE_EXCLUDED_TAG_PREFIX;
use zwipe_core::domain::deck::{Board, DeckCard, DeckName, DeckOtherTag, DeckTag};
use zwipe_core::http::contracts::suggestions::{
    ACTIVITY_DAYS, ACTIVITY_MIN_SEEN, ACTIVITY_PASSES_SENT, ACTIVITY_TAGS_SENT,
};

use crate::domain::deck::models::deck::activity::{
    ActivityCounts, ActivityDay, PASS_BASELINE_WEIGHT, PASS_MIN_SCORE, PassCount, RoleRates,
    SwipeOutcomes, TagRates,
};
use crate::domain::deck::models::deck::clear_deck_suppressions::ClearDeckSuppressionsError;
use crate::domain::deck::models::deck::clone_deck::CloneDeckError;
use crate::domain::deck::models::deck::commander_maybeboard::CommanderMaybeboardError;
use crate::domain::deck::models::deck::commander_suggestions;
use crate::domain::deck::models::deck::commander_suggestions::{
    CardColorCounts, CommanderDismissal, CommanderSuggestionsError, MainboardCard, ProfiledDeck,
};
use crate::domain::deck::models::deck::create_deck_profile::CreateDeckProfileError;
use crate::domain::deck::models::deck::delete_deck::DeleteDeckError;
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
use crate::domain::deck::models::deck::share_deck::ShareDeckError;
use crate::domain::deck::models::deck::skip_deck_card::SkipDeckCardError;
use crate::domain::deck::models::deck::update_deck_profile::UpdateDeckProfileError;
use crate::domain::deck::models::deck_card::create_deck_card::CreateDeckCardError;
use crate::domain::deck::models::deck_card::delete_deck_card::DeleteDeckCardError;
use crate::domain::deck::models::deck_card::get_deck_card::GetDeckCardError;
use crate::domain::deck::models::deck_card::import_deck_cards::ImportDeckCardsError;
use crate::domain::deck::models::deck_card::update_deck_card::UpdateDeckCardError;
use crate::domain::deck::ports::DeckRepository;
use crate::outbound::sqlx::deck::error::{IntoDeckCardError, IntoDeckProfileError};
use crate::outbound::sqlx::deck::helper::OwnsDeck;
use crate::outbound::sqlx::deck::models::{DatabaseDeckCard, DatabaseDeckProfile};
use crate::outbound::sqlx::postgres::Postgres;

/// Per-deck ceiling on suppression rows, enforced at ingest by evicting the
/// oldest `suppressed_at` beyond it.
pub(crate) const MAX_SUPPRESSIONS_PER_DECK: i64 = 5_000;

/// Serializes deck tags to a JSONB array of `snake_case` strings for storage.
fn deck_tags_to_json(tags: &[DeckTag]) -> serde_json::Value {
    serde_json::Value::Array(
        tags.iter()
            .map(|t| serde_json::Value::String(t.to_string()))
            .collect(),
    )
}

/// Serializes deck other-tags to a JSONB array of `snake_case` strings for storage.
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
    // == create ==
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
        // re-adds the card; the "doesn't fit" signal no longer holds).
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

    // == count ==
    async fn count_decks_by_user(&self, user_id: uuid::Uuid) -> Result<i64, anyhow::Error> {
        let count = sqlx::query_scalar!("SELECT COUNT(*) FROM decks WHERE user_id = $1", user_id)
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);
        Ok(count)
    }

    async fn count_cards_in_deck(&self, deck_id: uuid::Uuid) -> Result<i64, anyhow::Error> {
        // Counts ALL boards (mainboard + maybeboard + sideboard); this feeds the
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

    // == get ==
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
            .map(TryInto::try_into)
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
            .map(TryInto::try_into)
            .collect::<Result<Vec<DeckCard>, IntoDeckCardError>>()?;
        Ok(deck_cards)
    }

    /// Distinct `card_roles` across the deck's mainboard MVPs. Unnests each
    /// MVP's roles array and dedupes, so three MVPs sharing a role contribute
    /// it once: the serve term is a flat lift, not a tally.
    async fn get_mvp_card_roles(
        &self,
        request: &GetDeckProfile,
    ) -> Result<Vec<String>, GetDeckCardError> {
        if !request
            .user_id
            .owns_deck(request.deck_id, &self.pool)
            .await?
        {
            return Err(GetDeckCardError::Forbidden);
        }
        let roles = query_scalar!(
            r#"SELECT DISTINCT role AS "role!"
               FROM deck_cards dc
               JOIN card_profiles cp ON cp.scryfall_data_id = dc.scryfall_data_id
               CROSS JOIN LATERAL jsonb_array_elements_text(cp.card_roles) AS role
               WHERE dc.deck_id = $1 AND dc.board = 'deck' AND dc.mvp_at IS NOT NULL"#,
            request.deck_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(roles)
    }

    // == update ==
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
            .push(r" RETURNING id, name, commander_id, partner_commander_id, background_id, signature_spell_id, format, tags, power_level, other_tags, oracle_tags, land_target, price_target, price_target_currency, share_token, user_id,
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
                                            || ARRAY(SELECT dc2.scryfall_data_id FROM deck_cards dc2 WHERE dc2.deck_id = decks.id AND dc2.board = 'deck'))) as color_identity");
        let database_deck: DatabaseDeckProfile = qb.build_query_as().fetch_one(&mut *tx).await?;
        let deck_profile: DeckProfile = database_deck.try_into()?;

        tx.commit().await?;
        Ok(deck_profile)
    }

    /// Dynamically builds an `UPDATE` query for the provided fields.
    ///
    /// Supports setting quantity (absolute), board, printing, and/or
    /// MVP star. The database's check constraint on `quantity` stays as the
    /// backstop, surfacing as `QuantityUnderflow` if it ever trips. MVP rules (`context/plans/deck_mvps/`): mainboard
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
        if let Some(new_id) = request.new_scryfall_data_id {
            // A printing swap stays within the card: the row keeps its
            // oracle_id, so a printing of another card would break the
            // singleton and commander checks keyed on it.
            let current = query_scalar!(
                "SELECT oracle_id FROM deck_cards WHERE deck_id = $1 AND scryfall_data_id = $2",
                request.deck_id,
                request.scryfall_data_id
            )
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(UpdateDeckCardError::NotFound)?;
            let swapped =
                query_scalar!("SELECT oracle_id FROM scryfall_data WHERE id = $1", new_id)
                    .fetch_optional(&mut *tx)
                    .await?
                    .flatten();
            if swapped != Some(current) {
                return Err(UpdateDeckCardError::PrintingMismatch);
            }
        }
        if request.mvp == Some(true) {
            // Board rule: the star lands on the mainboard; either the board
            // this request sets, or the row's current board when untouched.
            let effective_board = if let Some(board) = &request.board {
                *board
            } else {
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
            // Absolute set: idempotent, can't underflow (>= 1 by
            // construction).
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

    // == delete ==
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
        // The user row lock serializes concurrent adds for one user; there
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

    async fn fetch_profiled_decks(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<ProfiledDeck>, CommanderSuggestionsError> {
        // card_colors is every color across the mainboard's identities, the
        // same union the deck views compute; a deck with a commander takes
        // the commander's identity instead. card_color_counts is the distinct
        // oracle ids across the mainboard and the command zone (commander,
        // partner, background) whose identity holds W, U, B, R and G, then
        // those with an empty identity.
        let rows = query!(
            r#"SELECT s.oracle_id AS "commander_oracle_id?",
                      s.color_identity AS "commander_colors?: Vec<String>",
                      (SELECT array_agg(DISTINCT ci)
                         FROM scryfall_data sci, unnest(sci.color_identity) AS ci
                        WHERE sci.id IN (SELECT dc2.scryfall_data_id FROM deck_cards dc2
                                          WHERE dc2.deck_id = d.id AND dc2.board = 'deck'))
                          AS "card_colors?: Vec<String>",
                      (SELECT ARRAY[
                                  COUNT(DISTINCT m.oracle_id) FILTER (WHERE 'W' = ANY(m.color_identity)),
                                  COUNT(DISTINCT m.oracle_id) FILTER (WHERE 'U' = ANY(m.color_identity)),
                                  COUNT(DISTINCT m.oracle_id) FILTER (WHERE 'B' = ANY(m.color_identity)),
                                  COUNT(DISTINCT m.oracle_id) FILTER (WHERE 'R' = ANY(m.color_identity)),
                                  COUNT(DISTINCT m.oracle_id) FILTER (WHERE 'G' = ANY(m.color_identity)),
                                  COUNT(DISTINCT m.oracle_id) FILTER (WHERE cardinality(m.color_identity) = 0)]
                         FROM (SELECT cc.oracle_id, cs.color_identity
                                 FROM deck_cards cc
                                 JOIN scryfall_data cs ON cs.id = cc.scryfall_data_id
                                WHERE cc.deck_id = d.id AND cc.board = 'deck'
                               UNION ALL
                               SELECT cz.oracle_id, cz.color_identity
                                 FROM scryfall_data cz
                                WHERE cz.id IN (d.commander_id, d.partner_commander_id, d.background_id)
                                  AND cz.oracle_id IS NOT NULL) AS m)
                          AS "card_color_counts!: Vec<i64>",
                      COALESCE(array_agg(dc.oracle_id) FILTER (WHERE dc.board = 'deck'), '{}')
                          AS "mainboard_oracle_ids!: Vec<uuid::Uuid>"
               FROM decks d
               LEFT JOIN scryfall_data s ON s.id = d.commander_id AND s.oracle_id IS NOT NULL
               LEFT JOIN deck_cards dc ON dc.deck_id = d.id
               WHERE d.user_id = $1
               GROUP BY d.id, s.oracle_id, s.color_identity
               HAVING s.oracle_id IS NOT NULL OR bool_or(dc.board = 'deck')
               ORDER BY d.updated_at DESC, d.id"#,
            user_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CommanderSuggestionsError::Database(e.into()))?;
        Ok(rows
            .into_iter()
            .map(|row| {
                let commander_colors = row
                    .commander_oracle_id
                    .map(|_| row.commander_colors.unwrap_or_default());
                ProfiledDeck {
                    commander_oracle_id: row.commander_oracle_id,
                    colors: commander_suggestions::deck_colors(
                        commander_colors,
                        row.card_colors.unwrap_or_default(),
                    ),
                    mainboard_oracle_ids: row.mainboard_oracle_ids,
                    card_colors: CardColorCounts::from_counts(&row.card_color_counts),
                }
            })
            .collect())
    }

    async fn fetch_mainboard_cards(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<MainboardCard>, CommanderSuggestionsError> {
        let rows = query!(
            r#"SELECT d.id AS deck_id, dc.oracle_id, dc.scryfall_data_id, s.name, s.type_line
               FROM decks d
               JOIN deck_cards dc ON dc.deck_id = d.id AND dc.board = 'deck'
               JOIN scryfall_data s ON s.id = dc.scryfall_data_id
               WHERE d.user_id = $1
               ORDER BY d.updated_at DESC, d.id, dc.oracle_id, dc.scryfall_data_id"#,
            user_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CommanderSuggestionsError::Database(e.into()))?;
        Ok(rows
            .into_iter()
            .map(|row| MainboardCard {
                deck_id: row.deck_id,
                oracle_id: row.oracle_id,
                scryfall_data_id: row.scryfall_data_id,
                name: row.name,
                type_line: row.type_line,
            })
            .collect())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one query per activity panel, read in order into one ActivityCounts"
    )]
    async fn fetch_activity_counts(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<ActivityCounts, CommanderSuggestionsError> {
        let db = |e: sqlx::Error| CommanderSuggestionsError::Database(e.into());
        let reach = i32::try_from(ACTIVITY_DAYS).unwrap_or(i32::MAX);
        let mut tx = self.pool.begin().await.map_err(db)?;
        let through = query_scalar!(r#"SELECT (now() AT TIME ZONE 'UTC')::date AS "through!""#)
            .fetch_one(&mut *tx)
            .await
            .map_err(db)?;
        // A day counts every swipe direction on every screen.
        let days = query!(
            r#"SELECT day,
                      (swipes_right + swipes_left + swipes_up + swipes_down)::int8 AS "swipes!",
                      searches::int8 AS "searches!"
               FROM user_daily_activity
               WHERE user_id = $1
                 AND day > $2::date - $3::int4
                 AND swipes_right + swipes_left + swipes_up + swipes_down + searches > 0
               ORDER BY day"#,
            user_id,
            through,
            reach,
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(db)?
        .into_iter()
        .map(|row| ActivityDay {
            day: row.day,
            swipes: saturating_u32(row.swipes),
            searches: saturating_u32(row.searches),
        })
        .collect();
        // The weekly scalars cover every deck; the per-card signals below
        // cover only decks with a commander.
        let outcomes = query!(
            r#"SELECT COALESCE(SUM(added), 0)::int8 AS "added!",
                      COALESCE(SUM(skipped), 0)::int8 AS "skipped!",
                      COALESCE(SUM(maybed), 0)::int8 AS "maybed!",
                      COALESCE(SUM(removed), 0)::int8 AS "removed!"
               FROM user_week_signal
               WHERE user_id = $1"#,
            user_id,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(db)?;
        let roles = query!(
            r#"SELECT r.role AS "role!",
                      SUM(s.added)::int8 AS "added!",
                      SUM(s.skipped)::int8 AS "skipped!",
                      SUM(s.maybed)::int8 AS "maybed!",
                      SUM(s.removed)::int8 AS "removed!"
               FROM user_card_signal s
               JOIN latest_cards lc ON lc.oracle_id = s.card_oracle_id
               JOIN card_profiles cp ON cp.scryfall_data_id = lc.id
               CROSS JOIN LATERAL jsonb_array_elements_text(COALESCE(cp.card_roles, '[]'::jsonb)) AS r(role)
               WHERE s.user_id = $1
               GROUP BY r.role
               HAVING SUM(s.added + s.skipped + s.maybed) > 0"#,
            user_id,
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(db)?
        .into_iter()
        .map(|row| RoleRates {
            role: row.role,
            outcomes: outcomes_of(row.added, row.skipped, row.maybed, row.removed),
        })
        .collect();
        // Each signal row counts once per tag under a role, the tag's role
        // the first by name when it sits under several. Every other
        // player's swipes on the tag come from the pooled table, which
        // holds the user's too, from every deck; theirs come off, floored at
        // zero. The pool is summed once, then read through a lateral
        // aggregate, one row per tag whatever the plan, so the columns'
        // nullability reads the same on every database `sqlx prepare
        // --check` describes it against.
        let excluded = format!("{PROFILE_EXCLUDED_TAG_PREFIX}%");
        let tags = query!(
            r#"WITH tagged AS (
                   SELECT s.commander_oracle_id, s.card_oracle_id, t.tag, MIN(r.role) AS role,
                          MIN(s.added) AS added, MIN(s.skipped) AS skipped,
                          MIN(s.maybed) AS maybed, MIN(s.removed) AS removed
                   FROM user_card_signal s
                   JOIN latest_cards lc ON lc.oracle_id = s.card_oracle_id
                   JOIN card_profiles cp ON cp.scryfall_data_id = lc.id
                   CROSS JOIN LATERAL jsonb_each(COALESCE(cp.oracle_tags_by_role, '{}'::jsonb)) AS r(role, slugs)
                   CROSS JOIN LATERAL jsonb_array_elements_text(r.slugs) AS t(tag)
                   WHERE s.user_id = $1 AND t.tag NOT LIKE $2
                   GROUP BY s.commander_oracle_id, s.card_oracle_id, t.tag
               ),
               mine AS (
                   SELECT tag, MIN(role) AS role,
                          SUM(added) AS added, SUM(skipped) AS skipped,
                          SUM(maybed) AS maybed, SUM(removed) AS removed
                   FROM tagged
                   GROUP BY tag
                   HAVING SUM(added + skipped + maybed)::int8 >= $3::int8
                   ORDER BY SUM(added + skipped + maybed) DESC, tag
                   LIMIT $4
               ),
               pooled AS (
                   SELECT oracle_tag AS tag,
                          SUM(added) AS added, SUM(skipped) AS skipped,
                          SUM(maybed) AS maybed, SUM(removed) AS removed
                   FROM otag_context_signal
                   WHERE oracle_tag IN (SELECT tag FROM mine)
                   GROUP BY oracle_tag
               )
               SELECT m.tag AS "tag!", m.role AS "role!",
                      m.added::int8 AS "added!", m.skipped::int8 AS "skipped!",
                      m.maybed::int8 AS "maybed!", m.removed::int8 AS "removed!",
                      GREATEST(COALESCE(p.added, 0) - m.added, 0)::int8 AS "all_added!",
                      GREATEST(COALESCE(p.skipped, 0) - m.skipped, 0)::int8 AS "all_skipped!",
                      GREATEST(COALESCE(p.maybed, 0) - m.maybed, 0)::int8 AS "all_maybed!",
                      GREATEST(COALESCE(p.removed, 0) - m.removed, 0)::int8 AS "all_removed!"
               FROM mine m
               CROSS JOIN LATERAL (
                   SELECT SUM(o.added) AS added, SUM(o.skipped) AS skipped,
                          SUM(o.maybed) AS maybed, SUM(o.removed) AS removed
                   FROM pooled o
                   WHERE o.tag = m.tag
               ) p"#,
            user_id,
            excluded,
            i64::try_from(ACTIVITY_MIN_SEEN).unwrap_or(i64::MAX),
            i64::try_from(ACTIVITY_TAGS_SENT).unwrap_or(i64::MAX),
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(db)?
        .into_iter()
        .map(|row| TagRates {
            tag: row.tag,
            role: row.role,
            outcomes: outcomes_of(row.added, row.skipped, row.maybed, row.removed),
            everyone: outcomes_of(row.all_added, row.all_skipped, row.all_maybed, row.all_removed),
        })
        .collect();
        // Skipped more than once and never added or saved, under any
        // commander, ranked by lift against other players. The pooled table
        // holds every player's swipes on each card, the user's included, so
        // theirs come off to leave everyone else's. Every candidate's own
        // swipes are skips, so the user's skip rate is 1 and the lift is the
        // inverse of the others' rate, smoothed toward the overall rate by
        // PASS_BASELINE_WEIGHT dealt cards. A card needs PASS_MIN_SCORE, so
        // one the others skip about as often takes many skips to show. The
        // pool is read through a lateral aggregate, as the tags' is.
        let passes = query!(
            r#"WITH mine AS (
                   SELECT card_oracle_id,
                          SUM(skipped)::float8 AS skipped,
                          array_agg(commander_oracle_id) FILTER (WHERE skipped > 0) AS commander_ids
                   FROM user_card_signal
                   WHERE user_id = $1
                   GROUP BY card_oracle_id
                   HAVING SUM(added) = 0 AND SUM(maybed) = 0 AND SUM(skipped) > 1
               ),
               overall AS (
                   SELECT COALESCE(SUM(skipped)::float8
                                   / NULLIF(SUM(added + skipped + maybed), 0)::float8, 0.5) AS rate
                   FROM commander_card_signal
               ),
               pooled AS (
                   SELECT c.card_oracle_id,
                          SUM(c.skipped)::float8 AS skipped,
                          SUM(c.added + c.skipped + c.maybed)::float8 AS seen
                   FROM commander_card_signal c
                   WHERE c.card_oracle_id IN (SELECT card_oracle_id FROM mine)
                   GROUP BY c.card_oracle_id
               ),
               scored AS (
                   SELECT m.card_oracle_id, m.skipped, m.commander_ids,
                          m.skipped * ln(
                              (GREATEST(COALESCE(p.seen, 0) - m.skipped, 0) + $3::float8)
                              / NULLIF(GREATEST(COALESCE(p.skipped, 0) - m.skipped, 0)
                                       + $3::float8 * o.rate, 0)
                          ) AS score
                   FROM mine m
                   CROSS JOIN overall o
                   CROSS JOIN LATERAL (
                       SELECT SUM(q.skipped) AS skipped, SUM(q.seen) AS seen
                       FROM pooled q
                       WHERE q.card_oracle_id = m.card_oracle_id
                   ) p
               )
               SELECT s.card_oracle_id AS "card_oracle_id!",
                      s.skipped::int8 AS "skipped!",
                      COALESCE((SELECT array_agg(lc.name ORDER BY lc.name)
                                FROM latest_cards lc
                                WHERE lc.oracle_id = ANY(s.commander_ids)), '{}')
                          AS "commanders!: Vec<String>"
               FROM scored s
               WHERE s.score >= $4
               ORDER BY s.score DESC, s.skipped DESC, s.card_oracle_id
               LIMIT $2"#,
            user_id,
            i64::try_from(ACTIVITY_PASSES_SENT).unwrap_or(i64::MAX),
            PASS_BASELINE_WEIGHT,
            PASS_MIN_SCORE,
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(db)?
        .into_iter()
        .map(|row| PassCount {
            oracle_id: row.card_oracle_id,
            skipped: saturating_u32(row.skipped),
            commanders: row.commanders,
        })
        .collect();
        tx.commit().await.map_err(db)?;
        Ok(ActivityCounts {
            through,
            days,
            outcomes: outcomes_of(
                outcomes.added,
                outcomes.skipped,
                outcomes.maybed,
                outcomes.removed,
            ),
            roles,
            tags,
            passes,
        })
    }

    async fn dismiss_commander(
        &self,
        request: &CommanderDismissal,
    ) -> Result<(), CommanderSuggestionsError> {
        query!(
            r#"INSERT INTO user_commander_dismissals (user_id, oracle_id)
               VALUES ($1, $2)
               ON CONFLICT (user_id, oracle_id) DO UPDATE SET dismissed_at = now()"#,
            request.user_id,
            request.oracle_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CommanderSuggestionsError::Database(e.into()))?;
        Ok(())
    }

    async fn undismiss_commander(
        &self,
        request: &CommanderDismissal,
    ) -> Result<(), CommanderSuggestionsError> {
        query!(
            "DELETE FROM user_commander_dismissals WHERE user_id = $1 AND oracle_id = $2",
            request.user_id,
            request.oracle_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CommanderSuggestionsError::Database(e.into()))?;
        Ok(())
    }

    async fn clear_commander_dismissals(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<u64, CommanderSuggestionsError> {
        let result = query!(
            "DELETE FROM user_commander_dismissals WHERE user_id = $1",
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CommanderSuggestionsError::Database(e.into()))?;
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

    #[expect(
        clippy::too_many_lines,
        reason = "one transaction: the ownership lock, the card limit, the upserts, then the replace-mode deletes"
    )]
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
                        // `mvp_at` must follow the board: an import that moves a starred
            // card off the mainboard has to drop the star, or it strands an MVP
            // the podium cap can't see (the cap counts mainboard stars only).
            // A CHECK constraint backs this up.
            " ON CONFLICT (deck_id, oracle_id) DO UPDATE SET quantity = EXCLUDED.quantity, board = EXCLUDED.board, mvp_at = CASE WHEN EXCLUDED.board = 'deck' THEN deck_cards.mvp_at ELSE NULL END RETURNING deck_id::TEXT, scryfall_data_id::TEXT, oracle_id::TEXT, quantity, board, mvp_at",
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

    // == clone ==
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

    // == share ==
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

/// Four database sums as outcomes, each 0 below zero.
fn outcomes_of(added: i64, skipped: i64, maybed: i64, removed: i64) -> SwipeOutcomes {
    let count = |n: i64| u64::try_from(n).unwrap_or(0);
    SwipeOutcomes {
        added: count(added),
        skipped: count(skipped),
        maybed: count(maybed),
        removed: count(removed),
    }
}

/// A count from the database as a `u32`: 0 below zero and `u32::MAX` past it.
fn saturating_u32(n: i64) -> u32 {
    u32::try_from(n.max(0)).unwrap_or(u32::MAX)
}
