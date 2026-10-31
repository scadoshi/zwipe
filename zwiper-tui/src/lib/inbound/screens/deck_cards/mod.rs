//! The deck cards view: every card in a deck grouped and filtered, with quick add, row actions (quantity, board, printing, MVP), the deck's undo log, and the doors to Add and Remove.
//!
//! Quantity taps apply at once and gather per card for 300 ms before one absolute write goes out; leaving sends every gathered write.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::{
    Board, Deck, DeckCard, DeckEntry, DeckProfile, Quantity, deck_metrics,
};
use zwipe_core::domain::user::hints::HINT_DECK_CARDS;
use zwipe_core::http::contracts::deck::HttpUpdateDeckProfile;
use zwipe_core::http::contracts::deck_card::{HttpCreateDeckCard, HttpPatchDeckCard};
use zwipe_core::http::helpers::Patch;

use super::card_sheets::filter::{FilterSheet, SheetConfig};
use super::card_sheets::printing::PrintingSheet;
use super::{Screen, remove_cards, swipe};
use crate::domain::deck_cards::{self, Boards, Burst, Bursts, Display, Write, Zone};
use crate::domain::deck_signals::{self, Crossing};
use crate::domain::filter::picker::Sources;
use crate::domain::stores::filters::FilterScope;
use crate::domain::stores::undo::{self, CommandZoneSlot, UndoAction};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::{self, ToastKind};
use crate::outbound::account::Account;

/// Quick add waits this long after the last keystroke before it searches.
const QUICK_ADD_DEBOUNCE: Duration = Duration::from_millis(300);

/// Quick add searches once the query is this long.
const QUICK_ADD_MIN_CHARS: usize = 2;

/// Quick add's result count.
const QUICK_ADD_LIMIT: u32 = 5;

/// What a card row is, which sets its actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// A main board row in a group or Lands: every action and the star.
    Main,
    /// A maybeboard or sideboard row.
    Board(Board),
    /// A command zone row: Printing only.
    Zone(CommandZoneSlot),
    /// A token: no actions.
    Token,
}

/// One line of the list.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Item {
    Header {
        key: String,
        title: String,
        pips: Option<Vec<zwipe_core::domain::card::scryfall_data::colors::Color>>,
        collapsible: bool,
    },
    Card {
        card: Box<Card>,
        kind: Kind,
    },
}

/// Quick add's box and results.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct QuickAdd {
    pub(crate) input: TextInput,
    pub(crate) focused: bool,
    pub(crate) generation: u64,
    pub(crate) results: Dropdown,
    pub(crate) highlight: usize,
}

/// Quick add's dropdown.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) enum Dropdown {
    #[default]
    Closed,
    Searching,
    Results(Vec<Card>),
}

/// Where a printing sheet's save goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrintingTarget {
    Deck,
    Zone(CommandZoneSlot),
}

/// The deck cards view.
#[derive(Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent toggles the app keeps as separate signals"
)]
pub(crate) struct DeckCardsView {
    pub(crate) deck_id: Uuid,
    pub(crate) loaded: bool,
    pub(crate) profile: Option<DeckProfile>,
    /// Every card on every board, the command zone left out.
    pub(crate) entries: Vec<DeckEntry>,
    pub(crate) command_zone_cards: Vec<Card>,
    pub(crate) zone: Zone,
    pub(crate) tokens: Option<Vec<Card>>,
    pub(crate) filter: CardQueryBuilder,
    pub(crate) group_by: GroupByOption,
    pub(crate) boards: Boards,
    pub(crate) show_tokens: bool,
    pub(crate) show_zone: bool,
    pub(crate) collapsed: HashSet<String>,
    /// The open row, by printing ID.
    pub(crate) expanded: Option<Uuid>,
    /// The open row's oracle tag whose definition shows, by its place in [`card_info::oracle_tags`](crate::inbound::widgets::card_info::oracle_tags).
    pub(crate) open_tag: Option<usize>,
    /// The face the open row shows.
    pub(crate) face: usize,
    pub(crate) display: Display,
    /// The selected item.
    pub(crate) cursor: usize,
    pub(crate) bursts: Bursts,
    /// Rows a burst deleted, kept until its write lands.
    pub(crate) removed_rows: HashMap<Uuid, DeckEntry>,
    pub(crate) quick: QuickAdd,
    pub(crate) sheet: Option<FilterSheet>,
    pub(crate) printing: Option<PrintingSheet>,
    pub(crate) printing_target: PrintingTarget,
    /// The image preview: a card and its face.
    pub(crate) preview: Option<(Box<Card>, usize)>,
    hint_seen: bool,
}

impl DeckCardsView {
    fn new(cx: &Context<'_>, deck_id: Uuid) -> Self {
        Self {
            deck_id,
            loaded: false,
            profile: None,
            entries: Vec::new(),
            command_zone_cards: Vec::new(),
            zone: Zone::default(),
            tokens: None,
            filter: cx
                .shared
                .cards
                .filters
                .restore(FilterScope::Cards, deck_id)
                .unwrap_or_default(),
            group_by: GroupByOption::CardType,
            boards: Boards::default(),
            show_tokens: false,
            show_zone: true,
            collapsed: HashSet::new(),
            expanded: None,
            open_tag: None,
            face: 0,
            display: Display::default(),
            cursor: 0,
            bursts: Bursts::default(),
            removed_rows: HashMap::new(),
            quick: QuickAdd::default(),
            sheet: None,
            printing: None,
            printing_target: PrintingTarget::Deck,
            preview: None,
            hint_seen: false,
        }
    }

    /// Fetches the deck again, as when Add or Remove closes over the view.
    pub(crate) fn reload(&mut self, cx: &mut Context<'_>) {
        cx.request(
            Request::Load {
                deck_id: self.deck_id,
            }
            .into(),
        );
    }

    /// The deck's currency for prices and the budget.
    pub(crate) fn currency(&self) -> PriceCurrency {
        self.profile
            .as_ref()
            .and_then(|p| p.price_target_currency)
            .unwrap_or(PriceCurrency::Usd)
    }

    fn land_target(&self) -> Option<i32> {
        self.profile.as_ref().and_then(|p| p.land_target)
    }

