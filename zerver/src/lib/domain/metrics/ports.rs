//! Port traits for metrics persistence.

use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

use crate::domain::metrics::models::{
    errors::MetricsError,
    kinds::{AuditAction, EventKind},
    lifetime_counters::LifetimeCounters,
    public_metrics::PublicMetrics,
};
use zwipe_core::http::contracts::metrics::HttpUsageBatch;

/// Database port for metrics writes and reads.
pub trait MetricsRepository: Clone + Send + Sync + 'static {
    /// Increments lifetime + daily counters for `user_id` by the batch values.
    fn apply_usage(
        &self,
        user_id: Uuid,
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
    fn public_metrics(
        &self,
    ) -> impl Future<Output = Result<PublicMetrics, MetricsError>> + Send;
}

/// Service port for metrics business logic.
pub trait MetricsService: Clone + Send + Sync + 'static {
    /// Applies a batched usage update from the client.
    fn apply_usage(
        &self,
        user_id: Uuid,
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

    /// Initializes the lifetime row for a new user (idempotent).
    fn insert_lifetime_row(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Increments the lifetime decks_created counter.
    fn increment_decks_created(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), MetricsError>> + Send;

    /// Increments the lifetime decks_completed counter.
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
    fn public_metrics(
        &self,
    ) -> impl Future<Output = Result<PublicMetrics, MetricsError>> + Send;
}

/// Object-safe wrapper used by `AppState` so the concrete service type stays
/// out of the generic parameter list. Auto-implemented for any `MetricsService`.
pub trait ErasedMetricsService: Send + Sync + 'static {
    /// See [`MetricsService::apply_usage`].
    fn apply_usage<'a>(
        &'a self,
        user_id: Uuid,
        batch: &'a HttpUsageBatch,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetricsError>> + Send + 'a>>;

    /// See [`MetricsService::lifetime_counters`].
    fn lifetime_counters<'a>(
        &'a self,
        user_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<LifetimeCounters, MetricsError>> + Send + 'a>>;

    /// See [`MetricsService::record_event`].
    fn record_event<'a>(
        &'a self,
        user_id: Uuid,
        kind: EventKind,
        deck_id: Option<Uuid>,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetricsError>> + Send + 'a>>;

    /// See [`MetricsService::record_audit`].
    fn record_audit<'a>(
        &'a self,
        user_id: Uuid,
        action: AuditAction,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetricsError>> + Send + 'a>>;

    /// See [`MetricsService::insert_lifetime_row`].
    fn insert_lifetime_row<'a>(
        &'a self,
        user_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetricsError>> + Send + 'a>>;

    /// See [`MetricsService::increment_decks_created`].
    fn increment_decks_created<'a>(
        &'a self,
        user_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetricsError>> + Send + 'a>>;

    /// See [`MetricsService::increment_decks_completed`].
    fn increment_decks_completed<'a>(
        &'a self,
        user_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetricsError>> + Send + 'a>>;

    /// See [`MetricsService::mark_deck_first_completed`].
    fn mark_deck_first_completed<'a>(
        &'a self,
        deck_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<bool, MetricsError>> + Send + 'a>>;

    /// See [`MetricsService::public_metrics`].
    fn public_metrics<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<PublicMetrics, MetricsError>> + Send + 'a>>;
}

impl<T> ErasedMetricsService for T
where
    T: MetricsService,
{
    fn apply_usage<'a>(
        &'a self,
        user_id: Uuid,
        batch: &'a HttpUsageBatch,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetricsError>> + Send + 'a>> {
        Box::pin(MetricsService::apply_usage(self, user_id, batch))
    }

    fn lifetime_counters<'a>(
        &'a self,
        user_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<LifetimeCounters, MetricsError>> + Send + 'a>> {
        Box::pin(MetricsService::lifetime_counters(self, user_id))
    }

    fn record_event<'a>(
        &'a self,
        user_id: Uuid,
        kind: EventKind,
        deck_id: Option<Uuid>,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetricsError>> + Send + 'a>> {
        Box::pin(MetricsService::record_event(self, user_id, kind, deck_id))
    }

    fn record_audit<'a>(
        &'a self,
        user_id: Uuid,
        action: AuditAction,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetricsError>> + Send + 'a>> {
        Box::pin(MetricsService::record_audit(self, user_id, action))
    }

    fn insert_lifetime_row<'a>(
        &'a self,
        user_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetricsError>> + Send + 'a>> {
        Box::pin(MetricsService::insert_lifetime_row(self, user_id))
    }

    fn increment_decks_created<'a>(
        &'a self,
        user_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetricsError>> + Send + 'a>> {
        Box::pin(MetricsService::increment_decks_created(self, user_id))
    }

    fn increment_decks_completed<'a>(
        &'a self,
        user_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetricsError>> + Send + 'a>> {
        Box::pin(MetricsService::increment_decks_completed(self, user_id))
    }

    fn mark_deck_first_completed<'a>(
        &'a self,
        deck_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<bool, MetricsError>> + Send + 'a>> {
        Box::pin(MetricsService::mark_deck_first_completed(self, deck_id))
    }

    fn public_metrics<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<PublicMetrics, MetricsError>> + Send + 'a>> {
        Box::pin(MetricsService::public_metrics(self))
    }
}
