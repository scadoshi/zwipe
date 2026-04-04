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
    DeckCard,
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

impl DeckRepository for Postgres {
    // ========
    //  create
    // ========
    async fn create_deck_profile(
        &self,
        request: &CreateDeckProfile,
    ) -> Result<DeckProfile, CreateDeckProfileError> {
        let mut tx = self.pool.begin().await?;
        let database_deck_profile = query_as!(
            DatabaseDeckProfile,
            r#"INSERT INTO decks (name, commander_id, format, user_id)
               VALUES ($1, $2, $3, $4)
               RETURNING id, name, commander_id, format, user_id,
                         0::bigint as "card_count",
                         (SELECT sd.name FROM scryfall_data sd WHERE sd.id = commander_id) as "commander_name?""#,
            request.name.to_string(),
            request.commander_id,
            request.format.map(|f| f.to_legality_key().to_string()) as Option<String>,
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
            "INSERT INTO deck_cards (deck_id, scryfall_data_id, quantity, maybeboard) VALUES ($1, $2, $3, $4) RETURNING deck_id, scryfall_data_id, quantity, maybeboard",
            request.deck_id,
            request.scryfall_data_id,
            *request.quantity,
            request.maybeboard
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
            "SELECT COALESCE(SUM(quantity), 0) FROM deck_cards WHERE deck_id = $1 AND maybeboard = false",
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
            r#"SELECT d.id, d.name, d.commander_id, d.format, d.user_id,
                      COALESCE(SUM(dc.quantity) FILTER (WHERE dc.maybeboard = false), 0) as "card_count",
                      sd.name as "commander_name?"
               FROM decks d
               LEFT JOIN deck_cards dc ON d.id = dc.deck_id
               LEFT JOIN scryfall_data sd ON d.commander_id = sd.id
               WHERE d.id = $1
               GROUP BY d.id, d.name, d.commander_id, d.format, d.user_id, sd.name"#,
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
            r#"SELECT d.id, d.name, d.commander_id, d.format, d.user_id,
                      COALESCE(SUM(dc.quantity) FILTER (WHERE dc.maybeboard = false), 0) as "card_count",
                      sd.name as "commander_name?"
               FROM decks d
               LEFT JOIN deck_cards dc ON d.id = dc.deck_id
               LEFT JOIN scryfall_data sd ON d.commander_id = sd.id
               WHERE d.user_id = $1
               GROUP BY d.id, d.name, d.commander_id, d.format, d.user_id, sd.name"#,
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
            "SELECT deck_id, scryfall_data_id, quantity, maybeboard FROM deck_cards WHERE deck_id = $1",
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
        if let Some(format) = &request.format {
            sep.push("format = ")
                .push_bind_unseparated(format.map(|f| f.to_legality_key().to_string()));
        }
        let now = chrono::Utc::now().naive_utc();
        sep.push("updated_at = ").push_bind_unseparated(now);

        qb.push(" WHERE id = ")
            .push_bind(request.deck_id)
            .push(r#" RETURNING id, name, commander_id, format, user_id,
                       (SELECT COALESCE(SUM(dc.quantity) FILTER (WHERE dc.maybeboard = false), 0) FROM deck_cards dc WHERE dc.deck_id = decks.id) as card_count,
                       (SELECT sd.name FROM scryfall_data sd WHERE sd.id = decks.commander_id) as commander_name"#);
        let database_deck: DatabaseDeckProfile = qb.build_query_as().fetch_one(&mut *tx).await?;
        let deck_profile: DeckProfile = database_deck.try_into()?;

        tx.commit().await?;
        Ok(deck_profile)
    }

    /// Dynamically builds an `UPDATE` query for the provided fields.
    ///
    /// Supports updating quantity (relative delta) and/or maybeboard status.
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
        if let Some(maybeboard) = request.maybeboard {
            sep.push("maybeboard = ").push_bind_unseparated(maybeboard);
        }
        let now = chrono::Utc::now().naive_utc();
        sep.push("updated_at = ").push_bind_unseparated(now);
        qb.push(" WHERE deck_id = ")
            .push_bind(request.deck_id)
            .push(" AND scryfall_data_id = ")
            .push_bind(request.scryfall_data_id)
            .push(" RETURNING deck_id::TEXT, scryfall_data_id::TEXT, quantity, maybeboard");
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
        cards: &[(uuid::Uuid, i32)],
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
            QueryBuilder::new("INSERT INTO deck_cards (deck_id, scryfall_data_id, quantity, maybeboard) ");
        qb.push_values(cards, |mut b, (scryfall_data_id, quantity)| {
            b.push_bind(request.deck_id)
                .push_bind(scryfall_data_id)
                .push_bind(quantity)
                .push_bind(false); // imports always go to mainboard
        });
        qb.push(
            " ON CONFLICT (deck_id, scryfall_data_id) DO UPDATE SET quantity = EXCLUDED.quantity, maybeboard = EXCLUDED.maybeboard RETURNING deck_id::TEXT, scryfall_data_id::TEXT, quantity, maybeboard",
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
}
