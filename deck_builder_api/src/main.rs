// Std
use std::time::Duration;

// External
use axum::{
    http::{header, HeaderValue, Method},
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
mod utils;

use crate::auth::jwt::JwtConfig;

type DbPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
struct AppState {
    db_pool: DbPool,
    jwt_config: JwtConfig,
}

impl AppState {
    fn initialize() -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = std::env::var("DATABASE_URL")?;

        let connection_manager = ConnectionManager::<PgConnection>::new(database_url);

        let db_pool = Pool::builder()
            .min_idle(Some(2)) // minimum 2 idle connections in pool
            .max_size(10) // maximum pool size of 10
            .idle_timeout(Some(Duration::from_secs(300))) // idle connection timeout of 5 min
            .connection_timeout(Duration::from_secs(5)) // new connection timeout of 5 sec
            .build(connection_manager)?;

        Ok(Self {
            db_pool,
            jwt_config: JwtConfig::from_env()?,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().expect("Failed to load .env file");

    let allowed_origins: Vec<HeaderValue> = std::env::var("")
        .expect("ALLOWED_ORIGINS environment variable must be set")
        .split(",")
        .map(|x| x.parse().expect("Failed to parse HeaderValue"))
        .collect();

    let app = Router::new()
        .route("/", get(handlers::health::root))
        .route("/health", get(handlers::health::health_check))
        .route("/health/deep", get(handlers::health::health_check_deep))
        .nest(
            "/api/v1",
            Router::new()
                .route("/auth/login", post(handlers::auth::login))
                .route("/auth/register", post(handlers::auth::register))
                .route("/cards", get(handlers::cards::list_cards))
                .route("/decks", get(handlers::decks::list_decks)),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origins)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::CONTENT_TYPE]),
        )
        .with_state(AppState::initialize()?);

    let bind_address =
        std::env::var("BIND_ADDRESS").expect("BIND_ADDRESS environment variable must be set");
    println!("🚀 Deck Builder API starting on {}", bind_address);

    let listener = tokio::net::TcpListener::bind(&bind_address).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
