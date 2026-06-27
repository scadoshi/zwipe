use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;
use zwipe::config::Config;
use zwipe::domain::{auth, card, deck, health, metrics, user};
use zwipe::inbound::http::{HttpServer, HttpServerConfig};
use zwipe::outbound::resend::Resend;
use zwipe::outbound::sqlx::postgres::Postgres;
use zwipe_core::domain::logo;

#[tokio::main]
#[allow(clippy::print_stderr)]
async fn main() {
    logo::Zerver::print();
    match run().await {
        Ok(_) => (),
        Err(e) => eprintln!("main failed: {e:?}"),
    }
}

async fn run() -> anyhow::Result<()> {
    let config: Config = Config::from_env()?;

    // EnvFilter::try_from_default_env() reads RUST_LOG directly from the process env;
    // when unset/invalid we fall back to the directive string loaded via Config (which
    // came from .env). Either way directives like "info,sqlx=warn,zwipe=debug" work.
    // Filters aren't Clone, so we build one per subscriber layer via this closure.
    let env_filter = || {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.rust_log))
    };

    std::fs::create_dir_all(&config.log_dir)
        .map_err(|e| anyhow::anyhow!("failed to create log directory: {e}"))?;

    // Rolling daily log file: $LOG_DIR/zerver.YYYY-MM-DD.log (default: /var/log/zwipe)
    // max_log_files(30) automatically deletes files older than 30 days
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("zerver")
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
    tracing::info!("zerver running v{}", env!("CARGO_PKG_VERSION"));
    let db = Postgres::new(&config.database_url).await?;
    let resend = Resend::new(config.resend_api_key, config.resend_from_email);
    let auth_service = auth::services::Service::new(
        db.clone(),
        db.clone(),
        resend,
        config.jwt_secret,
        config.web_base_url.clone(),
        config.support_email_address,
    );
    let user_service = user::services::Service::new(db.clone());
    let health_service = health::services::Service::new(db.clone());
    let card_service = card::services::Service::new(db.clone());
    let deck_service = deck::services::Service::new(db.clone(), db.clone());
    let metrics_service: Arc<dyn metrics::ports::ErasedMetricsService> =
        Arc::new(metrics::services::Service::new(db.clone()));
    let server_config = HttpServerConfig {
        bind_address: &config.bind_address,
        allowed_origins: config.allowed_origins,
        min_client_version: config.min_client_version,
        web_base_url: config.web_base_url,
    };
    let http_server = HttpServer::new(
        auth_service,
        user_service,
        health_service,
        card_service,
        deck_service,
        metrics_service,
        server_config,
    )
    .await?;
    http_server.run().await
}
