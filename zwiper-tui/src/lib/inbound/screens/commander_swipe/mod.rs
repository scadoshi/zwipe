//! The commander swipe, the app's `SwipeSelect` overlay in its two commander modes: Suggested deals Playstyle's held ranking narrowed by the same identity filter, and Commander deals `search_commanders` for the Commander format, most played first, leaving out oracle ids the host already has. Playstyle opens it with `s`; the Commander maybeboard opens it with its saved oracle ids excluded.
//!
//! Right picks the card on top: it closes the overlay and starts a deck with it through [`create_deck_with`]. Up saves it to the commander maybeboard, left skips it (in Suggested, hiding it for a week, quietly), and down undoes the last swipe, lifting a save or a dismissal. Closing leaves a [`Closed`] in the handoff, which tells the host to refetch and, from Suggested, which identity the Filter left picked.
//!
//! The app's eye button opens a details sheet over the card; the terminal shows the card's text and, in Suggested, its reasons right under it. Filter in Suggested picks one color identity from the Show chips; in Commander it opens the card filter sheet, whose Apply deals the stack again from the top.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::deck::format::Format;
use zwipe_core::http::contracts::suggestions::HttpCommanderSuggestion;

use super::card_sheets::filter::{FilterSheet, SheetConfig, SheetOutcome};
use super::playstyle::{DeckLimit, create_deck_with, suggestion_colors};
use super::{Screen, card_sheets};
use crate::domain::stack::Direction;
use crate::domain::{filter, playstyle};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::card_motion::CardMotion;
use crate::inbound::widgets::hint::{self, HintContent};
use crate::inbound::widgets::toast::{self, ToastKind};
use crate::outbound::account::Account;

/// Cards fetched per page in Commander mode, the backend's default.
pub(crate) const PAGE: u32 = 25;

/// The next page loads when this many cards or fewer are left ahead of the top.
pub(crate) const LOAD_MORE_AT: usize = 5;

/// Pages pulled in a row when exclusions empty a served page, before giving up until the next swipe.
pub(crate) const PULL_AHEAD: u8 = 4;

/// How long the maybeboard save toast stays up, as the app's `MAYBEBOARD_TOAST_MS`.
pub(crate) const SAVED_TOAST: Duration = Duration::from_millis(1500);

/// The Suggested mode's title and its hint's.
pub(crate) const SUGGESTED_TITLE: &str = "Suggested commanders";

/// The Commander mode's title and its hint's.
pub(crate) const COMMANDER_TITLE: &str = "Swipe select";

pub(crate) const SELECTED: &str = "Commander selected";
pub(crate) const SAVED: &str = "Added to your commander maybeboard";
pub(crate) const CANT_SAVE: &str = "This card can't be saved";
pub(crate) const SKIPPED: &str = "Skipped";
pub(crate) const NOTHING_TO_UNDO: &str = "Nothing to undo";
pub(crate) const UNSAVED: &str = "Removed from your commander maybeboard";
pub(crate) const UNDID_SKIP: &str = "Undid skip";
pub(crate) const REFRESHED: &str = "Stack refreshed";

/// The ranking a Suggested swipe deals: the host's held list, the deck count its reasons count against, and the identity its Show row picked (`None` is All).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SuggestedPile {
    pub(crate) held: Vec<HttpCommanderSuggestion>,
    pub(crate) deck_count: u32,
    pub(crate) identity: Option<Vec<Color>>,
}

/// What the swipe deals, chosen by its host.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Deal {
    /// Playstyle's held ranking.
    Suggested(SuggestedPile),
    /// Commanders for the Commander format, most played first, without these oracle ids.
    Commander { exclude: HashSet<Uuid> },
}

/// What the swipe leaves its host when it closes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Closed {
    /// The identity the Suggested Filter left picked; always `None` from Commander.
    pub(crate) identity: Option<Vec<Color>>,
}

