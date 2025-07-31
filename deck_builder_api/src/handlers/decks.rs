// External crate imports
use axum::{extract::State, http::StatusCode, response::Json};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    PgConnection, RunQueryDsl,
};
use serde_json::{json, Value};
use tracing::error;

// Internal crate imports (our code)
use crate::schema::decks;
use crate::{models::Deck, utils::connect_to};

// define DbPool from the more complex type
type DbPool = Pool<ConnectionManager<PgConnection>>;

pub async fn list_decks(State(pool): State<DbPool>) -> Result<Json<Value>, StatusCode> {
    let mut conn = connect_to(pool)?;

    let user_id_value = 1; // TODO: Extract user_id from JWT token

    let user_decks: Vec<Deck> = decks::table
        .filter(decks::user_id.eq(user_id_value))
        .load(&mut conn)
        .map_err(|e| {
            error!("Failed to load decks with error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(json!({
        "decks": user_decks,
        "user_id": user_id_value,
        "count": user_decks.len()
    })))
}
