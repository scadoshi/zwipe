//! zervice: the nightly job that syncs Scryfall, removes the printings Scryfall deleted, refreshes
//! the card views and prunes old rows. Every step records a `zervice_steps` row. `ZERVICE_DRY_RUN=true`
//! or `--dry-run` previews a whole run: every write rolls back, the view refreshes are skipped, and
//! there is no healthcheck ping.

use std::collections::BTreeMap;
use std::time::Instant;

use chrono::{DateTime, Utc};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;
use zwipe::config::ZerviceConfig;
use zwipe::domain::card::models::printing_removal::RemovalMode;
use zwipe::domain::card::models::zervice_step::{Step, StepOutcome, StepRecord};
use zwipe::domain::card::ports::CardService;
use zwipe::domain::upkeep::ports::UpkeepService;
use zwipe::domain::{card, upkeep};
use zwipe::inbound::external::scryfall::bulk::BulkEndpoint;
use zwipe::outbound::sqlx::postgres::{Jit, Postgres, Writes};
use zwipe_core::domain::logo;

/// Daily log files kept; the appender deletes older ones.
const MAX_LOG_FILES: usize = 30;

/// Cards per batch when deriving card roles.
const DERIVE_CATEGORIES_BATCH_SIZE: usize = 1000;

/// How long the healthcheck ping waits for an answer.
const HEALTHCHECK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[tokio::main]
#[expect(
    clippy::too_many_lines,
    reason = "the nightly steps in order, each logging and counting its own failure"
)]
async fn main() -> anyhow::Result<()> {
    logo::Zervice::print();
    let config = ZerviceConfig::from_env()?;

    // See zerver.rs for the rationale: RUST_LOG from the process env wins; otherwise
    // we use the directive string from Config. ONE filter, attached globally: a
    // per-layer EnvFilter copy on each fmt layer silently DROPS events (incl. the
    // step-result and error lines) once the directive set has per-target entries.
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.rust_log));

    std::fs::create_dir_all(&config.log_dir)
        .map_err(|e| anyhow::anyhow!("failed to create log directory: {e}"))?;

    // Rolling daily log file: $LOG_DIR/zervice.YYYY-MM-DD.log (default: /var/log/zwipe)
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("zervice")
        .filename_suffix("log")
        .max_log_files(MAX_LOG_FILES)
        .build(&config.log_dir)
        .map_err(|e| anyhow::anyhow!("failed to build log file appender: {e}"))?;

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .init();
    tracing::info!("zervice running v{}", env!("CARGO_PKG_VERSION"));

    // ZERVICE_DRY_RUN=true or --dry-run: every step does its work and keeps none of it.
    let dry_run = config.dry_run || std::env::args().skip(1).any(|arg| arg == "--dry-run");
    let writes = if dry_run {
        Writes::Discard
    } else {
        Writes::Keep
    };
    let db = Postgres::new(&config.database_url, Jit::Keep)
        .await?
        .with_writes(writes);
    let card_service = card::services::Service::new(db.clone());
    let upkeep_service = upkeep::services::Service::new(db.clone());
    if dry_run {
        tracing::info!("dry run: every step previews its changes and keeps none of them");
    }

    // Every step runs non-fatally: a failure logs and the pipeline continues, so one
    // broken step can't skip the rest (e.g. the serve-critical matview refreshes).
    // Failures are tallied and surfaced as a non-zero exit at the end.
    let mut night = Night {
        cards: card_service.clone(),
        run_id: Uuid::new_v4(),
        dry_run,
        failures: 0,
    };

    let clock = begin(Step::CardSync);
    let synced = match card_service.scryfall_sync(BulkEndpoint::DefaultCards).await {
        Ok(synced) => {
            let m = &synced.metrics;
            let outcome = done(
                Some(count(m.received_count())),
                &[
                    ("received", count(m.received_count())),
                    ("upserted", count(m.upserted_count())),
                    ("skipped", count(m.skipped_count())),
                    ("failed", count(m.error_count())),
                ],
            );
            night.record(Step::CardSync, clock, outcome).await;
            Some(synced)
        }
        Err(e) => {
            night.record(Step::CardSync, clock, failed(&e)).await;
            None
        }
    };

    // Only a complete bulk file says which printings Scryfall deleted, so the removal runs only
    // when step 1 downloaded and parsed it in full.
    let clock = begin(Step::PrintingRemoval);
    let mode = RemovalMode::of(config.printing_removal, dry_run);
    let outcome = match synced {
        Some(synced) => {
            match card_service
                .remove_deleted_printings(&synced.printing_ids, mode)
                .await
            {
                Ok(report) => {
                    for line in report.plan.lines() {
                        tracing::info!("printing removal ({mode:?}): {line}");
                    }
                    let applied = report.counts.unwrap_or_default();
                    done(
                        Some(applied.removed),
                        &[
                            ("to_remove", size(report.plan.removals.len())),
                            ("kept", size(report.plan.kept.len())),
                            ("removed", applied.removed),
                            ("moved_deck_cards", applied.remapped_deck_cards),
                            ("moved_slots", applied.remapped_slots),
                            ("applied", u64::from(report.counts.is_some())),
                        ],
                    )
                }
                Err(e) => failed(&e),
            }
        }
        None => StepOutcome::Skipped("the card sync did not complete"),
    };
    night.record(Step::PrintingRemoval, clock, outcome).await;

    let clock = begin(Step::OracleTags);
    match card_service.sync_oracle_tags().await {
        Ok((tags, correlations)) => {
            let outcome = done(
                Some(size(tags)),
                &[("tags", size(tags)), ("correlations", size(correlations))],
            );
            night.record(Step::OracleTags, clock, outcome).await;
            let clock = begin(Step::OracleTagProjection);
            let outcome = match card_service.refresh_card_oracle_tags().await {
                Ok(rows) => done(Some(rows), &[]),
                Err(e) => failed(&e),
            };
            night
                .record(Step::OracleTagProjection, clock, outcome)
                .await;
            let clock = begin(Step::OracleTagGrouping);
            let outcome = match card_service.refresh_oracle_tag_groups().await {
                Ok(rows) => done(Some(rows), &[]),
                Err(e) => failed(&e),
            };
            night.record(Step::OracleTagGrouping, clock, outcome).await;
        }
        Err(e) => {
            night.record(Step::OracleTags, clock, failed(&e)).await;
            for step in [Step::OracleTagProjection, Step::OracleTagGrouping] {
                let clock = begin(step);
                night
                    .record(
                        step,
                        clock,
                        StepOutcome::Skipped("the oracle tag sync failed"),
                    )
                    .await;
            }
        }
    }

    let clock = begin(Step::DeriveCategories);
    let outcome = match card_service
        .derive_card_categories(DERIVE_CATEGORIES_BATCH_SIZE)
        .await
    {
        Ok((otag_rows, merges)) => done(
            Some(otag_rows),
            &[("otag_rows", otag_rows), ("straggler_merges", size(merges))],
        ),
        Err(e) => failed(&e),
    };
    night.record(Step::DeriveCategories, clock, outcome).await;

    // A refresh only rebuilds what the other steps wrote, and latest_cards refreshes
    // CONCURRENTLY, which can't run in a transaction that rolls back: a dry run skips them.
    for step in [
        Step::RefreshLatestCards,
        Step::RefreshCardSignalRollup,
        Step::RefreshOtagContextSignalRollup,
    ] {
        let clock = begin(step);
        let outcome = if dry_run {
            StepOutcome::Skipped("dry run")
        } else {
            let refreshed = match step {
                Step::RefreshLatestCards => card_service.refresh_latest_cards().await,
                Step::RefreshCardSignalRollup => card_service.refresh_card_signal_rollup().await,
                Step::RefreshOtagContextSignalRollup => {
                    card_service.refresh_otag_context_signal_rollup().await
                }
                Step::CardSync
                | Step::PrintingRemoval
                | Step::OracleTags
                | Step::OracleTagProjection
                | Step::OracleTagGrouping
                | Step::DeriveCategories
                | Step::PruneClientErrors
                | Step::PruneCrashReports
                | Step::PruneExpiredSessions
                | Step::PruneEmailChanges => Ok(()),
            };
            match refreshed {
                Ok(()) => done(None, &[]),
                Err(e) => failed(&e),
            }
        };
        night.record(step, clock, outcome).await;
    }

    let clock = begin(Step::PruneClientErrors);
    let outcome = pruned(upkeep_service.prune_client_errors().await);
    night.record(Step::PruneClientErrors, clock, outcome).await;
    let clock = begin(Step::PruneCrashReports);
    let outcome = pruned(upkeep_service.prune_crash_reports().await);
    night.record(Step::PruneCrashReports, clock, outcome).await;
    let clock = begin(Step::PruneExpiredSessions);
    let outcome = pruned(upkeep_service.prune_expired_sessions().await);
    night
        .record(Step::PruneExpiredSessions, clock, outcome)
        .await;
    let clock = begin(Step::PruneEmailChanges);
    let outcome = pruned(upkeep_service.prune_expired_email_changes().await);
    night.record(Step::PruneEmailChanges, clock, outcome).await;

    let failures = night.failures;
    if failures == 0 {
        tracing::info!("zervice completed: every step ok");
        if dry_run {
            tracing::info!("dry run: nothing changed, and no healthcheck ping");
        } else {
            ping_healthcheck(config.healthcheck_ping_url.as_deref()).await;
        }
        Ok(())
    } else {
        tracing::error!("zervice completed with {failures} failed step(s), see the errors above");
        anyhow::bail!("zervice finished with {failures} failed step(s)")
    }
}

