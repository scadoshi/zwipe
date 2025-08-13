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
use crate::{
    auth::jwt::JwtConfig,
    database::card::{self, BulkInsert},
    models::scryfall_card::ScryfallCard,
    scryfall::get_oracle_card_dump,
};

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

        let db_pool = PgPoolOptions::new()
            .min_connections(2)
            .max_connections(10)
            .idle_timeout(Some(Duration::from_secs(300)))
            .acquire_timeout(Duration::from_secs(5))
            .connect(&database_url)
            .await
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
    let start = std::time::Instant::now();
    tracing_subscriber::fmt::init();
    info!("Tracing started at {:.2?}", start.elapsed());
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
    info!("Collected allowed origins at {:.2?}", start.elapsed());

    // protected routes - these need jwt authentication
    // all handlers here must include `AuthenticatedUser` parameter
    // which automatically enforces jwt validation via custom extractor
    // supreme rusty
    let protected_routes = Router::new().nest(
        "/api/v1",
        Router::new().route("/decks", get(handlers::decks::get_decks)),
    );
    info!("Generated protected routes at {:.2?}", start.elapsed());

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
    info!("Generated public routes at {:.2?}", start.elapsed());

    let app_state = AppState::initialize()
        .await
        .map_err(|e| anyhow!("Failed to initialize AppState. Error: {:?}", e))?;
    info!("Initialized app state at {:.2?}", start.elapsed());

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
                .await
                .map_err(|e| anyhow!("Failed to initialize AppState. Error: {:?}", e))?,
        )
        .layer(TraceLayer::new_for_http());
    info!("Put router together at {:.2?}", start.elapsed());

    let bind_address = std::env::var("BIND_ADDRESS").map_err(|e| {
        anyhow!(
            "BIND_ADDRESS environment variable must be set. Error: {:?}",
            e
        )
    })?;

    // let logo_path = include_str!("../../logos/deck_builder/ansi_shadow.txt");
    // println!("{}", logo);
    info!(
        "Deck Builder running on {} at {:.2?}",
        bind_address,
        start.elapsed()
    );

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .map_err(|e| anyhow!("failed to bind address to listener with error: {:?}", e))?;

    card::delete_all(&app_state.db_pool).await?;
    info!("Deleted all cards at {:.2?}", start.elapsed());

    info!("Getting dump of oracle cards at {:.2?}", start.elapsed());
    let dump: Vec<ScryfallCard> = get_oracle_card_dump().await?;
    let batch_size = 500;
    info!(
        "Recieved... Batch inserting by {:?} at {:.2?}",
        batch_size,
        start.elapsed()
    );
    dump.batch_insert(batch_size, &app_state.db_pool).await?;
    info!("Cards fully inserted at {:.2?}", start.elapsed());

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow!("Failed to serve application with error: {:?}", e))?;

    Ok(())
}
