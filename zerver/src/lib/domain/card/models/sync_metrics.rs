//! Scryfall sync metrics and status tracking.
//!
//! Tracks progress, errors, and outcomes of Scryfall bulk data sync operations.

use anyhow::anyhow;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =======
//  parts
// =======

/// Scryfall sync operation status.
///
/// # Status Evaluation
///
/// - **Success**: All cards upserted, no errors
/// - **PartialSuccess**: ≥70% of cards upserted successfully
/// - **Failure**: <70% success rate or catastrophic errors
/// - **InProgress**: Sync currently running
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    /// Sync completed successfully with no errors.
    Success,
    /// Sync completed but with some errors (≥70% success rate).
    PartialSuccess,
    /// Sync is currently running.
    InProgress,
    /// Sync failed (<70% success rate or critical error).
    Failure,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::PartialSuccess => write!(f, "partial_success"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Failure => write!(f, "failure"),
        }
    }
}

impl TryFrom<&str> for SyncStatus {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "success" => Ok(Self::Success),
            "partial_success" => Ok(Self::PartialSuccess),
            "in_progress" => Ok(Self::InProgress),
            "failure" => Ok(Self::Failure),
            x => Err(anyhow!("failed to parse SyncStatus from {x}")),
        }
    }
}

/// Error details for a failed card insertion during sync.
///
/// Records which card failed, its name, and the error message
/// for debugging and reporting purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Scryfall ID of the card that failed to insert.
    pub card_id: Uuid,
    /// Name of the card (for human-readable error reports).
    pub card_name: String,
    /// Error message describing what went wrong.
    pub error: String,
}

impl ErrorMetrics {
    /// Creates a new error metric entry.
    pub fn new(card_id: Uuid, card_name: impl Into<String>, error: impl Into<String>) -> Self {
        ErrorMetrics {
            card_id,
            card_name: card_name.into(),
            error: error.into(),
        }
    }
}

impl std::fmt::Display for ErrorMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "name: {} | id: {} | error: \"{}\"",
            self.card_name, self.card_id, self.error
        )
    }
}

/// Wrapper for `Vec<ErrorMetrics>` to implement custom traits.
///
/// Provides `Deref` to `Vec<ErrorMetrics>` for easy access while
/// allowing custom serialization and database encoding.
#[derive(Debug, Clone)]
pub struct VecErrorMetrics(Vec<ErrorMetrics>);

impl Serialize for VecErrorMetrics {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for VecErrorMetrics {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Vec::<ErrorMetrics>::deserialize(deserializer).map(VecErrorMetrics)
    }
}

impl std::ops::Deref for VecErrorMetrics {
    type Target = Vec<ErrorMetrics>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ======
//  main
// ======

/// Comprehensive metrics for a Scryfall bulk data sync operation.
///
/// Tracks timing, counts, errors, and final status. Used for:
/// - Monitoring sync progress
/// - Debugging failed syncs
/// - Reporting sync history to users/admins
///
/// # Example
///
/// ```rust,ignore
/// let mut metrics = SyncMetrics::new();
/// metrics.set_received_count(100_000);
/// // ... perform sync ...
/// metrics.add_upserted_count(99_500);
/// metrics.add_error(ErrorMetrics::new(card_id, "Card Name", "Parse error"));
/// metrics.mark_as_completed(); // Evaluates status, calculates duration
/// ```
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyncMetrics {
    /// Final status (Success, PartialSuccess, Failure, InProgress).
    status: SyncStatus,
    /// When sync started (UTC).
    started_at: NaiveDateTime,
    /// When sync ended (UTC, None if still in progress).
    ended_at: Option<NaiveDateTime>,
    /// Total duration in seconds.
    duration_in_seconds: i32,
    /// Total cards received from Scryfall.
    received_count: i32,
    /// Cards successfully inserted/updated.
    upserted_count: i32,
    /// Cards skipped (already up-to-date).
    skipped_count: i32,
    /// Number of errors encountered.
    error_count: i32,
    /// Detailed error information for failed cards.
    errors: Vec<ErrorMetrics>,
}

impl Default for SyncMetrics {
    fn default() -> Self {
        Self {
            status: SyncStatus::InProgress,
            started_at: chrono::Utc::now().naive_utc(),
            ended_at: None,
            duration_in_seconds: 0,
            received_count: 0,
            upserted_count: 0,
            skipped_count: 0,
            error_count: 0,
            errors: Vec::new(),
        }
    }
}

impl SyncMetrics {
    /// Creates a new sync metrics tracker with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the sync status.
    pub fn set_status(&mut self, status: SyncStatus) -> &mut Self {
        self.status = status;
        self
    }

