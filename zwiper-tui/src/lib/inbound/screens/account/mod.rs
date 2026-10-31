//! The Account screen: the user's details and verification, the About rows, the sheets that change the username, email and password, the More sheet with Delete account, and the Log out confirm.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use zwipe_client::ClientError;
use zwipe_core::domain::auth::models::secret::Secret;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::hints::HINT_PROFILE;
use zwipe_core::http::contracts::auth::{
    HttpChangeEmail, HttpChangePassword, HttpChangeUsername, HttpDeleteUser,
};

use super::{Password, Screen, failure, login};
use crate::domain::account::{
    self, DELETE_DELAY_SECS, RESEND_COOLDOWN_SECS, change_password_error, email_error,
    password_required, username_error,
};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::alert::AlertDialog;
use crate::inbound::widgets::hint::{self, HintTopic};
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::{self, ToastKind};
use crate::outbound::account::Account;

/// A button on one of the screen's rows, in the order the focus walks them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stop {
    ChangeUsername,
    ChangeEmail,
    /// Sends the verification email again. Only while the email is unverified.
    Resend,
    /// Asks the server whether the email is verified yet. Only while it is unverified.
    Check,
    ChangePassword,
    Website,
    Privacy,
    Changelog,
}

impl Stop {
    /// The `?` topic of the row this button sits on, if the row has one.
    pub(crate) fn topic(self) -> Option<HintTopic> {
        match self {
            Self::ChangeEmail => Some(HintTopic::Email),
            Self::Resend | Self::Check => Some(HintTopic::Verification),
            Self::Website => Some(HintTopic::Website),
            Self::Changelog => Some(HintTopic::Version),
            _ => None,
        }
    }
}

/// The Account screen's state.
#[derive(Debug)]
pub(crate) struct AccountView {
    pub(crate) user: User,
    /// The focused button, an index into [`AccountView::stops`].
    pub(crate) focus: usize,
    pub(crate) overlay: Option<Overlay>,
    /// When the verification email can be sent again.
    pub(crate) resend_ready_at: Option<Instant>,
    /// Set while a verification email is on its way.
    pub(crate) sending: bool,
    /// Set while the verification check is out.
    pub(crate) checking: bool,
}

impl AccountView {
    pub(crate) fn new(user: User) -> Self {
        Self {
            user,
            focus: 0,
            overlay: None,
            resend_ready_at: None,
            sending: false,
            checking: false,
        }
    }

    pub(crate) fn verified(&self) -> bool {
        self.user.email_verified_at.is_some()
    }

    /// The buttons that apply to this user, in focus order.
    pub(crate) fn stops(&self) -> Vec<Stop> {
        let mut stops = vec![Stop::ChangeUsername, Stop::ChangeEmail];
        if !self.verified() {
            stops.extend([Stop::Resend, Stop::Check]);
        }
        stops.extend([
            Stop::ChangePassword,
            Stop::Website,
            Stop::Privacy,
            Stop::Changelog,
        ]);
        stops
    }

    pub(crate) fn focused(&self) -> Option<Stop> {
        self.stops().get(self.focus).copied()
    }

    pub(crate) fn next(&mut self) {
        self.focus = self
            .focus
            .saturating_add(1)
            .min(self.stops().len().saturating_sub(1));
    }

    pub(crate) fn previous(&mut self) {
        self.focus = self.focus.saturating_sub(1);
    }

    /// Takes a fresh copy of the user, keeping the focus in range.
    pub(crate) fn set_user(&mut self, user: User) {
        self.user = user;
        self.focus = self.focus.min(self.stops().len().saturating_sub(1));
    }

    /// Seconds until the verification email can be sent again, `None` when it can be now.
    pub(crate) fn resend_wait(&self) -> Option<u64> {
        let left = self
            .resend_ready_at?
            .saturating_duration_since(Instant::now());
        (!left.is_zero()).then(|| left.as_secs().saturating_add(1))
    }

    /// The Resend button's label: `Sending...`, `Resend in {n}s`, or `Resend`.
    pub(crate) fn resend_label(&self) -> String {
        if self.sending {
            return "Sending...".to_owned();
        }
        self.resend_wait()
            .map_or_else(|| "Resend".to_owned(), |n| format!("Resend in {n}s"))
    }

    /// Whether Resend can be pressed: not while sending or cooling down.
    pub(crate) fn can_resend(&self) -> bool {
        !self.sending && self.resend_wait().is_none()
    }

    /// Whether a countdown is on screen: the wait to resend, or the delete button's.
    pub(crate) fn animating(&self) -> bool {
        self.resend_wait().is_some()
            || matches!(
                &self.overlay,
                Some(Overlay::More(Some(delete))) if delete.wait().is_some()
            )
    }

