//! The Commander maybeboard: the commanders the user saved for later, with a quick add search over every commander, a Show row of color pips, the card filter sheet, and the commander swipe to find more. The Decks screen's More sheet opens it.
//!
//! Entries arrive three ways: an up swipe in any commander swipe, an up swipe in this screen's own Swipe, or a result chip under Quick add. The Quick add search runs once the trimmed query is at least two characters and has rested for 300 ms; it never narrows the saved list, which only the Show pips do (every picked color must be in the card's identity).
//!
//! Each load takes a ticket and only the newest one's reply lands. A refetch with entries on screen is silent, and a remove or clear made while a load is out starts that load again, since its reply left before the edit and could bring a row back.
//!
//! The debounce is a request of its own: [`Request::Debounce`] sleeps 300 ms off the main loop and answers [`Update::Debounced`], and only then, if no edit took a newer ticket, does the search go out. The loop wakes on that answer, so nothing has to animate while a debounce waits, and tests apply the answer directly instead of sleeping.
//!
//! The app's "Art" chip toggles row thumbnails; the terminal's card rows draw none, so the Show row leaves it out. The app's Back pushes the deck list; here Esc goes back to the screen underneath. The app's Filter opens the card filter sheet, which belongs to the card filter screens, so [`open_filter`] stands in for it.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::time::Duration;

use crossterm::event::KeyCode;
use uuid::Uuid;
use zwipe_client::ClientError;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::card_sort_key::CardSortKey;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::deck::format::Format;
use zwipe_core::domain::user::models::hints::HINT_COMMANDER_MAYBEBOARD;

use super::card_sheets::filter::{FilterSheet, SheetConfig, SheetOutcome};
use super::playstyle::DeckLimit;
use super::{Screen, card_sheets, failure, oracle_tags};
use crate::domain::filter;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::alert::AlertDialog;
use crate::inbound::widgets::card_rows::CardRows;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::sheet::Sheet;
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::{self, ToastKind};
use crate::outbound::account::Account;

/// The screen's title and its hint's.
pub(crate) const TITLE: &str = "Commander maybeboard";

/// The Quick add row's label.
pub(crate) const QUICK_ADD: &str = "Quick add";

/// The Quick add box's placeholder.
pub(crate) const PLACEHOLDER: &str = "Search a commander to add";

/// The one result chip while a search is out.
pub(crate) const SEARCHING: &str = "Searching...";

/// The one result chip when the search found nothing not already saved.
pub(crate) const NO_RESULTS: &str = "No results";

/// Said when the first load failed.
pub(crate) const LOAD_FAILED: &str = "Could not load your commander maybeboard";

/// Said when nothing is saved.
pub(crate) const NO_COMMANDERS: &str = "No commanders";

/// Said when the Show pips hide every entry.
pub(crate) const NO_MATCH: &str = "No commanders match";

/// The list's group header.
pub(crate) const LIST_TITLE: &str = "Commanders";

/// The More sheet's title.
pub(crate) const MORE_TITLE: &str = "More actions";

/// The More sheet's one action, and its dialog's title.
pub(crate) const CLEAR: &str = "Clear maybeboard";

/// The Clear maybeboard dialog's body.
pub(crate) const CLEAR_BODY: &str =
    "Every commander on your maybeboard will be removed. This can't be undone.";

/// The toast after a remove.
pub(crate) const REMOVED: &str = "Removed";

/// The toast after Clear maybeboard.
pub(crate) const CLEARED: &str = "Maybeboard cleared";

/// The fewest bytes of a trimmed query that search, as the app's `MIN_QUERY_LEN` (it counts bytes too).
pub(crate) const MIN_QUERY_LEN: usize = 2;

/// How long a query rests before it searches, as the app's `DEBOUNCE_MS`.
pub(crate) const DEBOUNCE: Duration = Duration::from_millis(300);

