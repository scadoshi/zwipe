//! The account screen: the user's details, verification, and the forms that change them.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use zwipe_client::ClientError;
use zwipe_core::domain::auth::models::secret::Secret;
use zwipe_core::domain::user::User;
use zwipe_core::http::contracts::auth::{
    HttpChangeEmail, HttpChangePassword, HttpChangeUsername, HttpDeleteUser,
};

use super::{Password, Screen, failure, login};
use crate::domain::account::{
    self, DELETE_DELAY_SECS, RESEND_COOLDOWN_SECS, email_error, new_password_error,
    password_required, username_error,
};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::outbound::account::Account;

/// The account screen.
#[derive(Debug)]
pub(crate) struct AccountView {
    pub(crate) user: User,
    /// The selected row of [`AccountView::rows`].
    pub(crate) row: usize,
    pub(crate) dialog: Option<Dialog>,
    /// When the verification email can be sent again.
    pub(crate) resend_ready_at: Option<Instant>,
}

/// A row that does something when picked, in the app's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AccountRow {
    Username,
    Email,
    /// Sends the verification email again. Only while the email is unverified.
    Resend,
    /// Asks the server whether the email has been verified. Only while it is unverified.
    Check,
    Password,
    LogOut,
    Delete,
}

impl AccountView {
    pub(crate) fn new(user: User) -> Self {
        Self {
            user,
            row: 0,
            dialog: None,
            resend_ready_at: None,
        }
    }

    pub(crate) fn verified(&self) -> bool {
        self.user.email_verified_at.is_some()
    }

    /// The rows that apply to this user.
    pub(crate) fn rows(&self) -> Vec<AccountRow> {
        let mut rows = vec![AccountRow::Username, AccountRow::Email];
        if !self.verified() {
            rows.extend([AccountRow::Resend, AccountRow::Check]);
        }
        rows.extend([AccountRow::Password, AccountRow::LogOut, AccountRow::Delete]);
        rows
    }

    pub(crate) fn selected(&self) -> Option<AccountRow> {
        self.rows().get(self.row).copied()
    }

    pub(crate) fn next_row(&mut self) {
        self.row = self
            .row
            .saturating_add(1)
            .min(self.rows().len().saturating_sub(1));
    }

    pub(crate) fn previous_row(&mut self) {
        self.row = self.row.saturating_sub(1);
    }

    /// Takes a fresh copy of the user, keeping the selection in range.
    pub(crate) fn set_user(&mut self, user: User) {
        self.user = user;
        self.row = self.row.min(self.rows().len().saturating_sub(1));
    }

    /// Seconds until the verification email can be sent again, `None` when it can be now.
    pub(crate) fn resend_wait(&self) -> Option<u64> {
        let left = self
            .resend_ready_at?
            .saturating_duration_since(Instant::now());
        (!left.is_zero()).then(|| left.as_secs().saturating_add(1))
    }

    /// Whether a countdown is on screen: the wait to resend, or the delete form's.
    pub(crate) fn animating(&self) -> bool {
        self.resend_wait().is_some()
            || matches!(&self.dialog, Some(Dialog::Form(form)) if form.delete_wait().is_some())
    }

    /// Starts the wait between verification emails.
    pub(crate) fn start_resend_cooldown(&mut self) {
        self.resend_ready_at =
            Instant::now().checked_add(Duration::from_secs(RESEND_COOLDOWN_SECS));
    }
}

/// What is open over the account screen.
#[derive(Debug)]
pub(crate) enum Dialog {
    Form(Form),
    /// Asks before changing the password, since that signs out every other device.
    ConfirmPassword(Form),
    LogOut,
}

/// Which form is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FormKind {
    Username,
    Email,
    Password,
    Delete,
}

impl FormKind {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Username => "Change username",
            Self::Email => "Change email",
            Self::Password => "Change password",
            Self::Delete => "Delete account",
        }
    }

    /// Each field's label, and whether it hides what is typed.
    pub(crate) fn fields(self) -> &'static [(&'static str, bool)] {
        match self {
            Self::Username => &[("New username", false), ("Password", true)],
            Self::Email => &[("New email", false), ("Password", true)],
            Self::Password => &[
                ("Current password", true),
                ("New password", true),
                ("Confirm password", true),
            ],
            Self::Delete => &[("Password", true)],
        }
    }
}

