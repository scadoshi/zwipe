//! Every screen, and the three enums that register them: [`Screen`], [`Request`] and [`Update`].
//!
//! Each screen lives in its own folder: `mod.rs` for its state, its request and update enums, `perform` and `apply`; `keys.rs` for its keys; `draw.rs` for drawing it. This file only routes to them, one arm per screen. `context/ARCHITECTURE.md` lists every place a new screen registers.

pub(crate) mod account;
pub(crate) mod card_sheets;
pub(crate) mod changelog;
pub(crate) mod config;
pub(crate) mod deck;
pub(crate) mod deck_cards;
pub(crate) mod deck_export;
pub(crate) mod deck_form;
pub(crate) mod deck_import;
pub(crate) mod decks;
pub(crate) mod forgot_password;
pub(crate) mod home;
pub(crate) mod login;
pub(crate) mod privacy;
pub(crate) mod register;
pub(crate) mod remove_cards;
pub(crate) mod swipe;
pub(crate) mod update_required;

use std::fmt;
use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use ratatui::Frame;
use zwipe_client::ClientError;
use zwipe_core::domain::user::User;
use zwipe_core::http::contracts::changelog::HttpChangelog;
use zwipe_core::http::contracts::metrics::{AnonymousEventKind, HttpPublicMetrics};

use self::account::AccountView;
use self::changelog::ChangelogView;
use self::config::ConfigView;
use self::deck::DeckView;
use self::deck_export::ExportView;
use self::deck_form::DeckForm;
use self::deck_import::ImportView;
use self::decks::DeckList;
use self::forgot_password::ForgotForm;
use self::home::HomeView;
use self::login::LoginForm;
use self::privacy::PrivacyView;
use self::register::RegisterForm;
use self::swipe::SwipeView;
use super::app::{Context, Shared, session};
use super::nav::Nav;
use super::widgets;
use super::widgets::hero::{self, Counts, Hero};
use super::widgets::toast::ToastKind;
use crate::domain::changelog::ChangelogCache;
use crate::outbound::account::Account;
use crate::outbound::theme_file::{RememberedTheme, ThemeFile};

/// This build's version, which the server's minimum client version is checked against.
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

/// One open screen and its state.
#[derive(Debug)]
pub(crate) enum Screen {
    Login(Box<LoginForm>),
    Register(Box<RegisterForm>),
    ForgotPassword(Box<ForgotForm>),
    Home(Box<HomeView>),
    Decks(Box<DeckList>),
    Deck(Box<DeckView>),
    DeckForm(Box<DeckForm>),
    DeckImport(Box<ImportView>),
    DeckExport(Box<ExportView>),
    Swipe(Box<SwipeView>),
    DeckCards(Box<deck_cards::DeckCardsView>),
    RemoveCards(Box<remove_cards::RemoveView>),
    Account(Box<AccountView>),
    Config(Box<ConfigView>),
    Privacy(Box<PrivacyView>),
    Changelog(Box<ChangelogView>),
    /// The update notice as a screen of its own. The app shows it over the stack instead, while `Shared::update_required` holds; tests use it as a screen with no state.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the notice draws over the stack instead")
    )]
    UpdateRequired,
}

impl Screen {
    /// Hands a key to the screen.
    pub(crate) fn handle_key(&mut self, code: KeyCode, cx: &mut Context<'_>) {
        match self {
            Self::Login(form) => login::keys::handle(form, code, cx),
            Self::Register(form) => register::keys::handle(form, code, cx),
            Self::ForgotPassword(form) => forgot_password::keys::handle(form, code, cx),
            Self::Home(view) => home::keys::handle(view, code, cx),
            Self::Decks(list) => decks::keys::handle(list, code, cx),
            Self::Deck(view) => deck::keys::handle(view, code, cx),
            Self::DeckForm(form) => deck_form::keys::handle(form, code, cx),
            Self::DeckImport(view) => deck_import::keys::handle(view, code, cx),
            Self::DeckExport(view) => deck_export::keys::handle(view, code, cx),
            Self::Swipe(view) => swipe::keys::handle(view, code, cx),
            Self::DeckCards(view) => deck_cards::keys::handle(view, code, cx),
            Self::RemoveCards(view) => remove_cards::keys::handle(view, code, cx),
            Self::Account(view) => account::keys::handle(view, code, cx),
            Self::Config(view) => config::keys::handle(view, code, cx),
            Self::Privacy(view) => privacy::keys::handle(view, code, cx),
            Self::Changelog(view) => changelog::keys::handle(view, code, cx),
            Self::UpdateRequired => update_required::keys::handle(code, cx),
        }
    }