    fn budget(&self) -> Option<f64> {
        self.profile.as_ref().and_then(|p| p.price_target)
    }

    pub(crate) fn is_oathbreaker(&self) -> bool {
        self.profile
            .as_ref()
            .and_then(|p| p.format)
            .is_some_and(|f| f.has_signature_spell())
    }

    /// Reruns the filter and group pass, keeping the cursor in range.
    pub(crate) fn rerun(&mut self) {
        self.display = deck_cards::pass(&self.entries, &self.zone, &self.filter, self.group_by);
        self.cursor = self.cursor.min(self.items().len().saturating_sub(1));
    }

    /// Reruns the pass and closes the open row, as a filter or group change does.
    pub(crate) fn rerun_collapsed(&mut self) {
        self.expanded = None;
        self.rerun();
    }

    /// The entry for a printing.
    pub(crate) fn entry(&self, card_id: Uuid) -> Option<&DeckEntry> {
        self.entries
            .iter()
            .find(|e| e.deck_card.scryfall_data_id == card_id)
    }

    fn entry_mut(&mut self, card_id: Uuid) -> Option<&mut DeckEntry> {
        self.entries
            .iter_mut()
            .find(|e| e.deck_card.scryfall_data_id == card_id)
    }

    /// Copies of a printing, or 1 when it is not an entry.
    pub(crate) fn quantity(&self, card_id: Uuid) -> i32 {
        self.entry(card_id).map_or(1, |e| *e.deck_card.quantity)
    }

    /// The list's lines, top to bottom: tokens, maybeboard, sideboard, command zone, groups, lands.
    pub(crate) fn items(&self) -> Vec<Item> {
        let mut items = Vec::new();
        self.push_tokens(&mut items);
        self.push_side_boards(&mut items);
        if self.show_zone {
            self.push_zone(&mut items);
        }
        if self.boards.main {
            self.push_main(&mut items);
        }
        items
    }

    /// The copies across `cards`, counting each as its entry's quantity.
    fn qty_count(&self, cards: &[Card]) -> i32 {
        cards.iter().fold(0, |n, c| {
            n.saturating_add(self.quantity(c.scryfall_data.id))
        })
    }

    /// A collapsible section: its header, then its cards unless it is collapsed.
    fn push_section(
        &self,
        items: &mut Vec<Item>,
        key: String,
        title: String,
        pips: Option<Vec<zwipe_core::domain::card::scryfall_data::colors::Color>>,
        cards: Vec<(Card, Kind)>,
    ) {
        let collapsed = self.collapsed.contains(&key);
        items.push(Item::Header {
            key,
            title,
            pips,
            collapsible: true,
        });
        if !collapsed {
            items.extend(cards.into_iter().map(|(card, kind)| Item::Card {
                card: Box::new(card),
                kind,
            }));
        }
    }

    /// The Tokens section, sorted like the list, when tokens are shown and there are some.
    fn push_tokens(&self, items: &mut Vec<Item>) {
        if self.show_tokens
            && let Some(tokens) = self.tokens.as_ref().filter(|t| !t.is_empty())
        {
            let mut tokens = tokens.clone();
            let sort = self.filter.sort().unwrap_or(CardSortKey::Name);
            tokens = zwipe_core::domain::card::search_card::cards::Cards::from(tokens)
                .sorted(sort, self.filter.direction())
                .into();
            self.push_section(
                items,
                "Tokens".to_owned(),
                format!("Tokens ({})", tokens.len()),
                None,
                tokens.into_iter().map(|c| (c, Kind::Token)).collect(),
            );
        }
    }

    /// The Maybeboard and Sideboard sections, each when shown and not empty.
    fn push_side_boards(&self, items: &mut Vec<Item>) {
        for (board, on, list, title) in [
            (
                Board::Maybeboard,
                self.boards.maybe,
                &self.display.maybe,
                "Maybeboard",
            ),
            (
                Board::Sideboard,
                self.boards.side,
                &self.display.side,
                "Sideboard",
            ),
        ] {
            if on && !list.is_empty() {
                self.push_section(
                    items,
                    title.to_owned(),
                    format!("{title} ({})", list.len()),
                    None,
                    list.iter()
                        .map(|e| (e.card.clone(), Kind::Board(board)))
                        .collect(),
                );
            }
        }
    }

    /// The command zone: the leaders under one header, then the background and signature spell.
    fn push_zone(&self, items: &mut Vec<Item>) {
        let zone = &self.display.zone;
        let leaders: Vec<&(CommandZoneSlot, Card)> = zone
            .iter()
            .filter(|(s, _)| matches!(s, CommandZoneSlot::Commander | CommandZoneSlot::Partner))
            .collect();
        if !leaders.is_empty() {
            let has_partner = leaders.iter().any(|(s, _)| *s == CommandZoneSlot::Partner);
            let title = if self.is_oathbreaker() {
                "Oathbreaker"
            } else if has_partner {
                "Commanders"
            } else {
                "Commander"
            };
            items.push(zone_header(title));
            items.extend(
                leaders
                    .into_iter()
                    .map(|(slot, card)| zone_item(*slot, card)),
            );
        }
        for (slot, title) in [
            (CommandZoneSlot::Background, "Background"),
            (CommandZoneSlot::SignatureSpell, "Signature spell"),
        ] {
            if let Some((slot, card)) = zone.iter().find(|(s, _)| *s == slot) {
                items.push(zone_header(title));
                items.push(zone_item(*slot, card));
            }
        }
    }

    /// The main board: each group's section, then Lands.
    fn push_main(&self, items: &mut Vec<Item>) {
        for group in &self.display.groups {
            let title = if group.label.is_empty() {
                format!("({})", self.qty_count(&group.cards))
            } else {
                format!("{} ({})", group.label, self.qty_count(&group.cards))
            };
            self.push_section(
                items,
                group.key.clone(),
                title,
                group.pips.clone(),
                group
                    .cards
                    .iter()
                    .map(|c| (c.clone(), Kind::Main))
                    .collect(),
            );
        }
        if !self.display.lands.is_empty() {
            self.push_section(
                items,
                "Lands".to_owned(),
                format!("Lands ({})", self.qty_count(&self.display.lands)),
                None,
                self.display
                    .lands
                    .iter()
                    .map(|c| (c.clone(), Kind::Main))
                    .collect(),
            );
        }
    }

