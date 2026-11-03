//! The Login screen: the hero over a username-or-email and password form with `Log in` under it, `c Create profile` and `f Forgot password` in the key bar; and what signing in and out do to the app.
//!
//! What the screen knows and decides is zwipe-ui's [`State`]: the submit gate, the one sign-in out and where each button leads. This view adds the text boxes, the focus and Show and Hide. Esc ends typing: with no field active the letters are commands (`c`, `f`, `q` and `!`), and Tab or Shift+Tab goes back into the form. Esc never quits; `q` does, with no field active.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use zwipe_core::domain::auth::models::session::Session;
use zwipe_ui::account::Password;
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::login::{Action, Destination, Effect, Field, State};

use super::{Screen, config, forgot_password, home, register};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::hero::{self, Hero};
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The Login screen's state: the model plus what only the terminal keeps.
#[derive(Debug, PartialEq)]
pub(crate) struct LoginForm {
    pub(crate) model: State,
    /// The boxes the fields are typed in. Each value follows every keystroke into the model, and the model rewrites them when it fills or empties a field.
    pub(crate) identifier: TextInput,
    pub(crate) password: TextInput,
    /// The stop with the focus, or `None` once Esc ends typing.
    pub(crate) focus: Option<LoginField>,
    pub(crate) hero: Hero,
}

impl Default for LoginForm {
    fn default() -> Self {
        Self {
            model: State::new(),
            identifier: TextInput::default(),
            password: TextInput::masked(),
            focus: Some(LoginField::default()),
            hero: Hero::default(),
        }
    }
}

impl LoginForm {
    /// The text box with the cursor and the model's field it fills, if the focus is on one.
    pub(crate) fn focused_mut(&mut self) -> Option<(&mut TextInput, Field)> {
        match self.focus {
            Some(LoginField::Identifier) => Some((&mut self.identifier, Field::Identifier)),
            Some(LoginField::Password) => Some((&mut self.password, Field::Password)),
            _ => None,
        }
    }
}

/// The model's edit action for `field` holding `value`.
pub(crate) fn edit(field: Field, value: String) -> Action {
    match field {
        Field::Identifier => Action::EditIdentifier(value),
        Field::Password => Action::EditPassword(Password::new(value)),
    }
}

/// Applies `action` to the model and performs the effects it returns, in order.
pub(crate) fn dispatch(form: &mut LoginForm, action: Action, cx: &mut Context<'_>) {
    let effects = form.model.update(action);
    for effect in effects {
        match effect {
            Effect::SignIn {
                ticket,
                identifier,
                password,
            } => cx.request(
                Request::SignIn {
                    ticket,
                    identifier,
                    password,
                }
                .into(),
            ),
            Effect::SetIdentifier(value) => form.identifier.set_value(&value),
            Effect::ClearPassword => form.password.set_value(""),
            Effect::SignedIn(session) => adopt(cx, &session),
            Effect::Navigate(Destination::Home) => home::open(cx),
            Effect::Navigate(Destination::Register) => register::open(cx),
            Effect::Navigate(Destination::ForgotPassword) => forgot_password::open(cx),
            Effect::Toast(toast) => cx.show(toast),
        }
    }
}

/// A stop the focus can rest on: a field, the password's Show/Hide, or the `Log in` button under them.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoginField {
    #[default]
    Identifier,
    Password,
    /// The Show or Hide button inside the password field.
    ShowPassword,
    LogIn,
}

impl LoginField {
    /// Every stop, in Tab order.
    const ORDER: [Self; 4] = [
        Self::Identifier,
        Self::Password,
        Self::ShowPassword,
        Self::LogIn,
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

    /// The button this stop is, by its place in the row under the form.
    pub(crate) fn button(self) -> Option<usize> {
        (self == Self::LogIn).then_some(0)
    }
}

/// The Login screen's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    SignIn {
        ticket: Ticket,
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

/// The Login screen's results, each carrying its request's ticket back to the model.
#[derive(Debug)]
pub(crate) enum Update {
    SignedIn {
        ticket: Ticket,
        session: Box<Session>,
    },
    Failed {
        ticket: Ticket,
        error: String,
    },
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
    let Request::SignIn {
        ticket,
        identifier,
        password,
    } = request;
    Some(
        match account.sign_in(&identifier, password.read()).await {
            Ok(session) => Update::SignedIn {
                ticket,
                session: Box::new(session),
            },
            Err(e) => Update::Failed {
                ticket,
                error: screens::shown(&e),
            },
        }
        .into(),
    )
}

/// Hands a reply to the model. A failure raises its toast first, as the app's request runner does.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let action = match update {
        Update::SignedIn { ticket, session } => Action::LoggedIn { ticket, session },
        Update::Failed { ticket, error } => {
            cx.toast(ToastKind::Error, error);
            Action::Failed { ticket }
        }
    };
    if let Some(form) = nav.find_mut(pick) {
        dispatch(form, action, cx);
    }
}

/// Login comes back into view: the hero mounts again, and a username Register left in the handoff goes to the model to fill in, with the focus on the password.
pub(crate) fn on_return(form: &mut LoginForm, cx: &mut Context<'_>) {
    form.hero = hero::mount(cx);
    if let Some(username) = cx.shared.caches.handoff.login_prefill.take() {
        dispatch(form, Action::Prefill(username), cx);
        form.focus = Some(LoginField::Password);
    }
}

/// Starts over from home signed in with `session`, as a launch with a saved session does.
pub(crate) fn signed_in(cx: &mut Context<'_>, session: &Session) {
    adopt(cx, session);
    home::open(cx);
}

/// Takes `session` as the signed-in one: applies the account's theme, remembers it for the next launch, and holds the user.
fn adopt(cx: &mut Context<'_>, session: &Session) {
    cx.use_preferences(&session.preferences);
    cx.request(config::Request::AdoptPreferences.into());
    cx.shared.username = Some(session.user.username.to_string());
    cx.shared.user = Some(session.user.clone());
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