    pub(crate) fn draw(&self, f: &mut Frame<'_>, shared: &Shared) {
        match self {
            Self::Login(form) => login::draw::draw(f, shared, form),
            Self::Register(form) => register::draw::draw(f, shared, form),
            Self::ForgotPassword(form) => forgot_password::draw::draw(f, shared, form),
            Self::Home(view) => home::draw::draw(f, shared, view),
            Self::Decks(list) => decks::draw::draw(f, shared, list),
            Self::Deck(view) => deck::draw::draw(f, shared, view),
            Self::DeckForm(form) => deck_form::draw::draw(f, shared, form),
            Self::DeckImport(view) => deck_import::draw::draw(f, shared, view),
            Self::DeckExport(view) => deck_export::draw::draw(f, shared, view),
            Self::Swipe(view) => swipe::draw::draw(f, shared, view),
            Self::DeckCards(view) => deck_cards::draw::draw(f, shared, view),
            Self::RemoveCards(view) => remove_cards::draw::draw(f, shared, view),
            Self::Account(view) => account::draw::draw(f, shared, view),
            Self::Config(view) => config::draw::draw(f, shared, view),
            Self::Privacy(view) => privacy::draw::draw(f, shared, view),
            Self::Changelog(view) => changelog::draw::draw(f, shared, view),
            Self::UpdateRequired => update_required::draw::draw(f, shared),
        }
    }

    /// Whether the screen is moving, so the loop redraws it on a timer.
    pub(crate) fn animating(&self) -> bool {
        match self {
            Self::Home(view) => view.animating(),
            Self::Account(view) => view.animating(),
            Self::Login(form) => form.hero.animating(Instant::now()),
            Self::Changelog(view) => widgets::entrance::animating(view.filtered_at.elapsed()),
            Self::Register(form) => form.hero.animating(Instant::now()),
            Self::ForgotPassword(form) => form.sending() || form.hero.animating(Instant::now()),
            _ => false,
        }
    }

    /// The sign-in screens' hero, which takes everybody's counts as they arrive.
    pub(crate) fn hero_mut(&mut self) -> Option<&mut Hero> {
        match self {
            Self::Login(form) => Some(&mut form.hero),
            Self::Register(form) => Some(&mut form.hero),
            Self::ForgotPassword(form) => Some(&mut form.hero),
            _ => None,
        }
    }

    /// Called when the screen over this one closes, so this one can refresh.
    pub(crate) fn on_return(&mut self, cx: &mut Context<'_>) {
        match self {
            Self::Home(view) => view.refresh(cx),
            Self::Login(form) => form.hero = hero::mount(cx),
            Self::Decks(list) => list.reload(cx),
            Self::Deck(view) => view.reload(cx),
            Self::DeckCards(view) => view.reload(cx),
            _ => {}
        }
    }
}

/// Background work a handler asks for: the app's own, or one screen's.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    /// Refreshes the session if it needs it and compares this build with the server's minimum client version, every minute from launch.
    Upkeep,
    SignOut,
    /// Sends an anonymous funnel event, before anyone signs in.
    Record(AnonymousEventKind),
    /// Fetches the changelog, once at launch.
    FetchChangelog,
    /// Fetches everybody's counts for the hero, and saves them for the next launch.
    FetchCounts,
    /// Saves the account's theme for drawing before the next sign-in.
    RememberTheme(RememberedTheme),
    /// Tells the server a one-time hint has been shown, by its key.
    MarkHintShown(String),
    Login(login::Request),
    Register(register::Request),
    ForgotPassword(forgot_password::Request),
    Home(home::Request),
    Decks(decks::Request),
    Deck(deck::Request),
    DeckForm(deck_form::Request),
    DeckImport(deck_import::Request),
    DeckExport(deck_export::Request),
    Swipe(swipe::Request),
    DeckCards(deck_cards::Request),
    RemoveCards(remove_cards::Request),
    Account(account::Request),
    Config(config::Request),
    CardSheets(card_sheets::Request),
}

/// A result from a background request: the app's own, or one screen's.
#[derive(Debug)]
pub(crate) enum Update {
    /// The server's minimum client version is newer than this build.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "upkeep reports the version through Upkept")
    )]
    TooOld,
    /// An upkeep's result: whether the session ended, and whether this build is too old, `None` when the server could not say.
    Upkept {
        ended: bool,
        too_old: Option<bool>,
    },
    /// The changelog, or `None` when it could not be fetched.
    Changelog(Option<Box<HttpChangelog>>),
    /// Everybody's counts, freshly fetched.
    Counts(Box<HttpPublicMetrics>),
    /// The server no longer accepts the session.
    SessionEnded,
    SignedOut,
    /// A message for the status line.
    Status(String),
    /// A toast for a result that has no screen to show it on, up for the app's default time.
    Toast(ToastKind, String),
    /// A toast up for one of the app's three bands of time.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "for screens still to come; no request toasts its result yet"
        )
    )]
    ToastFor(ToastKind, String, Duration),
    /// The user as the server keeps it after marking a hint shown.
    HintMarked(Box<User>),
    Login(login::Update),
    Register(register::Update),
    ForgotPassword(forgot_password::Update),
    Home(home::Update),
    Decks(decks::Update),
    Deck(deck::Update),
    DeckForm(deck_form::Update),
    DeckImport(deck_import::Update),
    DeckExport(deck_export::Update),
    Swipe(swipe::Update),
    DeckCards(deck_cards::Update),
    RemoveCards(remove_cards::Update),
    Account(account::Update),
    Config(config::Update),
    CardSheets(card_sheets::Update),
}

