//! One deck: its Profile card and the app's sections in as many panels as the terminal fits, the warning fixes, and the More sheet with its dialogs.

pub(crate) mod draw;
pub(crate) mod keys;
pub(crate) mod layout;
#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::{Deck, DeckProfile};
use zwipe_core::domain::site::WEB_BASE;
use zwipe_core::http::contracts::deck::{HttpCloneDeck, HttpUpdateDeckProfile};
use zwipe_core::http::contracts::deck_card::HttpPatchDeckCard;
use zwipe_core::http::helpers::Patch;

use self::layout::Pane;
use super::{Screen, failure};
use crate::domain::deck::buy;
use crate::domain::deck::warning::{self, Fix};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::alert::AlertDialog;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::info::InfoPopup;
use crate::inbound::widgets::menu::{Menu, MenuItem};
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::{self, ToastKind};
use crate::outbound::account::Account;

/// The key the server stores the first-deck hint under.
pub(crate) const HINT_KEY: &str = "first_deck";

/// One section of the deck screen, in the app's order.
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

    /// The section's "?" explainers, as the app's info buttons word them.
    pub(crate) fn info(self) -> InfoPopup {
        match self {
            Self::Budget => InfoPopup::new(
                "Price target",
                &["A budget for the mainboard. Going over warns on the deck"],
            ),
            Self::Tags => InfoPopup::new(
                "Tags",
                &[
                    "Deck tags: the archetypes the deck is built around, like Aristocrats or Voltron",
                    "Oracle tags: the specific things the deck does, like spot removal, ramp or reanimation",
                    "Other tags: labels that do not change suggestions, like Budget or Jank",
                ],
            ),
            Self::Distributions => InfoPopup::new(
                "Distributions",
                &[
                    "Types: mainboard cards by type",
                    "Roles: mainboard cards by role, like ramp or removal. Cards with no role are left out. Tap a role to see its tags",
                ],
            ),
            Self::Mana => InfoPopup::new(
                "Mana",
                &[
                    "Mana curve: nonland mainboard cards by mana value",
                    "Colors: mainboard cards by color",
                    "Mana cost fulfillment: per color, the pips your lands and sources make against the pips your spells need",
                    "A check means you make at least as much as you need",
                ],
            ),
            Self::DrawOdds => InfoPopup::new(
                "Draw odds",
                &[
                    "Chance of at least one card from each group in the opening hand, or by a turn. - and + step the turn",
                    "Random draw only. Mulligans, tutors and card draw are not counted",
                ],
            ),
            Self::Warnings => InfoPopup::new(
                "Warnings",
                &["Warnings flag rule problems, each with a one-tap fix"],
            ),
        }
    }
}

/// Something inside a pane the focus can rest on and press.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Item {
    /// A role in the Roles chart that has tags to show.
    Role(String),
    /// An oracle tag under the open role.
    Tag(String),
    /// The open tag's Examples button.
    Examples(String),
    /// One fix button of one warning.
    Fix { warning: usize, button: usize },
}

/// A dialog over the deck screen.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Dialog {
    Buy(AlertDialog),
    Clone(CloneDialog),
    Share(AlertDialog),
    ClearSkips(AlertDialog),
    Delete(AlertDialog),
    Info(InfoPopup),
}

/// The Clone dialog: a name for the copy, and whether the copy is on its way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CloneDialog {
    pub(crate) name: TextInput,
    /// Focused button: 0 Cancel, 1 Save.
    pub(crate) focused: usize,
    pub(crate) busy: bool,
}

impl CloneDialog {
    /// Prefilled with "{deck name} (clone)".
    pub(crate) fn new(deck_name: &str) -> Self {
        Self {
            name: TextInput::default()
                .with_placeholder("New deck name")
                .with_value(&format!("{deck_name} (clone)")),
            focused: 1,
            busy: false,
        }
    }

    /// The name to send, or `None` while it is blank.
    pub(crate) fn trimmed(&self) -> Option<String> {
        let name = self.name.value().trim();
        (!name.is_empty()).then(|| name.to_owned())
    }
}

/// The furthest turn the draw odds look ahead, as in the app.
pub(crate) const MAX_TURN: u32 = 20;

