// Std
use std::time::Duration;

// External
use axum::{
    routing::{get, post},
    Router,
};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use tower_http::cors::CorsLayer;
use tracing_subscriber;

// Internal
mod auth;
mod handlers;
mod models;
mod schema;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Panic on configuration failures - these are startup issues
    dotenvy::dotenv().expect("Failed to load .env file");

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

    let connection_manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .min_idle(Some(2)) // minimum 2 idle connections in pool
        .max_size(10) // maximum pool size of 10
        .idle_timeout(Some(Duration::from_secs(300))) // idle connection timeout of 5 min
        .connection_timeout(Duration::from_secs(5)) // new connection timeout of 5 sec
        .build(connection_manager)
        .expect("Failed to build database connection pool");

    // Build our application with routes
    let app = Router::new()
        .route("/", get(handlers::health::root))
        .route("/health", get(handlers::health::health_check))
        .route("/health/deep", get(handlers::health::health_check_deep))
        .route("/api/v1/auth/login", post(handlers::auth::login))
        .route("/api/v1/cards", get(handlers::cards::list_cards))
        .route("/api/v1/decks", get(handlers::decks::list_decks))
        .layer(CorsLayer::permissive()) // TODO: Configure CORS properly
        .with_state(pool);

    let bind_address =
        std::env::var("BIND_ADDRESS").expect("BIND_ADDRESS environment variable must be set");

    println!("🚀 Deck Builder API starting on {}", bind_address);

    // Return errors for network operations - process manager might want to handle these
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
