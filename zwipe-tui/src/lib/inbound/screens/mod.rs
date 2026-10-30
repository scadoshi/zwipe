//! Every screen, and the three enums that register them: [`Screen`], [`Request`] and [`Update`].
//!
//! Each screen lives in its own folder: `mod.rs` for its state, its request and update enums, `perform` and `apply`; `keys.rs` for its keys; `draw.rs` for drawing it. This file only routes to them, one arm per screen. `context/ARCHITECTURE.md` lists every place a new screen registers.

pub(crate) mod account;
pub(crate) mod config;
pub(crate) mod deck;
pub(crate) mod decks;
pub(crate) mod home;
pub(crate) mod login;
pub(crate) mod swipe;
pub(crate) mod update_required;

use std::fmt;

use crossterm::event::KeyCode;
use ratatui::Frame;
use zwipe_client::ClientError;
use zwipe_core::domain::user::User;
use zwipe_core::version::version_at_least;

use self::account::AccountView;
use self::config::ConfigView;
use self::deck::DeckView;
use self::decks::DeckList;
use self::home::HomeView;
use self::login::LoginForm;
use self::swipe::SwipeView;
use super::app::{Context, Shared};
use super::nav::Nav;
use super::widgets::toast::ToastKind;
use crate::outbound::account::Account;
use crate::outbound::theme_file::{RememberedTheme, ThemeFile};

/// This build's version, which the server's minimum client version is checked against.
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

/// One open screen and its state.
#[derive(Debug)]
pub(crate) enum Screen {
    Login(LoginForm),
    Home(Box<HomeView>),
    Decks(DeckList),
    Deck(Box<DeckView>),
    Swipe(Box<SwipeView>),
    Account(Box<AccountView>),
    Config(Box<ConfigView>),
    /// The server no longer accepts this version. Only quitting is possible.
    UpdateRequired,
}

impl Screen {
    /// Hands a key to the screen.
    pub(crate) fn handle_key(&mut self, code: KeyCode, cx: &mut Context<'_>) {
        match self {
            Self::Login(form) => login::keys::handle(form, code, cx),
            Self::Home(view) => home::keys::handle(view, code, cx),
            Self::Decks(list) => decks::keys::handle(list, code, cx),
            Self::Deck(view) => deck::keys::handle(view, code, cx),
            Self::Swipe(view) => swipe::keys::handle(view, code, cx),
            Self::Account(view) => account::keys::handle(view, code, cx),
            Self::Config(view) => config::keys::handle(view, code, cx),
            Self::UpdateRequired => update_required::keys::handle(code, cx),
        }
    }

    pub(crate) fn draw(&self, f: &mut Frame<'_>, shared: &Shared) {
        match self {
            Self::Login(form) => login::draw::draw(f, shared, form),
            Self::Home(view) => home::draw::draw(f, shared, view),
            Self::Decks(list) => decks::draw::draw(f, shared, list),
            Self::Deck(view) => deck::draw::draw(f, shared, view),
            Self::Swipe(view) => swipe::draw::draw(f, shared, view),
            Self::Account(view) => account::draw::draw(f, shared, view),
            Self::Config(view) => config::draw::draw(f, shared, view),
            Self::UpdateRequired => update_required::draw::draw(f, shared),
        }
    }

    /// Whether the screen is moving, so the loop redraws it on a timer.
    pub(crate) fn animating(&self) -> bool {
        match self {
            Self::Home(view) => view.animating(),
            Self::Account(view) => view.animating(),
            _ => false,
        }
    }

    /// Called when the screen over this one closes, so this one can refresh.
    pub(crate) fn on_return(&mut self, cx: &mut Context<'_>) {
        match self {
            Self::Home(view) => view.refresh(cx),
            Self::Decks(list) => list.reload(cx),
            Self::Deck(view) => view.reload(cx),
            _ => {}
        }
    }
}

/// Background work a handler asks for: the app's own, or one screen's.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    /// Compares this build with the server's minimum client version.
    CheckVersion,
    SignOut,
    /// Saves the account's theme for drawing before the next sign-in.
    RememberTheme(RememberedTheme),
    /// Tells the server a one-time hint has been shown, by its key.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "sent by the hint widget, which no screen shows yet"
        )
    )]
    MarkHintShown(String),
    Login(login::Request),
    Home(home::Request),
    Decks(decks::Request),
    Deck(deck::Request),
    Swipe(swipe::Request),
    Account(account::Request),
    Config(config::Request),
}

