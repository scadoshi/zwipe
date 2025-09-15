#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService, card::ports::CardService, deck::ports::DeckService,
        health::ports::HealthService, user::ports::UserService,
    },
    inbound::http::AppState,
};
#[cfg(feature = "zerver")]
use axum::{extract::State, Json};
#[cfg(feature = "zerver")]
use chrono::Utc;
use serde::Serialize;
use serde_json::{json, Value};

// ======
//  root
// ======

#[derive(Debug, Serialize)]
pub struct RootResponse {
    pub message: String,
    pub version: String,
    pub status: String,
}

impl RootResponse {
    fn new(message: &str, version: &str, status: &str) -> Self {
        Self {
            message: message.to_string(),
            version: version.to_string(),
            status: status.to_string(),
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn root() -> Json<Value> {
    Json(json!(RootResponse::new(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        "ready",
    )))
}

// =========
//  health
// =========

#[derive(Debug, Serialize)]
struct HealthCheckResponse {
    status: String,
    timestamp: String,
}

impl HealthCheckResponse {
    fn new(status: &str) -> Self {
        Self {
            status: status.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn is_server_running() -> Json<Value> {
    Json(json!(HealthCheckResponse::new("healthy")))
}

#[cfg(feature = "zerver")]
pub async fn are_server_and_database_running<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
) -> Json<Value>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let result = match state.health_service.check_database().await {
        Ok(_) => "healthy",
        // add more errors here if we check other things in the future
        Err(_) => "cannot connect to database",
    };

    Json(json!(HealthCheckResponse::new(result)))
}
