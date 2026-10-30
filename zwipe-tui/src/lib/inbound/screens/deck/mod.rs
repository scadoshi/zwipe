//! One deck: its profile beside the app's stat sections, one tab at a time.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::{Deck, DeckProfile};

use super::{Screen, failure};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::outbound::account::Account;

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

    /// Fetches the deck again, keeping the tab, currency and turn; the stats show as loading until it arrives.
    pub(crate) fn reload(&mut self, cx: &mut Context<'_>) {
        cx.shared.status.clear();
        self.deck = None;
        self.metrics = None;
        self.error = None;
        cx.request(Request::Load(self.profile.id).into());
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

/// The deck screen's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    /// Fetches the full deck.
    Load(Uuid),
}

/// The deck screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded(Box<Deck>),
    Failed(String),
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Deck(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Deck(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut DeckView> {
    match screen {
        Screen::Deck(view) => Some(view),
        _ => None,
    }
}

/// Opens a deck over the current screen and fetches it.
pub(crate) fn open(cx: &mut Context<'_>, profile: DeckProfile) {
    cx.shared.status.clear();
    cx.request(Request::Load(profile.id).into());
    cx.push(Screen::Deck(Box::new(DeckView::opening(profile))));
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    match request {
        Request::Load(deck_id) => Some(
            match async { client.fetch_deck(deck_id, &account.fresh().await?).await }.await {
                Ok(deck) => Update::Loaded(Box::new(deck)).into(),
                Err(e) => failure(&e, |m| Update::Failed(m).into()),
            },
        ),
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, _cx: &mut Context<'_>) {
    let Some(view) = nav.find_mut(pick) else {
        return;
    };
    match update {
        Update::Loaded(deck) => {
            if view.profile.id == deck.deck_profile.id {
                view.load(*deck);
            }
        }
        Update::Failed(error) => view.error = Some(error),
    }
}
