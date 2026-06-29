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
                clone_deck::CloneDeckError,
                create_deck_profile::CreateDeckProfileError,
                delete_deck::DeleteDeckError,
                get_deck_profile::GetDeckProfileError,
                update_deck_profile::UpdateDeckProfileError,
            },
            deck_card::{
                create_deck_card::CreateDeckCardError,
                delete_deck_card::DeleteDeckCardError,
                get_deck_card::GetDeckCardError,
                import_deck_cards::ImportDeckCardsError,
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
use zwipe_core::domain::deck::{
    DeckCard, DeckName, DeckTag,
    deck_profile::DeckProfile,
    requests::{
        create_deck_card::CreateDeckCard,
        create_deck_profile::CreateDeckProfile,
        delete_deck::DeleteDeck,
        delete_deck_card::DeleteDeckCard,
        get_deck_profile::GetDeckProfile,
        get_deck_profiles::GetDeckProfiles,
        import_deck_cards::ImportDeckCards,
        update_deck_card::UpdateDeckCard,
        update_deck_profile::UpdateDeckProfile,
    },
};
use sqlx::{QueryBuilder, query, query_as};

/// Serializes deck tags to a JSONB array of snake_case strings for storage.
fn deck_tags_to_json(tags: &[DeckTag]) -> serde_json::Value {
    serde_json::Value::Array(
        tags.iter()
            .map(|t| serde_json::Value::String(t.to_string()))
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
        let database_deck_profile = query_as!(
            DatabaseDeckProfile,
            r#"INSERT INTO decks (name, commander_id, partner_commander_id, background_id, signature_spell_id, format, tags, land_target, price_target, price_target_currency, user_id)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
               RETURNING id, name, commander_id, partner_commander_id, background_id, signature_spell_id, format, tags as "tags?", land_target, price_target, price_target_currency, user_id,
                         0::bigint as "card_count",
                         (SELECT sd.name FROM scryfall_data sd WHERE sd.id = commander_id) as "commander_name?",
                         (SELECT sd.name FROM scryfall_data sd WHERE sd.id = partner_commander_id) as "partner_commander_name?",
                         (SELECT sd.name FROM scryfall_data sd WHERE sd.id = background_id) as "background_name?",
                         (SELECT sd.name FROM scryfall_data sd WHERE sd.id = signature_spell_id) as "signature_spell_name?""#,
            request.name.to_string(),
            request.commander_id,
            request.partner_commander_id,
            request.background_id,
            request.signature_spell_id,
            request.format.map(|f| f.to_legality_key().to_string()) as Option<String>,
            tags_json,
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
    ) -> Result<DeckCard, CreateDeckCardError> {
        if !request
            .user_id
            .owns_deck(request.deck_id, &self.pool)
            .await?
        {
            return Err(CreateDeckCardError::Forbidden);
        }
        let mut tx = self.pool.begin().await?;
        let database_deck_card = query_as!(
            DatabaseDeckCard,
            "INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, board) VALUES ($1, $2, $3, $4, $5) RETURNING deck_id, scryfall_data_id, oracle_id, quantity, board",
            request.deck_id,
            request.scryfall_data_id,
            request.oracle_id,
            *request.quantity,
            request.board.display_name()
        )
        .fetch_one(&mut *tx)
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
        let count = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(quantity), 0) FROM deck_cards WHERE deck_id = $1 AND board = 'deck'",
            deck_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);
        Ok(count)
    }

    async fn delete_deck_cards_not_in(
        &self,
        deck_id: uuid::Uuid,
        board: &str,
        keep_oracle_ids: &[uuid::Uuid],
    ) -> Result<(), anyhow::Error> {
        sqlx::query(
            "DELETE FROM deck_cards WHERE deck_id = $1 AND board = $2 AND NOT (oracle_id = ANY($3))",
        )
        .bind(deck_id)
        .bind(board)
        .bind(keep_oracle_ids)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn sum_quantities_for_oracle_ids(
        &self,
        deck_id: uuid::Uuid,
        oracle_ids: &[uuid::Uuid],
    ) -> Result<i64, anyhow::Error> {
        let sum = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(quantity), 0) FROM deck_cards WHERE deck_id = $1 AND oracle_id = ANY($2) AND board = 'deck'",
            deck_id,
            oracle_ids
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);
        Ok(sum)
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
                      d.format, d.tags as "tags?", d.land_target, d.price_target, d.price_target_currency, d.user_id,
                      COALESCE(SUM(dc.quantity) FILTER (WHERE dc.board = 'deck'), 0) as "card_count",
                      sd.name as "commander_name?",
                      (SELECT s2.name FROM scryfall_data s2 WHERE s2.id = d.partner_commander_id) as "partner_commander_name?",
                      (SELECT s3.name FROM scryfall_data s3 WHERE s3.id = d.background_id) as "background_name?",
                      (SELECT s4.name FROM scryfall_data s4 WHERE s4.id = d.signature_spell_id) as "signature_spell_name?"
               FROM decks d
               LEFT JOIN deck_cards dc ON d.id = dc.deck_id
               LEFT JOIN scryfall_data sd ON d.commander_id = sd.id
               WHERE d.id = $1
               GROUP BY d.id, d.name, d.commander_id, d.partner_commander_id, d.background_id, d.signature_spell_id,
                        d.format, d.land_target, d.price_target, d.price_target_currency, d.user_id, sd.name"#,
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
                      d.format, d.tags as "tags?", d.land_target, d.price_target, d.price_target_currency, d.user_id,
                      COALESCE(SUM(dc.quantity) FILTER (WHERE dc.board = 'deck'), 0) as "card_count",
                      sd.name as "commander_name?",
                      (SELECT s2.name FROM scryfall_data s2 WHERE s2.id = d.partner_commander_id) as "partner_commander_name?",
                      (SELECT s3.name FROM scryfall_data s3 WHERE s3.id = d.background_id) as "background_name?",
                      (SELECT s4.name FROM scryfall_data s4 WHERE s4.id = d.signature_spell_id) as "signature_spell_name?"
               FROM decks d
               LEFT JOIN deck_cards dc ON d.id = dc.deck_id
               LEFT JOIN scryfall_data sd ON d.commander_id = sd.id
               WHERE d.user_id = $1
               GROUP BY d.id, d.name, d.commander_id, d.partner_commander_id, d.background_id, d.signature_spell_id,
                        d.format, d.land_target, d.price_target, d.price_target_currency, d.user_id, sd.name"#,
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
            "SELECT deck_id, scryfall_data_id, oracle_id, quantity, board FROM deck_cards WHERE deck_id = $1",
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
        if let Some(land_target) = &request.land_target {
            sep.push("land_target = ").push_bind_unseparated(*land_target);
        }
        if let Some(price_target) = &request.price_target {
            sep.push("price_target = ").push_bind_unseparated(*price_target);
        }
        if let Some(price_target_currency) = &request.price_target_currency {
            sep.push("price_target_currency = ")
                .push_bind_unseparated(price_target_currency.map(|c| c.json_key().to_string()));
        }
        let now = chrono::Utc::now();
        sep.push("updated_at = ").push_bind_unseparated(now);

        qb.push(" WHERE id = ")
            .push_bind(request.deck_id)
            .push(r#" RETURNING id, name, commander_id, partner_commander_id, background_id, signature_spell_id, format, tags, land_target, price_target, price_target_currency, user_id,
                       (SELECT COALESCE(SUM(dc.quantity) FILTER (WHERE dc.board = 'deck'), 0) FROM deck_cards dc WHERE dc.deck_id = decks.id) as card_count,
                       (SELECT sd.name FROM scryfall_data sd WHERE sd.id = decks.commander_id) as commander_name,
                       (SELECT sd.name FROM scryfall_data sd WHERE sd.id = decks.partner_commander_id) as partner_commander_name,
                       (SELECT sd.name FROM scryfall_data sd WHERE sd.id = decks.background_id) as background_name,
                       (SELECT sd.name FROM scryfall_data sd WHERE sd.id = decks.signature_spell_id) as signature_spell_name"#);
        let database_deck: DatabaseDeckProfile = qb.build_query_as().fetch_one(&mut *tx).await?;
        let deck_profile: DeckProfile = database_deck.try_into()?;

        tx.commit().await?;
        Ok(deck_profile)
    }

    /// Dynamically builds an `UPDATE` query for the provided fields.
    ///
    /// Supports updating quantity (relative delta), board, and/or printing.
    /// The database enforces a check constraint on `quantity`, so negative deltas
    /// that would result in an invalid quantity surface as `QuantityUnderflow`.
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
        let mut qb: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("UPDATE deck_cards SET ");
        let mut sep = qb.separated(", ");
        if let Some(update_quantity) = &request.update_quantity {
            sep.push("quantity = quantity + ")
                .push_bind_unseparated(**update_quantity);
        }
        if let Some(board) = &request.board {
            sep.push("board = ").push_bind_unseparated(board.display_name().to_string());
        }
        if let Some(new_id) = request.new_scryfall_data_id {
            sep.push("scryfall_data_id = ").push_bind_unseparated(new_id);
        }
        let now = chrono::Utc::now();
        sep.push("updated_at = ").push_bind_unseparated(now);
        qb.push(" WHERE deck_id = ")
            .push_bind(request.deck_id)
            .push(" AND scryfall_data_id = ")
            .push_bind(request.scryfall_data_id)
            .push(" RETURNING deck_id::TEXT, scryfall_data_id::TEXT, oracle_id::TEXT, quantity, board");
        let database_deck_card: DatabaseDeckCard =
            qb.build_query_as().fetch_one(&mut *tx).await?;
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

    async fn delete_deck_card(&self, request: &DeleteDeckCard) -> Result<(), DeleteDeckCardError> {
        if !request
            .user_id
            .owns_deck(request.deck_id, &self.pool)
            .await?
        {
            return Err(DeleteDeckCardError::Forbidden);
        }
        let mut tx = self.pool.begin().await?;
        let result = query!(
            "DELETE FROM deck_cards WHERE deck_id = $1 AND scryfall_data_id = $2",
            request.deck_id,
            request.scryfall_data_id
        )
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() == 0 {
            return Err(DeleteDeckCardError::NotFound);
        }
        tx.commit().await?;
        Ok(())
    }

    async fn bulk_create_deck_cards(
        &self,
        request: &ImportDeckCards,
        cards: &[(uuid::Uuid, uuid::Uuid, i32, String)],
    ) -> Result<Vec<DeckCard>, ImportDeckCardsError> {
        if !request
            .user_id
            .owns_deck(request.deck_id, &self.pool)
            .await
            .map_err(|e| ImportDeckCardsError::Database(e.into()))?
        {
            return Err(ImportDeckCardsError::Forbidden);
        }
        if cards.is_empty() {
            return Ok(vec![]);
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ImportDeckCardsError::Database(e.into()))?;
        let mut qb: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, board) ");
        qb.push_values(cards, |mut b, (scryfall_data_id, oracle_id, quantity, board)| {
            b.push_bind(request.deck_id)
                .push_bind(scryfall_data_id)
                .push_bind(oracle_id)
                .push_bind(quantity)
                .push_bind(board);
        });
        qb.push(
            " ON CONFLICT (deck_id, oracle_id) DO UPDATE SET quantity = EXCLUDED.quantity, board = EXCLUDED.board RETURNING deck_id::TEXT, scryfall_data_id::TEXT, oracle_id::TEXT, quantity, board",
        );
        let rows: Vec<DatabaseDeckCard> = qb
            .build_query_as()
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| ImportDeckCardsError::Database(e.into()))?;
        let deck_cards: Vec<DeckCard> = rows
            .into_iter()
            .map(|r| {
                r.try_into()
                    .map_err(|e: IntoDeckCardError| ImportDeckCardsError::Database(e.into()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        tx.commit()
            .await
            .map_err(|e| ImportDeckCardsError::Database(e.into()))?;
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
                signature_spell_id, format, tags, land_target, price_target,
                price_target_currency, user_id
            )
            SELECT
                $1, commander_id, partner_commander_id, background_id,
                signature_spell_id, format, tags, land_target, price_target,
                price_target_currency, $2
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
            INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, board)
            SELECT $1, scryfall_data_id, oracle_id, quantity, board
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
}
