use crate::adapters::http::handlers;
use crate::adapters::AppState;
use crate::config::Config;
use crate::config::Config;
use crate::inbound::external::scryfall::scryfall_sync;
use crate::inbound::http::{HttpServer, HttpServerConfig};
use crate::outbound::sqlx::postgres::Postgres;

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
    let server_config = HttpServerConfig {
        bind_address: &config.bind_address,
        allowed_origins: &config.allowed_origins,
    };
    let http_server = HttpServer::new(user_service, server_config);
    http_server.run().await
}
