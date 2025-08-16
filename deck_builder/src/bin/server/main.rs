use crate::adapters::http::handlers;
use crate::adapters::AppState;
use crate::config::Config;
use crate::config::Config;
use crate::inbound::external::scryfall::scryfall_sync;

use anyhow::anyhow;
use axum::{
    http::{header, Method},
    routing::{get, post},
    Router,
};
use deck_builder::outbound::sqlx::postgres::Postgres;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    domain::print_logo();
    match run().await {
        Ok(_) => (),
        Err(e) => error!("Failed to run main. Error: {:?}", e),
    }
}

async fn run() -> Result<(), anyhow::Result<()>> {
    tracing_subscriber::fmt::init();
    let config: Config = Config::from_env()?;
    let postgres = Postgres::new(config.database_url)?;
    let user_service = crate::domain::user::services::Service::new(postgres);

    // wall of incorrectness everything below this is incorrect and must be hexarched

    let app_state = AppState::initialize()
        .await
        .map_err(|e| anyhow!("Failed to initialize AppState. Error: {:?}", e))?;
    info!("Initialized app state");

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(
            CorsLayer::new()
                .allow_origin(config.allowed_origins)
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

    scryfall_sync(&app_state.db_pool).await?;
    info!("Deck Builder running on {}", bind_address);

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .map_err(|e| anyhow!("failed to bind address to listener with error: {:?}", e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow!("Failed to serve application with error: {:?}", e))?;

    Ok(())
}
