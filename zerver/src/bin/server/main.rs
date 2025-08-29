use std::str::FromStr;
use zwipe::config::Config;
use zwipe::domain::{auth, card, health, logo, user};
use zwipe::inbound::http::{HttpServer, HttpServerConfig};
use zwipe::outbound::sqlx::postgres::Postgres;

#[tokio::main]
async fn main() {
    logo::print();
    match run().await {
        Ok(_) => (),
        Err(e) => tracing::error!("main failed: {:?}", e),
    }
}

async fn run() -> anyhow::Result<()> {
    let config: Config = Config::from_env()?;
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::from_str(&config.rust_log)?)
        .init();
    let db = Postgres::new(&config.database_url).await?;
    let auth_service = auth::services::Service::new(db.clone(), config.jwt_secret);
    let user_service = user::services::Service::new(db.clone());
    let health_service = health::services::Service::new(db.clone());
    let card_service = card::services::Service::new(db.clone());
    let server_config = HttpServerConfig {
        bind_address: &config.bind_address,
        allowed_origins: config.allowed_origins,
    };
    let http_server = HttpServer::new(
        auth_service,
        user_service,
        health_service,
        card_service,
        server_config,
    )
    .await?;
    http_server.run().await
}
