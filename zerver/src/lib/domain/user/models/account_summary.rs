//! What `zerver-admin show` prints about an account.

use chrono::{DateTime, Utc};
use zwipe_core::domain::user::User;

/// How many audit events an account summary carries, newest first.
pub const RECENT_AUDIT_EVENTS: i64 = 10;

/// One account at a glance, for an admin on the server.
#[derive(Debug, Clone)]
pub struct AccountSummary {
    /// The account: username, email, whether it is verified, a pending email change.
    pub user: User,
    /// When the account was made.
    pub created_at: DateTime<Utc>,
    /// The last authenticated request it made; `None` before its first.
    pub last_active_at: Option<DateTime<Utc>>,
    /// How many decks it owns.
    pub deck_count: i64,
    /// Its latest audit events, newest first, at most [`RECENT_AUDIT_EVENTS`].
    pub recent_audit: Vec<AuditEntry>,
}

/// One row of an account's audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    /// The action as stored, such as `username_changed`.
    pub action: String,
    /// When it happened.
    pub occurred_at: DateTime<Utc>,
}
