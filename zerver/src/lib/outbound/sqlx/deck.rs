use sqlx::{query, query_as, QueryBuilder};
use sqlx_macros::FromRow;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    domain::deck::{
        models::{
            deck::{
                CreateDeckProfile, CreateDeckProfileError, DeckName, DeckProfile, DeleteDeck,
                DeleteDeckError, GetDeck, GetDeckProfileError, InvalidDeckname, UpdateDeckProfile,
                UpdateDeckProfileError,
            },
            deck_card::{
                CreateDeckCard, CreateDeckCardError, DeckCard, DeleteDeckCard, DeleteDeckCardError, GetDeckCardError, InvalidQuantity, Quantity, UpdateDeckCard, UpdateDeckCardError
            },
        },
        ports::DeckRepository,
    },
    outbound::sqlx::postgres::{IsConstraintViolation, Postgres},
};

// ========
//  errors
// ========

#[derive(Debug, Error)]
pub enum ToDeckProfileError {
    #[error("invalid deck id: {0}")]
    Id(uuid::Error),
    #[error(transparent)]
    DeckName(InvalidDeckname),
    #[error("invalid user id: {0}")]
    UserId(uuid::Error),
}

impl From<ToDeckProfileError> for CreateDeckProfileError {
    fn from(value: ToDeckProfileError) -> Self {
        Self::DeckFromDb(value.into())
    }
}

impl From<ToDeckProfileError> for UpdateDeckProfileError {
    fn from(value: ToDeckProfileError) -> Self {
        Self::DeckFromDb(value.into())
    }
}

impl From<ToDeckProfileError> for GetDeckProfileError {
    fn from(value: ToDeckProfileError) -> Self {
        GetDeckProfileError::DeckProfileFromDb(value.into())
    }
}

impl From<sqlx::Error> for CreateDeckProfileError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            e if e.is_unique_constraint_violation() => Self::Duplicate,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for GetDeckProfileError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for UpdateDeckProfileError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            e if e.is_unique_constraint_violation() => Self::Duplicate,
            sqlx::Error::RowNotFound => Self::NotFound,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for DeleteDeckError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

#[derive(Debug, Error)]
pub enum ToDeckCardError {
    #[error(transparent)]
    InvalidId(uuid::Error),
    #[error(transparent)]
    InvalidDeckId(uuid::Error),
    #[error(transparent)]
    InvalidCardId(uuid::Error),
    #[error(transparent)]
    InvalidQuantity(InvalidQuantity),
}

impl From<InvalidQuantity> for ToDeckCardError {
    fn from(value: InvalidQuantity) -> Self {
        Self::InvalidQuantity(value)
    }
}

impl From<ToDeckCardError> for CreateDeckCardError {
    fn from(value: ToDeckCardError) -> Self {
        Self::DeckCardFromDb(value.into())
    }
}

impl From<ToDeckCardError> for GetDeckCardError {
    fn from(value: ToDeckCardError) -> Self {
        Self::DeckCardFromDb(value.into())
    }
}

impl From<ToDeckCardError> for UpdateDeckCardError {
    fn from(value: ToDeckCardError) -> Self {
        Self::DeckCardFromDb(value.into())
    }
}

impl From<sqlx::Error> for CreateDeckCardError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            e if e.is_unique_constraint_violation() => Self::Duplicate,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for GetDeckCardError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for UpdateDeckCardError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            e if e.is_check_constraint_violation() => Self::InvalidResultingQuantity,
            sqlx::Error::RowNotFound => Self::NotFound,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for DeleteDeckCardError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

// ===========
//   db types
// ===========

/// raw database deck record
/// (unvalidated data from `PostgreSQL`)
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseDeckProfile {
    pub id: String,
    pub name: String,
    pub user_id: String,
}

/// converts database deck to validated domain deck
impl TryFrom<DatabaseDeckProfile> for DeckProfile {
    type Error = ToDeckProfileError;

    fn try_from(value: DatabaseDeckProfile) -> Result<Self, Self::Error> {
        let id = Uuid::try_parse(&value.id).map_err(|e| ToDeckProfileError::Id(e.into()))?;
        let name =
            DeckName::new(&value.name).map_err(|e| ToDeckProfileError::DeckName(e.into()))?;
        let user_id =
            Uuid::try_parse(&value.user_id).map_err(|e| ToDeckProfileError::UserId(e.into()))?;
        Ok(Self { id, name, user_id })
    }
}

/// raw database deck card record
/// (unvalidated data from `PostgreSQL`)
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseDeckCard {
    pub deck_id: String,
    pub card_profile_id: String,
    pub quantity: i32,
}

