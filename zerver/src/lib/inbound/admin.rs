//! Account changes made from the server only, for `zerver-admin`. Nothing in the app reaches them.
//!
//! Each command takes the account `AuthService::fetch_account` found, goes through the services
//! the app's requests use, and writes an `admin_action_log` row naming the account by id, which
//! outlives the account. A change the account's own history should show also writes its
//! `user_audit_log` row. A failed log row is logged and never undoes the change.

use uuid::Uuid;
use zwipe_core::domain::Email;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::username::Username;

use crate::domain::auth::ports::AuthService;
use crate::domain::auth::requests::admin_change_email::AdminChangeEmailError;
use crate::domain::auth::requests::change_username::ChangeUsernameError;
use crate::domain::auth::requests::delete_user::DeleteUserError;
use crate::domain::auth::requests::revoke_sessions::{RevokeSessions, RevokeSessionsError};
use crate::domain::email::models::SendEmailError;
use crate::domain::metrics::models::kinds::{AdminAction, AuditAction};
use crate::domain::metrics::ports::MetricsService;
use crate::domain::user::models::account_summary::AccountSummary;
use crate::domain::user::models::get_user::GetUserError;
use crate::domain::user::ports::UserService;

/// Whether a rename tells the account, by email, that its old name is now reserved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameNotice {
    /// Sends the notice, with replies going to `reply_to`.
    Send {
        /// Where a reply goes: the server's support address unless the command names another.
        reply_to: Email,
    },
    /// Sends nothing.
    Skip,
}

/// What an email a command sends did.
#[derive(Debug)]
pub enum NoticeOutcome {
    /// The email went out.
    Sent,
    /// The command asked for none.
    Skipped,
    /// The email failed; the change stands.
    Failed(SendEmailError),
}

impl From<Result<(), SendEmailError>> for NoticeOutcome {
    fn from(sent: Result<(), SendEmailError>) -> Self {
        match sent {
            Ok(()) => Self::Sent,
            Err(e) => Self::Failed(e),
        }
    }
}

/// What [`rename_account`] did.
#[derive(Debug)]
pub struct Renamed {
    /// The account's name before.
    pub old_username: Username,
    /// The account as it is now.
    pub user: User,
    /// What happened to the notice.
    pub notice: NoticeOutcome,
}

/// What [`change_email`] did.
#[derive(Debug)]
pub struct EmailSwitched {
    /// The account's address before.
    pub old_email: Email,
    /// The account as it is now, its new address unverified.
    pub user: User,
    /// The verification email to the new address.
    pub verification: NoticeOutcome,
    /// The notice to the old address.
    pub notice: NoticeOutcome,
}

/// Records `action` in `user_id`'s own audit log, logging a failure.
async fn audit(metrics: &impl MetricsService, user_id: Uuid, action: AuditAction) {
    if let Err(e) = metrics.record_audit(user_id, action).await {
        tracing::warn!(error = ?e, %action, "admin: audit row failed");
    }
}

/// Records that `action` ran against `account_id` in the `admin_action_log`, logging a failure.
async fn log_action(metrics: &impl MetricsService, account_id: Uuid, action: AdminAction) {
    if let Err(e) = metrics.record_admin_action(account_id, action).await {
        tracing::warn!(error = ?e, %action, "admin: action log row failed");
    }
}

/// Renames `user` to `new_username` the way a user's own change does, minus the password and the
/// reserved-name check, records the `username_changed` audit row and the `rename` action, then
/// sends or skips the reserved-name notice.
///
/// # Errors
///
/// The [`ChangeUsernameError`] from the rename: the same name, a name another account holds in
/// any case, or a database failure. A failed notice comes back as [`NoticeOutcome::Failed`],
/// since the rename has gone through by then.
pub async fn rename_account(
    auth: &impl AuthService,
    metrics: &impl MetricsService,
    user: &User,
    new_username: &Username,
    notice: RenameNotice,
) -> Result<Renamed, ChangeUsernameError> {
    let renamed = auth.admin_change_username(user, new_username).await?;
    audit(metrics, renamed.id, AuditAction::UsernameChanged).await;
    log_action(metrics, renamed.id, AdminAction::Rename).await;
    let notice = match notice {
        RenameNotice::Skip => NoticeOutcome::Skipped,
        RenameNotice::Send { reply_to } => auth
            .send_reserved_name_notice(&renamed, &user.username, &reply_to)
            .await
            .into(),
    };
    Ok(Renamed {
        old_username: user.username.clone(),
        user: renamed,
        notice,
    })
}

/// `user` at a glance: whether it is locked, when it was made and last active, its deck count and
/// its latest audit events. Changes nothing, and records the `show` action.
///
/// # Errors
///
/// [`GetUserError::NotFound`] when the account is gone, and [`GetUserError::Database`] on a
/// database failure.
pub async fn show_account(
    users: &impl UserService,
    metrics: &impl MetricsService,
    user: &User,
) -> Result<AccountSummary, GetUserError> {
    let summary = users.fetch_account_summary(user.id).await?;
    log_action(metrics, user.id, AdminAction::Show).await;
    Ok(summary)
}

