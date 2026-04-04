use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use zwipe::{
    config::Config,
    domain::{
        auth::{
            ports::AuthService,
            services::Service as AuthService_,
        },
        card::{
            ports::CardService,
            services::Service as CardService_,
        },
    },
    inbound::external::scryfall::bulk::BulkEndpoint,
    outbound::{resend::Resend, sqlx::postgres::Postgres},
};
use zwipe_core::domain::logo;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logo::Zervice::print();
    let config = Config::from_env()?;

    let level_filter = LevelFilter::from(config.rust_log);

    std::fs::create_dir_all(&config.log_dir)
        .map_err(|e| anyhow::anyhow!("failed to create log directory: {e}"))?;

    // Rolling daily log file: $LOG_DIR/zervice.YYYY-MM-DD.log (default: /var/log/zwipe)
    // max_log_files(30) automatically deletes files older than 30 days
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("zervice")
        .filename_suffix("log")
        .max_log_files(30)
        .build(&config.log_dir)
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
    tracing::info!("zervice running v{}", env!("CARGO_PKG_VERSION"));

    let db = Postgres::new(&config.database_url).await?;
    let card_service = CardService_::new(db.clone());
    let resend = Resend::new(config.resend_api_key, config.resend_from_email);
    let auth_service = AuthService_::new(db.clone(), db.clone(), resend, config.jwt_secret);

    tracing::info!("syncing cards from scryfall");
    card_service.scryfall_sync(BulkEndpoint::OracleCards).await?;

    tracing::info!("cleaning up expired sessions");
    auth_service.delete_expired_sessions().await?;

    tracing::info!("zervice completed successfully");
    Ok(())
}
