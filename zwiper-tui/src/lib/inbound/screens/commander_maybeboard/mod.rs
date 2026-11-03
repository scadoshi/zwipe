//! The Commander maybeboard: the commanders the user saved for later, with a quick add search over every commander, a Show row of color pips, the card filter sheet, and the commander swipe to find more. The Decks screen's More sheet opens it.
//!
//! What the screen knows and decides (the saved list and its loads, the Quick add search and its debounce, the Show pips, the create gate, the edits and their toasts) is zwipe-ui's [`State`]; this view adds the Quick add box, the focus and chip cursors, the card rows, the More sheet and Clear dialog, and the card filter sheet with its applied filter.
//!
//! The debounce is a request of its own: [`Request::Debounce`] sleeps off the main loop and answers [`Update::Debounced`], which the model turns into the search when no newer edit replaced it. The loop wakes on that answer, so nothing has to animate while a debounce waits, and tests apply the answer directly instead of sleeping.
//!
//! The app's "Art" chip toggles row thumbnails; the terminal's card rows draw none, so the Show row leaves it out. The app's Back pushes the deck list; here Esc goes back to the screen underneath. The app's Filter opens the card filter sheet, which belongs to the card filter screens, so [`open_filter`] stands in for it. The app's Swipe is an overlay whose right swipe goes through this model's create gate; here it is the commander swipe screen, which takes its own [`DeckLimit`], so the view keeps the deck count the model reads too.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::time::Duration;

use crossterm::event::KeyCode;
use uuid::Uuid;
use zwipe_client::ClientError;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::query::CardQuery;
use zwipe_core::domain::user::models::hints::HINT_COMMANDER_MAYBEBOARD;
use zwipe_ui::filter;
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::commander_maybeboard::{
    Action, Destination, Effect, QuickAddResults, State, Status,
};

use super::card_sheets::filter::{FilterSheet, SheetOutcome};
use super::playstyle::DeckLimit;
use super::{Screen, card_sheets, deck_form, oracle_tags};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::alert::AlertDialog;
use crate::inbound::widgets::card_rows::CardRows;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::sheet::Sheet;
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

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

/// The Quick add box. Its value follows each edit into the model, which runs the search.
#[derive(Debug, Default)]
pub(crate) struct QuickAdd {
    pub(crate) query: TextInput,
    /// Whether keys type into the box.
    pub(crate) typing: bool,
    /// The result chip under the cursor.
    pub(crate) cursor: usize,
}

/// The Commander maybeboard's state: the model plus what only the terminal keeps.
#[derive(Debug)]
pub(crate) struct MaybeboardView {
    pub(crate) model: State,
    /// How many decks the user owns, once fetched, for the commander swipe's create gate. The model holds the same count for its own Create deck but does not show it.
    pub(crate) decks: Option<usize>,
    pub(crate) quick: QuickAdd,
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

impl Default for MaybeboardView {
    /// A view waiting on its first load.
    fn default() -> Self {
        Self {
            model: State::new(),
            decks: None,
            quick: QuickAdd::default(),
            show_cursor: 0,
            rows: CardRows::default(),
            focus: Focus::Input,
            overlay: None,
            filter: CardQueryBuilder::default(),
            sheet: None,
        }
    }
}

impl MaybeboardView {
    /// Whether the first load is still out, so the whole page ghosts.
    pub(crate) fn is_loading(&self) -> bool {
        self.model.status() == Status::Loading
    }

    /// The saved list the Show pips and the applied card filter let through, as the model narrows it.
    pub(crate) fn shown(&self) -> Vec<Card> {
        self.model.shown(&self.filter)
    }

    /// Whether Filter carries its dot: the card filter has a criterion or a sort.
    pub(crate) fn filter_dot(&self) -> bool {
        filter::shows_dot(&self.filter)
    }

    /// The result chips that can be added, empty while searching or with nothing found.
    pub(crate) fn results(&self) -> Vec<Card> {
        match self.model.quick_add() {
            Some(QuickAddResults::Found(cards)) => cards,
            Some(QuickAddResults::Searching | QuickAddResults::Empty) | None => Vec::new(),
        }
    }

    /// Everything the focus can land on, top to bottom.
    pub(crate) fn stops(&self) -> Vec<Focus> {
        let mut stops = Vec::new();
        if self.is_loading() {
            return stops;
        }
        stops.push(Focus::Input);
        if !self.results().is_empty() {
            stops.push(Focus::Results);
        }
        stops.push(Focus::Show);
        if self.model.status() != Status::Failed && !self.shown().is_empty() {
            stops.push(Focus::List);
        }
        stops
    }