/// An open form: what is typed in each field, which has the cursor, and why the last submit failed.
#[derive(Debug)]
pub(crate) struct Form {
    pub(crate) kind: FormKind,
    pub(crate) values: Vec<String>,
    pub(crate) focus: usize,
    pub(crate) error: Option<String>,
    /// Set while the request is out.
    pub(crate) busy: bool,
    pub(crate) opened_at: Instant,
}

impl Form {
    pub(crate) fn new(kind: FormKind) -> Self {
        Self {
            kind,
            values: vec![String::new(); kind.fields().len()],
            focus: 0,
            error: None,
            busy: false,
            opened_at: Instant::now(),
        }
    }

    pub(crate) fn value(&self, field: usize) -> &str {
        self.values.get(field).map_or("", String::as_str)
    }

    pub(crate) fn focused_mut(&mut self) -> Option<&mut String> {
        self.values.get_mut(self.focus)
    }

    pub(crate) fn next_field(&mut self) {
        self.focus = self.focus.saturating_add(1) % self.values.len().max(1);
    }

    pub(crate) fn previous_field(&mut self) {
        self.focus = self
            .focus
            .checked_sub(1)
            .unwrap_or_else(|| self.values.len().saturating_sub(1));
    }

    pub(crate) fn on_last_field(&self) -> bool {
        self.focus.saturating_add(1) >= self.values.len()
    }

    /// Seconds before the delete form can submit, `None` once it can.
    pub(crate) fn delete_wait(&self) -> Option<u64> {
        if self.kind != FormKind::Delete {
            return None;
        }
        let left = Duration::from_secs(DELETE_DELAY_SECS).saturating_sub(self.opened_at.elapsed());
        (!left.is_zero()).then(|| left.as_secs().saturating_add(1))
    }

    /// Checks the fields the way the app does before sending, giving the first problem.
    pub(crate) fn problem(&self) -> Option<String> {
        match self.kind {
            FormKind::Username => {
                username_error(self.value(0).trim()).or_else(|| password_required(self.value(1)))
            }
            FormKind::Email => {
                email_error(self.value(0).trim()).or_else(|| password_required(self.value(1)))
            }
            FormKind::Password => new_password_error(self.value(0), self.value(1), self.value(2)),
            FormKind::Delete => None,
        }
    }
}

/// The note under a pending email change, for `user`.
pub(crate) fn pending_note(user: &User) -> Option<String> {
    user.pending_email_expires_at
        .map(|at| account::pending_note(at, chrono::Utc::now()))
}

/// The account screen's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// Fetches the signed-in user.
    LoadUser,
    /// Sends the verification email again.
    Resend,
    /// Fetches the user to see whether the email is verified yet.
    CheckVerification,
    Change(AccountChange),
}

/// A change to the account, with the password the server checks it against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AccountChange {
    Username {
        username: String,
        password: Password,
    },
    Email {
        email: String,
        password: Password,
    },
    Password {
        current: Password,
        new: Password,
    },
    Delete {
        password: Password,
    },
}

