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

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;

    tracing_subscriber::fmt::init();
    info!("Tracing started");
    dotenvy::dotenv()?;

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

    info!("Deck Builder API running on {}", bind_address);

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .map_err(|e| anyhow!("failed to bind address to listener with error: {:?}", e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow!("Failed to serve application with error: {:?}", e))?;

    Ok(())
}
