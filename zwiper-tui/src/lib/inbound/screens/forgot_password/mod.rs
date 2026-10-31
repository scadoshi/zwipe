//! The Reset password screen: the hero over an email field, then `Check your email` once the request is in. The reset itself happens on the website, from the link in the email.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::time::Instant;

use zwipe_core::domain::user::Email;
use zwipe_core::http::contracts::auth::HttpRequestPasswordReset;

use super::Screen;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::hero::{self, Hero};
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The error under the email, whatever is wrong with it.
pub(crate) const INVALID_EMAIL: &str = "Please enter a valid email address";

/// The success panel's title and body.
pub(crate) const SENT_TITLE: &str = "Check your email";
pub(crate) const SENT_BODY: &str = "If that email is registered, a reset link is on its way.";

/// Where the focus is.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ForgotField {
    #[default]
    Email,
    BackToLogin,
    SendResetLink,
}

/// The Reset password screen's state.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ForgotForm {
    pub(crate) email: TextInput,
    pub(crate) focus: ForgotField,
    /// Whether the email has held something.
    pub(crate) touched: bool,
    pub(crate) attempted: bool,
    /// When the request went out, while it is out; the spinner turns from here.
    pub(crate) sending_since: Option<Instant>,
    /// The request went in: the form gives way to the success panel.
    pub(crate) sent: bool,
    pub(crate) hero: Hero,
}

impl Default for ForgotForm {
    fn default() -> Self {
        Self {
            email: TextInput::default().with_placeholder("Email"),
            focus: ForgotField::default(),
            touched: false,
            attempted: false,
            sending_since: None,
            sent: false,
            hero: Hero::default(),
        }
    }
}

impl ForgotForm {
    /// The stops in Tab order: the field and both buttons, or only `Back to login` once sent.
    pub(crate) fn stops(&self) -> &'static [ForgotField] {
        if self.sent {
            &[ForgotField::BackToLogin]
        } else {
            &[
                ForgotField::Email,
                ForgotField::BackToLogin,
                ForgotField::SendResetLink,
            ]
        }
    }

    /// Moves the focus `toward` the next or the previous of [`ForgotForm::stops`], wrapping round.
    pub(crate) fn step(&mut self, toward: Toward) {
        let stops = self.stops();
        let at = stops.iter().position(|s| *s == self.focus).unwrap_or(0);
        let next = toward.wrapped(at, stops.len());
        self.focus = stops.get(next).copied().unwrap_or_default();
    }

    /// The fixed error once the email has held something or a send was tried, checked untrimmed.
    pub(crate) fn validate(&mut self) {
        let invalid = Email::new(self.email.value()).is_err();
        let shows = (self.touched || self.attempted) && invalid;
        self.email
            .set_error(shows.then(|| INVALID_EMAIL.to_owned()));
    }

    /// Whether the spinner turns.
    pub(crate) fn sending(&self) -> bool {
        self.sending_since.is_some()
    }
}

/// The Reset password screen's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// Asks for a reset link for this email, as typed.
    Send(String),
}

/// The Reset password screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    Sent,
    Failed(String),
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::ForgotPassword(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::ForgotPassword(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut ForgotForm> {
    match screen {
        Screen::ForgotPassword(form) => Some(form),
        _ => None,
    }
}

/// Opens Reset password over Login.
pub(crate) fn open(cx: &mut Context<'_>) {
    let form = ForgotForm {
        hero: hero::mount(cx),
        ..ForgotForm::default()
    };
    cx.push(Screen::ForgotPassword(Box::new(form)));
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let Request::Send(email) = request;
    let result = account
        .client
        .request_password_reset(HttpRequestPasswordReset::new(&email))
        .await;
    Some(
        match result {
            Ok(()) => Update::Sent,
            Err(e) => Update::Failed(e.to_user_message()),
        }
        .into(),
    )
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let form = nav.find_mut(pick);
    match update {
        Update::Sent => {
            if let Some(form) = form {
                form.sending_since = None;
                form.sent = true;
                form.focus = ForgotField::BackToLogin;
            }
        }
        Update::Failed(error) => {
            if let Some(form) = form {
                form.sending_since = None;
            }
            cx.toast(ToastKind::Error, error);
        }
    }
}