/// A swipe that moved the pile on, so undo can reverse it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Swiped {
    /// A plain skip, or an up on a card that could not be saved.
    Skip,
    /// Saved this oracle id to the commander maybeboard.
    Maybe(Uuid),
    /// Dismissed this oracle id for a week, in Suggested.
    Dismissed(Uuid),
}

/// The Commander mode's paging.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Search {
    pub(crate) exclude: HashSet<Uuid>,
    /// The newest search's ticket; only its pages land.
    pub(crate) ticket: u64,
    /// The next page's offset.
    pub(crate) offset: u32,
    /// The server ran out.
    pub(crate) exhausted: bool,
    /// A page is out.
    pub(crate) loading: bool,
    /// Pages pulled in a row because exclusions emptied them.
    pub(crate) pulls: u8,
    /// The card filter as last applied; each page's query starts from it.
    pub(crate) filter: Box<CardQueryBuilder>,
}

/// Which pile the swipe deals.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Mode {
    Suggested(SuggestedPile),
    Commander(Search),
}

/// The Suggested Filter sheet, open with its chip cursor (0 is All).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Filter {
    pub(crate) cursor: usize,
}

/// A card's identity across printings: its oracle id, else its scryfall id.
pub(crate) fn card_key(card: &Card) -> Uuid {
    card.scryfall_data
        .oracle_id
        .unwrap_or(card.scryfall_data.id)
}

/// The pile re-dealt from a new ranking mid-session. The first `swiped` of `dealt` stay in front, so the top card's index and the undo history still line up with them; then every item of `fresh` whose key is not among them, in `fresh`'s order.
pub(crate) fn redeal<T: Clone, K: PartialEq>(
    dealt: &[T],
    swiped: usize,
    fresh: &[T],
    key: impl Fn(&T) -> K,
) -> Vec<T> {
    let mut pile: Vec<T> = dealt.iter().take(swiped).cloned().collect();
    let kept: Vec<K> = pile.iter().map(&key).collect();
    pile.extend(
        fresh
            .iter()
            .filter(|item| !kept.contains(&key(item)))
            .cloned(),
    );
    pile
}

/// The commander swipe's state.
#[derive(Debug)]
pub(crate) struct CommanderSwipeView {
    pub(crate) mode: Mode,
    /// Every card dealt so far, the top one at `index`.
    pub(crate) cards: Vec<Card>,
    pub(crate) index: usize,
    pub(crate) history: Vec<Swiped>,
    /// The create gate, as the host saw it when it opened the swipe.
    pub(crate) limit: DeckLimit,
    pub(crate) filter: Option<Filter>,
    /// Commander mode's card filter sheet, while it is open over the stack.
    pub(crate) sheet: Option<FilterSheet>,
    /// The swiped card flying off, or the undone one coming back. A pick closes the swipe, so it has no exit.
    pub(crate) motion: Option<CardMotion<Card>>,
}

impl CommanderSwipeView {
    pub(crate) fn new(deal: Deal, limit: DeckLimit) -> Self {
        let mode = match deal {
            Deal::Suggested(pile) => Mode::Suggested(pile),
            Deal::Commander { exclude } => Mode::Commander(Search {
                exclude,
                ticket: 0,
                offset: 0,
                exhausted: false,
                loading: false,
                pulls: 0,
                filter: Box::default(),
            }),
        };
        let mut view = Self {
            mode,
            cards: Vec::new(),
            index: 0,
            history: Vec::new(),
            limit,
            filter: None,
            sheet: None,
            motion: None,
        };
        if let Mode::Suggested(pile) = &view.mode {
            view.cards = dealt(pile);
        }
        view
    }

    pub(crate) fn suggested(&self) -> Option<&SuggestedPile> {
        match &self.mode {
            Mode::Suggested(pile) => Some(pile),
            Mode::Commander(_) => None,
        }
    }