    /// Sets the sync start time.
    pub fn set_started_at(&mut self, started_at: NaiveDateTime) -> &mut Self {
        self.started_at = started_at;
        self
    }

    /// Sets the sync end time.
    pub fn set_ended_at(&mut self, ended_at: Option<NaiveDateTime>) -> &mut Self {
        self.ended_at = ended_at;
        self
    }

    /// Sets the sync duration in seconds.
    pub fn set_duration_in_seconds(&mut self, duration_in_seconds: i32) -> &mut Self {
        self.duration_in_seconds = duration_in_seconds;
        self
    }

    /// Sets the total number of cards received from Scryfall.
    pub fn set_received_count(&mut self, count: i32) -> &mut Self {
        self.received_count = count;
        self
    }

    /// Sets the total number of cards successfully upserted.
    pub fn set_upserted_count(&mut self, count: i32) -> &mut Self {
        self.upserted_count = count;
        self
    }

    /// Increments the upserted count by the specified amount.
    pub fn add_upserted_count(&mut self, count: i32) -> &mut Self {
        self.upserted_count += count;
        self
    }

    /// Sets the total number of cards skipped (already up-to-date).
    pub fn set_skipped_count(&mut self, count: i32) -> &mut Self {
        self.skipped_count = count;
        self
    }

    /// Increments the skipped count by the specified amount.
    pub fn add_skipped_count(&mut self, count: i32) -> &mut Self {
        self.skipped_count += count;
        self
    }

    /// Sets the total error count.
    pub fn set_error_count(&mut self, count: i32) -> &mut Self {
        self.error_count = count;
        self
    }

    /// Adds an error to the metrics and increments error count.
    pub fn add_error(&mut self, error: ErrorMetrics) -> &mut Self {
        self.errors.push(error);
        self.error_count = self.errors.len() as i32;
        self
    }

    /// Sets the complete error list (replacing existing errors).
    pub fn set_errors<I>(&mut self, error_metrics: I) -> &mut Self
    where
        I: IntoIterator<Item = ErrorMetrics>,
    {
        self.errors = error_metrics.into_iter().collect();
        self
    }

    /// Marks sync as completed, evaluates status, and calculates duration.
    ///
    /// Call this when sync finishes (success or failure). This method:
    /// - Updates error_count from errors.len()
    /// - Evaluates final status based on success rate
    /// - Sets ended_at timestamp
    /// - Calculates duration_in_seconds
    pub fn mark_as_completed(&mut self) {
        self.error_count = self.errors.len() as i32;
        self.evaluate_status();
        self.ended_at = Some(chrono::Utc::now().naive_utc());
        if let Some(ended_at) = self.ended_at {
            self.duration_in_seconds = (ended_at - self.started_at).num_seconds() as i32;
        }
    }

    /// Returns when the sync started.
    pub fn started_at(&self) -> NaiveDateTime {
        self.started_at
    }

    /// Returns when the sync ended (None if still in progress).
    pub fn ended_at(&self) -> Option<NaiveDateTime> {
        self.ended_at
    }

    /// Returns sync duration in seconds.
    pub fn duration_in_seconds(&self) -> i32 {
        self.duration_in_seconds
    }

    /// Returns the final sync status.
    pub fn status(&self) -> SyncStatus {
        self.status
    }

    /// Returns total cards received from Scryfall.
    pub fn received_count(&self) -> i32 {
        self.received_count
    }

    /// Returns total cards successfully upserted.
    pub fn upserted_count(&self) -> i32 {
        self.upserted_count
    }

    /// Returns total cards skipped (already up-to-date).
    pub fn skipped_count(&self) -> i32 {
        self.skipped_count
    }

    /// Returns total error count.
    pub fn error_count(&self) -> i32 {
        self.error_count
    }

    /// Returns detailed error information for all failed cards.
    pub fn errors(&self) -> VecErrorMetrics {
        VecErrorMetrics(self.errors.clone())
    }

    /// Evaluates final status based on success rate.
    ///
    /// - **Success**: All intended cards upserted, no errors
    /// - **PartialSuccess**: ≥70% intended cards upserted
    /// - **Failure**: <70% success rate
    ///
    /// Intended cards = received_count - skipped_count
    fn evaluate_status(&mut self) -> &mut Self {
        self.status = SyncStatus::Failure;
        let intended_to_import = self.received_count - self.skipped_count;
        if self.upserted_count as f32 >= intended_to_import as f32 * 0.7 {
            self.status = SyncStatus::PartialSuccess;
        }
        if self.upserted_count == intended_to_import && self.error_count == 0 {
            self.status = SyncStatus::Success;
        }
        self
    }
}