    pub(crate) fn selected(&self) -> Option<Item> {
        self.items().into_iter().nth(self.cursor)
    }

    /// The selected card row, if open.
    pub(crate) fn open_row(&self) -> Option<(Card, Kind)> {
        match self.selected()? {
            Item::Card { card, kind } if Some(card.scryfall_data.id) == self.expanded => {
                Some((*card, kind))
            }
            _ => None,
        }
    }

    /// The deck as Add and Remove take it.
    pub(crate) fn deck(&self) -> Option<Deck> {
        Some(Deck {
            deck_profile: self.profile.clone()?,
            entries: self.entries.clone(),
            warnings: Vec::new(),
            command_zone_cards: self.command_zone_cards.clone(),
        })
    }

    /// Opens the filter sheet over the deck's cards.
    pub(crate) fn open_filter(&mut self, cx: &mut Context<'_>) {
        let cards: Vec<Card> = self.entries.iter().map(|e| e.card.clone()).collect();
        let config = SheetConfig {
            show_format: true,
            sources: Sources::deck(&cards),
            ..SheetConfig::default()
        };
        self.sheet = Some(FilterSheet::open(cx, &self.filter, config));
    }

    /// Reset filter: clears it at once, no sheet.
    pub(crate) fn reset_filter(&mut self, cx: &mut Context<'_>) {
        self.filter.clear();
        self.rerun_collapsed();
        cx.toast_for(ToastKind::Info, "Filter reset".to_owned(), toast::QUICK);
    }

    /// Opens the printing sheet over a row.
    pub(crate) fn open_printing(
        &mut self,
        cx: &mut Context<'_>,
        card: Card,
        target: PrintingTarget,
    ) {
        self.printing_target = target;
        self.printing = Some(PrintingSheet::open(
            cx,
            card,
            /* read_only */ false,
            "Printing saved",
        ));
    }

    fn signals(&self, cx: &mut Context<'_>, lands_before: i32, price_before: f64) {
        swipe::signal(
            cx,
            deck_signals::land_crossing(
                self.land_target(),
                lands_before,
                deck_signals::land_count(&self.entries),
                Crossing::Both,
            ),
        );
        swipe::signal(
            cx,
            deck_signals::budget_crossing(
                self.budget(),
                self.currency(),
                price_before,
                self.price(),
            ),
        );
    }

    fn price(&self) -> f64 {
        deck_metrics::deck_price(&self.entries, &self.command_zone_cards, self.currency())
    }

    /// One quantity tap: applies at once, and gathers into the card's burst for one write 300 ms after the last tap. Below one, the row is removed. `record` logs the change for the deck's Undo.
    pub(crate) fn change_quantity(
        &mut self,
        cx: &mut Context<'_>,
        card_id: Uuid,
        delta: i32,
        record: bool,
    ) {
        let Some(entry) = self.entry(card_id).cloned() else {
            return;
        };
        let current = *entry.deck_card.quantity;
        let (lands, price) = (deck_signals::land_count(&self.entries), self.price());
        let next = current.saturating_add(delta);
        if next < 1 {
            if record {
                let baseline = self.bursts.baseline(card_id).unwrap_or(current);
                cx.shared.cards.undo.push(
                    self.deck_id,
                    UndoAction::Removed {
                        entry: Box::new(entry.clone()),
                        baseline,
                    },
                );
            }
            self.removed_rows.insert(card_id, entry);
            self.entries
                .retain(|e| e.deck_card.scryfall_data_id != card_id);
            self.rerun();
            self.signals(cx, lands, price);
            cx.toast_for(ToastKind::Info, "Card removed".to_owned(), toast::QUICK);
        } else {
            if let (Some(entry), Ok(quantity)) = (self.entry_mut(card_id), Quantity::new(next)) {
                entry.deck_card.quantity = quantity;
            }
            self.rerun();
            self.signals(cx, lands, price);
        }
        let generation = self.bursts.tap(card_id, current, delta, record);
        cx.request(
            Request::FlushTimer {
                deck_id: self.deck_id,
                card_id,
                generation,
            }
            .into(),
        );
    }

    /// Sends a due burst's write.
    fn flush(&mut self, cx: &mut Context<'_>, card_id: Uuid, burst: Burst) {
        let removed = self.removed_rows.remove(&card_id);
        let write = burst.write();
        if write == Write::Nothing {
            return;
        }
        let name = self
            .entry(card_id)
            .or(removed.as_ref())
            .map(|e| e.card.scryfall_data.name.clone())
            .unwrap_or_default();
        cx.request(
            Request::WriteQuantity {
                deck_id: self.deck_id,
                card_id,
                card_name: name,
                burst,
                removed: removed.map(Box::new),
            }
            .into(),
        );
    }

    /// Sends every open burst now, as leaving does.
    pub(crate) fn flush_all(&mut self, cx: &mut Context<'_>) {
        for (card_id, burst) in self.bursts.drain() {
            self.flush(cx, card_id, burst);
        }
    }

    /// Moves a card to another board at once, putting it back if the write fails. `record` logs the move for the deck's Undo.
    pub(crate) fn move_to_board(
        &mut self,
        cx: &mut Context<'_>,
        card_id: Uuid,
        target: Board,
        record: bool,
    ) {
        let Some(entry) = self.entry_mut(card_id) else {
            return;
        };
        let from = entry.deck_card.board;
        let name = entry.card.scryfall_data.name.clone();
        entry.deck_card.board = target;
        if record {
            cx.shared.cards.undo.push(
                self.deck_id,
                UndoAction::MovedBoard {
                    card_id,
                    card_name: name,
                    from,
                },
            );
        }
        self.rerun();
        cx.request(
            Request::Move {
                deck_id: self.deck_id,
                card_id,
                from,
                target,
            }
            .into(),
        );
        if record {
            cx.toast_for(
                ToastKind::Info,
                format!("Moved to {}", target.display_name()),
                toast::QUICK,
            );
        }
    }

