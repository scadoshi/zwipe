//! Every screen, and the three enums that register them: [`Screen`], [`Request`] and [`Update`].
//!
//! Each screen lives in its own folder: `mod.rs` for its state, its request and update enums, `perform` and `apply`; `keys.rs` for its keys; `draw.rs` for drawing it. This file only routes to them, one arm per screen. `context/ARCHITECTURE.md` lists every place a new screen registers.

pub(crate) mod account;
pub(crate) mod activity;
pub(crate) mod card_sheets;
pub(crate) mod changelog;
pub(crate) mod commander_maybeboard;
pub(crate) mod commander_swipe;
pub(crate) mod config;
pub(crate) mod deck;
pub(crate) mod deck_cards;
pub(crate) mod deck_export;
pub(crate) mod deck_form;
pub(crate) mod deck_import;
pub(crate) mod decks;
pub(crate) mod forgot_password;
pub(crate) mod home;
pub(crate) mod image_preview;
pub(crate) mod login;
pub(crate) mod oracle_tags;
pub(crate) mod otag_examples;
pub(crate) mod playstyle;
pub(crate) mod printings;
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
use self::activity::ActivityView;
use self::card_sheets::filter::returned_from_dictionary;
use self::changelog::ChangelogView;
use self::commander_maybeboard::MaybeboardView;
use self::commander_swipe::CommanderSwipeView;
use self::config::ConfigView;
use self::deck::DeckView;
use self::deck_export::ExportView;
use self::deck_form::DeckForm;
use self::deck_import::ImportView;
use self::decks::DeckList;
use self::forgot_password::ForgotForm;
use self::home::HomeView;
use self::image_preview::ImagePreview;
use self::login::LoginForm;
use self::oracle_tags::OracleTagsView;
use self::otag_examples::ExamplesView;
use self::playstyle::PlaystyleView;
use self::printings::PrintingsView;
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
    Activity(Box<ActivityView>),
    Config(Box<ConfigView>),
    Privacy(Box<PrivacyView>),
    Changelog(Box<ChangelogView>),
    ImagePreview(Box<ImagePreview>),
    OracleTags(Box<OracleTagsView>),
    OtagExamples(Box<ExamplesView>),
    Printings(Box<PrintingsView>),
    Playstyle(Box<PlaystyleView>),
    CommanderSwipe(Box<CommanderSwipeView>),
    CommanderMaybeboard(Box<MaybeboardView>),
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
            Self::Activity(view) => activity::keys::handle(view, code, cx),
            Self::Config(view) => config::keys::handle(view, code, cx),
            Self::Privacy(view) => privacy::keys::handle(view, code, cx),
            Self::Changelog(view) => changelog::keys::handle(view, code, cx),
            Self::ImagePreview(view) => image_preview::keys::handle(view, code, cx),
            Self::OracleTags(view) => oracle_tags::keys::handle(view, code, cx),
            Self::OtagExamples(view) => otag_examples::keys::handle(view, code, cx),
            Self::Printings(view) => printings::keys::handle(view, code, cx),
            Self::Playstyle(view) => playstyle::keys::handle(view, code, cx),
            Self::CommanderSwipe(view) => commander_swipe::keys::handle(view, code, cx),
            Self::CommanderMaybeboard(view) => commander_maybeboard::keys::handle(view, code, cx),
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
            Self::Activity(view) => activity::draw::draw(f, shared, view),
            Self::Config(view) => config::draw::draw(f, shared, view),
            Self::Privacy(view) => privacy::draw::draw(f, shared, view),
            Self::Changelog(view) => changelog::draw::draw(f, shared, view),
            Self::ImagePreview(view) => image_preview::draw::draw(f, shared, view),
            Self::OracleTags(view) => oracle_tags::draw::draw(f, shared, view),
            Self::OtagExamples(view) => otag_examples::draw::draw(f, shared, view),
            Self::Printings(view) => printings::draw::draw(f, shared, view),
            Self::Playstyle(view) => playstyle::draw::draw(f, shared, view),
            Self::CommanderSwipe(view) => commander_swipe::draw::draw(f, shared, view),
            Self::CommanderMaybeboard(view) => commander_maybeboard::draw::draw(f, shared, view),
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
            Self::Swipe(view) => view.animating(),
            Self::RemoveCards(view) => view.animating(),
            Self::DeckForm(form) => {
                matches!(&form.overlay, Some(deck_form::Overlay::Swipe(picker)) if picker.animating())
            }
            Self::Register(form) => form.hero.animating(Instant::now()),
            Self::ForgotPassword(form) => form.sending() || form.hero.animating(Instant::now()),
            Self::Activity(view) => view.animating(),
            Self::Playstyle(view) => view.animating(),
            Self::CommanderSwipe(view) => view.animating(),
            Self::OtagExamples(view) => view.animating(),
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
            Self::DeckCards(view) => {
                if !returned_from_dictionary(view.sheet.as_mut(), cx) {
                    view.reload(cx);
                }
            }
            Self::DeckForm(form) => form.on_return(cx),
            Self::Swipe(view) => {
                returned_from_dictionary(view.sheet.as_mut(), cx);
            }
            Self::RemoveCards(view) => {
                returned_from_dictionary(view.sheet.as_mut(), cx);
            }
            Self::Activity(view) => view.on_return(cx),
            Self::Playstyle(view) => view.on_return(cx),
            Self::CommanderMaybeboard(view) => view.on_return(cx),
            Self::CommanderSwipe(view) => {
                returned_from_dictionary(view.sheet.as_mut(), cx);
            }
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
    Activity(activity::Request),
    Config(config::Request),
    CardSheets(card_sheets::Request),
    ImagePreview(image_preview::Request),
    OracleTags(oracle_tags::Request),
    OtagExamples(otag_examples::Request),
    Printings(printings::Request),
    Playstyle(playstyle::Request),
    CommanderSwipe(commander_swipe::Request),
    CommanderMaybeboard(commander_maybeboard::Request),
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
            reason = "the app-level twin of `Toast` for a timed band; every screen toasts its own results"
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
    Activity(activity::Update),
    Config(config::Update),
    CardSheets(card_sheets::Update),
    ImagePreview(image_preview::Update),
    OracleTags(oracle_tags::Update),
    OtagExamples(otag_examples::Update),
    Printings(printings::Update),
    Playstyle(playstyle::Update),
    CommanderSwipe(commander_swipe::Update),
    CommanderMaybeboard(commander_maybeboard::Update),
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
        Request::Activity(request) => activity::perform(account, request).await,
        Request::Config(request) => config::perform(account, request).await,
        Request::CardSheets(request) => card_sheets::perform(account, request).await,
        Request::ImagePreview(request) => image_preview::perform(account, request).await,
        Request::OracleTags(request) => oracle_tags::perform(account, request).await,
        Request::OtagExamples(request) => otag_examples::perform(account, request).await,
        Request::Printings(request) => printings::perform(account, request).await,
        Request::Playstyle(request) => playstyle::perform(account, request).await,
        Request::CommanderSwipe(request) => commander_swipe::perform(account, request).await,
        Request::CommanderMaybeboard(request) => {
            commander_maybeboard::perform(account, request).await
        }
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
            home::counts_arrived(nav, cx.shared);
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
        Update::Activity(update) => activity::apply(nav, update, cx),
        Update::Config(update) => config::apply(nav, update, cx),
        Update::CardSheets(update) => card_sheets::apply(nav, update, cx),
        Update::ImagePreview(update) => image_preview::apply(nav, update, cx),
        Update::OracleTags(update) => oracle_tags::apply(nav, update, cx),
        Update::OtagExamples(update) => otag_examples::apply(nav, update, cx),
        Update::Printings(update) => printings::apply(nav, update, cx),
        Update::Playstyle(update) => playstyle::apply(nav, update, cx),
        Update::CommanderSwipe(update) => commander_swipe::apply(nav, update, cx),
        Update::CommanderMaybeboard(update) => commander_maybeboard::apply(nav, update, cx),
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

    /// The screen's header row: the one holding `title` with the `!` corner first.
    fn header_row(text: &str, title: &str) -> Option<String> {
        let cells: Vec<char> = text.chars().collect();
        cells
            .chunks(100)
            .map(|line| line.iter().collect::<String>())
            .find(|line| line.trim_start().starts_with('!') && line.contains(title))
    }

    #[test]
    fn every_screen_has_the_help_corner_and_a_hint_corner_when_it_has_a_hint() {
        use uuid::Uuid;

        use crate::inbound::event_loop::harness::Harness;
        use crate::test_support::screen_text;

        type Open = Box<dyn FnOnce(&mut Context<'_>)>;
        let deck_id = Uuid::nil();
        let screens: Vec<(&str, bool, Open)> = vec![
            ("Activity", true, Box::new(activity::open)),
            ("Playstyle", true, Box::new(playstyle::open)),
            (
                "Commander maybeboard",
                true,
                Box::new(commander_maybeboard::open),
            ),
            ("Oracle tags", true, Box::new(oracle_tags::open)),
            (
                "Examples",
                true,
                Box::new(|cx: &mut Context<'_>| otag_examples::open(cx, "ramp")),
            ),
            ("Decks", true, Box::new(decks::open)),
            (
                "Deck cards",
                true,
                Box::new(move |cx: &mut Context<'_>| deck_cards::open(cx, deck_id)),
            ),
            (
                "Remove deck cards",
                true,
                Box::new(move |cx: &mut Context<'_>| remove_cards::open(cx, deck_id)),
            ),
        ];
        for (title, hint, open) in screens {
            let mut harness = Harness::on(Screen::Home(Box::default()));
            harness.open(open);
            harness.app.shared.hint = None;
            let text = screen_text(&harness.app);
            let row = header_row(&text, title).unwrap_or_else(|| panic!("{title} has no ! corner"));
            assert_eq!(row.trim_end().ends_with('?'), hint, "{title}'s ? corner");
            harness.press(KeyCode::Char('!'));
            assert!(harness.app.shared.help.is_some(), "! opens help on {title}");
        }
        let harness = Harness::on(Screen::Home(Box::default()));
        let text = screen_text(&harness.app);
        let row = header_row(&text, "Home").expect("Home has a ! corner");
        assert!(!row.trim_end().ends_with('?'), "Home has no ? corner");
    }
}