    pub(crate) fn title(&self) -> &'static str {
        match self.mode {
            Mode::Suggested(_) => SUGGESTED_TITLE,
            Mode::Commander(_) => COMMANDER_TITLE,
        }
    }

    /// The app's `swipe_select` hint for this mode: the suggested deal, or picking a commander.
    pub(crate) fn hint(&self) -> HintContent {
        hint::keys::swipe_select(
            matches!(self.mode, Mode::Suggested(_)),
            "commander",
            /* has_filter */ true,
        )
    }

    /// The card on top.
    pub(crate) fn current(&self) -> Option<&Card> {
        self.cards.get(self.index)
    }

    /// The top card's suggestion, for its reasons, in Suggested.
    pub(crate) fn current_suggestion(&self) -> Option<&HttpCommanderSuggestion> {
        let id = self.current()?.scryfall_data.id;
        self.suggested()?
            .held
            .iter()
            .find(|s| s.card.scryfall_data.id == id)
    }

    /// Whether a page is out in Commander.
    pub(crate) fn loading(&self) -> bool {
        matches!(&self.mode, Mode::Commander(search) if search.loading)
    }

    /// The identities the Filter offers: those among the held list plus the picked one.
    pub(crate) fn identities(&self) -> Vec<Vec<Color>> {
        self.suggested().map_or_else(Vec::new, |pile| {
            playstyle::identities_present(
                pile.held
                    .iter()
                    .map(suggestion_colors)
                    .chain(pile.identity.as_deref()),
            )
        })
    }

    /// Picks the Filter chip at `index` (0 is All; the picked identity again is All) and re-deals behind the cards already swiped.
    pub(crate) fn pick_identity(&mut self, index: usize) {
        let identities = self.identities();
        let Mode::Suggested(pile) = &mut self.mode else {
            return;
        };
        let picked = index
            .checked_sub(1)
            .and_then(|i| identities.into_iter().nth(i));
        pile.identity = if picked.is_some() && picked == pile.identity {
            None
        } else {
            picked
        };
        let fresh = dealt(pile);
        self.cards = redeal(&self.cards, self.index, &fresh, card_key);
        if let Some(filter) = &mut self.filter
            && pile.identity.is_none()
        {
            filter.cursor = 0;
        }
    }

    /// Whether a card is in motion, so the loop redraws.
    pub(crate) fn animating(&self) -> bool {
        self.motion
            .as_ref()
            .is_some_and(|m| m.running(Instant::now()))
    }

    /// The card under the top one, which a card coming back after an undo covers.
    pub(crate) fn under(&self) -> Option<&Card> {
        self.cards.get(self.index.checked_add(1)?)
    }

    /// Sends `card` off by `direction`, replacing any motion still running.
    fn exit(&mut self, direction: Direction, card: Card) {
        self.motion = Some(CardMotion::exit(direction, card, Instant::now()));
    }

    /// Moves past the top card, asking for more in Commander when few are left.
    fn advance(&mut self, cx: &mut Context<'_>) {
        self.index = self.index.saturating_add(1);
        let left = self.cards.len().saturating_sub(self.index);
        if let Mode::Commander(search) = &mut self.mode
            && !search.exhausted
            && left <= LOAD_MORE_AT
        {
            search.pulls = 0;
            load_more(search, cx);
        }
    }

    /// Right: picks the top card, closes, and starts a deck with it.
    pub(crate) fn select(&mut self, cx: &mut Context<'_>) {
        let Some(card) = self.current().cloned() else {
            return;
        };
        cx.shared
            .toasts
            .push_for(ToastKind::Success, SELECTED.to_owned(), toast::QUICK);
        self.close(cx);
        create_deck_with(cx, &card, self.limit);
    }

    /// Up: saves the top card to the commander maybeboard and moves on.
    pub(crate) fn save(&mut self, cx: &mut Context<'_>) {
        let Some(card) = self.current().cloned() else {
            return;
        };
        let oracle_id = card.scryfall_data.oracle_id;
        self.exit(Direction::Up, card);
        self.advance(cx);
        let Some(oracle_id) = oracle_id else {
            self.history.push(Swiped::Skip);
            cx.shared
                .toasts
                .push_for(ToastKind::Error, CANT_SAVE.to_owned(), toast::NORMAL);
            return;
        };
        self.history.push(Swiped::Maybe(oracle_id));
        cx.request(Request::Save(oracle_id).into());
    }

    /// Left: skips the top card, in Suggested hiding it for a week.
    pub(crate) fn skip(&mut self, cx: &mut Context<'_>) {
        let Some(card) = self.current().cloned() else {
            return;
        };
        let oracle_id = card.scryfall_data.oracle_id;
        self.exit(Direction::Left, card);
        self.advance(cx);
        match (&self.mode, oracle_id) {
            (Mode::Suggested(_), Some(oracle_id)) => {
                self.history.push(Swiped::Dismissed(oracle_id));
                cx.request(Request::Dismiss(oracle_id).into());
            }
            _ => self.history.push(Swiped::Skip),
        }
        cx.shared
            .toasts
            .push_for(ToastKind::Info, SKIPPED.to_owned(), toast::QUICK);
    }

    /// Down: brings the last card back, lifting its save or dismissal.
    pub(crate) fn undo(&mut self, cx: &mut Context<'_>) {
        let Some(index) = self.index.checked_sub(1) else {
            cx.shared
                .toasts
                .push_for(ToastKind::Info, NOTHING_TO_UNDO.to_owned(), toast::NORMAL);
            return;
        };
        self.index = index;
        let swiped = self.history.pop();
        let side = match swiped {
            Some(Swiped::Maybe(_)) => Direction::Up,
            Some(Swiped::Dismissed(_) | Swiped::Skip) | None => Direction::Left,
        };
        self.motion = Some(CardMotion::enter(side, Instant::now()));
        match swiped {
            Some(Swiped::Maybe(oracle_id)) => cx.request(Request::Unsave(oracle_id).into()),
            Some(Swiped::Dismissed(oracle_id)) => {
                cx.request(Request::Undismiss(oracle_id).into());
                cx.shared
                    .toasts
                    .push_for(ToastKind::Info, UNDID_SKIP.to_owned(), toast::QUICK);
            }
            Some(Swiped::Skip) | None => {
                cx.shared
                    .toasts
                    .push_for(ToastKind::Info, UNDID_SKIP.to_owned(), toast::QUICK);
            }
        }
    }

    /// Deals afresh from the top: the host's pile again, or a new search.
    pub(crate) fn refresh(&mut self, cx: &mut Context<'_>) {
        self.restart(cx);
        cx.shared
            .toasts
            .push_for(ToastKind::Info, REFRESHED.to_owned(), toast::QUICK);
    }

    /// Whether Filter carries its dot: an identity picked in Suggested, a criterion or sort in Commander.
    pub(crate) fn filter_dot(&self) -> bool {
        match &self.mode {
            Mode::Suggested(pile) => pile.identity.is_some(),
            Mode::Commander(search) => filter::shows_dot(&search.filter),
        }
    }

    /// Opens the Filter: the identity chips in Suggested, the card filter sheet without its format section in Commander.
    pub(crate) fn open_filter(&mut self, cx: &mut Context<'_>) {
        match &self.mode {
            Mode::Suggested(pile) => {
                let cursor = pile
                    .identity
                    .as_ref()
                    .and_then(|picked| self.identities().iter().position(|i| i == picked))
                    .map_or(0, |i| i.saturating_add(1));
                self.filter = Some(Filter { cursor });
            }
            Mode::Commander(search) => {
                let config = SheetConfig {
                    show_format: false,
                    ..SheetConfig::default()
                };
                self.sheet = Some(FilterSheet::open(cx, &search.filter, config));
            }
        }
    }

    /// Keys while the card filter sheet is open. Apply keeps the filter and deals the stack again from the top.
    pub(crate) fn on_sheet(&mut self, code: KeyCode, cx: &mut Context<'_>) {
        let Some(sheet) = &mut self.sheet else {
            return;
        };
        match card_sheets::filter::keys::handle(sheet, code, cx) {
            SheetOutcome::Open => {}
            SheetOutcome::Closed => self.sheet = None,
            SheetOutcome::Applied(applied) => {
                self.sheet = None;
                if let Mode::Commander(search) = &mut self.mode {
                    search.filter = applied;
                }
                self.restart(cx);
            }
        }
    }

    /// Deals from the top again without a word.
    fn restart(&mut self, cx: &mut Context<'_>) {
        self.motion = None;
        self.index = 0;
        self.history.clear();
        match &mut self.mode {
            Mode::Suggested(pile) => self.cards = dealt(pile),
            Mode::Commander(search) => {
                self.cards.clear();
                search.offset = 0;
                search.exhausted = false;
                search.pulls = 0;
                search.ticket = search.ticket.wrapping_add(1);
                search.loading = false;
                load_more(search, cx);
            }
        }
    }

    /// Closes the swipe, leaving its host the identity picked and the word to refetch.
    pub(crate) fn close(&self, cx: &mut Context<'_>) {
        let identity = self.suggested().and_then(|pile| pile.identity.clone());
        cx.shared.caches.handoff.commander_swipe = Some(Closed { identity });
        cx.back();
    }

    /// Takes a page: drops cards already dealt or excluded, and pulls ahead when exclusions emptied a page the server had more behind.
    fn land(&mut self, ticket: u64, offset: u32, page: Vec<Card>, cx: &mut Context<'_>) {
        let Mode::Commander(search) = &mut self.mode else {
            return;
        };
        if ticket != search.ticket || offset != search.offset {
            return;
        }
        search.loading = false;
        if page.is_empty() {
            search.exhausted = true;
            return;
        }
        search.offset = offset.saturating_add(PAGE);
        let seen: HashSet<Uuid> = self.cards.iter().map(|c| c.scryfall_data.id).collect();
        let fresh: Vec<Card> = page
            .into_iter()
            .filter(|c| !seen.contains(&c.scryfall_data.id))
            .filter(|c| {
                !c.scryfall_data
                    .oracle_id
                    .is_some_and(|o| search.exclude.contains(&o))
            })
            .collect();
        if fresh.is_empty() && search.pulls < PULL_AHEAD {
            search.pulls = search.pulls.saturating_add(1);
            load_more(search, cx);
            return;
        }
        self.cards.extend(fresh);
    }
}