    /// Stars or unstars a main board card at once; the server's answer sets the star's time.
    pub(crate) fn toggle_mvp(&mut self, cx: &mut Context<'_>, card_id: Uuid) {
        let Some(entry) = self.entry_mut(card_id) else {
            return;
        };
        let old = entry.deck_card.mvp_at;
        let star = old.is_none();
        entry.deck_card.mvp_at = star.then(Utc::now);
        self.rerun();
        cx.request(
            Request::Mvp {
                deck_id: self.deck_id,
                card_id,
                star,
                old,
            }
            .into(),
        );
    }

    /// The deck's Undo: reverses the newest entry in its log, or says it no longer applies.
    pub(crate) fn undo(&mut self, cx: &mut Context<'_>) {
        let Some(action) = cx.shared.cards.undo.pop(self.deck_id) else {
            return;
        };
        if deck_cards::is_stale(&action, &self.entries, &self.zone) {
            cx.toast_for(ToastKind::Info, "Already changed".to_owned(), toast::QUICK);
            return;
        }
        let message = deck_cards::undo_message(&action);
        let deck_id = self.deck_id;
        match action {
            UndoAction::Added { card_id, .. } => {
                self.entries
                    .retain(|e| e.deck_card.scryfall_data_id != card_id);
                self.rerun();
                cx.request(Request::UndoDelete { deck_id, card_id }.into());
            }
            UndoAction::Removed { entry, baseline } => {
                let mut entry = *entry;
                if let Ok(quantity) = Quantity::new(baseline) {
                    entry.deck_card.quantity = quantity;
                }
                self.entries.push(entry.clone());
                self.rerun();
                cx.request(
                    Request::UndoCreate {
                        deck_id,
                        entry: Box::new(entry),
                        baseline,
                    }
                    .into(),
                );
            }
            UndoAction::QuantityChanged { card_id, net, .. } => {
                self.change_quantity(cx, card_id, net.saturating_neg(), /* record */ false);
            }
            UndoAction::MovedBoard { card_id, from, .. } => {
                self.move_to_board(cx, card_id, from, /* record */ false);
            }
            UndoAction::PrintingChanged { old_card, new_id } => {
                let old_id = old_card.scryfall_data.id;
                if let Some(entry) = self.entry_mut(new_id) {
                    entry.deck_card.scryfall_data_id = old_id;
                    entry.card = *old_card;
                }
                self.rerun();
                cx.request(
                    Request::UndoPrinting {
                        deck_id,
                        new_id,
                        old_id,
                    }
                    .into(),
                );
            }
            UndoAction::CommandZonePrintingChanged { slot, old_card, .. } => {
                *self.zone.slot_mut(slot) = Some((*old_card).clone());
                self.rerun();
                cx.request(
                    Request::ZonePrinting {
                        deck_id,
                        slot,
                        new_card: old_card,
                        old_card: None,
                    }
                    .into(),
                );
            }
        }
        cx.toast_for(ToastKind::Info, message, toast::QUICK);
    }

    /// A quick add keystroke: closes the dropdown under two characters, else shows "Searching..." and searches after a pause.
    pub(crate) fn quick_edit(&mut self, cx: &mut Context<'_>) {
        self.quick.generation = self.quick.generation.wrapping_add(1);
        self.quick.highlight = 0;
        if self.quick.input.value().trim().chars().count() < QUICK_ADD_MIN_CHARS {
            self.quick.results = Dropdown::Closed;
            return;
        }
        self.quick.results = Dropdown::Searching;
        cx.request(
            Request::QuickDebounce {
                deck_id: self.deck_id,
                generation: self.quick.generation,
            }
            .into(),
        );
    }

    /// Adds the highlighted quick add result, dropping it from the list at once.
    pub(crate) fn quick_pick(&mut self, cx: &mut Context<'_>) {
        let Dropdown::Results(results) = &mut self.quick.results else {
            return;
        };
        let at = self.quick.highlight.min(results.len().saturating_sub(1));
        if at >= results.len() {
            return;
        }
        let card = results.remove(at);
        self.quick.highlight = self.quick.highlight.min(results.len().saturating_sub(1));
        cx.request(
            Request::QuickAdd {
                deck_id: self.deck_id,
                card: Box::new(card),
            }
            .into(),
        );
    }

    /// Leaves: sends open bursts, parks the filter, goes back.
    pub(crate) fn leave(&mut self, cx: &mut Context<'_>) {
        self.flush_all(cx);
        cx.shared
            .cards
            .filters
            .park(FilterScope::Cards, self.deck_id, self.filter.clone());
        cx.back();
    }

    /// Opens Add over the view.
    pub(crate) fn open_add(&mut self, cx: &mut Context<'_>) {
        if let Some(deck) = self.deck() {
            self.flush_all(cx);
            swipe::open(cx, &deck);
        }
    }

    /// Opens Remove over the view.
    pub(crate) fn open_remove(&mut self, cx: &mut Context<'_>) {
        if self.loaded {
            self.flush_all(cx);
            remove_cards::open(cx, self.deck_id);
        }
    }
}

fn zone_header(title: &str) -> Item {
    Item::Header {
        key: title.to_owned(),
        title: title.to_owned(),
        pips: None,
        collapsible: false,
    }
}

fn zone_item(slot: CommandZoneSlot, card: &Card) -> Item {
    Item::Card {
        card: Box::new(card.clone()),
        kind: Kind::Zone(slot),
    }
}

/// Opens the deck cards view for `deck_id` over the current screen and loads the deck and its tokens. The deck view's `c` opens it.
pub(crate) fn open(cx: &mut Context<'_>, deck_id: Uuid) {
    let view = DeckCardsView::new(cx, deck_id);
    cx.request(Request::Load { deck_id }.into());
    cx.request(Request::Tokens { deck_id }.into());
    cx.push(Screen::DeckCards(Box::new(view)));
}

/// What the load fetched.
#[derive(Debug, Clone)]
pub(crate) struct Loaded {
    pub(crate) deck: Deck,
    pub(crate) profile: Option<DeckProfile>,
    pub(crate) zone: Zone,
}

