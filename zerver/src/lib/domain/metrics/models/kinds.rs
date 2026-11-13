//! Enumerations for the sparse event log and audit log.

use std::fmt;

/// Rare, durably-logged user events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// User completed registration.
    Register,
    /// User logged in.
    Login,
    /// User refreshed their session.
    Refresh,
    /// User logged out (revoked their sessions).
    Logout,
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
            EventKind::Register => "register",
            EventKind::Login => "login",
            EventKind::Refresh => "refresh",
            EventKind::Logout => "logout",
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
    /// A switch to a new email was requested and is pending confirmation.
    EmailChangeRequested,
    /// Password was changed or reset.
    PasswordChanged,
    /// User logged in.
    Login,
    /// User refreshed their session.
    Refresh,
    /// User logged out (revoked their sessions).
    Logout,
    /// An admin revoked every session the account holds.
    SignedOutEverywhere,
    /// An admin sent the account a fresh verification email.
    VerificationEmailSent,
    /// An admin locked the account against sign-in.
    AccountLocked,
    /// An admin unlocked the account.
    AccountUnlocked,
}

impl AuditAction {
    /// String form stored in the `user_audit_log.action` column.
    pub fn as_str(self) -> &'static str {
        match self {
            AuditAction::UsernameChanged => "username_changed",
            AuditAction::EmailChanged => "email_changed",
            AuditAction::EmailChangeRequested => "email_change_requested",
            AuditAction::PasswordChanged => "password_changed",
            AuditAction::Login => "login",
            AuditAction::Refresh => "refresh",
            AuditAction::Logout => "logout",
            AuditAction::SignedOutEverywhere => "signed_out_everywhere",
            AuditAction::VerificationEmailSent => "verification_email_sent",
            AuditAction::AccountLocked => "account_locked",
            AuditAction::AccountUnlocked => "account_unlocked",
        }
    }
}

impl fmt::Display for AuditAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A `zerver-admin` command that ran against an account, for the `admin_action_log`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminAction {
    /// `show` read the account.
    Show,
    /// `rename` renamed it.
    Rename,
    /// `lock` locked it.
    Lock,
    /// `unlock` unlocked it.
    Unlock,
    /// `sign-out-all` revoked its sessions.
    SignOutAll,
    /// `resend-verification` mailed it a verify link.
    ResendVerification,
    /// `delete` deleted it.
    Delete,
    /// `change-email` switched its address.
    ChangeEmail,
}

impl AdminAction {
    /// String form stored in the `admin_action_log.action` column.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            AdminAction::Show => "show",
            AdminAction::Rename => "rename",
            AdminAction::Lock => "lock",
            AdminAction::Unlock => "unlock",
            AdminAction::SignOutAll => "sign_out_all",
            AdminAction::ResendVerification => "resend_verification",
            AdminAction::Delete => "delete",
            AdminAction::ChangeEmail => "change_email",
        }
    }
}

impl fmt::Display for AdminAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
