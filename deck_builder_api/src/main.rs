// Std
use std::{error::Error as StdError, time::Duration};

// External
use anyhow::anyhow;
use axum::{
    http::{header, HeaderValue, Method},
    routing::{get, post},
    Router,
};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
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
    fn initialize() -> Result<Self, Box<dyn StdError>> {
        let database_url = std::env::var("DATABASE_URL").map_err(|e| {
            anyhow!(
                "DATABASE_URL environment variable must be set. Error: {:?}",
                e
            )
        })?;

        let connection_manager = ConnectionManager::<PgConnection>::new(database_url);

        let db_pool = Pool::builder()
            .min_idle(Some(2)) // minimum 2 idle connections in pool
            .max_size(10) // maximum pool size of 10
            .idle_timeout(Some(Duration::from_secs(300))) // idle connection timeout of 5 min
            .connection_timeout(Duration::from_secs(5)) // new connection timeout of 5 sec
            .build(connection_manager)
            .map_err(|e| anyhow!("Failed to build connection. Error: {:?}", e))?;

        Ok(Self {
            db_pool,
            jwt_config: JwtConfig::from_env()?,
        })
    }
}

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to run main. Error: {:?}", e),
    }
}

async fn run() -> Result<(), Box<dyn StdError>> {
    tracing_subscriber::fmt::init();
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

    // protected routes - these need jwt authentication
    // all handlers here must include `AuthenticatedUser` parameter
    // which automatically enforces jwt validation via custom extractor
    // supreme rusty
    let protected_routes = Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/cards", get(handlers::cards::get_cards))
            .route("/decks", get(handlers::decks::get_decks)),
    );

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

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origins)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::CONTENT_TYPE]),
        )
        .with_state(
            AppState::initialize()
                .map_err(|e| anyhow!("Failed to initialize AppState. Error: {:?}", e))?,
        )
        .layer(TraceLayer::new_for_http());

    let bind_address = std::env::var("BIND_ADDRESS").map_err(|e| {
        anyhow!(
            "BIND_ADDRESS environment variable must be set. Error: {:?}",
            e
        )
    })?;

    let logo_str = include_str!("../../logos/ansi_shadow.txt");
    println!("{}", "=".repeat(69));
    println!("{}", logo_str);
    println!("{}", "=".repeat(69));
    println!("deck_builder API Running on {}", bind_address);

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .map_err(|e| anyhow!("failed to bind address to listener with error: {:?}", e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow!("failed to serve app with error: {:?}", e))?;

    Ok(())
}