/// The view's work.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    Load {
        deck_id: Uuid,
    },
    Tokens {
        deck_id: Uuid,
    },
    /// Waits out a quantity burst.
    FlushTimer {
        deck_id: Uuid,
        card_id: Uuid,
        generation: u64,
    },
    WriteQuantity {
        deck_id: Uuid,
        card_id: Uuid,
        card_name: String,
        burst: Burst,
        removed: Option<Box<DeckEntry>>,
    },
    Move {
        deck_id: Uuid,
        card_id: Uuid,
        from: Board,
        target: Board,
    },
    /// Swaps a deck card's printing.
    Printing {
        deck_id: Uuid,
        old_card: Box<Card>,
        new_card: Box<Card>,
    },
    /// Writes a command zone slot's printing; `old_card` is kept for the undo log, `None` for an undo.
    ZonePrinting {
        deck_id: Uuid,
        slot: CommandZoneSlot,
        new_card: Box<Card>,
        old_card: Option<Box<Card>>,
    },
    Mvp {
        deck_id: Uuid,
        card_id: Uuid,
        star: bool,
        old: Option<DateTime<Utc>>,
    },
    UndoDelete {
        deck_id: Uuid,
        card_id: Uuid,
    },
    UndoCreate {
        deck_id: Uuid,
        entry: Box<DeckEntry>,
        baseline: i32,
    },
    UndoPrinting {
        deck_id: Uuid,
        new_id: Uuid,
        old_id: Uuid,
    },
    QuickDebounce {
        deck_id: Uuid,
        generation: u64,
    },
    QuickSearch {
        deck_id: Uuid,
        generation: u64,
        query: String,
    },
    QuickAdd {
        deck_id: Uuid,
        card: Box<Card>,
    },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Load { .. } => "load",
            Self::Tokens { .. } => "tokens",
            Self::FlushTimer { .. } => "flush_timer",
            Self::WriteQuantity { .. } => "write_quantity",
            Self::Move { .. } => "move",
            Self::Printing { .. } => "printing",
            Self::ZonePrinting { .. } => "zone_printing",
            Self::Mvp { .. } => "mvp",
            Self::UndoDelete { .. } => "undo_delete",
            Self::UndoCreate { .. } => "undo_create",
            Self::UndoPrinting { .. } => "undo_printing",
            Self::QuickDebounce { .. } => "quick_debounce",
            Self::QuickSearch { .. } => "quick_search",
            Self::QuickAdd { .. } => "quick_add",
        }
    }
}

/// The view's results.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded {
        deck_id: Uuid,
        loaded: Box<Loaded>,
    },
    Tokens {
        deck_id: Uuid,
        tokens: Vec<Card>,
    },
    FlushDue {
        deck_id: Uuid,
        card_id: Uuid,
        generation: u64,
    },
    QuantityFailed {
        deck_id: Uuid,
        card_id: Uuid,
        burst: Burst,
        removed: Option<Box<DeckEntry>>,
        message: String,
    },
    QuantityWritten {
        deck_id: Uuid,
        card_id: Uuid,
        card_name: String,
        burst: Burst,
    },
    MoveFailed {
        deck_id: Uuid,
        card_id: Uuid,
        from: Board,
        message: String,
    },
    PrintingSaved {
        deck_id: Uuid,
        old_card: Box<Card>,
        new_card: Box<Card>,
    },
    ZonePrintingSaved {
        deck_id: Uuid,
        slot: CommandZoneSlot,
        old_card: Box<Card>,
        new_card: Box<Card>,
    },
    MvpSet {
        deck_id: Uuid,
        card_id: Uuid,
        /// The server's star time, or on failure the old one to restore.
        mvp_at: Option<DateTime<Utc>>,
        error: Option<String>,
    },
    UndoCreated {
        deck_id: Uuid,
        card_id: Uuid,
        result: Result<Box<DeckCard>, String>,
    },
    QuickDue {
        deck_id: Uuid,
        generation: u64,
    },
    QuickResults {
        deck_id: Uuid,
        generation: u64,
        cards: Option<Vec<Card>>,
    },
    QuickAdded {
        deck_id: Uuid,
        entry: Box<DeckEntry>,
    },
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::DeckCards(request)
    }
}