/// One deck: its profile, the full deck once loaded, and what is open over it.
#[derive(Debug)]
pub(crate) struct DeckView {
    pub(crate) profile: DeckProfile,
    pub(crate) deck: Option<Deck>,
    pub(crate) metrics: Option<DeckMetrics>,
    /// The pane the focus was put on. While it is not showing, the focus rests on [`DeckView::focused`]'s stand-in.
    pub(crate) focus: Pane,
    /// Panes newest first by when they were focused, so a tabbed panel shows the tab looked at last.
    pub(crate) recent: Vec<Pane>,
    /// The focused item in the focused pane, kept in range when read.
    pub(crate) item: usize,
    /// Whether the focused pane fills the body.
    pub(crate) zoomed: bool,
    /// How far the focused pane is scrolled when it has no items to move through, in rows.
    pub(crate) scroll: u16,
    /// The role whose tags show under it.
    pub(crate) role_open: Option<String>,
    /// The tag whose definition shows under the role's tags.
    pub(crate) tag_open: Option<String>,
    /// The budget's currency, seeded from the price target's.
    pub(crate) currency: PriceCurrency,
    /// The turn the draw odds are for; 0 is the opening hand.
    pub(crate) turn: u32,
    /// Whether the deck failed to load.
    pub(crate) failed: bool,
    /// The oracle tag catalog, for tag definitions.
    pub(crate) oracle_tags: Vec<OracleTag>,
    pub(crate) share_token: Option<Uuid>,
    pub(crate) more: Option<Menu>,
    pub(crate) dialog: Option<Dialog>,
}

impl DeckView {
    pub(crate) fn opening(profile: DeckProfile) -> Self {
        let currency = profile.price_target_currency.unwrap_or_default();
        let share_token = profile.share_token;
        Self {
            profile,
            deck: None,
            metrics: None,
            focus: Pane::Profile,
            recent: Vec::new(),
            item: 0,
            zoomed: false,
            scroll: 0,
            role_open: None,
            tag_open: None,
            currency,
            turn: 0,
            failed: false,
            oracle_tags: Vec::new(),
            share_token,
            more: None,
            dialog: None,
        }
    }

    /// Takes the loaded deck and works out its stats.
    pub(crate) fn load(&mut self, deck: Deck) {
        self.profile = deck.deck_profile.clone();
        self.share_token = self.profile.share_token;
        self.metrics =
            (!deck.entries.is_empty() || !deck.command_zone_cards.is_empty()).then(|| {
                DeckMetrics::from_entries_and_command_zone(&deck.entries, &deck.command_zone_cards)
            });
        self.deck = Some(deck);
        self.failed = false;
    }

    /// Fetches the deck again, keeping what is open; the stats show as loading until it arrives.
    pub(crate) fn reload(&mut self, cx: &mut Context<'_>) {
        cx.shared.status.clear();
        self.deck = None;
        self.metrics = None;
        self.failed = false;
        cx.request(Request::Load(self.profile.id).into());
    }

    /// Whether the deck has any card to export or buy: the app's `has_cards`.
    pub(crate) fn has_cards(&self) -> bool {
        self.metrics.is_some()
    }

    /// Whether stats could come once the deck loads, so a skeleton stands in for them meanwhile.
    pub(crate) fn stats_coming(&self) -> bool {
        self.deck.is_none()
            && !self.failed
            && (self.profile.card_count > 0
                || self.profile.commander_id.is_some()
                || self.profile.partner_commander_id.is_some()
                || self.profile.background_id.is_some()
                || self.profile.signature_spell_id.is_some())
    }

    /// The metrics when the deck has a mainboard card to chart.
    pub(crate) fn charted(&self) -> Option<&DeckMetrics> {
        self.metrics
            .as_ref()
            .filter(|m| m.land_count.saturating_add(m.nonland_count) > 0)
    }

    /// The sections that have something to show, under the app's rules.
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

    /// The panes that show, Profile first and the sections in the app's order, with ghost rows standing in for the stats while they load.
    pub(crate) fn panes(&self) -> Vec<Pane> {
        let mut panes = vec![Pane::Profile];
        if self.failed {
            return panes;
        }
        let sections = self.sections();
        let warnings = sections.contains(&Section::Warnings);
        panes.extend(
            sections
                .into_iter()
                .filter(|s| *s != Section::Warnings)
                .map(Pane::Section),
        );
        if self.stats_coming() {
            panes.push(Pane::Loading);
        }
        if warnings {
            panes.push(Pane::Section(Section::Warnings));
        }
        panes
    }

