// External
use axum::{extract::State, http::StatusCode, response::Json};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use serde_json::{json, Value};

// Internal
use crate::utils::connect_to;

// define DbPool from the more complex type
type DbPool = Pool<ConnectionManager<PgConnection>>;

pub async fn list_cards(State(pool): State<DbPool>) -> Result<Json<Value>, StatusCode> {
    let mut _conn = connect_to(pool)?;

    // TODO: Query cards table with filters/pagination
    // TODO: SELECT * FROM cards LIMIT 20 OFFSET ?
    // TODO: Add search/filter parameters from query string

    Ok(Json(json!({
        "cards": [],
        "message": "Card database integration - coming soon!",
        "todo": "Need to seed card data from MTG API"
    })))
}
