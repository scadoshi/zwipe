//! Account changes made from the server only, for `zerver-admin`. Nothing in the app reaches them.

use zwipe_core::domain::user::User;
use zwipe_core::domain::user::username::Username;

use crate::domain::auth::ports::AuthService;
use crate::domain::auth::requests::change_username::ChangeUsernameError;
use crate::domain::email::models::SendEmailError;
use crate::domain::metrics::models::kinds::AuditAction;
use crate::domain::metrics::ports::MetricsService;

/// Whether a rename tells the account, by email, that its old name is now reserved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameNotice {
    /// Sends the notice, with replies going to `reply_to` when given.
    Send {
        /// Where a reply goes; `None` replies to the sending address.
        reply_to: Option<String>,
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

/// Renames the account `account` (its exact username, id or email) to `new_username` the way a
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
    account: &str,
    new_username: &Username,
    notice: RenameNotice,
) -> Result<Renamed, ChangeUsernameError> {
    let (old_username, user) = auth.admin_change_username(account, new_username).await?;
    if let Err(e) = metrics
        .record_audit(user.id, AuditAction::UsernameChanged)
        .await
    {
        tracing::warn!(error = ?e, "admin rename: audit row failed");
    }
    let notice = match notice {
        RenameNotice::Skip => NoticeOutcome::Skipped,
        RenameNotice::Send { reply_to } => match auth
            .send_reserved_name_notice(&user, &old_username, reply_to.as_deref())
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
