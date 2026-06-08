//! Enumerations for the sparse event log and audit log.

use std::fmt;

/// Rare, durably-logged user events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// User completed signup.
    Signup,
    /// User created a new deck.
    DeckCreated,
    /// A deck first reached a valid state.
    DeckCompleted,
    /// User's first swipe ever (any direction).
    FirstSwipe,
}

impl EventKind {
    /// String form stored in the `user_events.kind` column.
    pub fn as_str(self) -> &'static str {
        match self {
            EventKind::Signup => "signup",
            EventKind::DeckCreated => "deck_created",
            EventKind::DeckCompleted => "deck_completed",
            EventKind::FirstSwipe => "first_swipe",
        }
    }
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Credential / identity changes recorded for audit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    /// Username was changed.
    UsernameChanged,
    /// Email was changed.
    EmailChanged,
    /// Password was changed or reset.
    PasswordChanged,
}

impl AuditAction {
    /// String form stored in the `user_audit_log.action` column.
    pub fn as_str(self) -> &'static str {
        match self {
            AuditAction::UsernameChanged => "username_changed",
            AuditAction::EmailChanged => "email_changed",
            AuditAction::PasswordChanged => "password_changed",
        }
    }
}

impl fmt::Display for AuditAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
