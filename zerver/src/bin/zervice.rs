use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};
use zwipe::{
    config::Config,
    domain::{
        auth::{ports::AuthService, services::Service as AuthService_},
        card::{ports::CardService, services::Service as CardService_},
    },
    inbound::external::scryfall::bulk::BulkEndpoint,
    outbound::{resend::Resend, sqlx::postgres::Postgres},
};
use zwipe_core::domain::logo;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logo::Zervice::print();
    let config = Config::from_env()?;
    let args: Vec<String> = std::env::args().collect();

    // See zerver.rs for the rationale — RUST_LOG from the process env wins; otherwise
    // we use the directive string from Config. Per-layer because EnvFilter isn't Clone.
    let env_filter =
        || EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.rust_log));

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
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter()))
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_filter(env_filter()),
        )
        .init();
    let recategorize = args
        .iter()
        .any(|a| a == "--recategorize" || a == "-rc" || a == "--rc");
    tracing::info!(
        "zervice running v{}{}",
        env!("CARGO_PKG_VERSION"),
        if recategorize {
            " [--recategorize]"
        } else {
            ""
        }
    );

    let db = Postgres::new(&config.database_url).await?;
    let card_service = CardService_::new(db.clone());
    let resend = Resend::new(config.resend_api_key, config.resend_from_email);
    let auth_service = AuthService_::new(
        db.clone(),
        db.clone(),
        resend,
        config.jwt_secret,
        config.web_base_url,
        config.support_email_address,
    );

    card_service
        .scryfall_sync(BulkEndpoint::DefaultCards)
        .await?;

    if recategorize {
        tracing::info!("clearing all categories for recategorization");
        card_service.clear_all_categories().await?;
    }

    let (classified, total) = card_service.classify_untagged_cards(1000).await?;
    tracing::info!("classification: {classified} / {total} cards categorized");

    card_service.refresh_latest_cards().await?;
    tracing::info!("latest_cards materialized view refreshed");

    auth_service.delete_expired_sessions().await?;

    tracing::info!("zervice completed");
    Ok(())
}
