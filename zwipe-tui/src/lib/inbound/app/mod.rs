//! What the screens draw from: which screen is up and its state.

mod account;
mod config;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::{Deck, DeckProfile};
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;

pub(crate) use self::account::{AccountRow, AccountView, Dialog, Form, FormKind, pending_note};
pub(crate) use self::config::{
    ConfigRow, ConfigView, ExceptionsEditor, PreferenceChange, ThemePicker,
};
use crate::domain::decode;
use crate::domain::home::{Figures, Scope};
use crate::domain::swipe::Stack;
use crate::domain::theme::Palette;

/// Everything the UI draws from.
#[derive(Debug)]
pub(crate) struct App {
    pub(crate) screen: Screen,
    /// The account's ZWIPE theme, or before sign-in the last account's, or gruvbox dark.
    pub(crate) palette: Palette,
    pub(crate) username: Option<String>,
    /// The signed-in user as last fetched, which the account screen opens on.
    pub(crate) user: Option<zwipe_core::domain::user::User>,
    /// A one-line message under the deck list and the swipe screen.
    pub(crate) status: String,
    /// How this terminal draws images, or `None` when card images are off.
    pub(crate) images: Option<Picker>,
}

impl App {
    /// Whether something on screen is moving, so the loop should redraw on a timer.
    pub(crate) fn animating(&self) -> bool {
        match &self.screen {
            Screen::Home(home) => home.logo_elapsed() < decode::DURATION,
            Screen::Account(view) => {
                view.resend_wait().is_some()
                    || matches!(&view.dialog, Some(Dialog::Form(form)) if form.delete_wait().is_some())
            }
            _ => false,
        }
    }

    pub(crate) fn new(palette: Palette) -> Self {
        Self {
            screen: Screen::Login(LoginForm::default()),
            palette,
            username: None,
            user: None,
            status: String::new(),
            images: None,
        }
    }
}

/// Which screen is up.
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

/// The home screen: the counters, the color map and the featured flavor card.
#[derive(Debug, Default)]
pub(crate) struct HomeView {
    pub(crate) scope: Scope,
    pub(crate) everybody: Option<Figures>,
    pub(crate) mine: Option<Figures>,
    pub(crate) play_style: Option<HttpPlayStyleSummary>,
    pub(crate) play_style_failed: bool,
    /// The card the server features this hour. Shown only when it has flavor text.
    pub(crate) flavor: Option<Card>,
    /// When the logo started decoding in. `None` draws it settled.
    pub(crate) logo_started: Option<Instant>,
}

impl HomeView {
    /// A home screen whose logo starts decoding in now.
    pub(crate) fn opening() -> Self {
        Self {
            logo_started: Some(Instant::now()),
            ..Self::default()
        }
    }

    /// How far the logo's entrance has run; settled when it never started.
    pub(crate) fn logo_elapsed(&self) -> Duration {
        self.logo_started.map_or(decode::DURATION, |t| t.elapsed())
    }

    /// The counters for the scope on screen, `None` until they arrive.
    pub(crate) fn figures(&self) -> Option<Figures> {
        match self.scope {
            Scope::Everybody => self.everybody,
            Scope::OnlyMe => self.mine,
        }
    }
}

/// One tab of the deck screen, in the app's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Section {
    Budget,
    Tags,
    Distributions,
    Mana,
    DrawOdds,
    Warnings,
}

impl Section {
    pub(crate) const ALL: [Self; 6] = [
        Self::Budget,
        Self::Tags,
        Self::Distributions,
        Self::Mana,
        Self::DrawOdds,
        Self::Warnings,
    ];

    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Budget => "Budget",
            Self::Tags => "Tags",
            Self::Distributions => "Distributions",
            Self::Mana => "Mana",
            Self::DrawOdds => "Draw odds",
            Self::Warnings => "Warnings",
        }
    }
}

/// Which way a tab change moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    Next,
    Previous,
}

/// The furthest turn the draw odds look ahead, as in the app.
pub(crate) const MAX_TURN: u32 = 20;

/// One deck: its profile, and the stats once the full deck has loaded.
#[derive(Debug)]
pub(crate) struct DeckView {
    pub(crate) profile: DeckProfile,
    pub(crate) deck: Option<Deck>,
    pub(crate) metrics: Option<DeckMetrics>,
    pub(crate) section: Section,
    pub(crate) currency: PriceCurrency,
    /// The turn the draw odds are for; 0 is the opening hand.
    pub(crate) turn: u32,
    pub(crate) error: Option<String>,
}

impl DeckView {
    pub(crate) fn opening(profile: DeckProfile) -> Self {
        let currency = profile.price_target_currency.unwrap_or_default();
        Self {
            profile,
            deck: None,
            metrics: None,
            section: Section::Budget,
            currency,
            turn: 0,
            error: None,
        }
    }

    /// Takes the loaded deck and works out its stats, keeping the tab when it still applies.
    pub(crate) fn load(&mut self, deck: Deck) {
        self.profile = deck.deck_profile.clone();
        self.metrics =
            (!deck.entries.is_empty() || !deck.command_zone_cards.is_empty()).then(|| {
                DeckMetrics::from_entries_and_command_zone(&deck.entries, &deck.command_zone_cards)
            });
        self.deck = Some(deck);
        self.error = None;
        let sections = self.sections();
        if !sections.contains(&self.section)
            && let Some(first) = sections.first()
        {
            self.section = *first;
        }
    }

