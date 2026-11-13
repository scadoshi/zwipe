//! Account changes made from the server only, for `zerver-admin`. Nothing in the app reaches them.

use zwipe_core::domain::Email;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::username::Username;

use crate::domain::auth::models::account_ref::AccountRef;
use crate::domain::auth::ports::AuthService;
use crate::domain::auth::requests::change_username::ChangeUsernameError;
use crate::domain::auth::requests::delete_user::DeleteUserError;
use crate::domain::auth::requests::revoke_sessions::{RevokeSessions, RevokeSessionsError};
use crate::domain::email::models::SendEmailError;
use crate::domain::metrics::models::kinds::AuditAction;
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

/// What a notice did.
#[derive(Debug)]
pub enum NoticeOutcome {
    /// The email went out.
    Sent,
    /// The rename asked for none.
    Skipped,
    /// The email failed; the rename stands.
    Failed(SendEmailError),
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

/// Renames the account `account` names (by exact username, id or exact email) to `new_username` the way a
/// user's own change does, minus the password and the reserved-name check, records the
/// `username_changed` audit row, then sends or skips the reserved-name notice.
///
/// # Errors
///
/// The [`ChangeUsernameError`] from the rename: no such account, the same name, a name another
/// account holds in any case, or a database failure. A failed audit row is logged and a failed
/// notice comes back as [`NoticeOutcome::Failed`], since the rename has gone through by then.
pub async fn rename_account(
    auth: &impl AuthService,
    metrics: &impl MetricsService,
    account: &AccountRef,
    new_username: &Username,
    notice: RenameNotice,
) -> Result<Renamed, ChangeUsernameError> {
    let (old_username, user) = auth.admin_change_username(account, new_username).await?;
    audit(metrics, user.id, AuditAction::UsernameChanged).await;
    let notice = match notice {
        RenameNotice::Skip => NoticeOutcome::Skipped,
        RenameNotice::Send { reply_to } => match auth
            .send_reserved_name_notice(&user, &old_username, &reply_to)
            .await
        {
            Ok(()) => NoticeOutcome::Sent,
            Err(e) => NoticeOutcome::Failed(e),
        },
    };
    Ok(Renamed {
        old_username,
        user,
        notice,
    })
}

/// Records `action` for `user_id`, logging a failure: the change it audits has gone through.
async fn audit(metrics: &impl MetricsService, user_id: uuid::Uuid, action: AuditAction) {
    if let Err(e) = metrics.record_audit(user_id, action).await {
        tracing::warn!(error = ?e, %action, "admin: audit row failed");
    }
}

/// `user` at a glance: when it was made and last active, its deck count and its latest audit
/// events. Reads only.
///
/// # Errors
///
/// [`GetUserError::NotFound`] when the account is gone, and [`GetUserError::Database`] on a
/// database failure.
pub async fn show_account(
    users: &impl UserService,
    user: &User,
) -> Result<AccountSummary, GetUserError> {
    users.fetch_account_summary(user.id).await
}

/// Revokes every refresh token `user` holds, so each device must sign in again once its access
/// token expires, and records the `signed_out_everywhere` audit row.
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
/// link, and records the `verification_email_sent` audit row.
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
    Ok(())
}

/// Deletes `user` and everything it owns, the way the user's own delete does but without the
/// password. Its audit rows go with it, so the delete is recorded in the server log.
///
/// # Errors
///
/// [`DeleteUserError::NotFound`] when the account is gone, and [`DeleteUserError::Database`] on a
/// database failure.
pub async fn delete_account(auth: &impl AuthService, user: &User) -> Result<(), DeleteUserError> {
    auth.admin_delete_user(user.id).await?;
    tracing::info!(event = "admin_account_deleted", user_id = %user.id, username = %user.username);
    Ok(())
}