/// Runs one request off the main loop.
pub(crate) async fn perform(
    account: &Account,
    theme_file: &ThemeFile,
    request: Request,
) -> Option<Update> {
    match request {
        Request::Upkeep => Some(session::upkeep(account).await),
        Request::Record(kind) => {
            session::record(account, kind).await;
            None
        }
        Request::FetchCounts => {
            // A failed fetch keeps whatever copy is on screen, with no message.
            let metrics = account.client.public_metrics().await.ok()?;
            account.counts.save(&metrics).ok();
            Some(Update::Counts(Box::new(metrics)))
        }
        Request::FetchChangelog => Some(Update::Changelog(
            account.client.fetch_changelog().await.ok().map(Box::new),
        )),
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
            let user = async {
                account
                    .client
                    .mark_hint_shown(&hint, &account.fresh().await?)
                    .await
            }
            .await
            .ok()?;
            account.set_user(user.clone()).await;
            Some(Update::HintMarked(Box::new(user)))
        }
        Request::Login(request) => login::perform(account, request).await,
        Request::Register(request) => register::perform(account, request).await,
        Request::ForgotPassword(request) => forgot_password::perform(account, request).await,
        Request::Home(request) => home::perform(account, request).await,
        Request::Decks(request) => decks::perform(account, request).await,
        Request::Deck(request) => deck::perform(account, request).await,
        Request::DeckForm(request) => deck_form::perform(account, request).await,
        Request::DeckImport(request) => deck_import::perform(account, request).await,
        Request::DeckExport(request) => deck_export::perform(account, request).await,
        Request::Swipe(request) => swipe::perform(account, request).await,
        Request::DeckCards(request) => deck_cards::perform(account, request).await,
        Request::RemoveCards(request) => remove_cards::perform(account, request).await,
        Request::Account(request) => account::perform(account, request).await,
        Request::Config(request) => config::perform(account, request).await,
        Request::CardSheets(request) => card_sheets::perform(account, request).await,
    }
}

/// Applies one result: the app's own here, a screen's by its module, which finds its screen in `nav` if it is open.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    match update {
        Update::TooOld => cx.shared.update_required = true,
        Update::Upkept { ended, too_old } => session::upkept(cx, ended, too_old),
        Update::Changelog(changelog) => {
            cx.shared.changelog =
                changelog.map_or(ChangelogCache::Failed, |c| ChangelogCache::Loaded(*c));
        }
        Update::SessionEnded => session::session_ended(cx),
        Update::Counts(metrics) => {
            let now = Instant::now();
            cx.shared.counts = Counts {
                metrics: Some(*metrics),
                fetched_at: Some(now),
            };
            for hero in nav.screens_mut().filter_map(Screen::hero_mut) {
                hero.take(&cx.shared.counts, now);
            }
        }
        Update::SignedOut => login::signed_out(cx, None),
        Update::Status(status) => cx.shared.status = status,
        Update::Toast(kind, message) => cx.toast(kind, message),
        Update::ToastFor(kind, message, duration) => cx.toast_for(kind, message, duration),
        Update::HintMarked(user) => {
            cx.shared.username = Some(user.username.to_string());
            cx.shared.user = Some(*user);
        }
        Update::Login(update) => login::apply(nav, update, cx),
        Update::Register(update) => register::apply(nav, update, cx),
        Update::ForgotPassword(update) => forgot_password::apply(nav, update, cx),
        Update::Home(update) => home::apply(nav, update, cx),
        Update::Decks(update) => decks::apply(nav, update, cx),
        Update::Deck(update) => deck::apply(nav, update, cx),
        Update::DeckForm(update) => deck_form::apply(nav, update, cx),
        Update::DeckImport(update) => deck_import::apply(nav, update, cx),
        Update::DeckExport(update) => deck_export::apply(nav, update, cx),
        Update::Swipe(update) => swipe::apply(nav, update, cx),
        Update::DeckCards(update) => deck_cards::apply(nav, update, cx),
        Update::RemoveCards(update) => remove_cards::apply(nav, update, cx),
        Update::Account(update) => account::apply(nav, update, cx),
        Update::Config(update) => config::apply(nav, update, cx),
        Update::CardSheets(update) => card_sheets::apply(nav, update, cx),
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
    fn a_toast_update_raises_a_toast_for_its_time() {
        use crate::inbound::event_loop::harness::Harness;
        use crate::inbound::widgets::toast;

        let mut harness = Harness::new();
        harness.apply(Update::ToastFor(
            ToastKind::Warning,
            "Careful".to_owned(),
            toast::LONG,
        ));
        harness.apply(Update::Toast(ToastKind::Info, "Hello".to_owned()));
        let toasts = harness.app.shared.toasts.items();
        assert_eq!(toasts.len(), 2);
        assert_eq!(
            toasts[0].expires_at.duration_since(toasts[0].shown_at),
            toast::LONG
        );
        assert_eq!(
            toasts[1].expires_at.duration_since(toasts[1].shown_at),
            toast::NORMAL
        );
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