    /// The pane the focus rests on: the one it was put on while that shows, else the stats that took its place, else Profile.
    pub(crate) fn focused(&self) -> Pane {
        let panes = self.panes();
        if panes.contains(&self.focus) {
            return self.focus;
        }
        let stats = |p: &Pane| {
            matches!(
                p,
                Pane::Loading
                    | Pane::Section(Section::Distributions | Section::Mana | Section::DrawOdds)
            )
        };
        if stats(&self.focus) {
            // The skeleton stands in first, since the pane comes back when it goes.
            let loading = panes.iter().find(|p| **p == Pane::Loading);
            if let Some(stand_in) = loading.or_else(|| panes.iter().find(|p| stats(p))) {
                return *stand_in;
            }
        }
        Pane::Profile
    }

    /// Puts the focus on `pane`, at its first item.
    pub(crate) fn focus_on(&mut self, pane: Pane) {
        self.focus = pane;
        self.item = 0;
        self.scroll = 0;
        self.recent.retain(|p| *p != pane);
        self.recent.insert(0, pane);
    }

    /// Moves the focus to the next pane that shows, wrapping around to Profile.
    pub(crate) fn next_pane(&mut self) {
        let panes = self.panes();
        let at = self.pane_index(&panes);
        let next = at.saturating_add(1).checked_rem(panes.len()).unwrap_or(0);
        if let Some(pane) = panes.get(next) {
            self.focus_on(*pane);
        }
    }

    /// Moves the focus to the previous pane that shows, wrapping around to the last.
    pub(crate) fn previous_pane(&mut self) {
        let panes = self.panes();
        let at = self.pane_index(&panes);
        let previous = at
            .checked_sub(1)
            .unwrap_or_else(|| panes.len().saturating_sub(1));
        if let Some(pane) = panes.get(previous) {
            self.focus_on(*pane);
        }
    }

    fn pane_index(&self, panes: &[Pane]) -> usize {
        let focused = self.focused();
        panes.iter().position(|p| *p == focused).unwrap_or(0)
    }

    /// The pane a panel holding `panes` shows: the focused one, else the one focused last, else its first.
    pub(crate) fn shown(&self, panes: &[Pane]) -> Option<Pane> {
        let focused = self.focused();
        if panes.contains(&focused) {
            return Some(focused);
        }
        self.recent
            .iter()
            .find(|p| panes.contains(p))
            .or_else(|| panes.first())
            .copied()
    }

    /// The focused pane's items in order: the roles with their open tags, or the warnings' fix buttons.
    pub(crate) fn items(&self) -> Vec<Item> {
        let mut items = Vec::new();
        match self.focused() {
            Pane::Section(Section::Distributions) => self.role_items(&mut items),
            Pane::Section(Section::Warnings) => {
                for (i, w) in self.deck.iter().flat_map(|d| &d.warnings).enumerate() {
                    if let Some(fix) = warning::fix(w) {
                        items.extend(
                            (0..warning::labels(fix).len())
                                .map(|button| Item::Fix { warning: i, button }),
                        );
                    }
                }
            }
            _ => {}
        }
        items
    }

    fn role_items(&self, items: &mut Vec<Item>) {
        let Some(metrics) = self.charted() else {
            return;
        };
        for (role, _, _) in metrics.card_role_bars().unwrap_or_default() {
            let Some(tags) = metrics.card_role_tags.get(role) else {
                continue;
            };
            items.push(Item::Role(role.to_owned()));
            if self.role_open.as_deref() != Some(role) {
                continue;
            }
            for tag in tags {
                items.push(Item::Tag(tag.clone()));
                if self.tag_open.as_deref() == Some(tag) {
                    items.push(Item::Examples(tag.clone()));
                }
            }
        }
    }

    /// The focused item, the last one when the list has shrunk under it.
    pub(crate) fn focused_item(&self) -> Option<Item> {
        let items = self.items();
        let last = items.len().checked_sub(1)?;
        items.get(self.item.min(last)).cloned()
    }

