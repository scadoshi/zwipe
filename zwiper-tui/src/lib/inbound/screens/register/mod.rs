//! The Create profile screen: the hero over username, email, password and confirm fields that validate as they are typed, with `Back to login` and `Create profile` in the action bar.
//!
//! A field starts showing its error once it has held something, or after a submit; from then on it checks again on every change. A good registration goes back to Login with the username filled in.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use zwipe_core::domain::auth::models::password;
use zwipe_core::http::contracts::auth::HttpRegisterUser;
use zwipe_core::http::contracts::metrics::AnonymousEventKind;

use super::{Password, Screen, login};
use crate::domain::account::{email_error, username_error};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::hero::{self, Hero};
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The success toast, sent before going back to Login.
pub(crate) const CHECK_INBOX: &str = "Check your inbox to verify your email, then log in";

/// The mismatch error, shown under Password.
pub(crate) const MISMATCH: &str = "Passwords do not match";

/// Where the focus is: a field, a password's Show/Hide, or a button.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegisterField {
    #[default]
    Username,
    Email,
    Password,
    ShowPassword,
    Confirm,
    ShowConfirm,
    BackToLogin,
    CreateProfile,
}

impl RegisterField {
    /// Every stop, in Tab order.
    const ORDER: [Self; 8] = [
        Self::Username,
        Self::Email,
        Self::Password,
        Self::ShowPassword,
        Self::Confirm,
        Self::ShowConfirm,
        Self::BackToLogin,
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

    /// The action bar button this stop is, by its place in the bar.
    pub(crate) fn button(self) -> Option<usize> {
        match self {
            Self::BackToLogin => Some(0),
            Self::CreateProfile => Some(1),
            _ => None,
        }
    }
}

/// The Create profile screen's state.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RegisterForm {
    pub(crate) username: TextInput,
    pub(crate) email: TextInput,
    pub(crate) password: TextInput,
    pub(crate) confirm: TextInput,
    pub(crate) focus: RegisterField,
    /// Which of username, email, password and confirm have held something.
    pub(crate) touched: [bool; 4],
    /// Set by the first submit, after which every field shows its error.
    pub(crate) attempted: bool,
    /// Set while the registration is out; both buttons are disabled meanwhile.
    pub(crate) busy: bool,
    pub(crate) hero: Hero,
}

impl Default for RegisterForm {
    fn default() -> Self {
        Self {
            username: TextInput::default().with_placeholder("Username"),
            email: TextInput::default().with_placeholder("Email"),
            password: TextInput::masked().with_placeholder("Password"),
            confirm: TextInput::masked().with_placeholder("Confirm password"),
            focus: RegisterField::default(),
            touched: [false; 4],
            attempted: false,
            busy: false,
            hero: Hero::default(),
        }
    }
}

impl RegisterForm {
    /// The text field with the cursor and its place in `touched`, if the focus is on one.
    pub(crate) fn focused_mut(&mut self) -> Option<(&mut TextInput, usize)> {
        match self.focus {
            RegisterField::Username => Some((&mut self.username, 0)),
            RegisterField::Email => Some((&mut self.email, 1)),
            RegisterField::Password => Some((&mut self.password, 2)),
            RegisterField::Confirm => Some((&mut self.confirm, 3)),
            _ => None,
        }
    }

    /// Whether a field's errors show: once it has held something, or after a submit.
    fn checks(&self, field: usize) -> bool {
        self.attempted || self.touched.get(field).copied().unwrap_or(false)
    }

    /// What is wrong with the password, in the app's order: the policy, then a mismatch with the confirmation once a submit was tried or the confirmation is as long as the password.
    pub(crate) fn password_error(&self) -> Option<String> {
        let password = self.password.value();
        if let Err(e) = password::validate(password) {
            return Some(e.to_string());
        }
        let confirm = self.confirm.value();
        let long_enough = confirm.chars().count() >= password.chars().count();
        (confirm != password && (self.attempted || long_enough)).then(|| MISMATCH.to_owned())
    }

    /// Checks every field the app's way and puts each error under its field.
    pub(crate) fn validate(&mut self) {
        let username = self
            .checks(0)
            .then(|| username_error(self.username.value()))
            .flatten();
        let email = self
            .checks(1)
            .then(|| email_error(self.email.value().trim()))
            .flatten();
        let password = (self.checks(2) || self.checks(3))
            .then(|| self.password_error())
            .flatten();
        self.username.set_error(username);
        self.email.set_error(email);
        self.password.set_error(password);
    }

    /// Whether every field passes, checked as a submit checks them.
    pub(crate) fn valid(&self) -> bool {
        username_error(self.username.value()).is_none()
            && email_error(self.email.value().trim()).is_none()
            && self.password_error().is_none()
    }
}

/// The Create profile screen's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// Registers the account: the username as typed, the email trimmed.
    Register {
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

/// The Create profile screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    /// The server sent the verification email; the username goes back to Login.
    Registered {
        username: String,
    },
    Failed(String),
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
        username,
        email,
        password,
    } = request;
    let result = account
        .client
        .register(HttpRegisterUser::new(&username, &email, &password.0))
        .await;
    Some(
        match result {
            Ok(_) => Update::Registered { username },
            Err(e) => Update::Failed(crate::inbound::screens::shown(&e)),
        }
        .into(),
    )
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let open = matches!(nav.top(), Screen::Register(_));
    if let Some(form) = nav.find_mut(pick) {
        form.busy = false;
    }
    match update {
        Update::Registered { username } => {
            login::prefill(nav, &username);
            cx.toast(ToastKind::Info, CHECK_INBOX.to_owned());
            if open {
                cx.back();
            }
        }
        Update::Failed(error) => cx.toast(ToastKind::Error, error),
    }
}