/// A result from a background request: the app's own, or one screen's.
#[derive(Debug)]
pub(crate) enum Update {
    /// The server's minimum client version is newer than this build.
    TooOld,
    /// The server no longer accepts the session.
    SessionEnded,
    SignedOut,
    /// A message for the status line.
    Status(String),
    /// A toast for a result that has no screen to show it on.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "for screens still to come; no request toasts its result yet"
        )
    )]
    Toast(ToastKind, String),
    /// The user as the server keeps it after marking a hint shown.
    HintMarked(Box<User>),
    Login(login::Update),
    Home(home::Update),
    Decks(decks::Update),
    Deck(deck::Update),
    Swipe(swipe::Update),
    Account(account::Update),
    Config(config::Update),
}

/// Runs one request off the main loop.
pub(crate) async fn perform(
    account: &Account,
    theme_file: &ThemeFile,
    request: Request,
) -> Option<Update> {
    match request {
        Request::CheckVersion => {
            let minimum = account.client.fetch_min_client_version().await.ok()?;
            (!version_at_least(VERSION, &minimum.min_version)).then_some(Update::TooOld)
        }
        Request::SignOut => {
            account.sign_out().await;
            Some(Update::SignedOut)
        }
        Request::RememberTheme(theme) => {
            // Only the next launch's first screen depends on it, so a failed write is not worth a message.
            theme_file.save(&theme).ok();
            None
        }
        Request::MarkHintShown(hint) => {
            // The hint is already marked on the local copy, so a failure only means it may show again next launch.
            async {
                account
                    .client
                    .mark_hint_shown(&hint, &account.fresh().await?)
                    .await
            }
            .await
            .ok()
            .map(|user| Update::HintMarked(Box::new(user)))
        }
        Request::Login(request) => login::perform(account, request).await,
        Request::Home(request) => home::perform(account, request).await,
        Request::Decks(request) => decks::perform(account, request).await,
        Request::Deck(request) => deck::perform(account, request).await,
        Request::Swipe(request) => swipe::perform(account, request).await,
        Request::Account(request) => account::perform(account, request).await,
        Request::Config(request) => config::perform(account, request).await,
    }
}

/// Applies one result: the app's own here, a screen's by its module, which finds its screen in `nav` if it is open.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    match update {
        Update::TooOld => cx.reset(Screen::UpdateRequired),
        Update::SessionEnded => {
            login::signed_out(cx, Some("Your session ended. Sign in again.".to_owned()));
            cx.request(Request::SignOut);
        }
        Update::SignedOut => login::signed_out(cx, None),
        Update::Status(status) => cx.shared.status = status,
        Update::Toast(kind, message) => cx.toast(kind, message),
        Update::HintMarked(user) => {
            cx.shared.username = Some(user.username.to_string());
            cx.shared.user = Some(*user);
        }
        Update::Login(update) => login::apply(nav, update, cx),
        Update::Home(update) => home::apply(nav, update, cx),
        Update::Decks(update) => decks::apply(nav, update, cx),
        Update::Deck(update) => deck::apply(nav, update, cx),
        Update::Swipe(update) => swipe::apply(nav, update, cx),
        Update::Account(update) => account::apply(nav, update, cx),
        Update::Config(update) => config::apply(nav, update, cx),
    }
}

/// An update for a failed request: the session ending when the server rejects it, else `report` with a message fit to show.
pub(crate) fn failure(error: &ClientError, report: impl FnOnce(String) -> Update) -> Update {
    match error {
        ClientError::Unauthorized(_) => Update::SessionEnded,
        other => report(other.to_user_message()),
    }
}

/// Starts signing out: says so on the status line and asks the server to end the session.
pub(crate) fn sign_out(cx: &mut Context<'_>, message: &str) {
    message.clone_into(&mut cx.shared.status);
    cx.request(Request::SignOut);
}

/// A password on its way to the server, never shown in `Debug` output.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Password(pub(crate) String);

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Password(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failed_request_ends_the_session_only_on_unauthorized() {
        assert!(matches!(
            failure(&ClientError::Unauthorized("x".to_owned()), Update::Status),
            Update::SessionEnded
        ));
        assert!(matches!(
            failure(&ClientError::NotFound("gone".to_owned()), Update::Status),
            Update::Status(m) if m == "gone"
        ));
    }

    #[test]
    fn the_password_never_shows_in_debug_output() {
        let request = Request::Login(login::Request::SignIn {
            identifier: "scotty".to_owned(),
            password: Password("hunter2".to_owned()),
        });
        assert!(!format!("{request:?}").contains("hunter2"));
    }
}
