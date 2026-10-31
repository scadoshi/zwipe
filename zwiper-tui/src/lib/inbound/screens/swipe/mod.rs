//! Add deck cards: swipe search results or the maybeboard into a deck, with the filter sheet, Synergy, undo that reconciles with the deck's undo log, and the land and budget signals.
//!
//! Search results come from the server a page at a time; the maybeboard is filtered in memory. Leaving parks the filter per source and the search stack per deck, so coming back resumes.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

use image::DynamicImage;
use ratatui_image::protocol::StatefulProtocol;
use uuid::Uuid;
use zwipe_client::ClientError;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::deck::{Board, Deck, DeckCard, DeckEntry};
use zwipe_core::domain::user::hints::HINT_ADD_DECK_CARDS;
use zwipe_core::http::contracts::deck::HttpDeckCardSearch;
use zwipe_core::http::contracts::deck_card::{HttpCreateDeckCard, HttpPatchDeckCard};

use super::Screen;
use super::card_sheets::details::DetailsSheet;
use super::card_sheets::filter::{FilterSheet, ResetRule, SheetConfig};
use super::card_sheets::printing::PrintingSheet;
use crate::domain::add::{self, AddDeck, LOAD_MORE_THRESHOLD};
use crate::domain::deck_signals::{self, Crossing, Level};
use crate::domain::filter::picker::Sources;
use crate::domain::stack::{
    AddAction, CARDS_WARNING_THRESHOLD, CardStack, Direction, MAX_CARDS_IN_STACK, MaybeboardAction,
    STACK_WARNING_EVERY, StackAction,
};
use crate::domain::stores::filters::FilterScope;
use crate::domain::stores::in_flight::AddOutcome;
use crate::domain::stores::parked::ParkedStack;
use crate::domain::stores::undo::UndoAction;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::card_motion::{CardMotion, Snapshot};
use crate::inbound::widgets::hint;
use crate::inbound::widgets::toast::{self, ToastKind};
use crate::outbound::account::Account;
use crate::outbound::images;

/// Images fetched ahead of the card on screen.
const IMAGES_AHEAD: usize = 3;

/// Where the cards come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Source {
    #[default]
    Search,
    Maybeboard,
}

impl Source {
    fn scope(self) -> FilterScope {
        match self {
            Self::Search => FilterScope::AddSearch,
            Self::Maybeboard => FilterScope::AddMaybeboard,
        }
    }
}

/// The Add screen.
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent states the app keeps as separate signals"
)]
#[derive(Debug)]
pub(crate) struct SwipeView {
    pub(crate) deck: AddDeck,
    pub(crate) filter: CardQueryBuilder,
    pub(crate) source: Source,
    /// The search stack: linear, swiped cards stay behind the cursor.
    pub(crate) stack: CardStack<AddAction>,
    /// The maybeboard stack: cycling.
    pub(crate) mb_stack: CardStack<MaybeboardAction>,
    /// The filter that produced the search stack.
    pub(crate) last_search_filter: Option<CardQueryBuilder>,
    /// Bumped by every fresh search, so answers for an older stack are dropped.
    pub(crate) generation: u64,
    pub(crate) loading_cards: bool,
    pub(crate) loading_more: bool,
    /// Paging ran dry.
    pub(crate) exhausted: bool,
    pub(crate) synergy_warming: bool,
    /// Printing IDs of cards served as like the deck rather than synergy picks.
    pub(crate) related_ids: std::collections::HashSet<Uuid>,
    pub(crate) related_announced: bool,
    pub(crate) sheet: Option<FilterSheet>,
    pub(crate) details: Option<DetailsSheet>,
    pub(crate) printing: Option<PrintingSheet>,
    /// The face of the top card shown.
    pub(crate) face: usize,
    /// Images by URL.
    pub(crate) images: HashMap<String, CardImage>,
    /// On a narrow terminal, whether the card shows as its image instead of its text.
    pub(crate) show_image: bool,
    /// The swiped card flying off, or the undone one coming back.
    pub(crate) motion: Option<Motion>,
}