    /// Moves to the next item in the focused pane, stopping at the last, or scrolls a pane with no items down a row.
    pub(crate) fn move_down(&mut self) {
        let items = self.items().len();
        if items == 0 {
            self.scroll = self.scroll.saturating_add(1).min(self.scroll_limit());
        } else {
            self.item = self.item.saturating_add(1).min(items.saturating_sub(1));
        }
    }

    /// Moves to the previous item in the focused pane, stopping at the first, or scrolls a pane with no items up a row.
    pub(crate) fn move_up(&mut self) {
        let items = self.items().len();
        if items == 0 {
            self.scroll = self.scroll.min(self.scroll_limit()).saturating_sub(1);
        } else {
            self.item = self.item.min(items.saturating_sub(1)).saturating_sub(1);
        }
    }

    /// How far `pane` is scrolled: the scroll while it has the focus, else none.
    pub(crate) fn scroll_of(&self, pane: Pane) -> u16 {
        if self.focused() == pane {
            self.scroll
        } else {
            0
        }
    }

    /// The most rows the focused pane can hold, so scrolling stops near its end; the draw stops it exactly at the end for the room it has.
    pub(crate) fn scroll_limit(&self) -> u16 {
        let profile = &self.profile;
        let rows = match self.focused() {
            // Each value can drop under its label.
            Pane::Profile => 18,
            // The chips, the total and the average, each of which can drop under its label.
            Pane::Section(Section::Budget) => 5,
            // A label, its chips one to a row at most, and a blank row, per group.
            Pane::Section(Section::Tags) => [
                profile.tags.len(),
                profile.oracle_tags.len(),
                profile.other_tags.len(),
            ]
            .iter()
            .filter(|n| **n > 0)
            .fold(0_usize, |rows, n| rows.saturating_add(n.saturating_add(2))),
            // The heading, a blank row, and a row per group.
            Pane::Section(Section::DrawOdds) => self
                .charted()
                .map_or(0, |m| m.draw_odds_buckets().1.len().saturating_add(2)),
            _ => 0,
        };
        u16::try_from(rows).unwrap_or(u16::MAX)
    }

    pub(crate) fn next_currency(&mut self) {
        self.currency = match self.currency {
            PriceCurrency::Usd => PriceCurrency::Eur,
            PriceCurrency::Eur => PriceCurrency::Tix,
            PriceCurrency::Tix => PriceCurrency::Usd,
        };
    }

    pub(crate) fn previous_currency(&mut self) {
        self.currency = match self.currency {
            PriceCurrency::Usd => PriceCurrency::Tix,
            PriceCurrency::Eur => PriceCurrency::Usd,
            PriceCurrency::Tix => PriceCurrency::Eur,
        };
    }

    pub(crate) fn later_turn(&mut self) {
        self.turn = self.turn.saturating_add(1).min(MAX_TURN);
    }

    pub(crate) fn earlier_turn(&mut self) {
        self.turn = self.turn.saturating_sub(1);
    }

    /// The description of `slug` from the catalog.
    pub(crate) fn tag_description(&self, slug: &str) -> Option<&str> {
        self.oracle_tags
            .iter()
            .find(|t| t.slug == slug)
            .and_then(|t| t.description.as_deref())
    }

    /// The share link for the current token.
    pub(crate) fn share_link(&self) -> Option<String> {
        self.share_token.map(|t| format!("{WEB_BASE}/deck/{t}"))
    }

    /// The More sheet, with Export and Buy off until the deck has cards.
    pub(crate) fn more_menu(&self) -> Menu {
        let has_cards = self.has_cards();
        Menu::new(
            "More actions",
            vec![
                MenuItem::new("Import cards"),
                MenuItem::new("Export cards").enabled(has_cards),
                MenuItem::new("Buy deck").enabled(has_cards),
                MenuItem::new("Clone deck"),
                MenuItem::new("Share deck"),
                MenuItem::new("Clear skips").danger(),
                MenuItem::new("Delete deck").danger(),
            ],
        )
    }

    /// The Buy dialog, with links once the deck has loaded.
    pub(crate) fn buy_dialog(&self) -> AlertDialog {
        let body = if self.deck.is_some() {
            "Buy this deck's cards from a retailer."
        } else {
            "No buy links are available for this deck."
        };
        let dialog = AlertDialog::new("Buy deck", body).with_button("Close", None);
        if self.deck.is_some() {
            dialog
                .with_button("TCGplayer \u{2197}", Some('t'))
                .with_button("Card Kingdom \u{2197}", Some('k'))
        } else {
            dialog
        }
    }

