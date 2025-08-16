use axum::routing::{get, post};
use axum::Router;

use crate::inbound::http::handlers;

pub fn protected_routes() -> Router {
    Router::new().nest(
        "/api/v1",
        Router::new().route("/decks", get(handlers::decks::get_decks)),
    )
}

pub fn public_routes() -> Router {
    Router::new()
        .route("/", get(handlers::health::root))
        .route("/health", get(handlers::health::health_check))
        .route("/health/deep", get(handlers::health::health_check_deep))
        .nest(
            "/api/v1",
            Router::new()
                .route("/auth/login", post(handlers::auth::login))
                .route("/auth/register", post(handlers::auth::register)),
        )
}