    /// Starts the wait between verification emails.
    pub(crate) fn start_resend_cooldown(&mut self) {
        self.resend_ready_at =
            Instant::now().checked_add(Duration::from_secs(RESEND_COOLDOWN_SECS));
    }
}

/// What is open over the Account screen.
#[derive(Debug)]
pub(crate) enum Overlay {
    /// A Change sheet.
    Form(Form),
    /// The confirm before a password change, the sheet waiting under it.
    ConfirmPassword {
        form: Form,
        alert: AlertDialog,
    },
    LogOut(AlertDialog),
    /// The More sheet, with the Delete account dialog over it when open.
    More(Option<DeleteDialog>),
}

/// The alert before a password change.
pub(crate) fn confirm_password_alert() -> AlertDialog {
    AlertDialog::new(
        "Change password",
        "Changing your password will log you out on all other devices.",
    )
    .with_button("Cancel", Some('n'))
    .with_button("Confirm", Some('y'))
    .with_focus(1)
}

/// The alert before logging out.
pub(crate) fn log_out_alert() -> AlertDialog {
    AlertDialog::new("Log out", "Are you sure you want to log out?")
        .with_button("Cancel", Some('n'))
        .with_danger_button("Log out", Some('y'))
        .with_focus(1)
}

/// Which Change sheet is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FormKind {
    Username,
    Email,
    Password,
}

impl FormKind {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Username => "Change username",
            Self::Email => "Change email",
            Self::Password => "Change password",
        }
    }

    /// Each field's label, placeholder, and whether it hides what is typed.
    pub(crate) fn fields(self) -> &'static [(&'static str, &'static str, bool)] {
        match self {
            Self::Username => &[
                ("New username", "New username", false),
                ("Password", "Password", true),
            ],
            Self::Email => &[
                ("New email", "New email", false),
                ("Password", "Password", true),
            ],
            Self::Password => &[
                ("Current password", "Current password", true),
                ("New password ?", "New password", true),
                ("Confirm password", "Confirm new", true),
            ],
        }
    }
}

/// An open Change sheet: its fields, which has the cursor, which have held something, and whether a save was tried.
#[derive(Debug)]
pub(crate) struct Form {
    pub(crate) kind: FormKind,
    pub(crate) inputs: Vec<TextInput>,
    pub(crate) focus: usize,
    pub(crate) touched: Vec<bool>,
    pub(crate) attempted: bool,
    /// Set while the change is out; the sheet takes no keys meanwhile.
    pub(crate) busy: bool,
}

impl Form {
    /// A sheet with every field empty, as each open is.
    pub(crate) fn new(kind: FormKind) -> Self {
        let inputs: Vec<TextInput> = kind
            .fields()
            .iter()
            .map(|&(_, placeholder, masked)| {
                let input = if masked {
                    TextInput::masked()
                } else {
                    TextInput::default()
                };
                input.with_placeholder(placeholder)
            })
            .collect();
        Self {
            kind,
            touched: vec![false; inputs.len()],
            inputs,
            focus: 0,
            attempted: false,
            busy: false,
        }
    }

    pub(crate) fn value(&self, field: usize) -> &str {
        self.inputs.get(field).map_or("", TextInput::value)
    }

    /// Fills the fields in order, for tests.
    #[cfg(test)]
    pub(crate) fn fill(&mut self, values: &[&str]) {
        for (input, value) in self.inputs.iter_mut().zip(values) {
            input.set_value(value);
        }
    }

    pub(crate) fn next_field(&mut self) {
        self.focus = self
            .focus
            .saturating_add(1)
            .checked_rem(self.inputs.len())
            .unwrap_or(0);
    }

    pub(crate) fn previous_field(&mut self) {
        self.focus = self
            .focus
            .checked_sub(1)
            .unwrap_or_else(|| self.inputs.len().saturating_sub(1));
    }

    pub(crate) fn on_last_field(&self) -> bool {
        self.focus.saturating_add(1) >= self.inputs.len()
    }

    fn checks(&self, field: usize) -> bool {
        self.attempted || self.touched.get(field).copied().unwrap_or(false)
    }

    /// Each field's error, the app's way: live once the field has held something, or after a save.
    fn errors(&self) -> Vec<Option<String>> {
        match self.kind {
            FormKind::Username => vec![
                self.checks(0)
                    .then(|| username_error(self.value(0)))
                    .flatten(),
                self.checks(1)
                    .then(|| password_required(self.value(1)))
                    .flatten(),
            ],
            FormKind::Email => vec![
                self.checks(0).then(|| email_error(self.value(0))).flatten(),
                self.checks(1)
                    .then(|| password_required(self.value(1)))
                    .flatten(),
            ],
            FormKind::Password => vec![
                None,
                (self.checks(1) || self.checks(2))
                    .then(|| {
                        change_password_error(
                            self.value(0),
                            self.value(1),
                            self.value(2),
                            self.attempted,
                        )
                    })
                    .flatten(),
                None,
            ],
        }
    }