    /// The Share dialog for whether the deck is shared now.
    pub(crate) fn share_dialog(&self) -> AlertDialog {
        if self.share_token.is_some() {
            AlertDialog::new(
                "Share deck",
                "Anyone with the link can view this deck. Stop sharing disables the link; sharing again creates a new one.",
            )
            .with_button("Close", None)
            .with_button("Copy link", None)
            .with_button("Stop share", None)
        } else {
            AlertDialog::new(
                "Share deck",
                "Anyone with the link can view this deck. You can stop sharing at any time.",
            )
            .with_button("Close", None)
            .with_button("Create link", None)
        }
    }

    /// The buy links: `TCGplayer`, then `CardKingdom`.
    pub(crate) fn buy_links(&self) -> Option<(String, String)> {
        self.deck.as_ref().map(buy::links)
    }
}

/// The Clear skips dialog in the app's order, Cancel then the danger Clear, focused on Cancel.
pub(crate) fn clear_skips_dialog() -> AlertDialog {
    AlertDialog::new(
        "Clear skips",
        "Skipped cards come back on their own after a week, and removed cards after a month. Clearing brings them all back now. This can't be undone.",
    )
    .with_button("Cancel", None)
    .with_danger_button("Clear", None)
    .with_focus(0)
}

/// The Delete dialog in the app's order, Cancel then the danger Delete, focused on Cancel.
pub(crate) fn delete_dialog() -> AlertDialog {
    AlertDialog::new("Delete deck", "Are you sure you want to delete this deck?")
        .with_button("Cancel", None)
        .with_danger_button("Delete", None)
        .with_focus(0)
}

/// Shows the first-deck hint again, whether or not it has been seen.
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    hint::reopen(cx, HINT_KEY);
}

/// The deck screen's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// Fetches the full deck.
    Load(Uuid),
    /// Fetches the oracle tag catalog for the role tags' definitions.
    LoadOracleTags,
    /// Sets a card's quantity, the copy-limit fix.
    FixQuantity {
        deck_id: Uuid,
        card: Uuid,
        quantity: i32,
    },
    /// Clears the commander, the invalid-commander fix.
    ClearCommander {
        deck_id: Uuid,
        oathbreaker: bool,
    },
    /// Takes a card out of the deck.
    RemoveCard {
        deck_id: Uuid,
        card: Uuid,
    },
    Clone {
        deck_id: Uuid,
        name: String,
    },
    Share(Uuid),
    Unshare(Uuid),
    ClearSkips(Uuid),
    Delete(Uuid),
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Load(..) => "load",
            Self::LoadOracleTags => "load_oracle_tags",
            Self::FixQuantity { .. } => "fix_quantity",
            Self::ClearCommander { .. } => "clear_commander",
            Self::RemoveCard { .. } => "remove_card",
            Self::Clone { .. } => "clone",
            Self::Share(..) => "share",
            Self::Unshare(..) => "unshare",
            Self::ClearSkips(..) => "clear_skips",
            Self::Delete(..) => "delete",
        }
    }
}

/// The deck screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded(Box<Deck>),
    Failed(String),
    OracleTags(Vec<OracleTag>),
    /// A warning fix went through: its toast, and whether the profile changed too.
    Fixed {
        deck_id: Uuid,
        message: String,
    },
    /// A request failed with this message for an error toast.
    Error(String),
    Cloned {
        profile: Box<DeckProfile>,
    },
    CloneFailed(String),
    Shared {
        deck_id: Uuid,
        token: Uuid,
    },
    Unshared(Uuid),
    SkipsCleared,
    Deleted(Uuid),
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

/// Opens a deck over the current screen and fetches it, then shows the first-deck hint once.
pub(crate) fn open(cx: &mut Context<'_>, profile: DeckProfile) {
    cx.shared.status.clear();
    cx.request(Request::Load(profile.id).into());
    cx.request(Request::LoadOracleTags.into());
    cx.push(Screen::Deck(Box::new(DeckView::opening(profile))));
    hint::once(cx, HINT_KEY);
}