impl Update {
    /// The deck the result belongs to.
    fn deck_id(&self) -> Uuid {
        match self {
            Self::Loaded { deck_id, .. }
            | Self::Tokens { deck_id, .. }
            | Self::FlushDue { deck_id, .. }
            | Self::QuantityWritten { deck_id, .. }
            | Self::QuantityFailed { deck_id, .. }
            | Self::MoveFailed { deck_id, .. }
            | Self::PrintingSaved { deck_id, .. }
            | Self::ZonePrintingSaved { deck_id, .. }
            | Self::MvpSet { deck_id, .. }
            | Self::UndoCreated { deck_id, .. }
            | Self::QuickDue { deck_id, .. }
            | Self::QuickResults { deck_id, .. }
            | Self::QuickAdded { deck_id, .. } => *deck_id,
        }
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::DeckCards(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut DeckCardsView> {
    match screen {
        Screen::DeckCards(view) => Some(view),
        _ => None,
    }
}

fn error_toast(message: String) -> screens::Update {
    screens::Update::Toast(ToastKind::Error, message)
}

/// A fresh session for a request that calls the server, none for a timer, or the failure's toast.
async fn session_for(
    account: &Account,
    request: &Request,
) -> Result<Option<Session>, screens::Update> {
    match request {
        Request::FlushTimer { .. } | Request::QuickDebounce { .. } => Ok(None),
        _ => match account.fresh().await {
            Ok(session) => Ok(Some(session)),
            Err(e) => Err(screens::failure(&e, |m| {
                screens::Update::Toast(ToastKind::Error, m)
            })),
        },
    }
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let session = match session_for(account, &request).await {
        Ok(session) => session,
        Err(update) => return Some(update),
    };
    let session = session.as_ref();
    match request {
        Request::Load { deck_id } => load(account, session, deck_id).await,
        Request::Tokens { deck_id } => fetch_tokens(account, session, deck_id).await,
        Request::FlushTimer {
            deck_id,
            card_id,
            generation,
        } => Some(flush_timer(deck_id, card_id, generation).await),
        Request::WriteQuantity {
            deck_id,
            card_id,
            card_name,
            burst,
            removed,
        } => {
            write_quantity(
                account, session, deck_id, card_id, card_name, burst, removed,
            )
            .await
        }
        Request::Move {
            deck_id,
            card_id,
            from,
            target,
        } => move_card(account, session, deck_id, card_id, from, target).await,
        Request::Printing {
            deck_id,
            old_card,
            new_card,
        } => save_printing(account, session, deck_id, old_card, new_card).await,
        Request::ZonePrinting {
            deck_id,
            slot,
            new_card,
            old_card,
        } => save_zone_printing(account, session, deck_id, slot, new_card, old_card).await,
        Request::Mvp {
            deck_id,
            card_id,
            star,
            old,
        } => set_mvp(account, session, deck_id, card_id, star, old).await,
        Request::UndoDelete { deck_id, card_id } => client
            .delete_deck_card(deck_id, card_id, session?)
            .await
            .err()
            .map(|e| error_toast(crate::inbound::screens::shown(&e))),
        Request::UndoCreate {
            deck_id,
            entry,
            baseline,
        } => undo_create(account, session, deck_id, &entry, baseline).await,
        Request::UndoPrinting {
            deck_id,
            new_id,
            old_id,
        } => undo_printing(account, session, deck_id, new_id, old_id).await,
        Request::QuickDebounce {
            deck_id,
            generation,
        } => {
            tokio::time::sleep(QUICK_ADD_DEBOUNCE).await;
            Some(
                Update::QuickDue {
                    deck_id,
                    generation,
                }
                .into(),
            )
        }
        Request::QuickSearch {
            deck_id,
            generation,
            query,
        } => quick_search(account, session, deck_id, generation, &query).await,
        Request::QuickAdd { deck_id, card } => quick_add(account, session, deck_id, card).await,
    }
}

/// Fetches the deck's tokens; a failure shows none.
async fn fetch_tokens(
    account: &Account,
    session: Option<&Session>,
    deck_id: Uuid,
) -> Option<screens::Update> {
    Some(
        Update::Tokens {
            deck_id,
            tokens: account
                .client
                .fetch_deck_tokens(deck_id, session?)
                .await
                .unwrap_or_default(),
        }
        .into(),
    )
}

/// Waits out a quantity burst, then says it is due to write.
async fn flush_timer(deck_id: Uuid, card_id: Uuid, generation: u64) -> screens::Update {
    tokio::time::sleep(Duration::from_millis(deck_cards::QTY_FLUSH_MS)).await;
    Update::FlushDue {
        deck_id,
        card_id,
        generation,
    }
    .into()
}

/// Puts an entry's old printing back; only a failure reports back.
async fn undo_printing(
    account: &Account,
    session: Option<&Session>,
    deck_id: Uuid,
    new_id: Uuid,
    old_id: Uuid,
) -> Option<screens::Update> {
    let patch = HttpPatchDeckCard::with_printing(&old_id.to_string());
    account
        .client
        .update_deck_card(deck_id, new_id, &patch, session?)
        .await
        .err()
        .map(|e| error_toast(crate::inbound::screens::shown(&e)))
}

/// Fetches the deck, its profile and its command zone cards.
async fn load(
    account: &Account,
    session: Option<&Session>,
    deck_id: Uuid,
) -> Option<screens::Update> {
    let client = &account.client;
    let session = session?;
    let deck = match client.fetch_deck(deck_id, session).await {
        Ok(deck) => deck,
        Err(e) => return Some(error_toast(crate::inbound::screens::shown(&e))),
    };
    let profile = client.fetch_deck_profile(deck_id, session).await.ok();
    let ids = profile.as_ref().unwrap_or(&deck.deck_profile);
    let mut zone = Zone::default();
    for (slot, id) in [
        (CommandZoneSlot::Commander, ids.commander_id),
        (CommandZoneSlot::SignatureSpell, ids.signature_spell_id),
        (CommandZoneSlot::Partner, ids.partner_commander_id),
        (CommandZoneSlot::Background, ids.background_id),
    ] {
        if let Some(id) = id {
            *zone.slot_mut(slot) = client.fetch_card(id).await.ok();
        }
    }
    Some(
        Update::Loaded {
            deck_id,
            loaded: Box::new(Loaded {
                deck,
                profile,
                zone,
            }),
        }
        .into(),
    )
}

/// Writes a settled quantity burst as a delete, a set, or nothing.
async fn write_quantity(
    account: &Account,
    session: Option<&Session>,
    deck_id: Uuid,
    card_id: Uuid,
    card_name: String,
    burst: Burst,
    removed: Option<Box<DeckEntry>>,
) -> Option<screens::Update> {
    let client = &account.client;
    let result = match burst.write() {
        Write::Nothing => Ok(()),
        Write::Delete => client.delete_deck_card(deck_id, card_id, session?).await,
        Write::Set(quantity) => client
            .update_deck_card(
                deck_id,
                card_id,
                &HttpPatchDeckCard::new(Some(quantity), None),
                session?,
            )
            .await
            .map(drop),
    };
    let update = match result {
        Ok(()) => Update::QuantityWritten {
            deck_id,
            card_id,
            card_name,
            burst,
        },
        Err(e) => Update::QuantityFailed {
            deck_id,
            card_id,
            burst,
            removed,
            message: crate::inbound::screens::shown(&e),
        },
    };
    Some(update.into())
}

/// Moves a card to another board; only a failure reports back.
async fn move_card(
    account: &Account,
    session: Option<&Session>,
    deck_id: Uuid,
    card_id: Uuid,
    from: Board,
    target: Board,
) -> Option<screens::Update> {
    let patch = HttpPatchDeckCard::new(None, Some(target.display_name().to_owned()));
    match account
        .client
        .update_deck_card(deck_id, card_id, &patch, session?)
        .await
    {
        Ok(_) => None,
        Err(e) => Some(
            Update::MoveFailed {
                deck_id,
                card_id,
                from,
                message: crate::inbound::screens::shown(&e),
            }
            .into(),
        ),
    }
}

/// Swaps an entry's printing.
async fn save_printing(
    account: &Account,
    session: Option<&Session>,
    deck_id: Uuid,
    old_card: Box<Card>,
    new_card: Box<Card>,
) -> Option<screens::Update> {
    let patch = HttpPatchDeckCard::with_printing(&new_card.scryfall_data.id.to_string());
    match account
        .client
        .update_deck_card(deck_id, old_card.scryfall_data.id, &patch, session?)
        .await
    {
        Ok(_) => Some(
            Update::PrintingSaved {
                deck_id,
                old_card,
                new_card,
            }
            .into(),
        ),
        Err(e) => Some(error_toast(crate::inbound::screens::shown(&e))),
    }
}

/// Swaps a command zone slot's printing; reports back only when there was an old card.
async fn save_zone_printing(
    account: &Account,
    session: Option<&Session>,
    deck_id: Uuid,
    slot: CommandZoneSlot,
    new_card: Box<Card>,
    old_card: Option<Box<Card>>,
) -> Option<screens::Update> {
    let id = Patch::Set(Some(new_card.scryfall_data.id));
    let builder = HttpUpdateDeckProfile::builder();
    let body = match slot {
        CommandZoneSlot::Commander => builder.commander_id(id),
        CommandZoneSlot::Partner => builder.partner_commander_id(id),
        CommandZoneSlot::Background => builder.background_id(id),
        CommandZoneSlot::SignatureSpell => builder.signature_spell_id(id),
    }
    .build();
    match account
        .client
        .update_deck_profile(deck_id, &body, session?)
        .await
    {
        Ok(_) => old_card.map(|old_card| {
            Update::ZonePrintingSaved {
                deck_id,
                slot,
                old_card,
                new_card,
            }
            .into()
        }),
        Err(e) => Some(error_toast(crate::inbound::screens::shown(&e))),
    }
}

/// Stars or unstars a card, reporting the old star back on failure.
async fn set_mvp(
    account: &Account,
    session: Option<&Session>,
    deck_id: Uuid,
    card_id: Uuid,
    star: bool,
    old: Option<DateTime<Utc>>,
) -> Option<screens::Update> {
    let result = account
        .client
        .update_deck_card(
            deck_id,
            card_id,
            &HttpPatchDeckCard::with_mvp(star),
            session?,
        )
        .await;
    let update = match result {
        Ok(deck_card) => Update::MvpSet {
            deck_id,
            card_id,
            mvp_at: deck_card.mvp_at,
            error: None,
        },
        Err(e) => Update::MvpSet {
            deck_id,
            card_id,
            mvp_at: old,
            error: Some(crate::inbound::screens::shown(&e)),
        },
    };
    Some(update.into())
}

/// Puts a deleted entry back at its baseline quantity.
async fn undo_create(
    account: &Account,
    session: Option<&Session>,
    deck_id: Uuid,
    entry: &DeckEntry,
    baseline: i32,
) -> Option<screens::Update> {
    let request = HttpCreateDeckCard::new(
        &entry.card.scryfall_data,
        baseline,
        undo::create_board_field(entry.deck_card.board),
    );
    let result = account
        .client
        .create_deck_card(deck_id, &request, session?)
        .await;
    Some(
        Update::UndoCreated {
            deck_id,
            card_id: entry.deck_card.scryfall_data_id,
            result: result
                .map(Box::new)
                .map_err(|e| crate::inbound::screens::shown(&e)),
        }
        .into(),
    )
}

/// Searches cards by name for quick add; a failed search reports no cards.
async fn quick_search(
    account: &Account,
    session: Option<&Session>,
    deck_id: Uuid,
    generation: u64,
    query: &str,
) -> Option<screens::Update> {
    let mut filter = CardQueryBuilder::with_name_contains(query.trim());
    filter
        .set_synergy(false)
        .set_include_skipped(true)
        .set_sort(CardSortKey::Name)
        .set_limit(QUICK_ADD_LIMIT);
    let cards = match filter.build() {
        Ok(query) => account
            .client
            .search_deck_cards(deck_id, &query, session?)
            .await
            .ok()
            .map(|s| s.cards),
        Err(_) => None,
    };
    Some(
        Update::QuickResults {
            deck_id,
            generation,
            cards,
        }
        .into(),
    )
}

/// Adds one copy of a card to the main board.
async fn quick_add(
    account: &Account,
    session: Option<&Session>,
    deck_id: Uuid,
    card: Box<Card>,
) -> Option<screens::Update> {
    let request = HttpCreateDeckCard::new(&card.scryfall_data, 1, None);
    match account
        .client
        .create_deck_card(deck_id, &request, session?)
        .await
    {
        Ok(deck_card) => Some(
            Update::QuickAdded {
                deck_id,
                entry: Box::new(DeckEntry {
                    card: *card,
                    deck_card,
                }),
            }
            .into(),
        ),
        Err(e) => Some(error_toast(crate::inbound::screens::shown(&e))),
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    // Writes that land after the view closed still keep the undo log true.
    record_undo(&update, cx);
    toast_failure(&update, cx);
    let Some(view) = nav.find_mut(pick) else {
        return;
    };
    apply_to_view(view, update, cx);
}

/// Pushes the undo action for a write that went through, and toasts a quick add.
fn record_undo(update: &Update, cx: &mut Context<'_>) {
    if let Update::QuantityWritten {
        deck_id,
        card_id,
        card_name,
        burst,
    } = update
        && burst.record
    {
        cx.shared.cards.undo.push(
            *deck_id,
            UndoAction::QuantityChanged {
                card_id: *card_id,
                card_name: card_name.clone(),
                net: burst.delta,
                baseline: burst.baseline,
            },
        );
    }
    if let Update::ZonePrintingSaved {
        deck_id,
        slot,
        old_card,
        new_card,
    } = update
    {
        cx.shared.cards.undo.push(
            *deck_id,
            UndoAction::CommandZonePrintingChanged {
                slot: *slot,
                old_card: old_card.clone(),
                new_id: new_card.scryfall_data.id,
            },
        );
    }
    if let Update::PrintingSaved {
        deck_id,
        old_card,
        new_card,
    } = update
    {
        cx.shared.cards.undo.push(
            *deck_id,
            UndoAction::PrintingChanged {
                old_card: old_card.clone(),
                new_id: new_card.scryfall_data.id,
            },
        );
    }
    if let Update::QuickAdded { deck_id, entry } = update {
        cx.shared.cards.undo.push(
            *deck_id,
            UndoAction::Added {
                card_id: entry.deck_card.scryfall_data_id,
                card_name: entry.card.scryfall_data.name.clone(),
            },
        );
        cx.toast_for(
            ToastKind::Info,
            format!("Added {}", entry.card.scryfall_data.name),
            toast::QUICK,
        );
    }
}

/// Toasts a failed write's message.
fn toast_failure(update: &Update, cx: &mut Context<'_>) {
    match update {
        Update::QuantityFailed { message, .. }
        | Update::MoveFailed { message, .. }
        | Update::MvpSet {
            error: Some(message),
            ..
        }
        | Update::UndoCreated {
            result: Err(message),
            ..
        } => cx.toast(ToastKind::Error, message.clone()),
        _ => {}
    }
}

/// Applies a result to the open view, ignoring one meant for another deck.
fn apply_to_view(view: &mut DeckCardsView, update: Update, cx: &mut Context<'_>) {
    if update.deck_id() != view.deck_id {
        return;
    }
    match update {
        Update::Loaded { loaded, .. } => loaded_deck(view, cx, *loaded),
        Update::Tokens { tokens, .. } => view.tokens = Some(tokens),
        Update::FlushDue {
            card_id,
            generation,
            ..
        } => {
            if let Some(burst) = view.bursts.due(card_id, generation) {
                view.flush(cx, card_id, burst);
            }
        }
        Update::QuantityWritten { .. } => {}
        Update::QuantityFailed {
            deck_id,
            card_id,
            burst,
            removed,
            ..
        } => quantity_failed(view, cx, deck_id, card_id, &burst, removed),
        Update::MoveFailed { card_id, from, .. } => {
            if let Some(entry) = view.entry_mut(card_id) {
                entry.deck_card.board = from;
                view.rerun();
            }
        }
        Update::PrintingSaved {
            old_card, new_card, ..
        } => printing_saved(view, &old_card, new_card),
        Update::ZonePrintingSaved { slot, new_card, .. } => {
            *view.zone.slot_mut(slot) = Some(*new_card);
            view.rerun();
        }
        Update::MvpSet {
            card_id, mvp_at, ..
        } => {
            if let Some(entry) = view.entry_mut(card_id) {
                entry.deck_card.mvp_at = mvp_at;
                view.rerun();
            }
        }
        Update::UndoCreated {
            card_id, result, ..
        } => undo_created(view, card_id, result),
        Update::QuickDue {
            deck_id,
            generation,
        } => {
            if generation == view.quick.generation {
                cx.request(
                    Request::QuickSearch {
                        deck_id,
                        generation,
                        query: view.quick.input.value().to_owned(),
                    }
                    .into(),
                );
            }
        }
        Update::QuickResults {
            generation, cards, ..
        } => {
            if generation == view.quick.generation {
                view.quick.results = cards.map_or(Dropdown::Closed, Dropdown::Results);
                view.quick.highlight = 0;
            }
        }
        Update::QuickAdded { entry, .. } => {
            view.entries.push(*entry);
            view.rerun();
        }
    }
}

/// Rolls a failed quantity write back: a failed delete restores the entry and drops its undo, a failed set restores the baseline.
fn quantity_failed(
    view: &mut DeckCardsView,
    cx: &mut Context<'_>,
    deck_id: Uuid,
    card_id: Uuid,
    burst: &Burst,
    removed: Option<Box<DeckEntry>>,
) {
    match burst.write() {
        Write::Delete => {
            cx.shared.cards.undo.take_newest(deck_id, |a| {
                matches!(a, UndoAction::Removed { entry, .. } if entry.deck_card.scryfall_data_id == card_id)
            });
            if view.entry(card_id).is_none()
                && let Some(mut entry) = removed.map(|e| *e)
            {
                if let Ok(quantity) = Quantity::new(burst.baseline) {
                    entry.deck_card.quantity = quantity;
                }
                view.entries.push(entry);
            }
        }
        Write::Set(_) | Write::Nothing => {
            if let (Some(entry), Ok(quantity)) =
                (view.entry_mut(card_id), Quantity::new(burst.baseline))
            {
                entry.deck_card.quantity = quantity;
            }
        }
    }
    view.rerun();
}

/// Swaps the saved printing into its entry and keeps it expanded if it was.
fn printing_saved(view: &mut DeckCardsView, old_card: &Card, new_card: Box<Card>) {
    let new_id = new_card.scryfall_data.id;
    if let Some(entry) = view.entry_mut(old_card.scryfall_data.id) {
        entry.deck_card.scryfall_data_id = new_id;
        entry.card = *new_card;
    }
    if view.expanded == Some(old_card.scryfall_data.id) {
        view.expanded = Some(new_id);
    }
    view.rerun();
}

/// Takes the recreated entry's row, or drops the entry when recreating failed.
fn undo_created(view: &mut DeckCardsView, card_id: Uuid, result: Result<Box<DeckCard>, String>) {
    match result {
        Ok(deck_card) => {
            if let Some(entry) = view.entry_mut(card_id) {
                entry.deck_card = *deck_card;
            }
        }
        Err(_) => view
            .entries
            .retain(|e| e.deck_card.scryfall_data_id != card_id),
    }
    view.rerun();
}

/// Takes a loaded deck: the command zone out of the entries, the profile's facts, the first-time hint, and the pass.
fn loaded_deck(view: &mut DeckCardsView, cx: &mut Context<'_>, loaded: Loaded) {
    let Loaded {
        deck,
        profile,
        zone,
    } = loaded;
    let mut entries = deck.entries;
    zone.strip(&mut entries);
    view.command_zone_cards = deck.command_zone_cards;
    view.profile = Some(profile.unwrap_or(deck.deck_profile));
    view.zone = zone;
    view.entries = entries;
    let first = !view.loaded;
    view.loaded = true;
    if first && !view.filter.is_empty() {
        cx.toast_for(
            ToastKind::Warning,
            "Filter is active".to_owned(),
            toast::QUICK,
        );
    }
    if !view.hint_seen && !view.entries.is_empty() {
        view.hint_seen = true;
        hint::once(cx, HINT_DECK_CARDS);
    }
    view.rerun();
}