/// A card motion on Add, an exit carrying the card that left.
pub(crate) type Motion = CardMotion<Snapshot>;

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
    /// Whether a card is in motion, so the loop redraws.
    pub(crate) fn animating(&self) -> bool {
        self.motion
            .as_ref()
            .is_some_and(|m| m.running(Instant::now()))
    }

    /// The card under the top one, which a card coming back after an undo covers.
    pub(crate) fn under(&self) -> Option<&Card> {
        match self.source {
            Source::Search => self.stack.next(),
            Source::Maybeboard => self.mb_stack.next_wrapping(),
        }
    }

    /// Sends the top card off by `direction`, keeping it and its face to draw it leaving. Replaces any motion still running.
    fn exit(&mut self, direction: Direction, card: Card) {
        let snapshot = Snapshot {
            card,
            face: self.face,
        };
        self.motion = Some(CardMotion::exit(direction, snapshot, Instant::now()));
    }

    /// The card on top: the search cursor's, or the maybeboard's, wrapping.
    pub(crate) fn current(&self) -> Option<&Card> {
        match self.source {
            Source::Search => self.stack.current(),
            Source::Maybeboard => self.mb_stack.current_wrapping(),
        }
    }

    /// Whether the Filter key shows its dot.
    pub(crate) fn filter_dot(&self) -> bool {
        self.deck.context().shows_dot(&self.filter)
    }

    /// Whether the Synergy chip shows: on Search, for a deck with a commander.
    pub(crate) fn synergy_shown(&self) -> bool {
        self.source == Source::Search && self.deck.has_commander
    }

    /// Runs the filter: a fresh server search on Search, an in-memory pass on the maybeboard.
    pub(crate) fn rerun(&mut self, cx: &mut Context<'_>) {
        match self.source {
            Source::Search => self.search(cx),
            Source::Maybeboard => self.refilter(),
        }
        self.face = 0;
        self.sync_images(cx);
    }

    /// A fresh search: drops the stack, then fetches the first page if the filter is enough to serve.
    pub(crate) fn search(&mut self, cx: &mut Context<'_>) {
        self.generation = self.generation.wrapping_add(1);
        self.loading_cards = false;
        self.loading_more = false;
        self.stack.reset();
        self.last_search_filter = None;
        self.exhausted = false;
        if !self.deck.context().serves(&self.filter) {
            cx.toast(
                ToastKind::Warning,
                "Try a filter to start swiping".to_owned(),
            );
            return;
        }
        let Ok(query) = add::normalized(&self.filter).build() else {
            return;
        };
        self.loading_cards = true;
        cx.request(
            Request::Search {
                deck_id: self.deck.id,
                generation: self.generation,
                query: Box::new(query),
                filter: Box::new(self.filter.clone()),
                page: Page::First,
            }
            .into(),
        );
    }

    /// Refresh: refetches the search without the gate, or reloads the maybeboard.
    pub(crate) fn refresh(&mut self, cx: &mut Context<'_>) {
        match self.source {
            Source::Maybeboard => {
                cx.request(
                    Request::ReloadMaybeboard {
                        deck_id: self.deck.id,
                    }
                    .into(),
                );
                cx.toast_for(
                    ToastKind::Info,
                    "Maybeboard refreshed".to_owned(),
                    toast::QUICK,
                );
            }
            Source::Search => {
                let Ok(query) = add::normalized(&self.filter).build() else {
                    cx.toast_for(
                        ToastKind::Warning,
                        "Filter is empty".to_owned(),
                        toast::QUICK,
                    );
                    return;
                };
                self.generation = self.generation.wrapping_add(1);
                self.stack.reset();
                self.last_search_filter = None;
                self.exhausted = false;
                self.loading_more = false;
                self.loading_cards = true;
                cx.request(
                    Request::Search {
                        deck_id: self.deck.id,
                        generation: self.generation,
                        query: Box::new(query),
                        filter: Box::new(self.filter.clone()),
                        page: Page::Refresh,
                    }
                    .into(),
                );
                cx.toast_for(ToastKind::Info, "Stack refreshed".to_owned(), toast::QUICK);
            }
        }
    }

    /// The maybeboard filtered and sorted in memory, the cursor back at the first card.
    pub(crate) fn refilter(&mut self) {
        let cards: Vec<Card> = self
            .deck
            .maybeboard
            .iter()
            .map(|e| e.card.clone())
            .collect();
        let mut cards = Cards::from(cards);
        if !self.filter.is_empty()
            && let Ok(criteria) = self.filter.build_criteria()
        {
            cards = cards.matching(&criteria);
        }
        let cards = cards.sorted(self.filter.sort(), self.filter.direction());
        self.mb_stack.replace(cards.into());
    }

    /// Fetches the next page when the stack runs low, unless one is out or the cap is reached.
    fn load_more(&mut self, cx: &mut Context<'_>) {
        if self.stack.len() >= MAX_CARDS_IN_STACK {
            cx.toast(
                ToastKind::Warning,
                "Card stack limit reached, please refresh your search to continue".to_owned(),
            );
            return;
        }
        if self.loading_more {
            return;
        }
        let Ok(query) = add::normalized(&self.filter).build() else {
            return;
        };
        self.loading_more = true;
        cx.request(
            Request::Search {
                deck_id: self.deck.id,
                generation: self.generation,
                query: Box::new(query),
                filter: Box::new(self.filter.clone()),
                page: Page::More,
            }
            .into(),
        );
    }

    /// Moves past a committed swipe: warns near the cap, says when results end, and fetches more when low.
    fn advance_after_commit(&mut self, cx: &mut Context<'_>) {
        let total = self.stack.len();
        if !self.stack.advance() {
            return;
        }
        if (CARDS_WARNING_THRESHOLD..MAX_CARDS_IN_STACK).contains(&total)
            && total.is_multiple_of(STACK_WARNING_EVERY)
        {
            cx.toast_for(
                ToastKind::Info,
                "Approaching the card stack limit, refresh your search to continue".to_owned(),
                toast::QUICK,
            );
        }
        let index = self.stack.index();
        if index >= total && self.exhausted {
            cx.toast(ToastKind::Warning, "End of results".to_owned());
        } else if total > 0 && index.saturating_add(1) >= total.saturating_sub(LOAD_MORE_THRESHOLD)
        {
            self.load_more(cx);
        }
    }

    /// Says once per fresh stack when the top card is the first one served as like the deck.
    fn announce_related(&mut self, cx: &mut Context<'_>) {
        if self.source != Source::Search || self.related_announced {
            return;
        }
        if self
            .stack
            .current()
            .is_some_and(|c| self.related_ids.contains(&c.card_profile.scryfall_data_id))
        {
            self.related_announced = true;
            cx.toast(
                ToastKind::Info,
                "That's every synergy pick. Showing cards like your deck".to_owned(),
            );
        }
    }

    /// Tracks whether synergy is still warming, saying so when it turns cold.
    fn set_warming(&mut self, warming: bool, cx: &mut Context<'_>) {
        if warming && !self.synergy_warming {
            cx.toast(
                ToastKind::Info,
                "Synergy warming up; showing all cards for now".to_owned(),
            );
        }
        self.synergy_warming = warming;
    }

    /// Swipes the top card right: adds it to the deck.
    pub(crate) fn swipe_right(&mut self, cx: &mut Context<'_>) {
        match self.source {
            Source::Search => {
                let Some(card) = self.stack.current().cloned() else {
                    return;
                };
                self.stack.record(AddAction::Add);
                self.exit(Direction::Right, card.clone());
                self.create(cx, &card, Board::Deck);
                cx.toast_for(ToastKind::Success, "Added to deck".to_owned(), toast::QUICK);
                if card.scryfall_data.is_land() {
                    let before = self.deck.land_count;
                    self.deck.land_count = before.saturating_add(1);
                    signal(
                        cx,
                        deck_signals::land_crossing(
                            self.deck.land_target,
                            before,
                            self.deck.land_count,
                            Crossing::Up,
                        ),
                    );
                }
                if self.deck.budget.is_some_and(|b| b > 0.0) {
                    let before = self.deck.total_price;
                    self.deck.total_price += deck_signals::price_of(&card, self.deck.currency);
                    signal(
                        cx,
                        deck_signals::budget_crossing(
                            self.deck.budget,
                            self.deck.currency,
                            before,
                            self.deck.total_price,
                        ),
                    );
                }
                self.advance_after_commit(cx);
            }
            Source::Maybeboard => {
                let Some(card) = self.mb_stack.current_wrapping().cloned() else {
                    return;
                };
                self.mb_stack.record(MaybeboardAction::Promote {
                    card: Box::new(card.clone()),
                });
                self.exit(Direction::Right, card.clone());
                let id = card.scryfall_data.id;
                self.deck
                    .maybeboard
                    .retain(|e| e.card.scryfall_data.id != id);
                self.mb_stack.remove_current();
                self.deck.in_deck.extend(card.scryfall_data.oracle_id);
                cx.request(
                    Request::Promote {
                        deck_id: self.deck.id,
                        card: Box::new(card),
                    }
                    .into(),
                );
                cx.toast_for(ToastKind::Success, "Moved to deck".to_owned(), toast::QUICK);
            }
        }
        self.after_swipe(cx);
    }

    /// Swipes the top card left: skips it for a week on Search, cycles past on the maybeboard.
    pub(crate) fn swipe_left(&mut self, cx: &mut Context<'_>) {
        match self.source {
            Source::Search => {
                let Some(card) = self.stack.current().cloned() else {
                    return;
                };
                if let Some(oracle_id) = card.scryfall_data.oracle_id {
                    cx.request(
                        Request::Skip {
                            deck_id: self.deck.id,
                            oracle_id,
                        }
                        .into(),
                    );
                }
                self.stack.record(AddAction::Skip);
                self.exit(Direction::Left, card);
                cx.toast_for(ToastKind::Info, "Skipped".to_owned(), toast::QUICK);
                self.advance_after_commit(cx);
            }
            Source::Maybeboard => {
                let Some(card) = self.mb_stack.current_wrapping().cloned() else {
                    return;
                };
                self.mb_stack.record(MaybeboardAction::Skip);
                self.exit(Direction::Left, card);
                cx.toast_for(ToastKind::Info, "Skipped".to_owned(), toast::QUICK);
                self.mb_stack.advance_wrapping();
            }
        }
        self.after_swipe(cx);
    }

    /// Swipes the top card up: to the maybeboard on Search; on the maybeboard it is already there.
    pub(crate) fn swipe_up(&mut self, cx: &mut Context<'_>) {
        match self.source {
            Source::Search => {
                let Some(card) = self.stack.current().cloned() else {
                    return;
                };
                self.stack.record(AddAction::Maybe);
                self.exit(Direction::Up, card.clone());
                self.create(cx, &card, Board::Maybeboard);
                cx.toast_for(
                    ToastKind::Info,
                    "Added to maybeboard".to_owned(),
                    toast::QUICK,
                );
                self.advance_after_commit(cx);
            }
            Source::Maybeboard => {
                if self.mb_stack.current_wrapping().is_some() {
                    cx.toast_for(
                        ToastKind::Info,
                        "Already on maybeboard".to_owned(),
                        toast::QUICK,
                    );
                }
            }
        }
        self.after_swipe(cx);
    }

    /// Sends a create for one copy, marking it in flight so an early undo waits for it.
    fn create(&mut self, cx: &mut Context<'_>, card: &Card, board: Board) {
        cx.shared.cards.in_flight.begin(card.scryfall_data.id);
        cx.request(
            Request::Create {
                deck_id: self.deck.id,
                card: Box::new(card.clone()),
                board,
            }
            .into(),
        );
    }

    /// Undoes the last swipe of the source in use.
    pub(crate) fn undo(&mut self, cx: &mut Context<'_>) {
        let entering = match self.source {
            Source::Search => {
                self.undo_search(cx);
                self.stack.entering()
            }
            Source::Maybeboard => {
                self.undo_maybeboard(cx);
                self.mb_stack.entering()
            }
        };
        if let Some(direction) = entering {
            self.motion = Some(CardMotion::enter(direction, Instant::now()));
        }
        self.after_swipe(cx);
    }

    fn undo_search(&mut self, cx: &mut Context<'_>) {
        let Some(action) = self.stack.pop_action() else {
            cx.toast(ToastKind::Info, "Nothing to undo".to_owned());
            return;
        };
        if !self.stack.step_back(&action) {
            cx.toast(ToastKind::Warning, "No previous card".to_owned());
            self.stack.record(action);
            return;
        }
        let Some(card) = self.stack.current().cloned() else {
            self.stack.unwind_undo(action);
            return;
        };
        let deck_id = self.deck.id;
        match action {
            AddAction::Skip => {
                if let Some(oracle_id) = card.scryfall_data.oracle_id {
                    cx.request(Request::Unskip { deck_id, oracle_id }.into());
                }
                cx.toast_for(ToastKind::Info, "Undid skip".to_owned(), toast::QUICK);
            }
            AddAction::Add | AddAction::Maybe => {
                let board = if action == AddAction::Add {
                    Board::Deck
                } else {
                    Board::Maybeboard
                };
                let toast = undone_message(board);
                let card_id = card.scryfall_data.id;
                if cx.shared.cards.in_flight.undo(card_id) {
                    if board == Board::Deck {
                        self.rewind_counts(&card);
                    }
                    cx.toast_for(ToastKind::Success, toast.to_owned(), toast::QUICK);
                    return;
                }
                let taken = cx.shared.cards.undo.take_newest(
                    deck_id,
                    |a| matches!(a, UndoAction::Added { card_id: id, .. } if *id == card_id),
                );
                if taken.is_none() && !self.deck.holds(&card) {
                    cx.toast_for(ToastKind::Success, toast.to_owned(), toast::QUICK);
                    return;
                }
                cx.request(
                    Request::UndoAdd {
                        deck_id,
                        card: Box::new(card),
                        board,
                        taken: taken.map(Box::new),
                    }
                    .into(),
                );
            }
        }
    }

    fn undo_maybeboard(&mut self, cx: &mut Context<'_>) {
        let Some(action) = self.mb_stack.pop_action() else {
            cx.toast(ToastKind::Info, "Nothing to undo".to_owned());
            return;
        };
        self.mb_stack.prime_entering(action.exited());
        match action {
            MaybeboardAction::Skip => {
                if !self.mb_stack.retreat_wrapping() {
                    self.mb_stack.cancel_entering();
                    return;
                }
                cx.toast_for(ToastKind::Info, "Undid skip".to_owned(), toast::QUICK);
            }
            MaybeboardAction::Promote { card } => {
                let deck_id = self.deck.id;
                let card_id = card.scryfall_data.id;
                let taken = cx.shared.cards.undo.take_newest(deck_id, |a| {
                    matches!(a, UndoAction::MovedBoard { card_id: id, from: Board::Maybeboard, .. } if *id == card_id)
                });
                let already_back = self
                    .deck
                    .maybeboard
                    .iter()
                    .any(|e| e.card.scryfall_data.id == card_id);
                if taken.is_none() && already_back {
                    self.mb_stack.cancel_entering();
                    cx.toast_for(ToastKind::Info, "Already changed".to_owned(), toast::QUICK);
                    return;
                }
                self.mb_stack.insert_current((*card).clone());
                cx.request(
                    Request::Demote {
                        deck_id,
                        card,
                        taken: taken.map(Box::new),
                    }
                    .into(),
                );
            }
        }
    }

    /// Takes back an undone add's land and price.
    fn rewind_counts(&mut self, card: &Card) {
        if card.scryfall_data.is_land() {
            self.deck.land_count = self.deck.land_count.saturating_sub(1).max(0);
        }
        let price = deck_signals::price_of(card, self.deck.currency);
        self.deck.total_price = (self.deck.total_price - price).max(0.0);
    }

    /// After any swipe or undo: shows the new top card's front and fetches the images around it.
    fn after_swipe(&mut self, cx: &mut Context<'_>) {
        self.face = 0;
        self.announce_related(cx);
        self.sync_images(cx);
    }

    /// Switches between Search and Maybeboard, each keeping its own filter.
    pub(crate) fn switch_source(&mut self, cx: &mut Context<'_>, to: Source) {
        if self.source == to {
            return;
        }
        let stores = &mut cx.shared.cards.filters;
        stores.park(self.source.scope(), self.deck.id, self.filter.clone());
        self.filter = match to {
            Source::Maybeboard => stores
                .restore(FilterScope::AddMaybeboard, self.deck.id)
                .unwrap_or_default(),
            Source::Search => stores
                .restore(FilterScope::AddSearch, self.deck.id)
                .unwrap_or_else(|| self.deck.context().fresh_search()),
        };
        self.source = to;
        self.motion = None;
        self.mb_stack.rewind();
        self.rerun(cx);
    }

    /// Flips Synergy and searches again at once.
    pub(crate) fn toggle_synergy(&mut self, cx: &mut Context<'_>) {
        if !self.synergy_shown() {
            return;
        }
        let on = self.filter.synergy();
        self.filter.set_synergy(!on);
        self.rerun(cx);
    }

    /// Opens the filter sheet for the source in use.
    pub(crate) fn open_filter(&mut self, cx: &mut Context<'_>) {
        let reset = match self.source {
            Source::Search => ResetRule::AddSearch(self.deck.context()),
            Source::Maybeboard => ResetRule::AddMaybeboard,
        };
        let config = SheetConfig {
            show_format: true,
            validate: self.source == Source::Search,
            reset,
            sources: Sources::Catalog,
        };
        self.sheet = Some(FilterSheet::open(cx, &self.filter, config));
    }

    /// Opens the details sheet over the top card, with Printings on Search.
    pub(crate) fn open_details(&mut self) {
        if let Some(card) = self.current().cloned() {
            self.details = Some(DetailsSheet::new(card, self.source == Source::Search));
        }
    }

    /// Leaves: parks the filter and the search stack, then goes back.
    pub(crate) fn leave(&mut self, cx: &mut Context<'_>) {
        self.park(cx);
        cx.back();
    }

    /// Parks the filter under its source and the search stack under the deck.
    pub(crate) fn park(&self, cx: &mut Context<'_>) {
        let stores = &mut cx.shared.cards;
        stores
            .filters
            .park(self.source.scope(), self.deck.id, self.filter.clone());
        if let Some(filter) = &self.last_search_filter
            && !self.stack.is_empty()
        {
            let (cards, index, history) = self.stack.park_state();
            stores.stacks.park(
                self.deck.id,
                ParkedStack {
                    cards,
                    index,
                    history,
                    filter: filter.clone(),
                    exhausted: self.exhausted,
                },
            );
        }
    }

    /// The images to have: the top card's face and the next few cards' fronts. Does nothing when card images are off.
    pub(crate) fn sync_images(&mut self, cx: &mut Context<'_>) {
        if cx.shared.images.is_none() {
            return;
        }
        let mut wanted: Vec<String> = Vec::new();
        if let Some(card) = self.current()
            && let Some(url) = card
                .scryfall_data
                .face_image_url(self.face, ImageSize::Normal)
        {
            wanted.push(url.to_owned());
        }
        let ahead: Vec<&Card> = match self.source {
            Source::Search => self
                .stack
                .window()
                .iter()
                .skip(1)
                .take(IMAGES_AHEAD)
                .collect(),
            Source::Maybeboard => Vec::new(),
        };
        wanted.extend(
            ahead
                .into_iter()
                .filter_map(|c| c.scryfall_data.primary_image_url(ImageSize::Normal))
                .map(str::to_owned),
        );
        self.images.retain(|url, _| wanted.contains(url));
        for url in wanted {
            if !self.images.contains_key(&url) {
                self.images.insert(url.clone(), CardImage::Loading);
                cx.request(Request::Image { url }.into());
            }
        }
    }
}