/// Swaps the screen on top for this deck, as after creating or cloning one, so going back reaches the list.
pub(crate) fn replace_with(cx: &mut Context<'_>, profile: DeckProfile) {
    cx.shared.status.clear();
    cx.request(Request::Load(profile.id).into());
    cx.request(Request::LoadOracleTags.into());
    cx.replace(Screen::Deck(Box::new(DeckView::opening(profile))));
    hint::once(cx, HINT_KEY);
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let error = |e: &zwipe_client::ClientError| failure(e, |m| Update::Error(m).into());
    match request {
        Request::Load(deck_id) => Some(
            match async { client.fetch_deck(deck_id, &account.fresh().await?).await }.await {
                Ok(deck) => Update::Loaded(Box::new(deck)).into(),
                Err(e) => failure(&e, |m| Update::Failed(m).into()),
            },
        ),
        // Definitions only add to the role tags, so a failure shows none.
        Request::LoadOracleTags => client
            .fetch_oracle_tags()
            .await
            .ok()
            .map(|tags| Update::OracleTags(tags).into()),
        Request::FixQuantity {
            deck_id,
            card,
            quantity,
        } => Some(fix_quantity(account, deck_id, card, quantity).await),
        Request::ClearCommander {
            deck_id,
            oathbreaker,
        } => Some(clear_commander(account, deck_id, oathbreaker).await),
        Request::RemoveCard { deck_id, card } => Some(remove_card(account, deck_id, card).await),
        Request::Clone { deck_id, name } => Some(clone_deck(account, deck_id, &name).await),
        Request::Share(deck_id) => Some(
            match async { client.share_deck(deck_id, &account.fresh().await?).await }.await {
                Ok(share) => Update::Shared {
                    deck_id,
                    token: share.share_token,
                }
                .into(),
                Err(e) => error(&e),
            },
        ),
        Request::Unshare(deck_id) => Some(
            match async { client.unshare_deck(deck_id, &account.fresh().await?).await }.await {
                Ok(()) => Update::Unshared(deck_id).into(),
                Err(e) => error(&e),
            },
        ),
        Request::ClearSkips(deck_id) => Some(
            match async {
                client
                    .clear_deck_suppressions(deck_id, &account.fresh().await?)
                    .await
            }
            .await
            {
                Ok(_) => Update::SkipsCleared.into(),
                Err(e) => error(&e),
            },
        ),
        Request::Delete(deck_id) => Some(
            match async { client.delete_deck(deck_id, &account.fresh().await?).await }.await {
                Ok(()) => Update::Deleted(deck_id).into(),
                Err(e) => error(&e),
            },
        ),
    }
}

/// Sets a card's quantity, the copy-limit fix.
async fn fix_quantity(
    account: &Account,
    deck_id: Uuid,
    card: Uuid,
    quantity: i32,
) -> screens::Update {
    match async {
        let body = HttpPatchDeckCard::new(Some(quantity), None);
        account
            .client
            .update_deck_card(deck_id, card, &body, &account.fresh().await?)
            .await
    }
    .await
    {
        Ok(_) => Update::Fixed {
            deck_id,
            message: format!("Quantity set to {quantity}"),
        }
        .into(),
        Err(e) => failure(&e, |m| Update::Error(m).into()),
    }
}

/// Clears the commander, or the oathbreaker, the invalid-commander fix.
async fn clear_commander(account: &Account, deck_id: Uuid, oathbreaker: bool) -> screens::Update {
    match async {
        let body = HttpUpdateDeckProfile::builder()
            .commander_id(Patch::Set(None))
            .build();
        account
            .client
            .update_deck_profile(deck_id, &body, &account.fresh().await?)
            .await
    }
    .await
    {
        Ok(_) => Update::Fixed {
            deck_id,
            message: if oathbreaker {
                "Oathbreaker cleared"
            } else {
                "Commander cleared"
            }
            .to_owned(),
        }
        .into(),
        Err(e) => failure(&e, |m| Update::Error(m).into()),
    }
}

/// Takes a card out of the deck.
async fn remove_card(account: &Account, deck_id: Uuid, card: Uuid) -> screens::Update {
    match async {
        account
            .client
            .delete_deck_card(deck_id, card, &account.fresh().await?)
            .await
    }
    .await
    {
        Ok(()) => Update::Fixed {
            deck_id,
            message: "Card removed".to_owned(),
        }
        .into(),
        Err(e) => failure(&e, |m| Update::Error(m).into()),
    }
}

