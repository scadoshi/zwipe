use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};
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

    // Every step runs non-fatally: a failure logs and the pipeline continues, so one
    // broken step can't skip the rest (e.g. the serve-critical matview refreshes).
    // Failures are tallied and surfaced as a non-zero exit at the end.
    let mut failures = 0u32;

    tracing::info!("step 1/5 card sync (default_cards): starting");
    match card_service.scryfall_sync(BulkEndpoint::DefaultCards).await {
        Ok(_) => tracing::info!("step 1/5 card sync: ok"),
        Err(e) => {
            failures += 1;
            tracing::error!("step 1/5 card sync FAILED (continuing): {e:#}");
        }
    }

    if recategorize {
        tracing::info!("clearing all categories (--recategorize)");
        if let Err(e) = card_service.clear_all_categories().await {
            failures += 1;
            tracing::error!("clear categories FAILED (continuing): {e:#}");
        }
    }

    tracing::info!("step 2/5 oracle tags: starting");
    match card_service.sync_oracle_tags().await {
        Ok((tags, correlations)) => {
            tracing::info!("step 2/5 oracle tags: synced {tags} tags, {correlations} correlations");
            match card_service.refresh_card_oracle_tags().await {
                Ok(()) => tracing::info!("step 2/5 oracle tags projection: ok"),
                Err(e) => {
                    failures += 1;
                    tracing::error!("step 2/5 oracle tags projection FAILED (continuing): {e:#}");
                }
            }
            match card_service.refresh_oracle_tag_groups().await {
                Ok(rows) => tracing::info!("step 2/5 oracle tags grouping: {rows} rows"),
                Err(e) => {
                    failures += 1;
                    tracing::error!("step 2/5 oracle tags grouping FAILED (continuing): {e:#}");
                }
            }
        }
        Err(e) => {
            failures += 1;
            tracing::error!(
                "step 2/5 oracle tags sync FAILED (skipping projection, continuing): {e:#}"
            );
        }
    }

    tracing::info!("step 3/5 derive categories (otags + gaps): starting");
    match card_service.derive_card_categories(1000).await {
        Ok((otag_rows, merges)) => {
            tracing::info!(
                "step 3/5 derive categories: {otag_rows} rows from otags, {merges} straggler merges"
            );
        }
        Err(e) => {
            failures += 1;
            tracing::error!("step 3/5 derive categories FAILED (continuing): {e:#}");
        }
    }

    tracing::info!("step 4/5 refresh materialized views: starting");
    if let Err(e) = card_service.refresh_latest_cards().await {
        failures += 1;
        tracing::error!("step 4/5 latest_cards refresh FAILED (continuing): {e:#}");
    } else {
        tracing::info!("step 4/5 latest_cards: refreshed");
    }
    if let Err(e) = card_service.refresh_card_signal_rollup().await {
        failures += 1;
        tracing::error!("step 4/5 card_signal_rollup refresh FAILED (continuing): {e:#}");
    } else {
        tracing::info!("step 4/5 card_signal_rollup: refreshed");
    }

    tracing::info!("step 5/5 prune expired sessions: starting");
    if let Err(e) = auth_service.delete_expired_sessions().await {
        failures += 1;
        tracing::error!("step 5/5 prune sessions FAILED (continuing): {e:#}");
    } else {
        tracing::info!("step 5/5 prune sessions: ok");
    }

    if failures == 0 {
        tracing::info!("zervice completed: all 5 steps ok");
        Ok(())
    } else {
        tracing::error!("zervice completed with {failures} failed step(s) — see errors above");
        anyhow::bail!("zervice finished with {failures} failed step(s)")
    }
}