/// Result chips per search, as the app's `RESULT_LIMIT`.
pub(crate) const RESULT_LIMIT: u32 = 8;

/// What has the keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    /// The Quick add box; Enter or `/` types into it.
    Input,
    /// The Quick add result chips.
    Results,
    /// The Show chips.
    Show,
    /// The saved rows, which move their own focus.
    List,
}

/// What covers the screen.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Overlay {
    /// The More actions sheet.
    More(Sheet),
    /// The Clear maybeboard dialog, over the More sheet.
    Clear(AlertDialog),
}

/// The Quick add box and its search.
#[derive(Debug)]
pub(crate) struct QuickAdd {
    pub(crate) query: TextInput,
    /// Whether keys type into the box.
    pub(crate) typing: bool,
    /// The newest query edit's ticket; only its debounce and search land.
    pub(crate) ticket: u64,
    /// A search for the current query is waiting or out.
    pub(crate) searching: bool,
    /// The last search's commanders, saved ones included.
    pub(crate) results: Vec<Card>,
    /// The result chip under the cursor.
    pub(crate) cursor: usize,
}

impl Default for QuickAdd {
    fn default() -> Self {
        Self {
            query: TextInput::default().with_placeholder(PLACEHOLDER),
            typing: false,
            ticket: 0,
            searching: false,
            results: Vec::new(),
            cursor: 0,
        }
    }
}

/// The Commander maybeboard's state.
#[derive(Debug)]
pub(crate) struct MaybeboardView {
    /// Until the first reply lands.
    pub(crate) is_loading: bool,
    /// The last load failed with nothing held.
    pub(crate) load_failed: bool,
    /// The saved commanders, newest save first.
    pub(crate) entries: Vec<Card>,
    /// The newest load's ticket; only its reply lands.
    pub(crate) ticket: u64,
    /// Whether the newest load is still out.
    pub(crate) load_out: bool,
    /// How many decks the user owns, once fetched, for the create gate.
    pub(crate) decks: Option<usize>,
    pub(crate) quick: QuickAdd,
    /// The picked Show colors; none is All.
    pub(crate) colors: Vec<Color>,
    /// The Show chip under the cursor: 0 is All, then W U B R G.
    pub(crate) show_cursor: usize,
    pub(crate) rows: CardRows,
    pub(crate) focus: Focus,
    pub(crate) overlay: Option<Overlay>,
    /// The card filter as last applied; the list narrows by it after the Show pips.
    pub(crate) filter: CardQueryBuilder,
    /// The card filter sheet, while it is open over the list.
    pub(crate) sheet: Option<FilterSheet>,
}

impl MaybeboardView {
    /// A view waiting on its first load.
    pub(crate) fn loading() -> Self {
        Self {
            is_loading: true,
            load_failed: false,
            entries: Vec::new(),
            ticket: 0,
            load_out: false,
            decks: None,
            quick: QuickAdd::default(),
            colors: Vec::new(),
            show_cursor: 0,
            rows: CardRows::default(),
            focus: Focus::Input,
            overlay: None,
            filter: CardQueryBuilder::default(),
            sheet: None,
        }
    }

    /// The oracle ids already saved.
    pub(crate) fn saved(&self) -> HashSet<Uuid> {
        self.entries
            .iter()
            .filter_map(|c| c.scryfall_data.oracle_id)
            .collect()
    }

