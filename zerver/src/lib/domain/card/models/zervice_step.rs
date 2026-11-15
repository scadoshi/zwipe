//! One recorded zervice step (`zwipe-context ideas/sync_remove_deleted_printings.md`): every step
//! of a nightly run writes a `zervice_steps` row saying when it ran, how long it took, how it
//! ended and what it counted, so the database holds the whole night, not only the card sync.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A step of zervice's nightly run, in the order it runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Step {
    /// Scryfall's `default_cards` into `scryfall_data`, `card_profiles` and `oou_sets`.
    CardSync,
    /// Removing the printings Scryfall deleted.
    PrintingRemoval,
    /// The oracle tag catalog and its card correlations.
    OracleTags,
    /// The `card_profiles.oracle_tags` projection.
    OracleTagProjection,
    /// The oracle tag grouping table.
    OracleTagGrouping,
    /// Card roles from oracle tags, with the straggler merges.
    DeriveCategories,
    /// The `latest_cards` view.
    RefreshLatestCards,
    /// The `card_signal_rollup` view.
    RefreshCardSignalRollup,
    /// The `otag_context_signal_rollup` view.
    RefreshOtagContextSignalRollup,
    /// Client errors past their retention.
    PruneClientErrors,
    /// Crash reports past their retention.
    PruneCrashReports,
    /// Expired and long-replaced refresh tokens.
    PruneExpiredSessions,
    /// Expired email change links.
    PruneEmailChanges,
}

impl Step {
    /// Every step, in run order.
    pub const ALL: [Self; 13] = [
        Self::CardSync,
        Self::PrintingRemoval,
        Self::OracleTags,
        Self::OracleTagProjection,
        Self::OracleTagGrouping,
        Self::DeriveCategories,
        Self::RefreshLatestCards,
        Self::RefreshCardSignalRollup,
        Self::RefreshOtagContextSignalRollup,
        Self::PruneClientErrors,
        Self::PruneCrashReports,
        Self::PruneExpiredSessions,
        Self::PruneEmailChanges,
    ];

    /// The name a `zervice_steps` row stores and `zcripts/metrics/zervice.sql` reads.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CardSync => "card_sync",
            Self::PrintingRemoval => "printing_removal",
            Self::OracleTags => "oracle_tags",
            Self::OracleTagProjection => "oracle_tag_projection",
            Self::OracleTagGrouping => "oracle_tag_grouping",
            Self::DeriveCategories => "derive_categories",
            Self::RefreshLatestCards => "refresh_latest_cards",
            Self::RefreshCardSignalRollup => "refresh_card_signal_rollup",
            Self::RefreshOtagContextSignalRollup => "refresh_otag_context_signal_rollup",
            Self::PruneClientErrors => "prune_client_errors",
            Self::PruneCrashReports => "prune_crash_reports",
            Self::PruneExpiredSessions => "prune_expired_sessions",
            Self::PruneEmailChanges => "prune_email_changes",
        }
    }
}

/// How a step ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepOutcome {
    /// It ran: its main count, when it has one, and its named counts.
    Ok {
        /// The step's main count: printings received, rows written, rows pruned.
        rows: Option<u64>,
        /// Its other counts by name.
        counts: BTreeMap<&'static str, u64>,
    },
    /// It failed, with the error as text.
    Failed(String),
    /// It did not run, and why.
    Skipped(&'static str),
}

impl StepOutcome {
    /// The status a `zervice_steps` row stores.
    #[must_use]
    pub fn status(&self) -> &'static str {
        match self {
            Self::Ok { .. } => "ok",
            Self::Failed(_) => "failed",
            Self::Skipped(_) => "skipped",
        }
    }
}

/// One step of one run, as `zervice_steps` stores it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRecord {
    /// The night's run, shared by all of its steps.
    pub run_id: Uuid,
    /// Which step.
    pub step: Step,
    /// When it started.
    pub started_at: DateTime<Utc>,
    /// How long it took, in milliseconds.
    pub duration_ms: u64,
    /// How it ended.
    pub outcome: StepOutcome,
    /// Whether the run was a dry run, whose steps changed nothing.
    pub dry_run: bool,
}

impl StepRecord {
    /// The named counts as the row's JSON object; a skipped step's reason sits under `skipped`
    /// as text instead.
    #[must_use]
    pub fn counts_json(&self) -> serde_json::Value {
        match &self.outcome {
            StepOutcome::Ok { counts, .. } => serde_json::Value::Object(
                counts
                    .iter()
                    .map(|(name, n)| ((*name).to_owned(), serde_json::Value::from(*n)))
                    .collect(),
            ),
            StepOutcome::Skipped(why) => serde_json::json!({ "skipped": why }),
            StepOutcome::Failed(_) => serde_json::json!({}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_step_has_its_own_snake_case_name_in_run_order() {
        let names: Vec<&str> = Step::ALL.iter().map(|s| s.as_str()).collect();
        assert_eq!(
            names,
            [
                "card_sync",
                "printing_removal",
                "oracle_tags",
                "oracle_tag_projection",
                "oracle_tag_grouping",
                "derive_categories",
                "refresh_latest_cards",
                "refresh_card_signal_rollup",
                "refresh_otag_context_signal_rollup",
                "prune_client_errors",
                "prune_crash_reports",
                "prune_expired_sessions",
                "prune_email_changes",
            ],
            "the names zervice.sql reads"
        );
    }

    #[test]
    fn an_outcome_stores_its_status_and_its_counts() {
        let record = |outcome| StepRecord {
            run_id: Uuid::nil(),
            step: Step::OracleTags,
            started_at: DateTime::UNIX_EPOCH,
            duration_ms: 5,
            outcome,
            dry_run: false,
        };
        let ok = record(StepOutcome::Ok {
            rows: Some(1830),
            counts: BTreeMap::from([("tags", 1830), ("correlations", 41_202)]),
        });
        assert_eq!(ok.outcome.status(), "ok");
        assert_eq!(
            ok.counts_json(),
            serde_json::json!({ "tags": 1830, "correlations": 41_202 })
        );
        let skipped = record(StepOutcome::Skipped("dry run"));
        assert_eq!(skipped.outcome.status(), "skipped");
        assert_eq!(
            skipped.counts_json(),
            serde_json::json!({ "skipped": "dry run" })
        );
        let failed = record(StepOutcome::Failed("boom".to_owned()));
        assert_eq!(failed.outcome.status(), "failed");
        assert_eq!(failed.counts_json(), serde_json::json!({}));
    }
}
