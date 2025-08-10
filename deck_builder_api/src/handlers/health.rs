// External crate imports
use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};
use sqlx::query;
use tracing::error;

use crate::AppState;

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
    query("SELECT 1")
        .fetch_one(&app_state.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to query database at all with error: {:?}", e);
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    Ok(Json(json!({
        "status": "healthy",
        "database": "connected",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
