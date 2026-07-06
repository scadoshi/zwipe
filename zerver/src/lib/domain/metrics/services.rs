//! Service implementation for the metrics domain.

use uuid::Uuid;

use crate::domain::metrics::{
    models::{
        errors::MetricsError,
        kinds::{AuditAction, EventKind},
        lifetime_counters::LifetimeCounters,
        public_metrics::PublicMetrics,
    },
    ports::{MetricsRepository, MetricsService},
};
use zwipe_core::http::contracts::metrics::{AnonymousEventKind, HttpUsageBatch};

/// Default metrics service.
#[derive(Debug, Clone)]
pub struct Service<R>
where
    R: MetricsRepository,
{
    repo: R,
}

impl<R> Service<R>
where
    R: MetricsRepository,
{
    /// Creates a new metrics service with the provided repository.
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: MetricsRepository> MetricsService for Service<R> {
    async fn apply_usage(&self, user_id: Uuid, batch: &HttpUsageBatch) -> Result<(), MetricsError> {
        self.repo.apply_usage(user_id, batch).await
    }

    async fn lifetime_counters(&self, user_id: Uuid) -> Result<LifetimeCounters, MetricsError> {
        self.repo.lifetime_counters(user_id).await
    }

    async fn record_event(
        &self,
        user_id: Uuid,
        kind: EventKind,
        deck_id: Option<Uuid>,
    ) -> Result<(), MetricsError> {
        self.repo.record_event(user_id, kind, deck_id).await
    }

    async fn record_audit(&self, user_id: Uuid, action: AuditAction) -> Result<(), MetricsError> {
        self.repo.record_audit(user_id, action).await
    }

    async fn record_anonymous_event(
        &self,
        session_id: Uuid,
        kind: AnonymousEventKind,
    ) -> Result<(), MetricsError> {
        self.repo.record_anonymous_event(session_id, kind).await
    }

    async fn insert_lifetime_row(&self, user_id: Uuid) -> Result<(), MetricsError> {
        self.repo.insert_lifetime_row(user_id).await
    }

    async fn increment_decks_created(&self, user_id: Uuid) -> Result<(), MetricsError> {
        self.repo.increment_decks_created(user_id).await
    }

    async fn increment_decks_completed(&self, user_id: Uuid) -> Result<(), MetricsError> {
        self.repo.increment_decks_completed(user_id).await
    }

    async fn mark_deck_first_completed(&self, deck_id: Uuid) -> Result<bool, MetricsError> {
        self.repo.mark_deck_first_completed(deck_id).await
    }

    async fn public_metrics(&self) -> Result<PublicMetrics, MetricsError> {
        self.repo.public_metrics().await
    }

    async fn touch_last_active(&self, user_id: Uuid) -> Result<(), MetricsError> {
        self.repo.touch_last_active(user_id).await
    }

    async fn mark_user_first_swiped(&self, user_id: Uuid) -> Result<bool, MetricsError> {
        self.repo.mark_user_first_swiped(user_id).await
    }
}