    /// Puts each field's error under it.
    pub(crate) fn validate(&mut self) {
        let errors = self.errors();
        for (input, error) in self.inputs.iter_mut().zip(errors) {
            input.set_error(error);
        }
    }

    /// Marks a save tried and checks every field; whether all pass.
    pub(crate) fn attempt(&mut self) -> bool {
        self.attempted = true;
        self.validate();
        self.inputs.iter().all(|i| i.error().is_none())
    }

    /// The change this sheet asks for, the values sent as typed.
    pub(crate) fn change(&self) -> AccountChange {
        let password = |field: usize| Password(self.value(field).to_owned());
        match self.kind {
            FormKind::Username => AccountChange::Username {
                username: self.value(0).to_owned(),
                password: password(1),
            },
            FormKind::Email => AccountChange::Email {
                email: self.value(0).to_owned(),
                password: password(1),
            },
            FormKind::Password => AccountChange::Password {
                current: password(0),
                new: password(1),
            },
        }
    }
}

/// Where the Delete account dialog's focus is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeleteFocus {
    Password,
    ShowPassword,
    Cancel,
    Delete,
}

/// The Delete account dialog: its password, when it opened for the countdown, and whether the delete is out.
#[derive(Debug)]
pub(crate) struct DeleteDialog {
    pub(crate) password: TextInput,
    pub(crate) opened_at: Instant,
    pub(crate) focus: DeleteFocus,
    pub(crate) busy: bool,
}

impl DeleteDialog {
    /// A dialog with an empty password, its countdown starting now.
    pub(crate) fn new() -> Self {
        Self {
            password: TextInput::masked().with_placeholder("Password"),
            opened_at: Instant::now(),
            focus: DeleteFocus::Password,
            busy: false,
        }
    }

    /// Seconds left before Delete can be pressed, `None` once it can.
    pub(crate) fn wait(&self) -> Option<u64> {
        let left = Duration::from_secs(DELETE_DELAY_SECS).saturating_sub(self.opened_at.elapsed());
        (!left.is_zero()).then(|| left.as_secs().saturating_add(1))
    }

    /// The Delete button's label: `Deleting...`, `Delete ({n})`, or `Delete`.
    pub(crate) fn label(&self) -> String {
        if self.busy {
            return "Deleting...".to_owned();
        }
        self.wait()
            .map_or_else(|| "Delete".to_owned(), |n| format!("Delete ({n})"))
    }
}

/// The note under a pending email change, for `user`.
pub(crate) fn pending_note(user: &User) -> Option<String> {
    user.pending_email_expires_at
        .map(|at| account::pending_note(at, chrono::Utc::now()))
}

/// The Account screen's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// Fetches the user with the session as it is, on every open.
    LoadUser,
    Resend,
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