/// The Suggested pile's cards: the held list narrowed to the picked identity.
fn dealt(pile: &SuggestedPile) -> Vec<Card> {
    playstyle::of_identity(&pile.held, suggestion_colors, pile.identity.as_deref())
        .into_iter()
        .map(|s| s.card)
        .collect()
}

/// Asks for the next page unless one is out.
fn load_more(search: &mut Search, cx: &mut Context<'_>) {
    if search.loading {
        return;
    }
    search.loading = true;
    cx.request(
        Request::Search {
            ticket: search.ticket,
            offset: search.offset,
            filter: search.filter.clone(),
        }
        .into(),
    );
}

/// The query for one page of commanders: the user's card filter, then the Commander format, no tokens, and the page. With no sort in the filter the server serves its most-played order.
pub(crate) fn commander_query(filter: &CardQueryBuilder, offset: u32) -> CardQueryBuilder {
    let mut builder = filter.clone();
    builder
        .set_is_commander_in_format(Format::Commander)
        .set_is_token(false)
        .set_limit(PAGE)
        .set_offset(offset);
    builder
}

/// Opens the swipe over its host, dealing `deal`, with the host's create gate. Commander mode asks for its first page; both show the swipe hint the first time.
pub(crate) fn open(cx: &mut Context<'_>, deal: Deal, limit: DeckLimit) {
    let mut view = CommanderSwipeView::new(deal, limit);
    if let Mode::Commander(search) = &mut view.mode {
        load_more(search, cx);
    }
    let content = view.hint();
    cx.push(Screen::CommanderSwipe(Box::new(view)));
    hint::once_with(cx, hint::keys::SWIPE_SELECT, content);
}