/// The toast for an undone add on `board`.
fn undone_message(board: Board) -> &'static str {
    if board == Board::Deck {
        "Undid add"
    } else {
        "Undid maybeboard"
    }
}

/// Raises a land or budget signal as a toast.
pub(crate) fn signal(cx: &mut Context<'_>, signal: Option<(Level, String)>) {
    if let Some((level, message)) = signal {
        let kind = match level {
            Level::Info => ToastKind::Info,
            Level::Warning => ToastKind::Warning,
        };
        cx.toast(kind, message);
    }
}

/// Which page a search fetches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Page {
    /// A fresh stack, which says when nothing matched.
    First,
    /// Refresh: a fresh stack without the empty-results toast.
    Refresh,
    /// The next page for the same stack.
    More,
}

/// How an undo's reversing call failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UndoFailure {
    /// The session could not be refreshed: the undo is rolled back.
    Session(String),
    /// The call itself failed: "Failed to undo".
    Call(String),
}

/// The Add screen's work.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    Search {
        deck_id: Uuid,
        generation: u64,
        query: Box<CardQuery>,
        /// The filter that built `query`, kept as the stack's filter when it lands.
        filter: Box<CardQueryBuilder>,
        page: Page,
    },
    /// Adds one copy to `board`.
    Create {
        deck_id: Uuid,
        card: Box<Card>,
        board: Board,
    },
    /// Deletes a card whose add was undone while in flight.
    DeleteUndone {
        deck_id: Uuid,
        card_id: Uuid,
    },
    Skip {
        deck_id: Uuid,
        oracle_id: Uuid,
    },
    Unskip {
        deck_id: Uuid,
        oracle_id: Uuid,
    },
    /// Undoes an add or a maybeboard add by deleting the card.
    UndoAdd {
        deck_id: Uuid,
        card: Box<Card>,
        board: Board,
        taken: Option<Box<UndoAction>>,
    },
    /// Moves a maybeboard card to the deck.
    Promote {
        deck_id: Uuid,
        card: Box<Card>,
    },
    /// Moves a promoted card back to the maybeboard.
    Demote {
        deck_id: Uuid,
        card: Box<Card>,
        taken: Option<Box<UndoAction>>,
    },
    ReloadMaybeboard {
        deck_id: Uuid,
    },
    /// Downloads one image.
    Image {
        url: String,
    },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Search { .. } => "search",
            Self::Create { .. } => "create",
            Self::DeleteUndone { .. } => "delete_undone",
            Self::Skip { .. } => "skip",
            Self::Unskip { .. } => "unskip",
            Self::UndoAdd { .. } => "undo_add",
            Self::Promote { .. } => "promote",
            Self::Demote { .. } => "demote",
            Self::ReloadMaybeboard { .. } => "reload_maybeboard",
            Self::Image { .. } => "image",
        }
    }
}