    /// The saved list narrowed by the Show pips, then by the applied card filter, then sorted when it set a sort; saved order otherwise.
    pub(crate) fn filtered(&self) -> Vec<&Card> {
        let shown: Vec<&Card> = self
            .entries
            .iter()
            .filter(|c| {
                self.colors
                    .iter()
                    .all(|color| c.scryfall_data.color_identity.contains(color))
            })
            .collect();
        let criteria = if self.filter.is_empty() {
            None
        } else {
            self.filter.build_criteria().ok()
        };
        let sort = self.filter.sort();
        if criteria.is_none() && sort.is_none() {
            return shown;
        }
        let mut cards = Cards::from(shown.iter().map(|&c| c.clone()).collect::<Vec<Card>>());
        if let Some(criteria) = &criteria {
            cards = cards.matching(criteria);
        }
        if let Some(sort) = sort {
            cards = cards.sorted(sort, self.filter.direction());
        }
        let order: Vec<Card> = cards.into();
        order
            .iter()
            .filter_map(|card| {
                shown
                    .iter()
                    .copied()
                    .find(|c| c.scryfall_data.id == card.scryfall_data.id)
            })
            .collect()
    }

    /// Whether Filter carries its dot: the card filter has a criterion or a sort.
    pub(crate) fn filter_dot(&self) -> bool {
        filter::shows_dot(&self.filter)
    }

    /// Whether the query is long enough to search, which is also when the result chips show.
    pub(crate) fn searching_catalog(&self) -> bool {
        self.quick.query.value().trim().len() >= MIN_QUERY_LEN
    }

    /// The result chips: the last search's commanders that have an oracle id and are not saved yet.
    pub(crate) fn shown_results(&self) -> Vec<&Card> {
        let saved = self.saved();
        self.quick
            .results
            .iter()
            .filter(|c| {
                c.scryfall_data
                    .oracle_id
                    .is_some_and(|o| !saved.contains(&o))
            })
            .collect()
    }

    /// Everything the focus can land on, top to bottom.
    pub(crate) fn stops(&self) -> Vec<Focus> {
        let mut stops = Vec::new();
        if self.is_loading {
            return stops;
        }
        stops.push(Focus::Input);
        if self.searching_catalog() && !self.quick.searching && !self.shown_results().is_empty() {
            stops.push(Focus::Results);
        }
        stops.push(Focus::Show);
        if !self.load_failed && !self.filtered().is_empty() {
            stops.push(Focus::List);
        }
        stops
    }

    /// Moves the focus `forward` or back one stop, entering the rows at their near end. Returns whether it moved.
    pub(crate) fn step(&mut self, forward: bool) -> bool {
        let stops = self.stops();
        let at = stops.iter().position(|&s| s == self.focus).unwrap_or(0);
        let next = if forward {
            at.checked_add(1)
        } else {
            at.checked_sub(1)
        };
        let Some(&focus) = next.and_then(|i| stops.get(i)) else {
            return false;
        };
        if focus == Focus::List {
            let last = self.filtered().len().saturating_sub(1);
            self.rows.set_focused(if forward { 0 } else { last });
        }
        self.focus = focus;
        true
    }

    /// Keeps the focus on a stop that still exists: the result chips fall back to the box, the rows to the Show row.
    pub(crate) fn settle_focus(&mut self) {
        let count = self.shown_results().len();
        self.quick.cursor = self.quick.cursor.min(count.saturating_sub(1));
        let stops = self.stops();
        if stops.is_empty() || stops.contains(&self.focus) {
            return;
        }
        self.focus = match self.focus {
            Focus::Results => Focus::Input,
            _ => Focus::Show,
        };
    }

    /// Picks the Show chip at `index`: 0 clears every color, any other toggles that color.
    pub(crate) fn pick_show(&mut self, index: usize) {
        let Some(color) = index
            .checked_sub(1)
            .and_then(|i| Color::all().get(i).copied())
        else {
            self.colors.clear();
            self.settle_focus();
            return;
        };
        if self.colors.contains(&color) {
            self.colors.retain(|&c| c != color);
        } else {
            self.colors.push(color);
        }
        self.settle_focus();
    }

    /// Starts a new load whose reply replaces any still out.
    pub(crate) fn refetch(&mut self, cx: &mut Context<'_>) {
        self.ticket = self.ticket.wrapping_add(1);
        self.load_out = true;
        cx.request(
            Request::Load {
                ticket: self.ticket,
            }
            .into(),
        );
    }

