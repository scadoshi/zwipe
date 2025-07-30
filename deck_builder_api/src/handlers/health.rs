// External crate imports
use axum::{extract::State, http::StatusCode, response::Json};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    sql_query, PgConnection, RunQueryDsl,
};
use serde_json::{json, Value};
use tracing::error;

// define DbPool from the more complex type
type DbPool = Pool<ConnectionManager<PgConnection>>;

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

pub async fn health_check_deep(State(pool): State<DbPool>) -> Result<Json<Value>, StatusCode> {
    let mut conn = pool.get().map_err(|e| {
        error!(
            "Failed to get connection to database connection pool with error: {:?}",
            e
        );
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

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
