use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use zwipe::config::Config;
use zwipe::domain::{auth, card, deck, health, logo, user};
use zwipe::inbound::http::{HttpServer, HttpServerConfig};
use zwipe::outbound::sqlx::postgres::Postgres;

#[tokio::main]
async fn main() {
    logo::Zerver::print();
    match run().await {
        Ok(_) => (),
        Err(e) => tracing::error!("main failed: {:?}", e),
    }
}

async fn run() -> anyhow::Result<()> {
    let config: Config = Config::from_env()?;

    let level_filter = LevelFilter::from(config.rust_log);

    // Rolling daily log file: /var/log/zwipe/zerver.YYYY-MM-DD.log
    // max_log_files(30) automatically deletes files older than 30 days
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("zerver")
        .filename_suffix("log")
        .max_log_files(30)
        .build("/var/log/zwipe")
        .map_err(|e| anyhow::anyhow!("failed to build log file appender: {e}"))?;

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_filter(level_filter),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_filter(level_filter),
        )
        .init();
    let db = Postgres::new(&config.database_url).await?;
    let auth_service = auth::services::Service::new(db.clone(), db.clone(), config.jwt_secret);
    let user_service = user::services::Service::new(db.clone());
    let health_service = health::services::Service::new(db.clone());
    let card_service = card::services::Service::new(db.clone());
    let deck_service = deck::services::Service::new(db.clone(), db.clone());
    let server_config = HttpServerConfig {
        bind_address: &config.bind_address,
        allowed_origins: config.allowed_origins,
    };
    let http_server = HttpServer::new(
        auth_service,
        user_service,
        health_service,
        card_service,
        deck_service,
        server_config,
    )
    .await?;
    http_server.run().await
}
