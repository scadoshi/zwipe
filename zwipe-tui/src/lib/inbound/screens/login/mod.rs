//! The sign-in screen: the form, the sign-in request, and what signing in and out do to the app.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use zwipe_core::domain::auth::models::session::Session;

use super::{Password, Screen, config, home};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::outbound::account::Account;

/// The sign-in form.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct LoginForm {
    /// Username or email.
    pub(crate) identifier: String,
    pub(crate) password: String,
    pub(crate) focus: LoginField,
    pub(crate) error: Option<String>,
    /// Set while the sign-in request is out.
    pub(crate) busy: bool,
}

impl LoginForm {
    /// The field the cursor is in.
    pub(crate) fn focused_mut(&mut self) -> &mut String {
        match self.focus {
            LoginField::Identifier => &mut self.identifier,
            LoginField::Password => &mut self.password,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoginField {
    #[default]
    Identifier,
    Password,
}

impl LoginField {
    pub(crate) fn other(self) -> Self {
        match self {
            Self::Identifier => Self::Password,
            Self::Password => Self::Identifier,
        }
    }
}

/// The sign-in screen's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    SignIn {
        identifier: String,
        password: Password,
    },
}

/// The sign-in screen's results.
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

/// The open sign-in form, if any.
fn pick(screen: &mut Screen) -> Option<&mut LoginForm> {
    match screen {
        Screen::Login(form) => Some(form),
        _ => None,
    }
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    match request {
        Request::SignIn {
            identifier,
            password,
        } => Some(
            match account.sign_in(&identifier, &password.0).await {
                Ok(session) => Update::SignedIn(Box::new(session)),
                Err(e) => Update::Failed(e.to_user_message()),
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
        Update::SignedIn(session) => signed_in(cx, &session),
        Update::Failed(error) => {
            form.busy = false;
            form.error = Some(error);
        }
    }
}

/// Applies the account's theme, remembers it for the next launch, and starts over from home.
pub(crate) fn signed_in(cx: &mut Context<'_>, session: &Session) {
    cx.use_preferences(&session.preferences);
    cx.request(config::Request::LoadPreferences.into());
    let username = session.user.username.to_string();
    cx.shared.status = format!("Hello, {username}!");
    cx.shared.username = Some(username);
    cx.shared.user = Some(session.user.clone());
    home::open(cx);
}

/// Back to sign-in, still in the account's theme, with `error` on the form.
pub(crate) fn signed_out(cx: &mut Context<'_>, error: Option<String>) {
    cx.shared.username = None;
    cx.shared.user = None;
    cx.shared.status.clear();
    cx.reset(Screen::Login(LoginForm {
        error,
        ..LoginForm::default()
    }));
}