/// converts database deck card to validated domain deck card
impl TryFrom<DatabaseDeckCard> for DeckCard {
    type Error = ToDeckCardError;

    fn try_from(value: DatabaseDeckCard) -> Result<Self, Self::Error> {
        let deck_id = Uuid::try_parse(&value.deck_id)
            .map_err(|e| ToDeckCardError::InvalidDeckId(e.into()))?;
        let card_profile_id = Uuid::try_parse(&value.card_profile_id)
            .map_err(|e| ToDeckCardError::InvalidCardId(e.into()))?;
        let quantity = Quantity::new(value.quantity)?;
        Ok(Self {
            deck_id,
            card_profile_id,
            quantity,
        })
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

        let database_deck_profile = query_as!(
            DatabaseDeckProfile,
            "INSERT INTO decks (name, user_id) VALUES ($1, $2) RETURNING id, name, user_id",
            request.name.to_string(),
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
        let mut tx = self.pool.begin().await?;

        let database_deck_card = query_as!(
            DatabaseDeckCard,
            "INSERT INTO deck_cards (deck_id, card_profile_id, quantity) VALUES ($1, $2, $3) RETURNING deck_id, card_profile_id, quantity",
            request.deck_id,
            request.card_profile_id,
            request.quantity.quantity()
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
        request: &GetDeck,
    ) -> Result<DeckProfile, GetDeckProfileError> {
        let database_deck_profile = query_as!(
            DatabaseDeckProfile,
            "SELECT id, name, user_id FROM decks WHERE id = $1",
            request.deck_id
        )
        .fetch_one(&self.pool)
        .await?;

        let deck_profile: DeckProfile = database_deck_profile.try_into()?;

        Ok(deck_profile)
    }

    async fn get_deck_cards(
        &self,
        request: &GetDeck,
    ) -> Result<Vec<DeckCard>, GetDeckCardError> {
        let database_deck_cards = query_as!(
            DatabaseDeckCard,
            "SELECT deck_id, card_profile_id, quantity FROM deck_cards WHERE deck_id = $1",
            request.deck_id
        )
        .fetch_all(&self.pool)
        .await?;

        let deck_cards: Vec<DeckCard> = database_deck_cards
            .into_iter()
            .map(|x| x.try_into())
            .collect::<Result<Vec<DeckCard>, ToDeckCardError>>()?;

        Ok(deck_cards)
    }
    // ========
    //  update
    // ========
    async fn update_deck_profile(
        &self,
        request: &UpdateDeckProfile,
    ) -> Result<DeckProfile, UpdateDeckProfileError> {
        let mut tx = self.pool.begin().await?;

        let mut qb = QueryBuilder::new("UPDATE decks SET ");
        let mut sep = qb.separated(", ");

        if let Some(name) = &request.name {
            sep.push("name = ").push_bind_unseparated(name.to_string());
        }

        let now = chrono::Utc::now().naive_utc();
        sep.push("updated_at = ").push_bind_unseparated(now);

        qb.push(" WHERE id = ")
            .push_bind(request.deck_id)
            .push(" RETURNING id, name, user_id");

        let database_deck: DatabaseDeckProfile = qb.build_query_as().fetch_one(&mut *tx).await?;

        let deck_profile: DeckProfile = database_deck.try_into()?;

        tx.commit().await?;

        Ok(deck_profile)
    }

    async fn update_deck_card(
        &self,
        request: &UpdateDeckCard,
    ) -> Result<DeckCard, UpdateDeckCardError> {
        let mut tx = self.pool.begin().await?;

        let database_deck_card = query_as!(
            DatabaseDeckCard,
            "UPDATE deck_cards SET quantity = quantity + $1 WHERE deck_id = $2 AND card_profile_id = $3 RETURNING deck_id, card_profile_id, quantity",
            request.update_quantity.value(),
            request.deck_id, 
            request.card_profile_id
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
        let mut tx = self.pool.begin().await?;

        let result = query!("DELETE FROM decks WHERE id = $1", request.id())
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DeleteDeckError::NotFound);
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_deck_card(&self, request: &DeleteDeckCard) -> Result<(), DeleteDeckCardError> {
        let mut tx = self.pool.begin().await?;

        let result = query!("DELETE FROM deck_cards WHERE deck_id = $1 AND card_profile_id = $2", request.deck_id, request.card_profile_id)
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DeleteDeckCardError::NotFound);
        }

        tx.commit().await?;

        Ok(())
    }
}
