use std::str::FromStr;

use deck_builder::domain::{auth, logo, user};
use deck_builder::inbound::http::{HttpServer, HttpServerConfig};
use deck_builder::{config::Config, outbound::sqlx::postgres::Postgres};

#[tokio::main]
async fn main() {
    logo::print();
    match run().await {
        Ok(_) => (),
        Err(e) => tracing::error!("Failed to run main. Error: {:?}", e),
    }
}

async fn run() -> anyhow::Result<()> {
    let config: Config = Config::from_env()?;
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::from_str(&config.rust_log)?)
        .init();
    let postgres = Postgres::new(&config.database_url).await?;
    let auth_service = auth::services::Service::new(postgres.clone(), config.jwt_secret);
    let user_service = user::services::Service::new(postgres.clone());
    let server_config = HttpServerConfig {
        bind_address: &config.bind_address,
        allowed_origins: config.allowed_origins,
    };
    let http_server = HttpServer::new(auth_service, user_service, server_config).await?;
    http_server.run().await
}
