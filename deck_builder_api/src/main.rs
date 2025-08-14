use anyhow::anyhow;
use axum::{
    http::{header, HeaderValue, Method},
    routing::{get, post},
    Router,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{error::Error as StdError, time::Duration};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber;

mod auth;
mod database;
mod handlers;
mod models;
mod scryfall;
use crate::auth::jwt::JwtConfig;

#[derive(Clone)]
struct AppState {
    db_pool: PgPool,
    jwt_config: JwtConfig,
}

impl AppState {
    async fn initialize() -> Result<Self, Box<dyn StdError>> {
        let database_url = std::env::var("DATABASE_URL").map_err(|e| {
            anyhow!(
                "DATABASE_URL environment variable must be set. Error: {:?}",
                e
            )
        })?;
        info!("Extracted database URL from environment");

        let db_pool = PgPoolOptions::new()
            .min_connections(2)
            .max_connections(10)
            .idle_timeout(Some(Duration::from_secs(300)))
            .acquire_timeout(Duration::from_secs(5))
            .connect(&database_url)
            .await
            .map_err(|e| anyhow!("Failed to build connection. Error: {:?}", e))?;
        info!("Generated database connection pool");

        Ok(Self {
            db_pool,
            jwt_config: JwtConfig::from_env()?,
        })
    }
}

#[tokio::main]
async fn main() {
    let logo = include_str!("../../logos/deck_builder/ansi_shadow.txt");
    println!("{}", logo);
    match run().await {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to run main. Error: {:?}", e),
    }
}

async fn run() -> Result<(), Box<dyn StdError>> {
    tracing_subscriber::fmt::init();
    info!("Tracing started");
    dotenvy::dotenv()?;

    let allowed_origins: Vec<HeaderValue> = std::env::var("ALLOWED_ORIGINS")
        .map_err(|e| {
            anyhow!(
                "ALLOWED_ORIGINS environment variable must be set. Error: {:?}",
                e
            )
        })?
        .split(",")
        .map(|x| x.parse())
        .collect::<Result<Vec<HeaderValue>, _>>()
        .map_err(|e| {
            anyhow!(
                "Failed to collect all values in ALLOWED_ORIGINS list. Error: {:?}",
                e
            )
        })?;
    info!("Collected allowed origins");

    // protected routes - these need jwt authentication
    // all handlers here must include `AuthenticatedUser` parameter
    // which automatically enforces jwt validation via custom extractor
    // supreme rusty
    let protected_routes = Router::new().nest(
        "/api/v1",
        Router::new().route("/decks", get(handlers::decks::get_decks)),
    );
    info!("Generated protected routes");

    // public routes - no authentication required
    // health checks and auth endpoints are accessible without tokens
    let public_routes = Router::new()
        .route("/", get(handlers::health::root))
        .route("/health", get(handlers::health::health_check))
        .route("/health/deep", get(handlers::health::health_check_deep))
        .nest(
            "/api/v1",
            Router::new()
                .route("/auth/login", post(handlers::auth::login))
                .route("/auth/register", post(handlers::auth::register)),
        );
    info!("Generated public routes");

    let app_state = AppState::initialize()
        .await
        .map_err(|e| anyhow!("Failed to initialize AppState. Error: {:?}", e))?;
    info!("Initialized app state");

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origins)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::CONTENT_TYPE]),
        )
        .with_state(app_state.clone())
        .layer(TraceLayer::new_for_http());
    info!("Put router together");

    let bind_address = std::env::var("BIND_ADDRESS").map_err(|e| {
        anyhow!(
            "BIND_ADDRESS environment variable must be set. Error: {:?}",
            e
        )
    })?;

    crate::database::card::scryfall_sync(&app_state.db_pool).await?;

    info!("Deck Builder API running on {}", bind_address);

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .map_err(|e| anyhow!("failed to bind address to listener with error: {:?}", e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow!("Failed to serve application with error: {:?}", e))?;

    Ok(())
}