/// Clones the deck under a new name, then fetches the clone's profile.
async fn clone_deck(account: &Account, deck_id: Uuid, name: &str) -> screens::Update {
    let client = &account.client;
    match async {
        let session = account.fresh().await?;
        let cloned = client
            .clone_deck(
                deck_id,
                &HttpCloneDeck {
                    new_name: name.to_owned(),
                },
                &session,
            )
            .await?;
        client.fetch_deck_profile(cloned.deck_id, &session).await
    }
    .await
    {
        Ok(profile) => Update::Cloned {
            profile: Box::new(profile),
        }
        .into(),
        Err(e) => failure(&e, |m| Update::CloneFailed(m).into()),
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let on_top = matches!(nav.top(), Screen::Deck(_));
    let Some(view) = nav.find_mut(pick) else {
        if let Update::Error(message) | Update::CloneFailed(message) = update {
            cx.toast(ToastKind::Error, message);
        }
        return;
    };
    match update {
        Update::Loaded(deck) => {
            if view.profile.id == deck.deck_profile.id {
                view.load(*deck);
            }
        }
        Update::Failed(error) => {
            view.failed = true;
            cx.toast(ToastKind::Error, error);
        }
        Update::OracleTags(tags) => view.oracle_tags = tags,
        Update::Fixed { deck_id, message } => {
            cx.toast_for(ToastKind::Info, message, toast::QUICK);
            if view.profile.id == deck_id {
                view.reload(cx);
            }
        }
        Update::Error(message) => cx.toast(ToastKind::Error, message),
        Update::Cloned { profile } => {
            cx.toast_for(
                ToastKind::Info,
                format!("Cloned as \"{}\"", profile.name),
                toast::QUICK,
            );
            if on_top {
                replace_with(cx, *profile);
            }
        }
        Update::CloneFailed(message) => {
            if let Some(Dialog::Clone(dialog)) = &mut view.dialog {
                dialog.busy = false;
            }
            cx.toast(ToastKind::Error, message);
        }
        Update::Shared { deck_id, token } => {
            if view.profile.id == deck_id {
                view.share_token = Some(token);
                if let Some(link) = view.share_link() {
                    copy(cx, &link, "Share link copied");
                }
            }
        }
        Update::Unshared(deck_id) => {
            if view.profile.id == deck_id {
                view.share_token = None;
            }
            cx.toast_for(ToastKind::Info, "Link disabled".to_owned(), toast::QUICK);
        }
        Update::SkipsCleared => {
            cx.toast_for(ToastKind::Info, "Skips cleared".to_owned(), toast::QUICK);
        }
        Update::Deleted(deck_id) => {
            if on_top && view.profile.id == deck_id {
                cx.back();
            }
        }
    }
}

/// Copies `text` to the clipboard and says `done`, or says it could not.
pub(crate) fn copy(cx: &mut Context<'_>, text: &str, done: &str) {
    if crate::inbound::widgets::clipboard::copy(text).is_ok() {
        cx.toast_for(ToastKind::Info, done.to_owned(), toast::QUICK);
    } else {
        cx.toast(ToastKind::Error, format!("Could not copy {text}"));
    }
}

/// Runs the fix behind a warning's button.
pub(crate) fn run_fix(view: &DeckView, fix: Fix, button: usize, cx: &mut Context<'_>) {
    let deck_id = view.profile.id;
    match fix {
        Fix::BelowMinimum if button == 0 => {
            if let Some(deck) = &view.deck {
                screens::swipe::open(cx, deck);
            }
        }
        Fix::BelowMinimum => screens::deck_import::open(cx, deck_id),
        Fix::Quantity { card, quantity } => cx.request(
            Request::FixQuantity {
                deck_id,
                card,
                quantity,
            }
            .into(),
        ),
        Fix::ClearCommander => cx.request(
            Request::ClearCommander {
                deck_id,
                oathbreaker: view.profile.format.is_some_and(|f| f.has_signature_spell()),
            }
            .into(),
        ),
        Fix::Remove { card } => cx.request(Request::RemoveCard { deck_id, card }.into()),
    }
}
