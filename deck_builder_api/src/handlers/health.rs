// External crate imports
use axum::{extract::State, http::StatusCode, response::Json};
use diesel::{sql_query, RunQueryDsl};
use serde_json::{json, Value};
use tracing::error;

use crate::{utils::connect_to, AppState};

pub async fn root() -> Json<Value> {
    Json(json!({
        "message": "MTG Deck Builder API",
        "version": "0.1.0",
        "status": "ready"
    }))
}

pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

pub async fn health_check_deep(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let mut conn = connect_to(app_state.db_pool)?;

    // Simple query to verify DB is responsive
    sql_query("SELECT 1").execute(&mut conn).map_err(|e| {
        error!("Failed to query database at all with error: {:?}", e);
        StatusCode::SERVICE_UNAVAILABLE
    })?;

    Ok(Json(json!({
        "status": "healthy",
        "database": "connected",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