/// The Add screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    Searched {
        deck_id: Uuid,
        generation: u64,
        filter: Box<CardQueryBuilder>,
        page: Page,
        result: Result<HttpDeckCardSearch, String>,
    },
    Created {
        deck_id: Uuid,
        card: Box<Card>,
        board: Board,
        result: Result<Box<DeckCard>, String>,
    },
    UndidAdd {
        deck_id: Uuid,
        card: Box<Card>,
        board: Board,
        taken: Option<Box<UndoAction>>,
        result: Result<(), UndoFailure>,
    },
    /// A promote landed, which the undo log records.
    Promoted { deck_id: Uuid, card: Box<Card> },
    Demoted {
        deck_id: Uuid,
        card: Box<Card>,
        taken: Option<Box<UndoAction>>,
        result: Result<Box<DeckCard>, UndoFailure>,
    },
    MaybeboardReloaded {
        deck_id: Uuid,
        entries: Vec<DeckEntry>,
    },
    Image {
        url: String,
        image: Option<Box<DynamicImage>>,
    },
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Swipe(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Swipe(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut SwipeView> {
    match screen {
        Screen::Swipe(view) => Some(view),
        _ => None,
    }
}

/// Opens Add over the current screen for `deck`: restores the deck's remembered filter or seeds one from the deck, resumes a parked stack made with the same filter, else searches.
pub(crate) fn open(cx: &mut Context<'_>, deck: &Deck) {
    let add_deck = AddDeck::from_deck(deck);
    let deck_id = add_deck.id;
    let remembered = cx
        .shared
        .cards
        .filters
        .restore(FilterScope::AddSearch, deck_id);
    let had_remembered = remembered.is_some();
    let mut filter = remembered.unwrap_or_default();
    add_deck.seed(&mut filter, had_remembered);
    let mut view = SwipeView {
        deck: add_deck,
        filter,
        source: Source::Search,
        stack: CardStack::default(),
        mb_stack: CardStack::default(),
        last_search_filter: None,
        generation: 0,
        loading_cards: false,
        loading_more: false,
        exhausted: false,
        synergy_warming: false,
        related_ids: std::collections::HashSet::new(),
        related_announced: false,
        sheet: None,
        details: None,
        printing: None,
        face: 0,
        images: HashMap::new(),
        show_image: false,
        motion: None,
    };
    let parked = cx.shared.cards.stacks.take(deck_id);
    match parked {
        Some(parked)
            if !parked.cards.is_empty()
                && add::normalized(&parked.filter) == add::normalized(&view.filter) =>
        {
            view.exhausted = parked.exhausted;
            view.last_search_filter = Some(parked.filter);
            view.stack
                .restore(parked.cards, parked.index, parked.history);
            view.sync_images(cx);
        }
        _ => view.search(cx),
    }
    hint::once(cx, HINT_ADD_DECK_CARDS);
    cx.shared.status.clear();
    cx.push(Screen::Swipe(Box::new(view)));
}

/// A client error as its toast words it.
fn message(error: &ClientError) -> String {
    crate::inbound::screens::shown(error)
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let update = match request {
        Request::Search {
            deck_id,
            generation,
            query,
            filter,
            page,
        } => {
            let result = async {
                let session = account.fresh().await?;
                client.search_deck_cards(deck_id, &query, &session).await
            }
            .await;
            return Some(searched_update(deck_id, generation, filter, page, result));
        }
        Request::Create {
            deck_id,
            card,
            board,
        } => {
            let result = create(account, deck_id, &card, board).await;
            Update::Created {
                deck_id,
                card,
                board,
                result,
            }
        }
        Request::DeleteUndone { deck_id, card_id } => {
            let result = async {
                let session = account.fresh().await?;
                client.delete_deck_card(deck_id, card_id, &session).await
            }
            .await;
            return result
                .err()
                .map(|e| screens::Update::Toast(ToastKind::Error, message(&e)));
        }
        Request::Skip { deck_id, oracle_id } => {
            // Quiet, as in the app: a failed skip only means the card may come back.
            async {
                let session = account.fresh().await?;
                client.skip_deck_card(deck_id, oracle_id, &session).await
            }
            .await
            .ok();
            return None;
        }
        Request::Unskip { deck_id, oracle_id } => {
            let result = async {
                let session = account.fresh().await?;
                client.unskip_deck_card(deck_id, oracle_id, &session).await
            }
            .await;
            return result
                .err()
                .map(|e| screens::Update::Toast(ToastKind::Error, message(&e)));
        }
        Request::UndoAdd {
            deck_id,
            card,
            board,
            taken,
        } => {
            let result = undo_add(account, deck_id, &card).await;
            Update::UndidAdd {
                deck_id,
                card,
                board,
                taken,
                result,
            }
        }
        Request::Promote { deck_id, card } => return promote(account, deck_id, card).await,
        Request::Demote {
            deck_id,
            card,
            taken,
        } => {
            let result = demote(account, deck_id, &card).await;
            Update::Demoted {
                deck_id,
                card,
                taken,
                result,
            }
        }
        Request::ReloadMaybeboard { deck_id } => return reload_maybeboard(account, deck_id).await,
        Request::Image { url } => Update::Image {
            image: images::fetch(&client.client, &url).await.ok().map(Box::new),
            url,
        },
    };
    Some(update.into())
}

/// A search answer as its update: a signed-out session ends it, anything else lands on the stack.
fn searched_update(
    deck_id: Uuid,
    generation: u64,
    filter: Box<CardQueryBuilder>,
    page: Page,
    result: Result<HttpDeckCardSearch, ClientError>,
) -> screens::Update {
    if let Err(e @ ClientError::Unauthorized(_)) = &result {
        return screens::failure(e, screens::Update::Status);
    }
    Update::Searched {
        deck_id,
        generation,
        filter,
        page,
        result: result.map_err(|e| message(&e)),
    }
    .into()
}

/// Adds one copy of `card` to `board`.
async fn create(
    account: &Account,
    deck_id: Uuid,
    card: &Card,
    board: Board,
) -> Result<Box<DeckCard>, String> {
    let request = HttpCreateDeckCard::new(
        &card.scryfall_data,
        1,
        crate::domain::stores::undo::create_board_field(board),
    );
    let result = async {
        let session = account.fresh().await?;
        account
            .client
            .create_deck_card(deck_id, &request, &session)
            .await
    }
    .await;
    result.map(Box::new).map_err(|e| message(&e))
}

/// Moves a maybeboard `card` to the deck, or toasts the failure.
async fn promote(account: &Account, deck_id: Uuid, card: Box<Card>) -> Option<screens::Update> {
    let patch = HttpPatchDeckCard::new(None, Some("deck".to_owned()));
    let result = async {
        let session = account.fresh().await?;
        account
            .client
            .update_deck_card(deck_id, card.scryfall_data.id, &patch, &session)
            .await
    }
    .await;
    if let Err(e) = &result {
        return Some(screens::Update::Toast(ToastKind::Error, message(e)));
    }
    Some(Update::Promoted { deck_id, card }.into())
}

/// Deletes `card` from the deck, undoing its add.
async fn undo_add(account: &Account, deck_id: Uuid, card: &Card) -> Result<(), UndoFailure> {
    let session = account
        .fresh()
        .await
        .map_err(|e| UndoFailure::Session(message(&e)))?;
    account
        .client
        .delete_deck_card(deck_id, card.scryfall_data.id, &session)
        .await
        .map_err(|e| UndoFailure::Call(e.to_string()))
}

/// Moves a promoted `card` back to the maybeboard.
async fn demote(
    account: &Account,
    deck_id: Uuid,
    card: &Card,
) -> Result<Box<DeckCard>, UndoFailure> {
    let patch = HttpPatchDeckCard::new(None, Some("maybeboard".to_owned()));
    let session = account
        .fresh()
        .await
        .map_err(|e| UndoFailure::Session(message(&e)))?;
    account
        .client
        .update_deck_card(deck_id, card.scryfall_data.id, &patch, &session)
        .await
        .map(Box::new)
        .map_err(|e| UndoFailure::Call(e.to_string()))
}

/// Fetches the deck again and keeps its maybeboard, or toasts the failure.
async fn reload_maybeboard(account: &Account, deck_id: Uuid) -> Option<screens::Update> {
    let result = async {
        let session = account.fresh().await?;
        account.client.fetch_deck(deck_id, &session).await
    }
    .await;
    match result {
        Ok(deck) => Some(
            Update::MaybeboardReloaded {
                deck_id,
                entries: deck
                    .entries
                    .into_iter()
                    .filter(|e| e.deck_card.board.is_maybeboard())
                    .collect(),
            }
            .into(),
        ),
        Err(e) => Some(screens::Update::Toast(ToastKind::Error, message(&e))),
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    // Adds and promotes settle in the shared stores whether or not the screen is still open.
    let update = match update {
        Update::Created {
            deck_id,
            card,
            board,
            result,
        } => {
            created(nav, cx, deck_id, &card, board, result);
            return;
        }
        Update::Promoted { deck_id, card } => {
            cx.shared.cards.undo.push(
                deck_id,
                UndoAction::MovedBoard {
                    card_id: card.scryfall_data.id,
                    card_name: card.scryfall_data.name.clone(),
                    from: Board::Maybeboard,
                },
            );
            return;
        }
        other => other,
    };
    let Some(view) = nav.find_mut(pick) else {
        if let Update::UndidAdd {
            deck_id,
            taken: Some(taken),
            result: Err(_),
            ..
        }
        | Update::Demoted {
            deck_id,
            taken: Some(taken),
            result: Err(_),
            ..
        } = update
        {
            cx.shared.cards.undo.push(deck_id, *taken);
        }
        return;
    };
    match update {
        Update::Searched {
            deck_id,
            generation,
            filter,
            page,
            result,
        } => {
            if deck_id != view.deck.id || generation != view.generation {
                return;
            }
            searched(view, cx, *filter, page, result);
        }
        Update::Created { .. } | Update::Promoted { .. } => {}
        Update::UndidAdd {
            deck_id,
            card,
            board,
            taken,
            result,
        } => {
            if deck_id != view.deck.id {
                return;
            }
            undid_add(view, cx, deck_id, &card, board, taken, result);
        }
        Update::Demoted {
            deck_id,
            card,
            taken,
            result,
        } => demoted(view, cx, deck_id, *card, taken, result),
        Update::MaybeboardReloaded { deck_id, entries } => {
            if deck_id != view.deck.id {
                return;
            }
            view.deck.maybeboard = entries;
            view.mb_stack.rewind();
            if view.source == Source::Maybeboard {
                view.refilter();
                view.sync_images(cx);
            }
        }
        Update::Image { url, image } => {
            if let Some(picker) = &cx.shared.images
                && let Some(slot @ CardImage::Loading) = view.images.get_mut(&url)
            {
                *slot = match image {
                    Some(image) => {
                        CardImage::Ready(Box::new(RefCell::new(picker.new_resize_protocol(*image))))
                    }
                    None => CardImage::Failed,
                };
            }
        }
    }
}

/// Settles an undone add on the open screen: the card leaves the deck, or the failure toasts and the log gets its entry back.
fn undid_add(
    view: &mut SwipeView,
    cx: &mut Context<'_>,
    deck_id: Uuid,
    card: &Card,
    board: Board,
    taken: Option<Box<UndoAction>>,
    result: Result<(), UndoFailure>,
) {
    match result {
        Ok(()) => {
            if let Some(oracle_id) = card.scryfall_data.oracle_id {
                view.deck.in_deck.remove(&oracle_id);
            }
            if board == Board::Deck {
                view.rewind_counts(card);
            } else {
                let id = card.scryfall_data.id;
                view.deck
                    .maybeboard
                    .retain(|e| e.card.scryfall_data.id != id);
            }
            cx.toast_for(
                ToastKind::Success,
                undone_message(board).to_owned(),
                toast::QUICK,
            );
        }
        Err(failure) => {
            if let UndoFailure::Session(m) = &failure {
                cx.toast(ToastKind::Error, m.clone());
                let action = if board == Board::Deck {
                    AddAction::Add
                } else {
                    AddAction::Maybe
                };
                view.stack.unwind_undo(action);
            }
            if let UndoFailure::Call(e) = &failure {
                cx.toast(ToastKind::Error, format!("Failed to undo: {e}"));
            }
            if let Some(taken) = taken {
                cx.shared.cards.undo.push(deck_id, *taken);
            }
        }
    }
}

/// Settles a demote on the open screen: the card rejoins the maybeboard, or the failure toasts and the log gets its entry back.
fn demoted(
    view: &mut SwipeView,
    cx: &mut Context<'_>,
    deck_id: Uuid,
    card: Card,
    taken: Option<Box<UndoAction>>,
    result: Result<Box<DeckCard>, UndoFailure>,
) {
    match result {
        Ok(deck_card) => {
            if let Some(oracle_id) = card.scryfall_data.oracle_id {
                view.deck.in_deck.remove(&oracle_id);
            }
            let mut deck_card = *deck_card;
            deck_card.board = Board::Maybeboard;
            deck_card.mvp_at = None;
            view.deck.maybeboard.push(DeckEntry { card, deck_card });
            cx.toast_for(
                ToastKind::Success,
                "Undid move to deck".to_owned(),
                toast::QUICK,
            );
        }
        Err(failure) => {
            match failure {
                UndoFailure::Session(m) => {
                    cx.toast(ToastKind::Error, m);
                    view.mb_stack.cancel_entering();
                }
                UndoFailure::Call(e) => {
                    cx.toast(ToastKind::Error, format!("Failed to undo: {e}"));
                }
            }
            if let Some(taken) = taken {
                cx.shared.cards.undo.push(deck_id, *taken);
            }
        }
    }
}

/// A search answer for the current stack.
fn searched(
    view: &mut SwipeView,
    cx: &mut Context<'_>,
    filter: CardQueryBuilder,
    page: Page,
    result: Result<HttpDeckCardSearch, String>,
) {
    let search = match result {
        Ok(search) => search,
        Err(message) => {
            view.loading_cards = false;
            view.loading_more = false;
            cx.toast(ToastKind::Error, message);
            return;
        }
    };
    view.set_warming(!search.synergy_applied, cx);
    let related = add::related_ids(&search);
    match page {
        Page::First | Page::Refresh => {
            view.related_ids = related;
            view.related_announced = false;
            let served: Vec<Card> = search
                .cards
                .into_iter()
                .filter(|c| !view.deck.holds(c))
                .collect();
            if served.is_empty() && page == Page::First {
                cx.toast(
                    ToastKind::Warning,
                    add::no_results_message(filter.synergy(), view.synergy_warming),
                );
            }
            view.stack.replace(served);
            view.last_search_filter = Some(filter);
            view.loading_cards = false;
        }
        Page::More => {
            view.related_ids.extend(related);
            let held: std::collections::HashSet<Uuid> = view
                .stack
                .cards()
                .iter()
                .map(|c| c.card_profile.scryfall_data_id)
                .collect();
            let fresh: Vec<Card> = search
                .cards
                .into_iter()
                .filter(|c| !held.contains(&c.card_profile.scryfall_data_id) && !view.deck.holds(c))
                .collect();
            if fresh.is_empty() {
                view.exhausted = true;
            } else {
                view.stack.append(fresh);
            }
            view.loading_more = false;
        }
    }
    view.announce_related(cx);
    view.sync_images(cx);
}

/// An add's answer: deletes it if it was undone in flight, else records it in the undo log and the open screen.
fn created(
    nav: &mut Nav,
    cx: &mut Context<'_>,
    deck_id: Uuid,
    card: &Card,
    board: Board,
    result: Result<Box<DeckCard>, String>,
) {
    let card_id = card.scryfall_data.id;
    let outcome = cx.shared.cards.in_flight.finish(card_id);
    let deck_card = match result {
        Ok(deck_card) => deck_card,
        Err(message) => {
            cx.toast(ToastKind::Error, message);
            return;
        }
    };
    if outcome == AddOutcome::Undo {
        cx.request(Request::DeleteUndone { deck_id, card_id }.into());
        return;
    }
    cx.shared.cards.undo.push(
        deck_id,
        UndoAction::Added {
            card_id,
            card_name: card.scryfall_data.name.clone(),
        },
    );
    if let Some(view) = nav.find_mut(pick)
        && view.deck.id == deck_id
    {
        view.deck.in_deck.extend(card.scryfall_data.oracle_id);
        if board == Board::Maybeboard {
            view.deck.maybeboard.push(DeckEntry {
                card: card.clone(),
                deck_card: *deck_card,
            });
        }
    }
}