    /// Moves the focus one stop `toward` the next or the previous, entering the rows at their near end. Returns whether it moved.
    pub(crate) fn step(&mut self, toward: Toward) -> bool {
        let stops = self.stops();
        let at = stops.iter().position(|&s| s == self.focus).unwrap_or(0);
        let Some(&focus) = toward.neighbor(at).and_then(|i| stops.get(i)) else {
            return false;
        };
        if focus == Focus::List {
            let last = self.shown().len().saturating_sub(1);
            self.rows
                .set_focused(if toward == Toward::Next { 0 } else { last });
        }
        self.focus = focus;
        true
    }

    /// Keeps the focus on a stop that still exists: the result chips fall back to the box, the rows to the Show row.
    pub(crate) fn settle_focus(&mut self) {
        let count = self.results().len();
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
    pub(crate) fn pick_show(&mut self, index: usize, cx: &mut Context<'_>) {
        let action = match index
            .checked_sub(1)
            .and_then(|i| Color::all().get(i).copied())
        {
            Some(color) => Action::ToggleColor(color),
            None => Action::ShowAll,
        };
        dispatch(self, action, cx);
    }

    /// Hands the box's query to the model after an edit.
    pub(crate) fn query_changed(&mut self, cx: &mut Context<'_>) {
        self.quick.cursor = 0;
        let query = self.quick.query.value().to_owned();
        dispatch(self, Action::Type(query), cx);
    }

    /// Saves the result chip at `index`: it leaves the chips at once, and the row joins the list when the server confirms.
    pub(crate) fn add_result(&mut self, index: usize, cx: &mut Context<'_>) {
        let Some(oracle_id) = self
            .results()
            .get(index)
            .and_then(|c| c.scryfall_data.oracle_id)
        else {
            return;
        };
        dispatch(self, Action::Add(oracle_id), cx);
    }

    /// The create gate the commander swipe takes for the signed-in user.
    pub(crate) fn deck_limit(&self, cx: &Context<'_>) -> DeckLimit {
        DeckLimit::of(cx.shared.user.as_ref(), self.decks)
    }

    /// Refetches when the commander swipe closed, since its up swipes saved entries, and swaps in a printing picked from a row, keeping that row expanded.
    pub(crate) fn on_return(&mut self, cx: &mut Context<'_>) {
        if card_sheets::filter::returned_from_dictionary(self.sheet.as_mut(), cx) {
            return;
        }
        if cx.shared.caches.handoff.commander_swipe.take().is_some() {
            dispatch(self, Action::SwipeClosed, cx);
        }
        if let Some(printing) = cx.shared.caches.handoff.printing.take() {
            let id = printing.scryfall_data.id;
            dispatch(self, Action::PickPrinting(Box::new(printing)), cx);
            if self
                .model
                .entries()
                .iter()
                .any(|c| c.scryfall_data.id == id)
            {
                self.rows.expand(id);
            }
        }
    }
}

/// Applies `action` to the model, performs the effects it returns in order, and keeps the focus on a stop that still exists.
pub(crate) fn dispatch(view: &mut MaybeboardView, action: Action, cx: &mut Context<'_>) {
    let mut seed = None;
    for effect in view.model.update(action) {
        match effect {
            Effect::OfferHint(offer) => {
                hint::once(cx, offer.key());
            }
            Effect::FetchMaybeboard(ticket) => cx.request(Request::Load { ticket }.into()),
            Effect::FetchDeckProfiles => cx.request(Request::DeckProfiles.into()),
            Effect::Debounce { ticket, wait } => {
                cx.request(Request::Debounce { ticket, wait }.into());
            }
            Effect::SearchCommanders { ticket, query } => {
                cx.request(Request::Search { ticket, query }.into());
            }
            Effect::AddCommander { ticket, oracle_id } => {
                cx.request(Request::Add { ticket, oracle_id }.into());
            }
            Effect::RemoveCommander { ticket, oracle_id } => {
                cx.request(Request::Remove { ticket, oracle_id }.into());
            }
            Effect::ClearMaybeboard(ticket) => cx.request(Request::Clear { ticket }.into()),
            Effect::SeedCommander(card) => seed = Some(*card),
            Effect::Navigate(Destination::DeckList) => cx.back(),
            Effect::Navigate(Destination::CreateDeck) => {
                if let Some(card) = seed.take() {
                    deck_form::open_create_with_commander(cx, card);
                }
            }
            Effect::Toast(toast) => cx.show(toast),
        }
    }
    view.settle_focus();
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
            if view.focus == Focus::List && view.shown().is_empty() {
                view.focus = Focus::Show;
            }
        }
    }
}

/// The screen's work, each request but the deck count carrying its ticket.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    /// The saved list.
    Load { ticket: Ticket },
    /// The user's decks, for the create gate.
    DeckProfiles,
    /// Waits out the debounce for the query edit with this ticket.
    Debounce { ticket: Ticket, wait: Duration },
    /// Searches commanders with the query the model built.
    Search {
        ticket: Ticket,
        query: Box<CardQuery>,
    },
    /// Saves a commander from the result chips.
    Add { ticket: Ticket, oracle_id: Uuid },
    /// Takes a commander off.
    Remove { ticket: Ticket, oracle_id: Uuid },
    /// Takes every commander off.
    Clear { ticket: Ticket },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Load { .. } => "load",
            Self::DeckProfiles => "deck_profiles",
            Self::Debounce { .. } => "debounce",
            Self::Search { .. } => "search",
            Self::Add { .. } => "add",
            Self::Remove { .. } => "remove",
            Self::Clear { .. } => "clear",
        }
    }
}

