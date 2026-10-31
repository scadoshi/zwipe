//! The Login screen: the hero over a username-or-email and password form, with `Log in`, `Create profile` and `Forgot password` in the action bar; and what signing in and out do to the app.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use zwipe_core::domain::auth::models::session::Session;

use super::{Password, Screen, config, home};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::hero::{self, Hero};
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The empty-form message, the app's toast.
pub(crate) const EMPTY_FORM: &str = "Enter your username or email and password";

/// The Login screen's state.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LoginForm {
    /// Username or email, sent as typed.
    pub(crate) identifier: TextInput,
    pub(crate) password: TextInput,
    pub(crate) focus: LoginField,
    /// Set while the sign-in request is out; every button is disabled meanwhile.
    pub(crate) busy: bool,
    pub(crate) hero: Hero,
}

impl Default for LoginForm {
    fn default() -> Self {
        Self {
            identifier: TextInput::default().with_placeholder("Username or email"),
            password: TextInput::masked().with_placeholder("Password"),
            focus: LoginField::default(),
            busy: false,
            hero: Hero::default(),
        }
    }
}

impl LoginForm {
    /// The text field with the cursor, if the focus is on one.
    pub(crate) fn focused_mut(&mut self) -> Option<&mut TextInput> {
        match self.focus {
            LoginField::Identifier => Some(&mut self.identifier),
            LoginField::Password => Some(&mut self.password),
            _ => None,
        }
    }
}

/// Where the focus is: a field, the password's Show/Hide, or a button in the action bar.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoginField {
    #[default]
    Identifier,
    Password,
    /// The Show or Hide button inside the password field.
    ShowPassword,
    LogIn,
    CreateProfile,
    ForgotPassword,
}

impl LoginField {
    /// Every stop, in Tab order.
    const ORDER: [Self; 6] = [
        Self::Identifier,
        Self::Password,
        Self::ShowPassword,
        Self::LogIn,
        Self::CreateProfile,
        Self::ForgotPassword,
    ];

    fn index(self) -> usize {
        Self::ORDER.iter().position(|f| *f == self).unwrap_or(0)
    }

    /// The next stop, wrapping round.
    pub(crate) fn next(self) -> Self {
        let next = self
            .index()
            .saturating_add(1)
            .checked_rem(Self::ORDER.len())
            .unwrap_or(0);
        Self::ORDER.get(next).copied().unwrap_or_default()
    }

    /// The stop before, wrapping round.
    pub(crate) fn previous(self) -> Self {
        let index = self
            .index()
            .checked_sub(1)
            .unwrap_or(Self::ORDER.len().saturating_sub(1));
        Self::ORDER.get(index).copied().unwrap_or_default()
    }

    /// The action bar button this stop is, by its place in the bar.
    pub(crate) fn button(self) -> Option<usize> {
        match self {
            Self::LogIn => Some(0),
            Self::CreateProfile => Some(1),
            Self::ForgotPassword => Some(2),
            _ => None,
        }
    }
}

/// The Login screen's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    SignIn {
        identifier: String,
        password: Password,
    },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::SignIn { .. } => "sign_in",
        }
    }
}

/// The Login screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    SignedIn(Box<Session>),
    Failed(String),
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Login(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Login(update)
    }
}

/// The open Login screen, if any.
pub(crate) fn pick(screen: &mut Screen) -> Option<&mut LoginForm> {
    match screen {
        Screen::Login(form) => Some(form),
        _ => None,
    }
}

/// A fresh Login screen, its hero fetching the counts when they are stale.
pub(crate) fn screen(cx: &mut Context<'_>) -> Screen {
    Screen::Login(Box::new(LoginForm {
        hero: hero::mount(cx),
        ..LoginForm::default()
    }))
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    match request {
        Request::SignIn {
            identifier,
            password,
        } => Some(
            match account.sign_in(&identifier, &password.0).await {
                Ok(session) => Update::SignedIn(Box::new(session)),
                Err(e) => Update::Failed(crate::inbound::screens::shown(&e)),
            }
            .into(),
        ),
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let Some(form) = nav.find_mut(pick) else {
        return;
    };
    match update {
        Update::SignedIn(session) => {
            form.password.set_value("");
            signed_in(cx, &session);
        }
        Update::Failed(error) => {
            form.busy = false;
            cx.toast(ToastKind::Error, error);
        }
    }
}

/// Puts `username` in the identifier field and clears the password, as Register does on its way back.
pub(crate) fn prefill(nav: &mut Nav, username: &str) {
    if let Some(form) = nav.find_mut(pick) {
        form.identifier.set_value(username);
        form.password.set_value("");
        form.focus = LoginField::Password;
    }
}

/// Applies the account's theme, remembers it for the next launch, and starts over from home.
pub(crate) fn signed_in(cx: &mut Context<'_>, session: &Session) {
    cx.use_preferences(&session.preferences);
    cx.request(config::Request::LoadPreferences.into());
    cx.shared.username = Some(session.user.username.to_string());
    cx.shared.user = Some(session.user.clone());
    home::open(cx);
}

/// Back to Login, still in the account's theme. The `error` is for callers that have one to report; the app reports none, so it shows as a toast.
pub(crate) fn signed_out(cx: &mut Context<'_>, error: Option<String>) {
    cx.shared.username = None;
    cx.shared.user = None;
    cx.shared.status.clear();
    if let Some(error) = error {
        cx.toast(ToastKind::Error, error);
    }
    let login = screen(cx);
    cx.reset(login);
}
