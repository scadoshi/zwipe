//! The Config screen: the theme with its Dark/Light button and the Themes sheet, the pip style, Universes Beyond and its Exceptions sheet.
//!
//! What the screen knows and decides is zwipe-ui's [`State`]: Universes Beyond and its exceptions, the optimistic toggles' tickets, which save lands, and every toast. The theme and the pips are the client's live values, kept here in [`ConfigView::shown`] and drawn by the whole app; the model hands each change to them back as an effect. The franchises the exceptions can name are the client's too.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use zwipe_client::ClientError;
use zwipe_core::domain::card::scryfall_data::universe::UbFranchiseView;
use zwipe_core::domain::user::preferences::UserPreferences;
use zwipe_core::http::contracts::user::HttpUpdatePreferences;
use zwipe_ui::generation::Ticket;
use zwipe_ui::preferences::PreferenceChange;
use zwipe_ui::screens::config::{
    Action, Destination, Effect, SaveTicket, Sheet as ModelSheet, State, Theme,
};

use super::Screen;
use crate::domain::theme::{self, Palette, PickerRow};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::hint::{self, HintTopic};
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// A button on Config's rows, in focus order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stop {
    /// The Dark or Light button in the Theme row.
    Mode,
    /// The Theme row's Change, which opens the Themes sheet.
    Theme,
    Pips,
    UniversesBeyond,
    Exceptions,
}

impl Stop {
    pub(crate) const ALL: [Self; 5] = [
        Self::Mode,
        Self::Theme,
        Self::Pips,
        Self::UniversesBeyond,
        Self::Exceptions,
    ];
}

/// The Themes sheet: the theme it opened on, and the row under the selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ThemeSheet {
    /// The theme the sheet opened on, which Back puts back.
    pub(crate) original: String,
    /// The selected row of [`theme::picker_rows`], always a theme.
    pub(crate) row: usize,
}

impl ThemeSheet {
    /// A sheet on `current`, or on the first theme when `current` is not listed.
    pub(crate) fn opening(current: &str) -> Self {
        let slug = theme::current_slug(current);
        let rows = theme::picker_rows();
        let row = rows
            .iter()
            .position(|r| matches!(r, PickerRow::Theme(s) if *s == slug))
            .or_else(|| rows.iter().position(|r| matches!(r, PickerRow::Theme(_))))
            .unwrap_or(0);
        Self {
            original: slug.to_owned(),
            row,
        }
    }

    /// The theme under the selection.
    pub(crate) fn theme(&self) -> Option<&'static str> {
        match theme::picker_rows().get(self.row) {
            Some(PickerRow::Theme(slug)) => Some(slug),
            _ => None,
        }
    }

    /// Whether a different theme than the one it opened on is picked.
    pub(crate) fn picked(&self) -> bool {
        self.theme().is_some_and(|t| t != self.original)
    }

    /// Moves to the next theme down, or up toward the previous, skipping section labels.
    pub(crate) fn step(&mut self, toward: Toward) {
        let rows = theme::picker_rows();
        let is_theme = |i: &usize| matches!(rows.get(*i), Some(PickerRow::Theme(_)));
        let found = if toward == Toward::Next {
            (self.row.saturating_add(1)..rows.len()).find(is_theme)
        } else {
            (0..self.row).rev().find(is_theme)
        };
        if let Some(row) = found {
            self.row = row;
        }
    }
}

/// What is open over Config. The Exceptions sheet's draft is the model's; the sheet keeps only the focused chip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Sheet {
    Themes(ThemeSheet),
    Exceptions { chip: usize },
}

/// The Config screen's state: the model plus what only the terminal keeps.
#[derive(Debug)]
pub(crate) struct ConfigView {
    pub(crate) model: State,
    /// The theme, mode and pips the app draws in now: the preferences as last applied, with any preview or optimistic toggle over them. Universes Beyond is the model's, so its fields here are never read.
    pub(crate) shown: UserPreferences,
    pub(crate) focus: usize,
    pub(crate) sheet: Option<Sheet>,
    /// The franchises the exceptions can name, as the server lists them; empty until they load.
    pub(crate) franchises: Vec<UbFranchiseView>,
}

impl ConfigView {
    /// Config on `shown`, the preferences as last applied, before [`Action::Load`].
    pub(crate) fn new(shown: UserPreferences) -> Self {
        Self {
            model: State::new(),
            shown,
            focus: 0,
            sheet: None,
            franchises: Vec::new(),
        }
    }

    pub(crate) fn focused(&self) -> Stop {
        Stop::ALL.get(self.focus).copied().unwrap_or(Stop::Mode)
    }

    pub(crate) fn next(&mut self) {
        self.focus = self
            .focus
            .saturating_add(1)
            .min(Stop::ALL.len().saturating_sub(1));
    }

    pub(crate) fn previous(&mut self) {
        self.focus = self.focus.saturating_sub(1);
    }

