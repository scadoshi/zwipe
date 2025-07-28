// Standard library imports
use std::time::Duration;

// External crate imports
use axum::{routing::get, Router};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use tower_http::cors::CorsLayer;
use tracing_subscriber;

// Internal crate imports
mod handlers;
mod models;
mod schema;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL not found")
        .to_string();
    let connection_manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .min_idle(Some(2)) // always keep 2 connections always ready
        .max_size(10) // maximum of 10 connections at a time
        .idle_timeout(Some(Duration::from_secs(300))) // after 5 minutes of no usage, connection goes into idle
        .connection_timeout(Duration::from_secs(5)) // timeout a connection after 5 seconds
        .build(connection_manager)
        .expect("Failed to build connection pool from connection_manager");

    // Build our application with routes
    let app = Router::new()
        .route("/", get(handlers::health::root))
        .route("/health", get(handlers::health::health_check))
        .route("/health/deep", get(handlers::health::health_check_deep))
        .route("/api/v1/cards", get(handlers::cards::list_cards))
        .route("/api/v1/decks", get(handlers::decks::list_decks))
        .layer(CorsLayer::permissive()) // TODO: Configure CORS properly
        .with_state(pool);

    // Get bind address from environment
    let bind_address =
        std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    println!("🚀 Deck Builder API starting on {}", bind_address);

    // Run the server
    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
