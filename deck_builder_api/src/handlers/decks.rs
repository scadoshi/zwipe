// External crate imports
use axum::{extract::State, http::StatusCode, response::Json};
use diesel::{prelude::*, RunQueryDsl};
use serde_json::{json, Value};
use tracing::error;

// Internal crate imports (our code)
use crate::{auth::middleware::AuthenticatedUser, models::Deck, utils::connect_to};
use crate::{schema::decks, AppState};

pub async fn get_decks(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<Value>, StatusCode> {
    let mut conn = connect_to(app_state.db_pool)?;
    let user_decks: Vec<Deck> = decks::table
        .filter(decks::user_id.eq(authenticated_user.user_id))
        .load(&mut conn)
        .map_err(|e| {
            error!("Failed to load decks with error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(json!({
        "decks": user_decks,
        "user_id": authenticated_user.user_id,
        "count": user_decks.len()
    })))
}