/// Shows the hint again, for `?`.
pub(crate) fn reopen_hint(view: &CommanderSwipeView, cx: &mut Context<'_>) {
    hint::open(cx, view.hint());
}

/// The swipe's work.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    /// One page of commanders under a ticket, narrowed by the card filter.
    Search {
        ticket: u64,
        offset: u32,
        filter: Box<CardQueryBuilder>,
    },
    /// Saves to the commander maybeboard.
    Save(Uuid),
    /// Takes a save back off, for undo.
    Unsave(Uuid),
    /// Hides a suggestion for a week, quietly.
    Dismiss(Uuid),
    /// Lifts a dismissal, quietly, for undo.
    Undismiss(Uuid),
}

/// The swipe's results.
#[derive(Debug)]
pub(crate) enum Update {
    Page {
        ticket: u64,
        offset: u32,
        cards: Vec<Card>,
    },
    /// A page failed; the swipe stops loading and keeps what it has.
    SearchFailed {
        ticket: u64,
    },
    Saved,
    Unsaved,
    /// A save or its undo failed with this message.
    Failed(String),
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::CommanderSwipe(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::CommanderSwipe(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut CommanderSwipeView> {
    match screen {
        Screen::CommanderSwipe(view) => Some(view),
        _ => None,
    }
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    match request {
        Request::Search {
            ticket,
            offset,
            filter,
        } => {
            let Ok(query) = commander_query(&filter, offset).build() else {
                return Some(Update::SearchFailed { ticket }.into());
            };
            match async {
                client
                    .search_commanders(&query, &account.fresh().await?)
                    .await
            }
            .await
            {
                Ok(cards) => Some(
                    Update::Page {
                        ticket,
                        offset,
                        cards,
                    }
                    .into(),
                ),
                Err(e) => Some(screens::failure(&e, |_| {
                    Update::SearchFailed { ticket }.into()
                })),
            }
        }
        Request::Save(oracle_id) => {
            match async {
                client
                    .add_commander_maybeboard_card(oracle_id, &account.fresh().await?)
                    .await
            }
            .await
            {
                Ok(()) => Some(Update::Saved.into()),
                Err(e) => Some(screens::failure(&e, |message| {
                    Update::Failed(message).into()
                })),
            }
        }
        Request::Unsave(oracle_id) => {
            match async {
                client
                    .remove_commander_maybeboard_card(oracle_id, &account.fresh().await?)
                    .await
            }
            .await
            {
                Ok(()) => Some(Update::Unsaved.into()),
                Err(e) => Some(screens::failure(&e, |message| {
                    Update::Failed(message).into()
                })),
            }
        }
        // Quiet like the app's: the pile already moved on, and a failure only means the commander comes back sooner.
        Request::Dismiss(oracle_id) => {
            let _ = async {
                client
                    .dismiss_commander(oracle_id, &account.fresh().await?)
                    .await
            }
            .await;
            None
        }
        Request::Undismiss(oracle_id) => {
            let _ = async {
                client
                    .undismiss_commander(oracle_id, &account.fresh().await?)
                    .await
            }
            .await;
            None
        }
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    match update {
        Update::Page {
            ticket,
            offset,
            cards,
        } => {
            if let Some(view) = nav.find_mut(pick) {
                view.land(ticket, offset, cards, cx);
            }
        }
        Update::SearchFailed { ticket } => {
            if let Some(view) = nav.find_mut(pick)
                && let Mode::Commander(search) = &mut view.mode
                && search.ticket == ticket
            {
                search.loading = false;
            }
        }
        Update::Saved => {
            cx.shared
                .toasts
                .push_for(ToastKind::Success, SAVED.to_owned(), SAVED_TOAST);
        }
        Update::Unsaved => {
            cx.shared
                .toasts
                .push_for(ToastKind::Info, UNSAVED.to_owned(), toast::QUICK);
        }
        Update::Failed(message) => {
            cx.shared
                .toasts
                .push_for(ToastKind::Error, message, toast::NORMAL);
        }
    }
}