    /// Starts again a load that went out before a local edit.
    fn restart_stale_load(&mut self, cx: &mut Context<'_>) {
        if self.load_out {
            self.refetch(cx);
        }
    }

    /// Takes the query after an edit: a new ticket, then either the results cleared (too short) or a debounce before searching.
    pub(crate) fn query_changed(&mut self, cx: &mut Context<'_>) {
        self.quick.ticket = self.quick.ticket.wrapping_add(1);
        if self.searching_catalog() {
            self.quick.searching = true;
            cx.request(
                Request::Debounce {
                    ticket: self.quick.ticket,
                }
                .into(),
            );
        } else {
            self.quick.searching = false;
            self.quick.results.clear();
        }
        self.settle_focus();
    }

    /// Saves the result chip at `index`: it leaves the chips at once, and the row joins the list when the server confirms.
    pub(crate) fn add_result(&mut self, index: usize, cx: &mut Context<'_>) {
        let Some(card) = self.shown_results().get(index).map(|&c| c.clone()) else {
            return;
        };
        let Some(oracle_id) = card.scryfall_data.oracle_id else {
            return;
        };
        self.quick
            .results
            .retain(|c| c.scryfall_data.oracle_id != Some(oracle_id));
        self.settle_focus();
        cx.request(Request::Add(Box::new(card)).into());
    }

    /// The create gate for the signed-in user.
    pub(crate) fn deck_limit(&self, cx: &Context<'_>) -> DeckLimit {
        DeckLimit::of(cx.shared.user.as_ref(), self.decks)
    }

    /// Refetches when the commander swipe closed, since its up swipes saved entries, and swaps in a printing picked from a row.
    pub(crate) fn on_return(&mut self, cx: &mut Context<'_>) {
        if card_sheets::filter::returned_from_dictionary(self.sheet.as_mut(), cx) {
            return;
        }
        if cx.shared.caches.handoff.commander_swipe.take().is_some() {
            self.refetch(cx);
        }
        self.rows.take_printing(cx, &mut self.entries, itself);
    }
}

/// A saved entry is the card itself, for [`CardRows::take_printing`].
fn itself(card: &mut Card) -> &mut Card {
    card
}

/// Opens the card filter sheet over the list's own filter, as the app's Filter does: no format section, Reset back to blank. The list changes only when Apply commits.
pub(crate) fn open_filter(view: &mut MaybeboardView, cx: &mut Context<'_>) {
    let config = SheetConfig {
        show_format: false,
        ..SheetConfig::default()
    };
    view.sheet = Some(FilterSheet::open(cx, &view.filter, config));
}

/// Keys while the filter sheet is open: Apply narrows the list and puts the focus back at its top.
pub(crate) fn on_sheet(view: &mut MaybeboardView, code: KeyCode, cx: &mut Context<'_>) {
    let Some(sheet) = &mut view.sheet else {
        return;
    };
    match card_sheets::filter::keys::handle(sheet, code, cx) {
        SheetOutcome::Open => {}
        SheetOutcome::Closed => view.sheet = None,
        SheetOutcome::Applied(filter) => {
            view.sheet = None;
            view.filter = *filter;
            view.rows.set_focused(0);
            if view.focus == Focus::List && view.filtered().is_empty() {
                view.focus = Focus::Show;
            }
        }
    }
}

/// The Quick add search for `query`: commanders in the Commander format by name, no tokens, sorted by name, eight at most.
pub(crate) fn search_query(query: &str) -> CardQueryBuilder {
    let mut builder = CardQueryBuilder::with_name_contains(query.trim());
    builder
        .set_is_commander_in_format(Format::Commander)
        .set_is_token(false)
        .set_sort(CardSortKey::Name)
        .set_limit(RESULT_LIMIT);
    builder
}