/// The screen's results, each carrying its request's ticket back to the model.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded {
        ticket: Ticket,
        cards: Vec<Card>,
    },
    /// How many decks the user owns.
    DeckProfiles(usize),
    /// The debounce for this query edit is over.
    Debounced {
        ticket: Ticket,
    },
    Searched {
        ticket: Ticket,
        cards: Vec<Card>,
    },
    Added {
        ticket: Ticket,
    },
    Removed {
        ticket: Ticket,
    },
    Cleared {
        ticket: Ticket,
    },
    /// A request failed; `message` is the error toast's text, `None` for a quiet search.
    Failed {
        ticket: Ticket,
        message: Option<String>,
    },
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

/// Opens the Commander maybeboard: the model offers the hint and fetches the list and the user's decks, and the tag catalog warms for the rows.
pub(crate) fn open(cx: &mut Context<'_>) {
    cx.shared.caches.handoff.commander_swipe = None;
    let mut view = MaybeboardView::default();
    dispatch(&mut view, Action::Load, cx);
    oracle_tags::ensure_catalog(cx);
    cx.push(Screen::CommanderMaybeboard(Box::new(view)));
}

/// Shows the hint again, for `H`.
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    hint::reopen(cx, HINT_COMMANDER_MAYBEBOARD);
}

/// Runs the list edit `call`, its success as `done` and its failure with the message to toast.
async fn edit(
    ticket: Ticket,
    call: impl Future<Output = Result<(), ClientError>>,
    done: Update,
) -> Option<screens::Update> {
    match call.await {
        Ok(()) => Some(done.into()),
        Err(e) => Some(screens::failure(&e, |message| {
            Update::Failed {
                ticket,
                message: Some(message),
            }
            .into()
        })),
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
                Err(e) => Some(screens::failure(&e, |message| {
                    Update::Failed {
                        ticket,
                        message: Some(message),
                    }
                    .into()
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
        Request::Debounce { ticket, wait } => {
            tokio::time::sleep(wait).await;
            Some(Update::Debounced { ticket }.into())
        }
        Request::Search { ticket, query } => {
            match async {
                client
                    .search_commanders(&query, &account.fresh().await?)
                    .await
            }
            .await
            {
                Ok(cards) => Some(Update::Searched { ticket, cards }.into()),
                // Quiet, as the app's `run_quiet`: a toast per keystroke would bury the screen.
                Err(e) => Some(screens::failure(&e, |_| {
                    Update::Failed {
                        ticket,
                        message: None,
                    }
                    .into()
                })),
            }
        }
        Request::Add { ticket, oracle_id } => {
            edit(
                ticket,
                async {
                    client
                        .add_commander_maybeboard_card(oracle_id, &account.fresh().await?)
                        .await
                },
                Update::Added { ticket },
            )
            .await
        }
        Request::Remove { ticket, oracle_id } => {
            edit(
                ticket,
                async {
                    client
                        .remove_commander_maybeboard_card(oracle_id, &account.fresh().await?)
                        .await
                },
                Update::Removed { ticket },
            )
            .await
        }
        Request::Clear { ticket } => {
            edit(
                ticket,
                async {
                    client
                        .clear_commander_maybeboard(&account.fresh().await?)
                        .await
                },
                Update::Cleared { ticket },
            )
            .await
        }
    }
}

/// Hands a reply to the model. A failure's error toast goes up even when the screen has closed.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let cleared = matches!(update, Update::Cleared { .. });
    let action = match update {
        Update::Loaded { ticket, cards } => Action::Loaded { ticket, cards },
        Update::DeckProfiles(count) => {
            if let Some(view) = nav.find_mut(pick) {
                view.decks = Some(count);
            }
            Action::DeckProfilesLoaded { count }
        }
        Update::Debounced { ticket } => Action::Debounced { ticket },
        Update::Searched { ticket, cards } => Action::Searched { ticket, cards },
        Update::Added { ticket } => Action::Added { ticket },
        Update::Removed { ticket } => Action::Removed { ticket },
        Update::Cleared { ticket } => Action::Cleared { ticket },
        Update::Failed { ticket, message } => {
            if let Some(message) = message {
                cx.toast(ToastKind::Error, message);
            }
            Action::Failed { ticket }
        }
    };
    if let Some(view) = nav.find_mut(pick) {
        dispatch(view, action, cx);
        if cleared && view.model.entries().is_empty() {
            view.rows.collapse();
        }
    }
}
