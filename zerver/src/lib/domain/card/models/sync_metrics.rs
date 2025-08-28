// std
use std::{fmt::Display, ops::Deref};
// external
use anyhow::anyhow;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =======
//  parts
// =======

/// represents the type of sync which occured
/// in this entry of SyncMetrics
///
///  - **Partial**: incremental addition of new cards
/// that do not exist in database
/// - **Full**: comprehensive refresh of all given cards
/// even if already exists in database
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncType {
    Partial,
    Full,
}

impl Display for SyncType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Partial => write!(f, "partial"),
            Self::Full => write!(f, "full"),
        }
    }
}

impl TryFrom<&str> for SyncType {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "partial" => Ok(Self::Partial),
            "full" => Ok(Self::Full),
            x => Err(anyhow!("failed to parse SyncType from {x}")),
        }
    }
}

/// for tracking sync status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Success,
    PartialSuccess,
    InProgress,
    Failure,
}

impl Display for SyncStatus {
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

/// tracks and persists error metrics for
/// errors encountered while inserting card data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub card_id: Uuid,
    pub card_name: String,
    pub error: String,
}

impl ErrorMetrics {
    pub fn new(card_id: Uuid, card_name: &str, error: &str) -> Self {
        ErrorMetrics {
            card_id,
            card_name: card_name.to_string(),
            error: error.to_string(),
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

/// wrapped version of `Vec<ErrorMetrics>`
/// so we can implement the likes of encode, decode, type
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

impl Deref for VecErrorMetrics {
    type Target = Vec<ErrorMetrics>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ======
//  main
// ======

/// stores metrics about a scryfall database card sync
///
/// keeping fields private for more controlled design pattern
#[derive(Debug, Deserialize, Serialize)]
pub struct SyncMetrics {
    sync_type: SyncType,
    status: SyncStatus,
    started_at: NaiveDateTime,
    ended_at: Option<NaiveDateTime>,
    duration_in_seconds: i32,
    received: i32,
    imported: i32,
    skipped: i32,
    error_count: i32,
    errors: Vec<ErrorMetrics>,
}

impl SyncMetrics {
    /// constructs SyncMetrics
    /// with sensible defaults
    pub fn generate(sync_type: SyncType) -> Self {
        Self {
            sync_type,
            status: SyncStatus::InProgress,
            started_at: chrono::Utc::now().naive_utc(),
            ended_at: None,
            duration_in_seconds: 0,
            received: 0,
            imported: 0,
            skipped: 0,
            error_count: 0,
            errors: Vec::new(),
        }
    }

    /// constructs SyncMetrics
    /// out of raw types
    pub fn new(
        sync_type: SyncType,
        started_at: NaiveDateTime,
        ended_at: Option<NaiveDateTime>,
        duration_in_seconds: i32,
        status: SyncStatus,
        received: i32,
        imported: i32,
        skipped: i32,
        error_count: i32,
        errors: Vec<ErrorMetrics>,
    ) -> Self {
        Self {
            sync_type,
            started_at,
            ended_at,
            duration_in_seconds,
            status,
            received,
            imported,
            skipped,
            error_count,
            errors,
        }
    }

    // for mutating SyncMetrics
    pub fn set_received(&mut self, count: i32) {
        self.received = count;
    }

    pub fn set_imported(&mut self, count: i32) {
        self.imported = count;
    }

    pub fn add_imported(&mut self, count: i32) {
        self.imported += count;
    }

    pub fn set_skipped(&mut self, count: i32) {
        self.skipped = count;
    }

    pub fn add_skipped(&mut self, count: i32) {
        self.skipped += count;
    }

    pub fn add_error(&mut self, error: ErrorMetrics) {
        self.errors.push(error);
        self.error_count = self.errors.len() as i32;
    }

    pub fn mark_as_completed(&mut self) {
        self.error_count = self.errors.len() as i32;

        self.evaluate_status();

        self.ended_at = Some(chrono::Utc::now().naive_utc());
        if let Some(ended_at) = self.ended_at {
            self.duration_in_seconds = (ended_at - self.started_at).num_seconds() as i32;
        }
    }

    // for getting from SyncMetrics
    pub fn sync_type(&self) -> SyncType {
        self.sync_type.clone()
    }
    pub fn started_at(&self) -> NaiveDateTime {
        self.started_at.clone()
    }
    pub fn ended_at(&self) -> Option<NaiveDateTime> {
        self.ended_at.clone()
    }
    pub fn duration_in_seconds(&self) -> i32 {
        self.duration_in_seconds
    }
    pub fn status(&self) -> SyncStatus {
        self.status.clone()
    }
    pub fn received(&self) -> i32 {
        self.received
    }
    pub fn imported(&self) -> i32 {
        self.imported
    }
    pub fn skipped(&self) -> i32 {
        self.skipped
    }
    pub fn error_count(&self) -> i32 {
        self.error_count
    }
    pub fn errors(&self) -> VecErrorMetrics {
        VecErrorMetrics(self.errors.clone())
    }

    // helpers
    fn evaluate_status(&mut self) {
        self.status = SyncStatus::Failure;

        let intended_to_import = self.received - self.skipped;

        if self.imported as f32 >= intended_to_import as f32 * 0.7 && self.error_count > 0 {
            self.status = SyncStatus::PartialSuccess;
        }

        if self.imported >= intended_to_import && self.error_count == 0 {
            self.status = SyncStatus::Success;
        }
    }
}
