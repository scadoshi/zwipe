use crate::{auth::middleware::AuthenticatedUser, AppState};
use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};

pub async fn get_cards(
    State(_app_state): State<AppState>,
    _authenticated_user: AuthenticatedUser,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Query cards table with filters/pagination
    // TODO: SELECT * FROM cards LIMIT 20 OFFSET ?
    // TODO: Add search/filter parameters from query string

    Ok(Json(json!({
        "cards": [],
        "message": "Card database integration - coming soon!",
        "todo": "Need to seed card data from MTG API"
    })))
}