/// The Account screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    /// A fresh copy of the user, and the toast that answers a verification check.
    User {
        user: Box<User>,
        toast: Option<(ToastKind, &'static str)>,
    },
    CheckFailed(String),
    /// A change went through: the user when the server sent one back, and the success toast for its time.
    Changed {
        user: Option<Box<User>>,
        message: String,
        duration: Duration,
    },
    ChangeFailed(String),
    Deleted,
    DeleteFailed(String),
    Resent,
    /// The server asked to wait before another verification email.
    ResendLimited,
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

/// Opens Account over the current screen on the user as last known, fetches a fresh copy, and shows the `profile` hint the first time.
pub(crate) fn open(cx: &mut Context<'_>) {
    let Some(user) = cx.shared.user.clone() else {
        return;
    };
    cx.push(Screen::Account(Box::new(AccountView::new(user))));
    cx.request(Request::LoadUser.into());
    hint::once(cx, HINT_PROFILE);
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let update = match request {
        Request::LoadUser => {
            // The app fetches with the session as it stands and says nothing when that fails.
            let session = account.session().await?;
            let user = client.fetch_user(&session).await.ok()?;
            Update::User {
                user: Box::new(user),
                toast: None,
            }
        }
        Request::CheckVerification => {
            match async { client.fetch_user(&account.fresh().await?).await }.await {
                Ok(user) => {
                    account.set_user(user.clone()).await;
                    let toast = if user.email_verified_at.is_some() {
                        (ToastKind::Success, "Email verified")
                    } else {
                        (ToastKind::Info, "Not verified yet. Check your inbox")
                    };
                    Update::User {
                        user: Box::new(user),
                        toast: Some(toast),
                    }
                }
                Err(e) => return Some(failure(&e, |m| Update::CheckFailed(m).into())),
            }
        }
        Request::Resend => {
            let failed = |m| Update::ResendFailed(m).into();
            let session = match account.fresh().await {
                Ok(session) => session,
                Err(e) => return Some(failure(&e, failed)),
            };
            match client.resend_verification(&session).await {
                Ok(()) => Update::Resent,
                Err(ClientError::TooManyRequests(_)) => Update::ResendLimited,
                Err(e) => return Some(failure(&e, failed)),
            }
        }
        Request::Change(change) => return Some(change_account(account, change).await),
    };
    Some(update.into())
}

/// Sends an account change. A wrong password is the server's to judge, so its refusal is a toast rather than a sign-out.
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
                (Some(user), message, toast::QUICK)
            }),
        AccountChange::Email { email, password } => client
            .change_email(HttpChangeEmail::new(&email, &password.0), &session)
            .await
            .map(|user| {
                let message = format!("Check {email} for a link to confirm the change");
                (Some(user), message, toast::NORMAL)
            }),
        AccountChange::Password { current, new } => client
            .change_password(HttpChangePassword::new(&current.0, &new.0), &session)
            .await
            .map(|()| (None, "Password change successful".to_owned(), toast::QUICK)),
        AccountChange::Delete { password } => {
            let request = HttpDeleteUser {
                password: Secret::new(&password.0),
            };
            return match client.delete_user(request, &session).await {
                Ok(()) => {
                    account.sign_out().await;
                    Update::Deleted
                }
                Err(e) => Update::DeleteFailed(crate::inbound::screens::shown(&e)),
            }
            .into();
        }
    };
    match result {
        Ok((user, message, duration)) => {
            if let Some(user) = &user {
                account.set_user(user.clone()).await;
            }
            Update::Changed {
                user: user.map(Box::new),
                message,
                duration,
            }
        }
        Err(e) => Update::ChangeFailed(crate::inbound::screens::shown(&e)),
    }
    .into()
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let mut view = nav.find_mut(pick);
    match update {
        Update::User { user, toast } => {
            if let Some(view) = view.as_deref_mut() {
                view.checking = false;
            }
            take_user(view, cx, *user);
            if let Some((kind, message)) = toast {
                cx.toast(kind, message.to_owned());
            }
        }
        Update::CheckFailed(error) => {
            if let Some(view) = view {
                view.checking = false;
            }
            cx.toast(ToastKind::Error, error);
        }
        Update::Changed {
            user,
            message,
            duration,
        } => {
            if let Some(view) = view.as_deref_mut()
                && matches!(
                    view.overlay,
                    Some(Overlay::Form(_) | Overlay::ConfirmPassword { .. })
                )
            {
                view.overlay = None;
            }
            if let Some(user) = user {
                take_user(view, cx, *user);
            }
            cx.toast_for(ToastKind::Success, message, duration);
        }
        Update::ChangeFailed(error) => {
            if let Some(view) = view {
                view.change_failed();
            }
            cx.toast(ToastKind::Error, error);
        }
        Update::Deleted => login::signed_out(cx, None),
        Update::DeleteFailed(error) => {
            if let Some(view) = view
                && let Some(Overlay::More(Some(delete))) = &mut view.overlay
            {
                delete.busy = false;
            }
            cx.toast(ToastKind::Error, error);
        }
        Update::Resent | Update::ResendLimited => {
            if let Some(view) = view {
                view.sending = false;
                view.start_resend_cooldown();
            }
            if matches!(update, Update::Resent) {
                cx.toast(ToastKind::Success, "Verification email sent".to_owned());
            } else {
                cx.toast(ToastKind::Info, "Please wait a moment".to_owned());
            }
        }
        Update::ResendFailed(error) => {
            if let Some(view) = view {
                view.sending = false;
                view.resend_ready_at = None;
            }
            cx.toast_for(ToastKind::Error, error, toast::LONG);
        }
    }
}

/// Takes a fresh copy of the user everywhere it shows.
fn take_user(view: Option<&mut AccountView>, cx: &mut Context<'_>, user: User) {
    cx.shared.username = Some(user.username.to_string());
    if let Some(view) = view {
        view.set_user(user.clone());
    }
    cx.shared.user = Some(user);
}

impl AccountView {
    /// Puts the open sheet back, ready to try again with what was typed.
    fn change_failed(&mut self) {
        let form = match self.overlay.take() {
            Some(Overlay::Form(form) | Overlay::ConfirmPassword { form, .. }) => Some(form),
            other => {
                self.overlay = other;
                None
            }
        };
        if let Some(mut form) = form {
            form.busy = false;
            self.overlay = Some(Overlay::Form(form));
        }
    }
}