    /// The metrics when the deck has cards to chart.
    pub(crate) fn charted(&self) -> Option<&DeckMetrics> {
        self.metrics
            .as_ref()
            .filter(|m| m.land_count.saturating_add(m.nonland_count) > 0)
    }

    /// The tabs that have something to show, under the same rules as the app's sections.
    pub(crate) fn sections(&self) -> Vec<Section> {
        let profile = &self.profile;
        let has_tags = !profile.tags.is_empty()
            || !profile.oracle_tags.is_empty()
            || !profile.other_tags.is_empty();
        let warnings = self.deck.as_ref().is_some_and(|d| !d.warnings.is_empty());
        let charted = self.charted().is_some();
        Section::ALL
            .into_iter()
            .filter(|section| match section {
                Section::Budget => profile.price_target.is_some() || self.metrics.is_some(),
                Section::Tags => has_tags,
                Section::Distributions | Section::DrawOdds => charted,
                Section::Mana => charted || profile.land_target.is_some(),
                Section::Warnings => warnings,
            })
            .collect()
    }

    /// Moves to the next or previous tab that applies, wrapping around.
    pub(crate) fn step_section(&mut self, step: Step) {
        let sections = self.sections();
        let Some(here) = sections.iter().position(|s| *s == self.section) else {
            if let Some(first) = sections.first() {
                self.section = *first;
            }
            return;
        };
        let count = sections.len();
        let next = match step {
            Step::Next => here.saturating_add(1) % count,
            Step::Previous => here
                .checked_sub(1)
                .unwrap_or_else(|| count.saturating_sub(1)),
        };
        if let Some(section) = sections.get(next) {
            self.section = *section;
        }
    }

    pub(crate) fn next_currency(&mut self) {
        self.currency = match self.currency {
            PriceCurrency::Usd => PriceCurrency::Eur,
            PriceCurrency::Eur => PriceCurrency::Tix,
            PriceCurrency::Tix => PriceCurrency::Usd,
        };
    }

    pub(crate) fn later_turn(&mut self) {
        self.turn = self.turn.saturating_add(1).min(MAX_TURN);
    }

    pub(crate) fn earlier_turn(&mut self) {
        self.turn = self.turn.saturating_sub(1);
    }
}

/// The user's decks.
#[derive(Debug, Default)]
pub(crate) struct DeckList {
    pub(crate) decks: Vec<DeckProfile>,
    pub(crate) selected: usize,
    pub(crate) loading: bool,
    pub(crate) error: Option<String>,
}

impl DeckList {
    pub(crate) fn loading() -> Self {
        Self {
            loading: true,
            ..Self::default()
        }
    }

    /// Takes the decks, sorted by name ignoring case as in the app, keeping the selection in range.
    pub(crate) fn set_decks(&mut self, mut decks: Vec<DeckProfile>) {
        decks.sort_by_cached_key(|d| d.name.to_lowercase());
        self.decks = decks;
        self.loading = false;
        self.selected = self.selected.min(self.decks.len().saturating_sub(1));
    }

    pub(crate) fn selected_deck(&self) -> Option<&DeckProfile> {
        self.decks.get(self.selected)
    }

    pub(crate) fn next(&mut self) {
        self.selected = self
            .selected
            .saturating_add(1)
            .min(self.decks.len().saturating_sub(1));
    }

    pub(crate) fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
}

/// Swiping cards into one deck.
#[derive(Debug)]
pub(crate) struct SwipeView {
    pub(crate) deck: DeckProfile,
    /// `None` until the deck has loaded and the query is known.
    pub(crate) stack: Option<Stack>,
    pub(crate) query: Option<CardQuery>,
    /// Set while a page of cards is being fetched.
    pub(crate) fetching: bool,
    /// Copies added to the main board here, less the ones undone.
    pub(crate) added: i64,
    pub(crate) error: Option<String>,
    /// Images for the cards around the one on screen, by printing ID.
    pub(crate) images: HashMap<Uuid, CardImage>,
    /// On a narrow terminal, whether the card shows as its image instead of its text.
    pub(crate) show_image: bool,
}

/// A card's image as it loads.
pub(crate) enum CardImage {
    Loading,
    /// Encoded for this terminal on first draw, and again when its area changes.
    Ready(Box<RefCell<StatefulProtocol>>),
    Failed,
}

impl fmt::Debug for CardImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Loading => "Loading",
            Self::Ready(_) => "Ready",
            Self::Failed => "Failed",
        })
    }
}

impl SwipeView {
    pub(crate) fn opening(deck: DeckProfile) -> Self {
        Self {
            deck,
            stack: None,
            query: None,
            fetching: false,
            added: 0,
            error: None,
            images: HashMap::new(),
            show_image: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_focus_toggles_between_the_two_fields() {
        assert_eq!(LoginField::Identifier.other(), LoginField::Password);
        assert_eq!(LoginField::Password.other(), LoginField::Identifier);
    }

    #[test]
    fn decks_sort_by_name_ignoring_case() {
        let mut list = DeckList::loading();
        list.set_decks(vec![
            crate::test_support::deck_profile("satya"),
            crate::test_support::deck_profile("Big fat chud"),
            crate::test_support::deck_profile("isshin"),
        ]);
        let names: Vec<String> = list.decks.iter().map(|d| d.name.to_string()).collect();
        assert_eq!(names, ["Big fat chud", "isshin", "satya"]);
        assert!(!list.loading);
    }

    #[test]
    fn an_empty_deck_list_keeps_the_selection_at_zero() {
        let mut list = DeckList::default();
        list.next();
        list.previous();
        assert_eq!(list.selected, 0);
        assert!(list.selected_deck().is_none());
    }
}