/// When a step started: the wall time its row stores and a monotonic clock for its duration.
#[derive(Debug, Clone, Copy)]
struct Clock {
    /// The wall time it started.
    started_at: DateTime<Utc>,
    /// The monotonic instant it started.
    instant: Instant,
}

impl Clock {
    /// Milliseconds since the step started.
    fn elapsed_ms(self) -> u64 {
        u64::try_from(self.instant.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
}

/// The night's run: it logs and records each step and tallies the failures.
struct Night {
    /// Where step rows are written.
    cards: card::services::Service<Postgres>,
    /// The run's id, shared by its step rows.
    run_id: Uuid,
    /// Whether the run is a dry run.
    dry_run: bool,
    /// Steps that failed so far.
    failures: u32,
}

impl Night {
    /// Logs `step`'s `outcome`, counts it when it failed, and writes its `zervice_steps` row. A
    /// row that won't write only warns: the step's own outcome stands.
    async fn record(&mut self, step: Step, clock: Clock, outcome: StepOutcome) {
        let name = step.as_str();
        match &outcome {
            StepOutcome::Ok { rows, counts } => {
                tracing::info!("step {name}: ok, rows {rows:?}, {counts:?}");
            }
            StepOutcome::Failed(error) => {
                self.failures = self.failures.saturating_add(1);
                tracing::error!("step {name} FAILED (continuing): {error}");
            }
            StepOutcome::Skipped(why) => tracing::warn!("step {name}: skipped, {why}"),
        }
        let record = StepRecord {
            run_id: self.run_id,
            step,
            started_at: clock.started_at,
            duration_ms: clock.elapsed_ms(),
            outcome,
            dry_run: self.dry_run,
        };
        if let Err(e) = self.cards.record_zervice_step(&record).await {
            tracing::warn!("step {name}: its zervice_steps row did not write: {e:#}");
        }
    }
}

/// Logs that `step` is starting and starts its clock.
fn begin(step: Step) -> Clock {
    tracing::info!("step {}: starting", step.as_str());
    Clock {
        started_at: Utc::now(),
        instant: Instant::now(),
    }
}

/// A step that ran, with its main count and its named counts.
fn done(rows: Option<u64>, counts: &[(&'static str, u64)]) -> StepOutcome {
    StepOutcome::Ok {
        rows,
        counts: counts.iter().copied().collect::<BTreeMap<_, _>>(),
    }
}

/// A step that failed with `error`.
fn failed(error: &anyhow::Error) -> StepOutcome {
    StepOutcome::Failed(format!("{error:#}"))
}

/// A prune's outcome: the rows it deleted (or, on a dry run, would have).
fn pruned(result: anyhow::Result<u64>) -> StepOutcome {
    match result {
        Ok(rows) => done(Some(rows), &[]),
        Err(e) => failed(&e),
    }
}

/// A metrics column's count as a step count; a negative one, which never happens, reads as 0.
fn count(n: i32) -> u64 {
    u64::try_from(n).unwrap_or(0)
}

/// A length as a step count.
fn size(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

/// Dead-man's-switch ping on an all-steps-ok run: the receiving side
/// (healthchecks.io) alerts on SILENCE, covering "never ran at all"; the
/// blind spot `OnFailure=` email can't see (masked timer, wedged box).
/// Skipped when the URL is unset (dev
/// runs stay quiet), and a failed ping only warns: the sync itself succeeded,
/// and a false "zervice is down" email from their side is the correct outcome
/// if the ping can't get out.
async fn ping_healthcheck(url: Option<&str>) {
    let Some(url) = url else {
        return;
    };
    let result = reqwest::Client::new()
        .get(url)
        .timeout(HEALTHCHECK_TIMEOUT)
        .send()
        .await;
    match result {
        Ok(res) if res.status().is_success() => tracing::info!("healthcheck ping: ok"),
        Ok(res) => tracing::warn!("healthcheck ping answered {}", res.status()),
        Err(e) => tracing::warn!("healthcheck ping failed: {e}"),
    }
}
