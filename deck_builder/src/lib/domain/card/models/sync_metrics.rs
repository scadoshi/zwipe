use std::fmt::Display;

use anyhow::anyhow;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ===================================
//              parts (*3*)
// ===================================

/// represents the type of sync which occured
/// in this entry of SyncMetrics
///
///  - Partial: incremental addition of only cards
/// the database did not already have
/// - Full: comprehensive refresh of all cards
/// supplied to sync function (involves
/// removing all cards from database from
/// given data and then inserting back in)
///     - intention is to ensure card data
///     in database is up to date in case
///     anything in scryfall changes
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

/// high level status of the sync
///
/// technically in progress only ever lives in memory
/// (at least i dont intend on persisting that status)
/// but need something to set to while it is running in memory
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

/// tracks error specifics while syncing cards
/// not your typical rust error type
/// meant more to contain information so we can narrow down what happened in post
/// hopefully will never need this XD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub card_id: Uuid,
    pub card_name: String,
    pub error: String,
}

impl ErrorMetrics {
    pub fn new(card_id: Uuid, card_name: String, error: String) -> Self {
        Self {
            card_id,
            card_name,
            error,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ErrorMetricsVec(Vec<ErrorMetrics>);

impl Serialize for ErrorMetricsVec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ErrorMetricsVec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Vec::<ErrorMetrics>::deserialize(deserializer).map(ErrorMetricsVec)
    }
}

// ====================================
//            main
// ====================================

/// stores metrics about a scryfall-database sync
///
/// not going to pub the fields and
/// instead going to build helper functions
/// to update in a more controlled manner
/// design patterns woot
#[derive(Debug)]
pub struct SyncMetrics {
    sync_type: SyncType,
    started_at: NaiveDateTime,
    ended_at: Option<NaiveDateTime>,
    duration_in_seconds: i64,
    status: SyncStatus,
    total_cards_count: i32,
    imported_cards_count: i32,
    skipped_cards_count: i32,
    error_count: i32,
    errors: Vec<ErrorMetrics>,
}

impl SyncMetrics {
    // =============
    //  constructor
    // =============

    /// sets up a new instance of SyncMetrics
    /// with sensible defaults
    pub fn new(sync_type: SyncType) -> Self {
        Self {
            sync_type,
            started_at: chrono::Utc::now().naive_utc(),
            ended_at: None,
            duration_in_seconds: 0,
            status: SyncStatus::InProgress,
            total_cards_count: 0,
            imported_cards_count: 0,
            skipped_cards_count: 0,
            error_count: 0,
            errors: Vec::new(),
        }
    }

    // =============
    //  adjustors
    // =============
    pub fn set_total_cards_count(&mut self, count: i32) {
        self.total_cards_count = count;
    }

    pub fn set_imported_cards_count(&mut self, count: i32) {
        self.imported_cards_count = count;
    }

    pub fn add_imported_cards_count(&mut self, count: i32) {
        self.imported_cards_count += count;
    }

    pub fn set_skipped_count(&mut self, count: i32) {
        self.skipped_cards_count = count;
    }

    pub fn add_skipped_count(&mut self, count: i32) {
        self.skipped_cards_count += count;
    }

    pub fn add_error(&mut self, error: ErrorMetrics) {
        self.errors.push(error);
        self.error_count = self.errors.len() as i32;
    }

    pub fn mark_as_completed(&mut self) {
        // set error count based on what we have collected
        self.error_count = self.errors.len() as i32;

        // report failure if no cards were brought in and errors were encountered
        if self.total_cards_count > 0 && self.imported_cards_count == 0 && self.error_count > 0 {
            self.status = SyncStatus::Failure;
        }
        // report partial if any errors were received
        if self.error_count > 0 {
            self.status = SyncStatus::PartialSuccess;
        } else {
            // if no errors report success
            self.status = SyncStatus::Success;
        }

        // add ended at and set duration
        self.ended_at = Some(chrono::Utc::now().naive_utc());
        if let Some(ended_at) = self.ended_at {
            self.duration_in_seconds = (ended_at - self.started_at).num_seconds();
        }
    }

    // ===============================
    //  getters - they clone for now
    // ===============================

    pub fn sync_type(&self) -> SyncType {
        self.sync_type.clone()
    }
    pub fn started_at(&self) -> NaiveDateTime {
        self.started_at.clone()
    }
    pub fn ended_at(&self) -> Option<NaiveDateTime> {
        self.ended_at.clone()
    }
    pub fn duration_in_seconds(&self) -> i64 {
        self.duration_in_seconds
    }
    pub fn status(&self) -> SyncStatus {
        self.status.clone()
    }
    pub fn total_cards_count(&self) -> i32 {
        self.total_cards_count
    }
    pub fn imported_cards_count(&self) -> i32 {
        self.imported_cards_count
    }
    pub fn skipped_cards_count(&self) -> i32 {
        self.skipped_cards_count
    }
    pub fn error_count(&self) -> i32 {
        self.error_count
    }
    pub fn errors(&self) -> ErrorMetricsVec {
        ErrorMetricsVec(self.errors.clone())
    }
}