    /// Whether `stop` can be pressed, as the model says for the two Universes Beyond buttons.
    pub(crate) fn enabled(&self, stop: Stop) -> bool {
        match stop {
            Stop::UniversesBeyond => self.model.universes_beyond_enabled(),
            Stop::Exceptions => self.model.exceptions_enabled(),
            Stop::Mode | Stop::Theme | Stop::Pips => true,
        }
    }

    /// The theme drawn now, as the model's actions carry it.
    pub(crate) fn theme(&self) -> Theme {
        Theme {
            name: self.shown.theme.clone(),
            dark_mode: self.shown.dark_mode,
        }
    }
}

/// Applies `action` to the model and performs the effects it returns, in order.
pub(crate) fn dispatch(view: &mut ConfigView, action: Action, cx: &mut Context<'_>) {
    for effect in view.model.update(action) {
        perform_effect(view, effect, cx);
    }
}

fn perform_effect(view: &mut ConfigView, effect: Effect, cx: &mut Context<'_>) {
    match effect {
        Effect::OfferHint(offer) => {
            hint::once(cx, offer.key());
        }
        Effect::FetchPreferences(ticket) => {
            cx.request(Request::Job(Job::LoadPreferences { ticket }).into());
        }
        Effect::Save { ticket, change } => {
            cx.request(Request::Job(Job::Save { ticket, change }).into());
        }
        Effect::ShowTheme(theme) => {
            set_theme(view, theme);
            show(cx, &view.shown);
        }
        Effect::ShowPips(style) => {
            view.shown.pip_style = style;
            show(cx, &view.shown);
        }
        Effect::ThemeSaved(theme) => {
            set_theme(view, theme);
            keep(cx, &view.shown);
        }
        Effect::PipsSaved(style) => {
            view.shown.pip_style = style;
            keep(cx, &view.shown);
        }
        Effect::OpenSheet(ModelSheet::Themes) => {
            view.sheet = Some(Sheet::Themes(ThemeSheet::opening(&view.shown.theme)));
        }
        Effect::OpenSheet(ModelSheet::Exceptions) => {
            view.sheet = Some(Sheet::Exceptions { chip: 0 });
        }
        Effect::CloseSheet(closing) => {
            let open = match &view.sheet {
                Some(Sheet::Themes(_)) => Some(ModelSheet::Themes),
                Some(Sheet::Exceptions { .. }) => Some(ModelSheet::Exceptions),
                None => None,
            };
            if open == Some(closing) {
                view.sheet = None;
            }
        }
        Effect::Navigate(Destination::Previous) => cx.back(),
        Effect::Toast(toast) => cx.show(toast),
    }
}

/// Puts `theme` on the view's live preferences.
fn set_theme(view: &mut ConfigView, theme: Theme) {
    view.shown.theme = theme.name;
    view.shown.dark_mode = theme.dark_mode;
}

/// Draws in `shown`'s theme and pips, as the app's live theme.
fn show(cx: &mut Context<'_>, shown: &UserPreferences) {
    cx.shared.palette = Palette::for_preferences(shown);
}

/// Draws in `shown`'s theme and pips from now on and keeps them with the app's preferences, so the next launch opens in them.
fn keep(cx: &mut Context<'_>, shown: &UserPreferences) {
    let mut kept = cx.shared.preferences.clone().unwrap_or_default();
    kept.theme.clone_from(&shown.theme);
    kept.dark_mode = shown.dark_mode;
    kept.pip_style = shown.pip_style;
    cx.use_preferences(&kept);
}

/// Config's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// Fetches the preferences at sign-in, to draw in the account's theme.
    AdoptPreferences,
    /// The screen's own work.
    Job(Job),
}

/// One piece of Config's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Job {
    /// Fetches the preferences. A failure says nothing.
    LoadPreferences { ticket: Ticket },
    /// Sends one change as its sparse update.
    Save {
        ticket: SaveTicket,
        change: PreferenceChange,
    },
    /// Fetches the franchises the exceptions can name.
    LoadFranchises,
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::AdoptPreferences | Self::Job(Job::LoadPreferences { .. }) => "load_preferences",
            Self::Job(Job::Save { .. }) => "save",
            Self::Job(Job::LoadFranchises) => "load_franchises",
        }
    }
}

/// Config's results.
#[derive(Debug)]
pub(crate) enum Update {
    /// The account's preferences fetched at sign-in.
    Adopt(Box<UserPreferences>),
    /// A reply to the screen's own work.
    Reply(Reply),
}

