//! The Create profile screen: the hero over username, email, password and confirm fields that validate as they are typed, with `Create profile` under them.
//!
//! What the screen knows and decides is zwipe-ui's [`State`]: the checks, when each field shows its line, the one registration out and what a good one does. This view adds the text boxes, the focus and the Show and Hide toggles. Esc ends typing, and Esc again with no field active goes back to Login; Tab or Shift+Tab goes back into the form.
//!
//! A good registration leaves the username in `cx.shared.caches.handoff.login_prefill` and goes back; Login takes it in its arm of `Screen::on_return`.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use zwipe_core::http::contracts::auth::HttpRegisterUser;
use zwipe_core::http::contracts::metrics::AnonymousEventKind;
use zwipe_ui::account::Password;
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::register::{Action, Destination, Effect, Field, State};

use super::Screen;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::hero::{self, Hero};
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// A stop the focus can rest on: a field, a password's Show/Hide, or the `Create profile` button.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegisterField {
    #[default]
    Username,
    Email,
    Password,
    ShowPassword,
    Confirm,
    ShowConfirm,
    CreateProfile,
}

impl RegisterField {
    /// Every stop, in Tab order.
    const ORDER: [Self; 7] = [
        Self::Username,
        Self::Email,
        Self::Password,
        Self::ShowPassword,
        Self::Confirm,
        Self::ShowConfirm,
        Self::CreateProfile,
    ];

    fn index(self) -> usize {
        Self::ORDER.iter().position(|f| *f == self).unwrap_or(0)
    }

    pub(crate) fn next(self) -> Self {
        let next = self
            .index()
            .saturating_add(1)
            .checked_rem(Self::ORDER.len())
            .unwrap_or(0);
        Self::ORDER.get(next).copied().unwrap_or_default()
    }

    pub(crate) fn previous(self) -> Self {
        let index = self
            .index()
            .checked_sub(1)
            .unwrap_or(Self::ORDER.len().saturating_sub(1));
        Self::ORDER.get(index).copied().unwrap_or_default()
    }

    /// The button this stop is, by its place in the row under the form.
    pub(crate) fn button(self) -> Option<usize> {
        (self == Self::CreateProfile).then_some(0)
    }
}

/// The Create profile screen's state: the model plus what only the terminal keeps.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RegisterForm {
    pub(crate) model: State,
    /// The boxes the fields are typed in. Each value follows every keystroke into the model, and each error line is the model's.
    pub(crate) username: TextInput,
    pub(crate) email: TextInput,
    pub(crate) password: TextInput,
    pub(crate) confirm: TextInput,
    /// The stop with the focus, or `None` once Esc ends typing.
    pub(crate) focus: Option<RegisterField>,
    pub(crate) hero: Hero,
}

impl Default for RegisterForm {
    fn default() -> Self {
        Self {
            model: State::new(),
            username: TextInput::default(),
            email: TextInput::default(),
            password: TextInput::masked(),
            confirm: TextInput::masked(),
            focus: Some(RegisterField::default()),
            hero: Hero::default(),
        }
    }
}

impl RegisterForm {
    /// The text box with the cursor and the model's field it fills, if the focus is on one.
    pub(crate) fn focused_mut(&mut self) -> Option<(&mut TextInput, Field)> {
        match self.focus {
            Some(RegisterField::Username) => Some((&mut self.username, Field::Username)),
            Some(RegisterField::Email) => Some((&mut self.email, Field::Email)),
            Some(RegisterField::Password) => Some((&mut self.password, Field::Password)),
            Some(RegisterField::Confirm) => Some((&mut self.confirm, Field::ConfirmPassword)),
            _ => None,
        }
    }
}

/// The model's edit action for `field` holding `value`.
pub(crate) fn edit(field: Field, value: String) -> Action {
    match field {
        Field::Username => Action::EditUsername(value),
        Field::Email => Action::EditEmail(value),
        Field::Password => Action::EditPassword(Password::new(value)),
        Field::ConfirmPassword => Action::EditConfirm(Password::new(value)),
    }
}

/// Applies `action` to the model, puts its error lines under the boxes, and performs the effects it returns, in order.
pub(crate) fn dispatch(form: &mut RegisterForm, action: Action, cx: &mut Context<'_>) {
    let effects = form.model.update(action);
    form.username.set_error(form.model.error(Field::Username));
    form.email.set_error(form.model.error(Field::Email));
    form.password.set_error(form.model.error(Field::Password));
    for effect in effects {
        match effect {
            Effect::Register {
                ticket,
                username,
                email,
                password,
            } => {
                cx.request(screens::Request::Record(
                    AnonymousEventKind::RegisterSubmitted,
                ));
                cx.request(
                    Request::Register {
                        ticket,
                        username,
                        email,
                        password,
                    }
                    .into(),
                );
            }
            Effect::Prefill(username) => cx.shared.caches.handoff.login_prefill = Some(username),
            Effect::Navigate(Destination::Previous) => cx.back(),
            Effect::Toast(toast) => cx.show(toast),
        }
    }
}

/// The Create profile screen's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// Registers the account: the username as typed, the email trimmed.
    Register {
        ticket: Ticket,
        username: String,
        email: String,
        password: Password,
    },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Register { .. } => "register",
        }
    }
}

/// The Create profile screen's results, each carrying its request's ticket back to the model.
#[derive(Debug)]
pub(crate) enum Update {
    /// The server sent the verification email.
    Registered {
        ticket: Ticket,
    },
    Failed {
        ticket: Ticket,
        error: String,
    },
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Register(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Register(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut RegisterForm> {
    match screen {
        Screen::Register(form) => Some(form),
        _ => None,
    }
}

/// Opens Create profile over Login and tells the server, anonymously, that it was viewed.
pub(crate) fn open(cx: &mut Context<'_>) {
    let form = RegisterForm {
        hero: hero::mount(cx),
        ..RegisterForm::default()
    };
    cx.request(screens::Request::Record(AnonymousEventKind::RegisterViewed));
    cx.push(Screen::Register(Box::new(form)));
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let Request::Register {
        ticket,
        username,
        email,
        password,
    } = request;
    let result = account
        .client
        .register(HttpRegisterUser::new(&username, &email, password.read()))
        .await;
    Some(
        match result {
            Ok(_) => Update::Registered { ticket },
            Err(e) => Update::Failed {
                ticket,
                error: screens::shown(&e),
            },
        }
        .into(),
    )
}

/// Hands a reply to the model. A failure raises its toast even when the screen has closed; a success after it closed has no model to land on, as in the app.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let action = match update {
        Update::Registered { ticket } => Action::Registered { ticket },
        Update::Failed { ticket, error } => {
            cx.toast(ToastKind::Error, error);
            Action::Failed { ticket }
        }
    };
    if let Some(form) = nav.find_mut(pick) {
        dispatch(form, action, cx);
    }
}
