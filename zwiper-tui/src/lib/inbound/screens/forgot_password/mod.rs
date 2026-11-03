//! The Reset password screen: the hero over an email field with `Send reset link` under it, then `Check your email` once the request is in. The reset itself happens on the website, from the link in the email.
//!
//! What the screen knows and decides is zwipe-ui's [`State`]; this view adds the text box, the focus and the spinner's clock. Esc ends typing, and Esc again with no field active goes back to Login; Tab or Shift+Tab goes back into the form.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::time::Instant;

use zwipe_core::http::contracts::auth::HttpRequestPasswordReset;
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::forgot_password::{Action, Destination, Effect, State};

use super::Screen;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::hero::{self, Hero};
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// A stop the focus can rest on: the email field or the `Send reset link` button under it.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ForgotField {
    #[default]
    Email,
    SendResetLink,
}

/// The Reset password screen's state: the model plus what only the terminal keeps.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ForgotForm {
    pub(crate) model: State,
    /// The box the email is typed in. Its value follows each keystroke into the model, and its error line is the model's.
    pub(crate) email: TextInput,
    /// The stop with the focus, or `None` once Esc ends typing or the request is in.
    pub(crate) focus: Option<ForgotField>,
    /// When the latest request went out; the spinner turns from here while the model is loading.
    pub(crate) spin_since: Instant,
    pub(crate) hero: Hero,
}

impl Default for ForgotForm {
    fn default() -> Self {
        Self {
            model: State::new(),
            email: TextInput::default(),
            focus: Some(ForgotField::default()),
            spin_since: Instant::now(),
            hero: Hero::default(),
        }
    }
}

impl ForgotForm {
    /// The stops in Tab order: the field and its button, or none once sent.
    pub(crate) fn stops(&self) -> &'static [ForgotField] {
        if self.model.is_sent() {
            &[]
        } else {
            &[ForgotField::Email, ForgotField::SendResetLink]
        }
    }

    /// Moves the focus `toward` the next or the previous of [`ForgotForm::stops`], wrapping round. With no stop focused, the next is the first and the previous the last.
    pub(crate) fn step(&mut self, toward: Toward) {
        let stops = self.stops();
        let next = match self.focus.and_then(|f| stops.iter().position(|s| *s == f)) {
            Some(at) => toward.wrapped(at, stops.len()),
            None if toward == Toward::Next => 0,
            None => stops.len().saturating_sub(1),
        };
        self.focus = stops.get(next).copied();
    }
}

/// Applies `action` to the model, brings the box and the focus in line with it, and performs the effects it returns, in order.
pub(crate) fn dispatch(form: &mut ForgotForm, action: Action, cx: &mut Context<'_>) {
    let effects = form.model.update(action);
    form.email.set_error(form.model.error());
    if form.model.is_sent() {
        form.focus = None;
    }
    for effect in effects {
        match effect {
            Effect::RequestReset { ticket, email } => {
                form.spin_since = Instant::now();
                cx.request(Request::Send { ticket, email }.into());
            }
            Effect::Navigate(Destination::Previous) => cx.back(),
        }
    }
}

/// The Reset password screen's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// Asks for a reset link for this email, as typed.
    Send { ticket: Ticket, email: String },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Send { .. } => "send",
        }
    }
}

/// The Reset password screen's results, each carrying its request's ticket back to the model.
#[derive(Debug)]
pub(crate) enum Update {
    Sent { ticket: Ticket },
    Failed { ticket: Ticket, error: String },
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
    let Request::Send { ticket, email } = request;
    let result = account
        .client
        .request_password_reset(HttpRequestPasswordReset::new(&email))
        .await;
    Some(
        match result {
            Ok(()) => Update::Sent { ticket },
            Err(e) => Update::Failed {
                ticket,
                error: screens::shown(&e),
            },
        }
        .into(),
    )
}

/// Hands a reply to the model. A failure raises its toast even when the screen has closed.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let action = match update {
        Update::Sent { ticket } => Action::Sent { ticket },
        Update::Failed { ticket, error } => {
            cx.toast(ToastKind::Error, error);
            Action::Failed { ticket }
        }
    };
    if let Some(form) = nav.find_mut(pick) {
        dispatch(form, action, cx);
    }
}