/// The screen's work.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    /// The saved list, under a ticket.
    Load { ticket: u64 },
    /// The user's decks, for the create gate.
    DeckProfiles,
    /// Waits out the debounce for the query edit with this ticket.
    Debounce { ticket: u64 },
    /// Searches commanders by name for the query edit with this ticket.
    Search { ticket: u64, query: String },
    /// Saves a commander from the result chips.
    Add(Box<Card>),
    /// Takes a commander off by oracle id.
    Remove(Uuid),
    /// Takes every commander off.
    Clear,
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Load { .. } => "load",
            Self::DeckProfiles => "deck_profiles",
            Self::Debounce { .. } => "debounce",
            Self::Search { .. } => "search",
            Self::Add(..) => "add",
            Self::Remove(..) => "remove",
            Self::Clear => "clear",
        }
    }
}

/// The screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded {
        ticket: u64,
        cards: Vec<Card>,
    },
    LoadFailed {
        ticket: u64,
        message: String,
    },
    /// How many decks the user owns.
    DeckProfiles(usize),
    /// The debounce for this query edit is over.
    Debounced {
        ticket: u64,
    },
    Searched {
        ticket: u64,
        cards: Vec<Card>,
    },
    /// A search failed, quietly; the old results stay.
    SearchFailed {
        ticket: u64,
    },
    Added(Box<Card>),
    Removed(Uuid),
    Cleared,
    /// An add, remove or clear failed with this message.
    ActionFailed(String),
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::CommanderMaybeboard(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::CommanderMaybeboard(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut MaybeboardView> {
    match screen {
        Screen::CommanderMaybeboard(view) => Some(view),
        _ => None,
    }
}

/// Opens the Commander maybeboard: fetches the list, the user's decks and the tag catalog, and shows the hint the first time.
pub(crate) fn open(cx: &mut Context<'_>) {
    cx.shared.caches.handoff.commander_swipe = None;
    let mut view = MaybeboardView::loading();
    view.refetch(cx);
    cx.push(Screen::CommanderMaybeboard(Box::new(view)));
    cx.request(Request::DeckProfiles.into());
    oracle_tags::ensure_catalog(cx);
    hint::once(cx, HINT_COMMANDER_MAYBEBOARD);
}

/// Shows the hint again, for `H`.
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    hint::reopen(cx, HINT_COMMANDER_MAYBEBOARD);
}

/// Runs `call` for an action, its success as `done` and its failure as a message.
async fn act(
    call: impl Future<Output = Result<(), ClientError>>,
    done: Update,
) -> Option<screens::Update> {
    match call.await {
        Ok(()) => Some(done.into()),
        Err(e) => Some(failure(&e, |message| Update::ActionFailed(message).into())),
    }
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    match request {
        Request::Load { ticket } => {
            match async {
                client
                    .fetch_commander_maybeboard(&account.fresh().await?)
                    .await
            }
            .await
            {
                Ok(cards) => Some(Update::Loaded { ticket, cards }.into()),
                Err(e) => Some(failure(&e, |message| {
                    Update::LoadFailed { ticket, message }.into()
                })),
            }
        }
        Request::DeckProfiles => {
            match async { client.fetch_deck_profiles(&account.fresh().await?).await }.await {
                Ok(profiles) => Some(Update::DeckProfiles(profiles.len()).into()),
                Err(ClientError::Unauthorized(_)) => Some(screens::Update::SessionEnded),
                // Quiet, as the app's `run_quiet`: the gate only lets creation through.
                Err(_) => None,
            }
        }
        Request::Debounce { ticket } => {
            tokio::time::sleep(DEBOUNCE).await;
            Some(Update::Debounced { ticket }.into())
        }
        Request::Search { ticket, query } => {
            let Ok(filter) = search_query(&query).build() else {
                return Some(Update::SearchFailed { ticket }.into());
            };
            match async {
                client
                    .search_commanders(&filter, &account.fresh().await?)
                    .await
            }
            .await
            {
                Ok(cards) => Some(Update::Searched { ticket, cards }.into()),
                // Quiet, as the app's `run_quiet`: a toast per keystroke would bury the screen.
                Err(e) => Some(failure(&e, |_| Update::SearchFailed { ticket }.into())),
            }
        }
        Request::Add(card) => {
            let oracle_id = card.scryfall_data.oracle_id?;
            act(
                async {
                    client
                        .add_commander_maybeboard_card(oracle_id, &account.fresh().await?)
                        .await
                },
                Update::Added(card),
            )
            .await
        }
        Request::Remove(oracle_id) => {
            act(
                async {
                    client
                        .remove_commander_maybeboard_card(oracle_id, &account.fresh().await?)
                        .await
                },
                Update::Removed(oracle_id),
            )
            .await
        }
        Request::Clear => {
            act(
                async {
                    client
                        .clear_commander_maybeboard(&account.fresh().await?)
                        .await
                },
                Update::Cleared,
            )
            .await
        }
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let view = nav.find_mut(pick);
    match update {
        Update::Loaded { ticket, cards } => {
            if let Some(view) = view.filter(|v| v.ticket == ticket) {
                view.entries = cards;
                view.load_failed = false;
                view.is_loading = false;
                view.load_out = false;
                view.settle_focus();
            }
        }
        Update::LoadFailed { ticket, message } => {
            cx.shared
                .toasts
                .push_for(ToastKind::Error, message, toast::NORMAL);
            if let Some(view) = view.filter(|v| v.ticket == ticket) {
                view.load_failed = view.entries.is_empty();
                view.is_loading = false;
                view.load_out = false;
                view.settle_focus();
            }
        }
        Update::DeckProfiles(count) => {
            if let Some(view) = view {
                view.decks = Some(count);
            }
        }
        Update::Debounced { ticket } => {
            if let Some(view) = view.filter(|v| v.quick.ticket == ticket) {
                let query = view.quick.query.value().trim().to_owned();
                cx.request(Request::Search { ticket, query }.into());
            }
        }
        Update::Searched { ticket, cards } => {
            if let Some(view) = view.filter(|v| v.quick.ticket == ticket) {
                view.quick.results = cards;
                view.quick.searching = false;
                view.quick.cursor = 0;
                view.settle_focus();
            }
        }
        Update::SearchFailed { ticket } => {
            if let Some(view) = view.filter(|v| v.quick.ticket == ticket) {
                view.quick.searching = false;
                view.settle_focus();
            }
        }
        Update::Added(card) => {
            let name = card.scryfall_data.name.clone();
            if let Some(view) = view {
                let oracle_id = card.scryfall_data.oracle_id;
                if !view
                    .entries
                    .iter()
                    .any(|c| c.scryfall_data.oracle_id == oracle_id)
                {
                    view.entries.insert(0, *card);
                }
                view.refetch(cx);
                view.settle_focus();
            }
            cx.shared
                .toasts
                .push_for(ToastKind::Info, format!("Added {name}"), toast::QUICK);
        }
        Update::Removed(oracle_id) => {
            if let Some(view) = view {
                view.entries
                    .retain(|c| c.scryfall_data.oracle_id != Some(oracle_id));
                view.restart_stale_load(cx);
                view.settle_focus();
            }
            cx.shared
                .toasts
                .push_for(ToastKind::Info, REMOVED.to_owned(), toast::QUICK);
        }
        Update::Cleared => {
            if let Some(view) = view {
                view.entries.clear();
                view.rows.collapse();
                view.restart_stale_load(cx);
                view.settle_focus();
            }
            cx.shared
                .toasts
                .push_for(ToastKind::Info, CLEARED.to_owned(), toast::QUICK);
        }
        Update::ActionFailed(message) => {
            cx.shared
                .toasts
                .push_for(ToastKind::Error, message, toast::NORMAL);
        }
    }
}
