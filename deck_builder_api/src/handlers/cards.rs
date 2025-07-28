// External crate imports
use axum::{extract::State, http::StatusCode, response::Json};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use serde_json::{json, Value};

// define DbPool from the more complex type
type DbPool = Pool<ConnectionManager<PgConnection>>;

pub async fn list_cards(State(pool): State<DbPool>) -> Result<Json<Value>, StatusCode> {
    let mut _conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // TODO: Query cards table with filters/pagination
    // TODO: SELECT * FROM cards LIMIT 20 OFFSET ?
    // TODO: Add search/filter parameters from query string

    Ok(Json(json!({
        "cards": [],
        "message": "Card database integration - coming soon!",
        "todo": "Need to seed card data from MTG API"
    })))
}