/// The account screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    /// A fresh copy of the user, with a message to show when it answers a check.
    User {
        user: Box<User>,
        message: Option<String>,
    },
    Changed {
        user: Option<Box<User>>,
        message: String,
    },
    ChangeFailed(String),
    Deleted,
    /// Sending the verification email failed, so it can be tried again at once.
    ResendFailed(String),
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Account(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Account(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut AccountView> {
    match screen {
        Screen::Account(view) => Some(view),
        _ => None,
    }
}

/// Opens the account screen over the current one on the user as last known, and fetches a fresh copy.
pub(crate) fn open(cx: &mut Context<'_>) {
    let Some(user) = cx.shared.user.clone() else {
        return;
    };
    cx.shared.status.clear();
    cx.push(Screen::Account(Box::new(AccountView::new(user))));
    cx.request(Request::LoadUser.into());
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let update = match request {
        Request::LoadUser => {
            match async { client.fetch_user(&account.fresh().await?).await }.await {
                Ok(user) => {
                    account.set_user(user.clone()).await;
                    Update::User {
                        user: Box::new(user),
                        message: None,
                    }
                }
                Err(e) => return Some(failure(&e, screens::Update::Status)),
            }
        }
        Request::CheckVerification => {
            match async { client.fetch_user(&account.fresh().await?).await }.await {
                Ok(user) => {
                    account.set_user(user.clone()).await;
                    let message = if user.email_verified_at.is_some() {
                        "Email verified"
                    } else {
                        "Not verified yet. Check your inbox"
                    };
                    Update::User {
                        user: Box::new(user),
                        message: Some(message.to_owned()),
                    }
                }
                Err(e) => return Some(failure(&e, screens::Update::Status)),
            }
        }
        Request::Resend => {
            let resend_failed = |m| Update::ResendFailed(m).into();
            let session = match account.fresh().await {
                Ok(session) => session,
                Err(e) => return Some(failure(&e, resend_failed)),
            };
            return Some(match client.resend_verification(&session).await {
                Ok(()) => screens::Update::Status("Verification email sent".to_owned()),
                Err(ClientError::TooManyRequests(_)) => {
                    screens::Update::Status("Please wait a moment".to_owned())
                }
                Err(e) => failure(&e, resend_failed),
            });
        }
        Request::Change(change) => return Some(change_account(account, change).await),
    };
    Some(update.into())
}

/// Sends an account change. The password is the server's to judge, so its refusal is the form's error rather than a sign-out.
async fn change_account(account: &Account, change: AccountChange) -> screens::Update {
    let client = &account.client;
    let session = match account.fresh().await {
        Ok(session) => session,
        Err(e) => return failure(&e, |m| Update::ChangeFailed(m).into()),
    };
    let result = match change {
        AccountChange::Username { username, password } => client
            .change_username(HttpChangeUsername::new(&username, &password.0), &session)
            .await
            .map(|user| {
                let message = format!("Username changed to {}", user.username);
                (Some(user), message)
            }),
        AccountChange::Email { email, password } => client
            .change_email(HttpChangeEmail::new(&email, &password.0), &session)
            .await
            .map(|user| {
                (
                    Some(user),
                    format!("Check {email} for a link to confirm the change"),
                )
            }),
        AccountChange::Password { current, new } => client
            .change_password(HttpChangePassword::new(&current.0, &new.0), &session)
            .await
            .map(|()| (None, "Password change successful".to_owned())),
        AccountChange::Delete { password } => {
            let request = HttpDeleteUser {
                password: Secret::new(&password.0),
            };
            return match client.delete_user(request, &session).await {
                Ok(()) => {
                    account.forget().await;
                    Update::Deleted
                }
                Err(e) => Update::ChangeFailed(e.to_user_message()),
            }
            .into();
        }
    };
    match result {
        Ok((user, message)) => {
            if let Some(user) = &user {
                account.set_user(user.clone()).await;
            }
            Update::Changed {
                user: user.map(Box::new),
                message,
            }
        }
        Err(e) => Update::ChangeFailed(e.to_user_message()),
    }
    .into()
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let view = nav.find_mut(pick);
    match update {
        Update::User { user, message } => take_user(view, cx, *user, message),
        Update::Changed { user, message } => {
            let mut view = view;
            if let Some(view) = view.as_deref_mut() {
                view.dialog = None;
            }
            match user {
                Some(user) => take_user(view, cx, *user, Some(message)),
                None => cx.shared.status = message,
            }
        }
        Update::ChangeFailed(error) => {
            if let Some(view) = view {
                view.change_failed(error);
            }
        }
        Update::Deleted => {
            login::signed_out(cx, None);
            "Account deleted".clone_into(&mut cx.shared.status);
        }
        Update::ResendFailed(error) => {
            if let Some(view) = view {
                view.resend_ready_at = None;
            }
            cx.shared.status = error;
        }
    }
}

/// Takes a fresh copy of the user everywhere it shows, and the message that came with it.
fn take_user(
    view: Option<&mut AccountView>,
    cx: &mut Context<'_>,
    user: User,
    message: Option<String>,
) {
    cx.shared.username = Some(user.username.to_string());
    if let Some(view) = view {
        view.set_user(user.clone());
    }
    cx.shared.user = Some(user);
    if let Some(message) = message {
        cx.shared.status = message;
    }
}

impl AccountView {
    /// Puts the open form back with the server's reason, ready to try again.
    fn change_failed(&mut self, error: String) {
        let form = match self.dialog.take() {
            Some(Dialog::Form(form) | Dialog::ConfirmPassword(form)) => Some(form),
            other => {
                self.dialog = other;
                None
            }
        };
        if let Some(mut form) = form {
            form.busy = false;
            form.error = Some(error);
            self.dialog = Some(Dialog::Form(form));
        }
    }
}
