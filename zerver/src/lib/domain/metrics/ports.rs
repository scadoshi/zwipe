//! Port traits for metrics persistence.

use std::future::Future;

use uuid::Uuid;
use zwipe_core::domain::auth::models::platform::ClientPlatform;
use zwipe_core::http::contracts::metrics::{AnonymousEventKind, HttpCrashReport, HttpUsageBatch};

use crate::domain::metrics::models::errors::MetricsError;
use crate::domain::metrics::models::kinds::{AuditAction, EventKind};
use crate::domain::metrics::models::lifetime_counters::LifetimeCounters;
use crate::domain::metrics::models::public_metrics::PublicMetrics;

/// Database port for metrics writes and reads.
pub trait MetricsRepository: Clone + Send + Sync + 'static {
    /// Increments lifetime + daily counters for `user_id` by the batch values,
    /// and the day's totals for the session's `platform` (`None` counts as
    /// unknown).
    fn apply_usage(
        &self,
        user_id: Uuid,
        platform: Option<ClientPlatform>,
        batch: &HttpUsageBatch,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Inserts the lifetime counter row for a new user (idempotent).
    fn insert_lifetime_row(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Increments `decks_created` by one.
    fn increment_decks_created(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Increments `decks_completed` by one.
    fn increment_decks_completed(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Appends a sparse event row.
    fn record_event(
        &self,
        user_id: Uuid,
        kind: EventKind,
        deck_id: Option<Uuid>,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Appends a credential / identity audit row.
    fn record_audit(
        &self,
        user_id: Uuid,
        action: AuditAction,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Appends a pre-auth funnel event row keyed by the client session id.
    fn record_anonymous_event(
        &self,
        session_id: Uuid,
        kind: AnonymousEventKind,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Stores a crash report exactly once (`ON CONFLICT (crash_id) DO NOTHING`).
    fn record_crash(
        &self,
        report: &HttpCrashReport,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Fetches lifetime counters for a user.
    fn lifetime_counters(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<LifetimeCounters, MetricsError>> + Send;

    /// Marks a deck as first-completed if `first_completed_at` is null. Returns
    /// `true` if this call performed the transition (caller should emit the
    /// completion event), `false` if the deck was already marked.
    fn mark_deck_first_completed(
        &self,
        deck_id: Uuid,
    ) -> impl Future<Output = Result<bool, MetricsError>> + Send;

    /// Aggregates lifetime counters across every user. Used for public stats.
    fn public_metrics(&self) -> impl Future<Output = Result<PublicMetrics, MetricsError>> + Send;

    /// Stamps `users.last_active_at` to now.
    fn touch_last_active(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Marks a user as having swiped if `first_swiped_at` is null. Returns
    /// `true` if this call performed the transition (caller should emit the
    /// first-swipe event), `false` if the user was already marked.
    fn mark_user_first_swiped(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<bool, MetricsError>> + Send;
}

/// Service port for metrics business logic.
pub trait MetricsService: Clone + Send + Sync + 'static {
    /// Applies a batched usage update from a client on `platform`.
    fn apply_usage(
        &self,
        user_id: Uuid,
        platform: Option<ClientPlatform>,
        batch: &HttpUsageBatch,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Returns lifetime counters for the caller.
    fn lifetime_counters(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<LifetimeCounters, MetricsError>> + Send;

    /// Fire-and-forget event emission.
    fn record_event(
        &self,
        user_id: Uuid,
        kind: EventKind,
        deck_id: Option<Uuid>,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Fire-and-forget audit emission.
    fn record_audit(
        &self,
        user_id: Uuid,
        action: AuditAction,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Records a pre-auth funnel event from an unauthenticated client.
    fn record_anonymous_event(
        &self,
        session_id: Uuid,
        kind: AnonymousEventKind,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Stores a crash report from an unauthenticated client (exactly once
    /// per `crash_id`: the client retries until it sees a 2xx).
    fn record_crash(
        &self,
        report: &HttpCrashReport,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Initializes the lifetime row for a new user (idempotent).
    fn insert_lifetime_row(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Increments the lifetime `decks_created` counter.
    fn increment_decks_created(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Increments the lifetime `decks_completed` counter.
    fn increment_decks_completed(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Stamps `decks.first_completed_at` if currently null. Returns true if
    /// this call performed the transition (so the caller fires the event).
    fn mark_deck_first_completed(
        &self,
        deck_id: Uuid,
    ) -> impl Future<Output = Result<bool, MetricsError>> + Send;

    /// Aggregates app-wide totals across every user.
    fn public_metrics(&self) -> impl Future<Output = Result<PublicMetrics, MetricsError>> + Send;

    /// Stamps `users.last_active_at` to now.
    fn touch_last_active(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Stamps `users.first_swiped_at` if currently null. Returns true if
    /// this call performed the transition (so the caller fires the event).
    fn mark_user_first_swiped(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<bool, MetricsError>> + Send;
}