/// One reply, carrying its request's ticket back to the model.
#[derive(Debug)]
pub(crate) enum Reply {
    Preferences {
        ticket: Ticket,
        preferences: Box<UserPreferences>,
    },
    PreferencesUnavailable {
        ticket: Ticket,
    },
    Saved {
        ticket: SaveTicket,
        preferences: Box<UserPreferences>,
    },
    SaveFailed {
        ticket: SaveTicket,
        message: String,
    },
    Franchises(Vec<UbFranchiseView>),
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Config(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Config(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut ConfigView> {
    match screen {
        Screen::Config(view) => Some(view),
        _ => None,
    }
}

/// Opens Config on the preferences as last applied; the model fetches them and offers the `config` hint, and the franchises load beside them.
pub(crate) fn open(cx: &mut Context<'_>) {
    let shown = cx.shared.preferences.clone().unwrap_or_default();
    let mut view = ConfigView::new(shown);
    dispatch(&mut view, Action::Load, cx);
    cx.request(Request::Job(Job::LoadFranchises).into());
    cx.push(Screen::Config(Box::new(view)));
}

/// The `?` topic of a stop's row.
pub(crate) fn topic(stop: Stop) -> Option<HintTopic> {
    match stop {
        Stop::Mode | Stop::Theme => Some(HintTopic::Theme),
        Stop::Pips => Some(HintTopic::Pips),
        Stop::UniversesBeyond => Some(HintTopic::UniversesBeyond),
        Stop::Exceptions => None,
    }
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let job = match request {
        Request::AdoptPreferences => {
            let preferences = async { client.fetch_preferences(&account.fresh().await?).await };
            return match preferences.await {
                Ok(preferences) => {
                    account.set_preferences(preferences.clone()).await;
                    Some(Update::Adopt(Box::new(preferences)).into())
                }
                Err(ClientError::Unauthorized(_)) => Some(screens::Update::SessionEnded),
                // Reported, and otherwise quiet: the theme stays the one the session saved.
                Err(e) => {
                    screens::shown(&e);
                    None
                }
            };
        }
        Request::Job(job) => job,
    };
    let reply = match job {
        Job::LoadPreferences { ticket } => {
            match async { client.fetch_preferences(&account.fresh().await?).await }.await {
                Ok(preferences) => {
                    account.set_preferences(preferences.clone()).await;
                    Reply::Preferences {
                        ticket,
                        preferences: Box::new(preferences),
                    }
                }
                Err(e) => {
                    return Some(screens::failure(&e, |_| {
                        Update::Reply(Reply::PreferencesUnavailable { ticket }).into()
                    }));
                }
            }
        }
        Job::Save { ticket, change } => {
            let result = async {
                let session = account.fresh().await?;
                client
                    .update_preferences(HttpUpdatePreferences::from(change), &session)
                    .await
            }
            .await;
            match result {
                Ok(preferences) => {
                    account.set_preferences(preferences.clone()).await;
                    Reply::Saved {
                        ticket,
                        preferences: Box::new(preferences),
                    }
                }
                Err(e) => {
                    return Some(screens::failure(&e, |message| {
                        Update::Reply(Reply::SaveFailed { ticket, message }).into()
                    }));
                }
            }
        }
        Job::LoadFranchises => Reply::Franchises(client.fetch_ub_franchises().await.ok()?),
    };
    Some(Update::Reply(reply).into())
}

/// Hands a reply to the open screen's model, which drops one whose ticket it did not issue. With Config closed, fetched or saved preferences become the app's, and a failure's toast still shows.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let reply = match update {
        Update::Adopt(preferences) => {
            cx.use_preferences(&preferences);
            return;
        }
        Update::Reply(reply) => reply,
    };
    let view = nav.find_mut(pick);
    let action = match reply {
        Reply::Preferences {
            ticket,
            preferences,
        } => {
            let Some(view) = view else {
                cx.use_preferences(&preferences);
                return;
            };
            dispatch(
                view,
                Action::Loaded {
                    ticket,
                    preferences,
                },
                cx,
            );
            return;
        }
        Reply::PreferencesUnavailable { ticket } => Action::Failed { ticket },
        Reply::Saved {
            ticket,
            preferences,
        } => {
            let Some(view) = view else {
                cx.use_preferences(&preferences);
                return;
            };
            // Universes Beyond as the server holds it, for the screens that filter by it.
            let mut kept = cx.shared.preferences.clone().unwrap_or_default();
            kept.exclude_universes_beyond = preferences.exclude_universes_beyond;
            kept.universes_beyond_exceptions
                .clone_from(&preferences.universes_beyond_exceptions);
            cx.shared.preferences = Some(kept);
            dispatch(
                view,
                Action::Saved {
                    ticket,
                    preferences,
                },
                cx,
            );
            return;
        }
        Reply::SaveFailed { ticket, message } => {
            cx.toast(ToastKind::Error, message);
            Action::SaveFailed { ticket }
        }
        Reply::Franchises(franchises) => {
            if let Some(view) = view {
                view.franchises = franchises;
            }
            return;
        }
    };
    if let Some(view) = view {
        dispatch(view, action, cx);
    }
}
