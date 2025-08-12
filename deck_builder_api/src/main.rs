// Std
use std::{error::Error as StdError, time::Duration};

// External
use anyhow::anyhow;
use axum::{
    http::{header, HeaderValue, Method},
    routing::{get, post},
    Router,
};

use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber;

// Internal
mod auth;
mod database;
mod handlers;
mod models;
mod scryfall;
use crate::{
    auth::jwt::JwtConfig,
    database::card::{delete_all_cards, Insert},
    models::card::scryfall_card::ScryfallCard,
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
            .min_connections(2) // min 2 idle connections in pool
            .max_connections(10) // max pool size of 10
            .idle_timeout(Some(Duration::from_secs(300))) // existing conn times out after 5 min
            .acquire_timeout(Duration::from_secs(5)) // acquiring new conn times out after 5 sec
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
        Router::new().route("/decks", get(handlers::decks::get_decks)),
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

    let app_state = AppState::initialize()
        .await
        .map_err(|e| anyhow!("Failed to initialize AppState. Error: {:?}", e))?;

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

    let bind_address = std::env::var("BIND_ADDRESS").map_err(|e| {
        anyhow!(
            "BIND_ADDRESS environment variable must be set. Error: {:?}",
            e
        )
    })?;

    // println!(
    //     "{}",
    //     include_str!("../../logos/deck_builder/ansi_shadow.txt")
    // );
    println!("deck_builder_api running on {}", bind_address);

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .map_err(|e| anyhow!("failed to bind address to listener with error: {:?}", e))?;

    delete_all_cards(&app_state.db_pool).await?;

    // insert_card(
    //     &app_state.db_pool,
    //     scryfall::card_search("satya")
    //         .await?
    //         .into_iter()
    //         .next()
    //         .expect("no cards found in search"),
    // )
    // .await?;

    let dump: Vec<ScryfallCard> = get_oracle_card_dump().await?;
    for (i, card) in dump.into_iter().enumerate() {
        match card.insert(&app_state.db_pool).await {
            Err(e) => println!("(*3*)<(failed to insert {:?}\nerror: {:?})", card.name, e),
            _ => (),
        }

        if i % 100 == 0 {
            println!("(*3*)<({} cards inserted!)", i);
        }
    }

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow!("failed to serve app with error: {:?}", e))?;

    Ok(())
}
