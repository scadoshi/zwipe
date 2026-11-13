//! An admin's lock on an account: while it stands, sign-in and session refresh refuse the account.

use chrono::{DateTime, Utc};

/// A lock `zerver-admin lock` set, until `zerver-admin unlock` clears it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountLock {
    /// When the account was first locked.
    pub locked_at: DateTime<Utc>,
    /// Why, in the admin's words; `None` when none was given.
    pub reason: Option<String>,
}