/// Locks `user`: sign-in answers it as the wrong-password lockout does and its sessions stop
/// refreshing, until [`unlock_account`]. Records the `account_locked` audit row and the `lock`
/// action.
///
/// # Errors
///
/// A database failure.
pub async fn lock_account(
    auth: &impl AuthService,
    metrics: &impl MetricsService,
    user: &User,
    reason: Option<&str>,
) -> anyhow::Result<()> {
    auth.lock_account(user.id, reason).await?;
    audit(metrics, user.id, AuditAction::AccountLocked).await;
    log_action(metrics, user.id, AdminAction::Lock).await;
    Ok(())
}

/// Clears `user`'s lock, recording the `account_unlocked` audit row and the `unlock` action. A
/// running zerver's lockout for wrong passwords is its own and expires on its own.
///
/// # Errors
///
/// A database failure.
pub async fn unlock_account(
    auth: &impl AuthService,
    metrics: &impl MetricsService,
    user: &User,
) -> anyhow::Result<()> {
    auth.unlock_account(user.id).await?;
    audit(metrics, user.id, AuditAction::AccountUnlocked).await;
    log_action(metrics, user.id, AdminAction::Unlock).await;
    Ok(())
}

/// Revokes every refresh token `user` holds, so each device must sign in again once its access
/// token expires, and records the `signed_out_everywhere` audit row and the `sign_out_all`
/// action.
///
/// # Errors
///
/// The [`RevokeSessionsError`] on a database failure.
pub async fn sign_out_all(
    auth: &impl AuthService,
    metrics: &impl MetricsService,
    user: &User,
) -> Result<(), RevokeSessionsError> {
    auth.revoke_sessions(&RevokeSessions::new(user.id)).await?;
    audit(metrics, user.id, AuditAction::SignedOutEverywhere).await;
    log_action(metrics, user.id, AdminAction::SignOutAll).await;
    Ok(())
}

/// Why [`resend_verification`] sent nothing.
#[derive(Debug, thiserror::Error)]
pub enum ResendVerificationError {
    /// The account's email is verified already, as the app refuses too.
    #[error("the email is already verified")]
    AlreadyVerified,
    /// Storing the token or sending the email failed.
    #[error(transparent)]
    Send(#[from] anyhow::Error),
}

/// Sends `user` a fresh verification email the way the app's resend does, replacing any earlier
/// link, and records the `verification_email_sent` audit row and the `resend_verification`
/// action.
///
/// # Errors
///
/// [`ResendVerificationError::AlreadyVerified`] when the email is verified, and
/// [`ResendVerificationError::Send`] when the token or the email fails.
pub async fn resend_verification(
    auth: &impl AuthService,
    metrics: &impl MetricsService,
    user: &User,
) -> Result<(), ResendVerificationError> {
    if user.email_verified_at.is_some() {
        return Err(ResendVerificationError::AlreadyVerified);
    }
    auth.send_verification_email(user.id, user.email.as_ref())
        .await?;
    audit(metrics, user.id, AuditAction::VerificationEmailSent).await;
    log_action(metrics, user.id, AdminAction::ResendVerification).await;
    Ok(())
}

/// Deletes `user` and everything it owns, the way the user's own delete does but without the
/// password. Its audit rows go with it; the `delete` action stays in the `admin_action_log`.
///
/// # Errors
///
/// [`DeleteUserError::NotFound`] when the account is gone, and [`DeleteUserError::Database`] on a
/// database failure.
pub async fn delete_account(
    auth: &impl AuthService,
    metrics: &impl MetricsService,
    user: &User,
) -> Result<(), DeleteUserError> {
    auth.admin_delete_user(user.id).await?;
    tracing::info!(event = "admin_account_deleted", user_id = %user.id);
    log_action(metrics, user.id, AdminAction::Delete).await;
    Ok(())
}

/// Switches `user` to `new_email` at once: the same taken check as the user's own change, the new
/// address unverified and sent a verification email the way registration does, and the old
/// address told, with replies going to `reply_to`. Records the `email_changed` audit row and the
/// `change_email` action.
///
/// # Errors
///
/// [`AdminChangeEmailError::Unchanged`] when it already has the address,
/// [`AdminChangeEmailError::Taken`] when another account has it, or a database failure. A failed
/// email comes back in [`EmailSwitched`], since the switch has gone through by then.
pub async fn change_email(
    auth: &impl AuthService,
    metrics: &impl MetricsService,
    user: &User,
    new_email: &Email,
    reply_to: &Email,
) -> Result<EmailSwitched, AdminChangeEmailError> {
    let switched = auth.admin_change_email(user, new_email).await?;
    audit(metrics, switched.id, AuditAction::EmailChanged).await;
    log_action(metrics, switched.id, AdminAction::ChangeEmail).await;
    let verification = auth
        .send_verification_email(switched.id, switched.email.as_ref())
        .await
        .map_err(SendEmailError::Network)
        .into();
    let notice = auth
        .send_email_changed_notice(&switched, &user.email, reply_to)
        .await
        .into();
    Ok(EmailSwitched {
        old_email: user.email.clone(),
        user: switched,
        verification,
        notice,
    })
}
