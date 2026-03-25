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
                create_deck_profile::{CreateDeckProfile, CreateDeckProfileError},
                deck_profile::DeckProfile,
                delete_deck::{DeleteDeck, DeleteDeckError},
                get_deck_profile::{GetDeckProfile, GetDeckProfileError},
                get_deck_profiles::GetDeckProfiles,
                update_deck_profile::{UpdateDeckProfile, UpdateDeckProfileError},
            },
            deck_card::{
                DeckCard,
                create_deck_card::{CreateDeckCard, CreateDeckCardError},
                delete_deck_card::{DeleteDeckCard, DeleteDeckCardError},
                get_deck_card::GetDeckCardError,
                import_deck_cards::{ImportDeckCards, ImportDeckCardsError},
                update_deck_card::{UpdateDeckCard, UpdateDeckCardError},
            },
        },
        ports::DeckRepository,
    },
    outbound::sqlx::{
        deck::{
            error::{IntoDeckCardError, IntoDeckProfileError},
            helper::OwnsDeck,
            models::{DatabaseDeckCard, DatabaseDeckProfile, UpdateDeckCardGuard},
        },
        postgres::Postgres,
    },
};
use sqlx::{QueryBuilder, query, query_as};

impl Postgres {
    /// Clamps all deck_card quantities to `max` for the given deck.
    async fn truncate_deck_card_quantities(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        deck_id: uuid::Uuid,
        max: i32,
    ) -> Result<(), UpdateDeckProfileError> {
        sqlx::query!(
            "UPDATE deck_cards SET quantity = $1, updated_at = now()
             WHERE deck_id = $2 AND quantity > $1",
            max,
            deck_id,
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| UpdateDeckProfileError::Database(e.into()))?;
        Ok(())
    }
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
        let database_copy_max = request.copy_max.map(|cm| *cm);
        let database_deck_profile = query_as!(
            DatabaseDeckProfile,
            "INSERT INTO decks (name, commander_id, copy_max, user_id) VALUES ($1, $2, $3, $4) RETURNING id, name, commander_id, copy_max, user_id",
            request.name.to_string(),
            request.commander_id,
            database_copy_max,
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
            "INSERT INTO deck_cards (deck_id, scryfall_data_id, quantity) VALUES ($1, $2, $3) RETURNING deck_id, scryfall_data_id, quantity",
            request.deck_id,
            request.scryfall_data_id,
            *request.quantity
        )
        .fetch_one(&mut *tx)
        .await?;
        let deck_card: DeckCard = database_deck_card.try_into()?;
        tx.commit().await?;
        Ok(deck_card)
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
            "SELECT id, name, commander_id, copy_max, user_id FROM decks WHERE id = $1",
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
            "SELECT id, name, commander_id, copy_max, user_id FROM decks WHERE user_id = $1",
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
            "SELECT deck_id, scryfall_data_id, quantity FROM deck_cards WHERE deck_id = $1",
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
        if let Some(copy_max) = &request.copy_max {
            sep.push("copy_max = ")
                .push_bind_unseparated(copy_max.as_ref().map(|cm| **cm));
        }
        let now = chrono::Utc::now().naive_utc();
        sep.push("updated_at = ").push_bind_unseparated(now);

        qb.push(" WHERE id = ")
            .push_bind(request.deck_id)
            .push(" RETURNING id, name, commander_id, copy_max, user_id");
        let database_deck: DatabaseDeckProfile = qb.build_query_as().fetch_one(&mut *tx).await?;
        let deck_profile: DeckProfile = database_deck.try_into()?;

        // If copy_max was tightened, clamp existing card quantities
        if let Some(Some(new_max)) = &request.copy_max {
            Self::truncate_deck_card_quantities(&mut tx, request.deck_id, **new_max).await?;
        }

        tx.commit().await?;
        Ok(deck_profile)
    }

    /// Applies a **relative delta** to card quantity (`quantity + $1`).
    ///
    /// Runs a guard SELECT to enforce copy-max limits before the update.
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
        if let Some(guard) = query_as!(
            UpdateDeckCardGuard,
            "SELECT dc.quantity, d.copy_max, sd.type_line \
             FROM deck_cards dc \
             JOIN decks d ON d.id = dc.deck_id \
             JOIN scryfall_data sd ON sd.id = dc.scryfall_data_id \
             WHERE dc.deck_id = $1 AND dc.scryfall_data_id = $2",
            request.deck_id,
            request.scryfall_data_id
        )
        .fetch_optional(&mut *tx)
        .await?
            && let Some(copy_max) = guard.copy_max
        {
            let is_basic_land = guard
                .type_line
                .as_deref()
                .map(|tl| tl.to_lowercase().contains("basic land"))
                .unwrap_or(false);
            let resulting = guard.quantity + *request.update_quantity;
            if !is_basic_land && resulting > copy_max {
                return Err(UpdateDeckCardError::ExceedsCopyMax);
            }
        }
        let database_deck_card = query_as!(
            DatabaseDeckCard,
            "UPDATE deck_cards SET quantity = quantity + $1 WHERE deck_id = $2 AND scryfall_data_id = $3 RETURNING deck_id, scryfall_data_id, quantity",
            *request.update_quantity,
            request.deck_id,
            request.scryfall_data_id
        )
        .fetch_one(&mut *tx)
        .await?;
        let deck_card = database_deck_card.try_into()?;
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
        let mut tx = self.pool.begin().await
            .map_err(|e| ImportDeckCardsError::Database(e.into()))?;
        let mut qb: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
            "INSERT INTO deck_cards (deck_id, scryfall_data_id, quantity) ",
        );
        qb.push_values(cards, |mut b, (scryfall_data_id, quantity)| {
            b.push_bind(request.deck_id)
                .push_bind(scryfall_data_id)
                .push_bind(quantity);
        });
        qb.push(
            " ON CONFLICT (deck_id, scryfall_data_id) DO UPDATE SET quantity = EXCLUDED.quantity RETURNING deck_id::TEXT, scryfall_data_id::TEXT, quantity",
        );
        let rows: Vec<DatabaseDeckCard> = qb.build_query_as().fetch_all(&mut *tx).await
            .map_err(|e| ImportDeckCardsError::Database(e.into()))?;
        let deck_cards: Vec<DeckCard> = rows
            .into_iter()
            .map(|r| r.try_into().map_err(|e: IntoDeckCardError| ImportDeckCardsError::Database(e.into())))
            .collect::<Result<Vec<_>, _>>()?;
        tx.commit().await
            .map_err(|e| ImportDeckCardsError::Database(e.into()))?;
        Ok(deck_cards)
    }
}
