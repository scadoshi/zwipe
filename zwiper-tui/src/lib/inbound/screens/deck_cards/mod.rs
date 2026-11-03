//! The deck cards view: every card in a deck grouped and filtered, with quick add, row actions (quantity, board, printing, MVP), the deck's undo log, and the doors to Add and Remove.
//!
//! The view drives zwipe-ui's `screens::deck_cards` model, which owns the deck, the groups, the quantity bursts and every write; the view keeps the client's filter, the cursor, the open row and the sheets.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::time::Duration;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::query::CardQuery;
use zwipe_core::domain::deck::{Board, Deck, DeckCard, DeckProfile};
use zwipe_core::http::contracts::deck::HttpUpdateDeckProfile;
use zwipe_core::http::contracts::deck_card::{HttpCreateDeckCard, HttpPatchDeckCard};
use zwipe_ui::deck::form::Slot;
use zwipe_ui::deck_cards::{Display, Write, Zone};
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::filter::picker::Sources;
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::deck_cards::{
    self, Action, Destination, Effect, Op, QuickAddResults, State,
};
use zwipe_ui::stores::filters::FilterScope;
use zwipe_ui::stores::undo::UndoAction;

use super::card_sheets::filter::FilterSheet;
use super::card_sheets::printing::PrintingSheet;
use super::deck_form::Body;
use super::{Screen, remove_cards, swipe};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The printing sheet's toast on Save.
const PRINTING_SAVED: &str = "Printing saved";

/// What a card row is, which sets its actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// A main board row in a group or Lands: every action and the star.
    Main,
    /// A maybeboard or sideboard row.
    Board(Board),
    /// A command zone row: Printing only.
    Zone(Slot),
    /// A token: no actions.
    Token,
}

/// One line of the list.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Item {
    Header {
        key: String,
        title: String,
        pips: Option<Vec<Color>>,
        collapsible: bool,
    },
    Card {
        card: Box<Card>,
        kind: Kind,
    },
}

/// Quick add's box and the result the cursor is on; the query's results are the model's.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct QuickAdd {
    pub(crate) input: TextInput,
    pub(crate) focused: bool,
    pub(crate) highlight: usize,
}

/// The deck cards view.
#[derive(Debug, Default)]
pub(crate) struct DeckCardsView {
    pub(crate) model: State,
    /// The client's filter: the filter sheet edits it, and the model hears it on every apply and every `Refilter`.
    pub(crate) filter: CardQueryBuilder,
    /// The model's lists under `filter`, taken after every action.
    pub(crate) display: Display,
    /// The open row, by printing ID.
    pub(crate) expanded: Option<Uuid>,
    /// The open row's oracle tag whose definition shows, by its place in [`card_info::oracle_tags`](crate::inbound::widgets::card_info::oracle_tags).
    pub(crate) open_tag: Option<usize>,
    /// The face the open row shows.
    pub(crate) face: usize,
    /// The selected item.
    pub(crate) cursor: usize,
    pub(crate) quick: QuickAdd,
    pub(crate) sheet: Option<FilterSheet>,
    pub(crate) printing: Option<PrintingSheet>,
    /// The image preview: a card and its face.
    pub(crate) preview: Option<(Box<Card>, usize)>,
    /// Whether Add or Remove opened over the view, so their return loads the deck again.
    reload_on_return: bool,
}

impl DeckCardsView {
    /// Loads the deck again when Add or Remove closes over the view, under the filter as it stands.
    pub(crate) fn on_return(&mut self, cx: &mut Context<'_>) {
        if !std::mem::take(&mut self.reload_on_return) {
            return;
        }
        if let Some(deck_id) = self.model.deck_id() {
            let filter = Box::new(self.filter.clone());
            dispatch(self, Action::Load { deck_id, filter }, cx);
        }
    }

    /// The list's lines, top to bottom: tokens, maybeboard, sideboard, command zone, groups, lands.
    pub(crate) fn items(&self) -> Vec<Item> {
        let mut items = Vec::new();
        self.push_tokens(&mut items);
        self.push_side_boards(&mut items);
        if self.model.show_command_zone() {
            self.push_zone(&mut items);
        }
        if self.model.board_on(Board::Deck) {
            self.push_main(&mut items);
        }
        items
    }

    /// A collapsible section: its header, then its cards unless it is collapsed.
    fn push_section(
        &self,
        items: &mut Vec<Item>,
        key: &str,
        title: String,
        pips: Option<Vec<Color>>,
        cards: Vec<(Card, Kind)>,
    ) {
        items.push(Item::Header {
            key: key.to_owned(),
            title,
            pips,
            collapsible: true,
        });
        if !self.model.is_collapsed(key) {
            items.extend(cards.into_iter().map(|(card, kind)| Item::Card {
                card: Box::new(card),
                kind,
            }));
        }
    }

    /// The Tokens section, when tokens are shown and there are some.
    fn push_tokens(&self, items: &mut Vec<Item>) {
        if self.model.show_tokens()
            && let Some(tokens) = self.model.tokens(&self.filter).filter(|t| !t.is_empty())
        {
            self.push_section(
                items,
                deck_cards::TOKENS_KEY,
                deck_cards::tokens_title(tokens.len()),
                None,
                tokens.into_iter().map(|c| (c, Kind::Token)).collect(),
            );
        }
    }

    /// The Maybeboard and Sideboard sections, each when shown and not empty.
    fn push_side_boards(&self, items: &mut Vec<Item>) {
        for (board, list, key, title) in [
            (
                Board::Maybeboard,
                &self.display.maybe,
                deck_cards::MAYBEBOARD_KEY,
                deck_cards::maybeboard_title(self.display.maybe.len()),
            ),
            (
                Board::Sideboard,
                &self.display.side,
                deck_cards::SIDEBOARD_KEY,
                deck_cards::sideboard_title(self.display.side.len()),
            ),
        ] {
            if self.model.board_on(board) && !list.is_empty() {
                self.push_section(
                    items,
                    key,
                    title,
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
        let format = self.model.profile().and_then(|p| p.format);
        let leaders: Vec<&(Slot, Card)> = zone
            .iter()
            .filter(|(s, _)| matches!(s, Slot::Commander | Slot::Partner))
            .collect();
        if let Some((slot, _)) = leaders.first() {
            let has_partner = leaders.iter().any(|(s, _)| *s == Slot::Partner);
            items.push(zone_header(slot.group_header(format, has_partner)));
            items.extend(
                leaders
                    .into_iter()
                    .map(|(slot, card)| zone_item(*slot, card)),
            );
        }
        for slot in [Slot::Background, Slot::SignatureSpell] {
            if let Some((slot, card)) = zone.iter().find(|(s, _)| *s == slot) {
                items.push(zone_header(slot.group_header(format, false)));
                items.push(zone_item(*slot, card));
            }
        }
    }

    /// The main board: each group's section, then Lands.
    fn push_main(&self, items: &mut Vec<Item>) {
        for group in &self.display.groups {
            self.push_section(
                items,
                &group.key,
                deck_cards::group_title(&group.label, self.model.copies(&group.cards)),
                group.pips.clone(),
                group
                    .cards
                    .iter()
                    .map(|c| (c.clone(), Kind::Main))
                    .collect(),
            );
        }
        let lands = &self.display.lands;
        if !lands.is_empty() {
            self.push_section(
                items,
                deck_cards::LANDS_KEY,
                deck_cards::lands_title(self.model.copies(lands)),
                None,
                lands.iter().map(|c| (c.clone(), Kind::Main)).collect(),
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

    /// The board a row's card sits on, if it is a deck entry.
    pub(crate) fn board(&self, card_id: Uuid) -> Option<Board> {
        self.model.row(card_id).map(|r| r.board)
    }

    /// Opens the filter sheet over the deck's cards as they loaded.
    pub(crate) fn open_filter(&mut self, cx: &mut Context<'_>) {
        let config = SheetConfig {
            show_format: true,
            sources: Sources::deck(self.model.filter_cards()),
            ..SheetConfig::default()
        };
        self.sheet = Some(FilterSheet::open(cx, &self.filter, config));
    }

    /// Takes the filter the sheet applied: the open row closes and the model regroups under it.
    pub(crate) fn apply_filter(&mut self, filter: CardQueryBuilder, cx: &mut Context<'_>) {
        self.filter = filter;
        self.expanded = None;
        let filter = Box::new(self.filter.clone());
        dispatch(self, Action::Filter(filter), cx);
    }

    /// The deck's Undo: hands the model the newest entry of the deck's log.
    pub(crate) fn undo(&mut self, cx: &mut Context<'_>) {
        let Some(deck_id) = self.model.deck_id() else {
            return;
        };
        if let Some(entry) = cx.shared.cards.undo.pop(deck_id) {
            dispatch(self, Action::Undo(Box::new(entry)), cx);
        }
    }

    /// Adds the highlighted quick add result.
    pub(crate) fn quick_pick(&mut self, cx: &mut Context<'_>) {
        let Some(QuickAddResults::Found(results)) = self.model.quick_add() else {
            return;
        };
        let at = self.quick.highlight.min(results.len().saturating_sub(1));
        if let Some(card) = results.into_iter().nth(at) {
            dispatch(self, Action::AddCard(Box::new(card)), cx);
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

fn zone_item(slot: Slot, card: &Card) -> Item {
    Item::Card {
        card: Box::new(card.clone()),
        kind: Kind::Zone(slot),
    }
}

/// Opens the deck cards view for `deck_id` over the current screen under the filter remembered for it, and loads the deck and its tokens. The deck view's `c` opens it.
pub(crate) fn open(cx: &mut Context<'_>, deck_id: Uuid) {
    let filter = cx
        .shared
        .cards
        .filters
        .restore(FilterScope::Cards, deck_id)
        .unwrap_or_default();
    let mut view = DeckCardsView {
        filter: filter.clone(),
        ..DeckCardsView::default()
    };
    dispatch(
        &mut view,
        Action::Load {
            deck_id,
            filter: Box::new(filter),
        },
        cx,
    );
    cx.push(Screen::DeckCards(Box::new(view)));
}

/// Applies `action` to the model and performs the effects it returns, in order. An effect the client answers at once (`Refilter` with the filter, a leave with `Leave`) queues its action, which goes to the model after the batch.
pub(crate) fn dispatch(view: &mut DeckCardsView, action: Action, cx: &mut Context<'_>) {
    let mut queue = VecDeque::from([action]);
    while let Some(action) = queue.pop_front() {
        let printing = view.model.printing().map(|c| c.scryfall_data.id);
        let results = view.model.quick_add();
        for effect in view.model.update(action) {
            if let Some(next) = perform_effect(view, effect, cx) {
                queue.push_back(next);
            }
        }
        // A saved printing replaces the open row's card, which stays open.
        let saved = view.model.printing().map(|c| c.scryfall_data.id);
        if let (Some(old), Some(new)) = (printing, saved)
            && old != new
            && view.expanded == Some(old)
        {
            view.expanded = Some(new);
        }
        if view.model.quick_add() != results {
            view.quick.highlight = 0;
        }
    }
    view.display = view.model.display(&view.filter);
    view.cursor = view.cursor.min(view.items().len().saturating_sub(1));
}

/// The request a request effect sends, or the effect back when it is not one.
fn request_for(effect: Effect) -> Result<Request, Effect> {
    Ok(match effect {
        Effect::FetchDeck { ticket, deck_id } => Request::FetchDeck { ticket, deck_id },
        Effect::FetchTokens { ticket, deck_id } => Request::FetchTokens { ticket, deck_id },
        Effect::Debounce { ticket, wait } => Request::Debounce { ticket, wait },
        Effect::SearchCards {
            ticket,
            deck_id,
            query,
        } => Request::SearchCards {
            ticket,
            deck_id,
            query,
        },
        Effect::CreateCard {
            ticket,
            deck_id,
            request,
            op,
        } => Request::CreateCard {
            ticket,
            deck_id,
            body: Body(request),
            op,
        },
        Effect::PatchCard {
            ticket,
            deck_id,
            card_id,
            request,
            op,
        } => Request::PatchCard {
            ticket,
            deck_id,
            card_id,
            body: Body(request),
            op,
        },
        Effect::DeleteCard {
            ticket,
            deck_id,
            card_id,
            op,
        } => Request::DeleteCard {
            ticket,
            deck_id,
            card_id,
            op,
        },
        Effect::PatchProfile {
            ticket,
            deck_id,
            request,
            op,
        } => Request::PatchProfile {
            ticket,
            deck_id,
            body: Body(request),
            op,
        },
        Effect::Flush {
            deck_id,
            card_id,
            write,
        } => Request::Flush {
            deck_id,
            card_id,
            write,
        },
        other => return Err(other),
    })
}

/// Performs one effect, returning the action that answers it at once, if any.
fn perform_effect(
    view: &mut DeckCardsView,
    effect: Effect,
    cx: &mut Context<'_>,
) -> Option<Action> {
    let effect = match request_for(effect) {
        Ok(request) => {
            cx.request(request.into());
            return None;
        }
        Err(effect) => effect,
    };
    match effect {
        Effect::OfferHint(offer) => {
            hint::once(cx, offer.key());
        }
        // The requests went out through `request_for`, and the terminal's usage batches count no removals.
        Effect::FetchDeck { .. }
        | Effect::FetchTokens { .. }
        | Effect::Debounce { .. }
        | Effect::SearchCards { .. }
        | Effect::CreateCard { .. }
        | Effect::PatchCard { .. }
        | Effect::DeleteCard { .. }
        | Effect::PatchProfile { .. }
        | Effect::Flush { .. }
        | Effect::RecordRemoval { .. } => {}
        Effect::RecordUndo(entry) => {
            if let Some(deck_id) = view.model.deck_id() {
                cx.shared.cards.undo.push(deck_id, *entry);
            }
        }
        Effect::ForgetRemoval { card_id } => {
            if let Some(deck_id) = view.model.deck_id() {
                cx.shared.cards.undo.take_newest(deck_id, |a| {
                    matches!(a, UndoAction::Removed { entry, .. } if entry.card.scryfall_data.id == card_id)
                });
            }
        }
        Effect::ClearFilter => {
            view.filter.clear();
        }
        Effect::Refilter => return Some(Action::Filter(Box::new(view.filter.clone()))),
        Effect::Collapse => view.expanded = None,
        Effect::OpenPrintings => {
            view.printing = view.model.printing().cloned().map(|card| {
                PrintingSheet::open(cx, card, /* read_only */ false, PRINTING_SAVED)
            });
        }
        Effect::Navigate(to) => return Some(navigate(view, to, cx)),
        Effect::Toast(toast) => cx.show(toast),
    }
    None
}

/// Goes where the model sends the user. Leaving the view, or opening Add or Remove over it, writes every waiting burst through `Leave`.
fn navigate(view: &mut DeckCardsView, to: Destination, cx: &mut Context<'_>) -> Action {
    match to {
        Destination::Previous => {
            if let Some(deck_id) = view.model.deck_id() {
                cx.shared
                    .cards
                    .filters
                    .park(FilterScope::Cards, deck_id, view.filter.clone());
            }
            cx.back();
        }
        Destination::AddCards(deck_id) => {
            view.reload_on_return = true;
            swipe::open(cx, deck_id);
        }
        Destination::RemoveCards(deck_id) => {
            view.reload_on_return = true;
            remove_cards::open(cx, deck_id);
        }
    }
    Action::Leave
}

/// The view's work, each of the model's requests carrying its ticket.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    /// Fetches the deck, then its profile and its command zone cards quietly.
    FetchDeck { ticket: Ticket, deck_id: Uuid },
    /// Fetches the deck's tokens; a failure shows none.
    FetchTokens { ticket: Ticket, deck_id: Uuid },
    /// Waits out a quick add pause or a quantity burst.
    Debounce { ticket: Ticket, wait: Duration },
    /// A quick add search, quiet on failure.
    SearchCards {
        ticket: Ticket,
        deck_id: Uuid,
        query: Box<CardQuery>,
    },
    CreateCard {
        ticket: Ticket,
        deck_id: Uuid,
        body: Body<HttpCreateDeckCard>,
        op: Op,
    },
    PatchCard {
        ticket: Ticket,
        deck_id: Uuid,
        card_id: Uuid,
        body: Body<HttpPatchDeckCard>,
        op: Op,
    },
    DeleteCard {
        ticket: Ticket,
        deck_id: Uuid,
        card_id: Uuid,
        op: Op,
    },
    PatchProfile {
        ticket: Ticket,
        deck_id: Uuid,
        body: Body<HttpUpdateDeckProfile>,
        op: Op,
    },
    /// Writes a burst the view left behind; only a failure reports back.
    Flush {
        deck_id: Uuid,
        card_id: Uuid,
        write: Write,
    },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports: the app's names, and a write's [`Op`].
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::FetchDeck { .. } => "load_deck",
            Self::FetchTokens { .. } => "load_tokens",
            Self::Debounce { .. } => "debounce",
            Self::SearchCards { .. } => "quick_add_search",
            Self::CreateCard { op, .. }
            | Self::PatchCard { op, .. }
            | Self::DeleteCard { op, .. }
            | Self::PatchProfile { op, .. } => op.name(),
            Self::Flush { .. } => "flush_quantity",
        }
    }
}

/// The view's results, each carrying the ticket of the request it answers.
#[derive(Debug)]
pub(crate) enum Update {
    /// The deck, with its profile when that fetch went through and the command zone cards it could fetch.
    Loaded {
        ticket: Ticket,
        profile: Option<Box<DeckProfile>>,
        deck: Box<Deck>,
        zone: Box<Zone>,
    },
    TokensLoaded {
        ticket: Ticket,
        tokens: Vec<Card>,
    },
    Debounced {
        ticket: Ticket,
    },
    Searched {
        ticket: Ticket,
        cards: Vec<Card>,
    },
    Created {
        ticket: Ticket,
        deck_card: Box<DeckCard>,
    },
    Patched {
        ticket: Ticket,
        deck_card: Box<DeckCard>,
    },
    Deleted {
        ticket: Ticket,
    },
    ProfilePatched {
        ticket: Ticket,
    },
    /// A request failed: `Some` holds the error toast's message, `None` is a quiet search.
    Failed {
        ticket: Ticket,
        message: Option<String>,
    },
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::DeckCards(request)
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

/// The update a write's answer comes back as: `ok`'s on success, else the failure carrying `ticket` and its toast.
fn reply<T>(
    ticket: Ticket,
    result: Result<T, zwipe_client::ClientError>,
    ok: impl FnOnce(T) -> Update,
) -> screens::Update {
    match result {
        Ok(value) => ok(value).into(),
        Err(e) => failed(ticket, &e),
    }
}

/// A request's failure carrying `ticket` and its toast, or the end of the session.
fn failed(ticket: Ticket, error: &zwipe_client::ClientError) -> screens::Update {
    screens::failure(error, |message| {
        Update::Failed {
            ticket,
            message: Some(message),
        }
        .into()
    })
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    Some(match request {
        Request::FetchDeck { ticket, deck_id } => load(account, ticket, deck_id).await,
        Request::FetchTokens { ticket, deck_id } => tokens(account, ticket, deck_id).await,
        Request::Debounce { ticket, wait } => {
            tokio::time::sleep(wait).await;
            Update::Debounced { ticket }.into()
        }
        Request::SearchCards {
            ticket,
            deck_id,
            query,
        } => search(account, ticket, deck_id, &query).await,
        Request::CreateCard {
            ticket,
            deck_id,
            body,
            ..
        } => reply(
            ticket,
            async {
                client
                    .create_deck_card(deck_id, &body.0, &account.fresh().await?)
                    .await
            }
            .await,
            |deck_card| Update::Created {
                ticket,
                deck_card: Box::new(deck_card),
            },
        ),
        Request::PatchCard {
            ticket,
            deck_id,
            card_id,
            body,
            ..
        } => reply(
            ticket,
            async {
                client
                    .update_deck_card(deck_id, card_id, &body.0, &account.fresh().await?)
                    .await
            }
            .await,
            |deck_card| Update::Patched {
                ticket,
                deck_card: Box::new(deck_card),
            },
        ),
        Request::DeleteCard {
            ticket,
            deck_id,
            card_id,
            ..
        } => reply(
            ticket,
            async {
                client
                    .delete_deck_card(deck_id, card_id, &account.fresh().await?)
                    .await
            }
            .await,
            |()| Update::Deleted { ticket },
        ),
        Request::PatchProfile {
            ticket,
            deck_id,
            body,
            ..
        } => reply(
            ticket,
            async {
                client
                    .update_deck_profile(deck_id, &body.0, &account.fresh().await?)
                    .await
            }
            .await,
            |_| Update::ProfilePatched { ticket },
        ),
        Request::Flush {
            deck_id,
            card_id,
            write,
        } => return flush(account, deck_id, card_id, write).await,
    })
}

/// Fetches the deck's tokens. They only decorate the list, so a failure shows none.
async fn tokens(account: &Account, ticket: Ticket, deck_id: Uuid) -> screens::Update {
    let tokens = async {
        account
            .client
            .fetch_deck_tokens(deck_id, &account.fresh().await?)
            .await
    }
    .await
    .unwrap_or_default();
    Update::TokensLoaded { ticket, tokens }.into()
}

/// A quick add search. One goes out on every pause in typing, so a failure only closes the results.
async fn search(
    account: &Account,
    ticket: Ticket,
    deck_id: Uuid,
    query: &CardQuery,
) -> screens::Update {
    let page = async {
        account
            .client
            .search_deck_cards(deck_id, query, &account.fresh().await?)
            .await
    }
    .await;
    match page {
        Ok(page) => Update::Searched {
            ticket,
            cards: page.cards,
        },
        Err(_) => Update::Failed {
            ticket,
            message: None,
        },
    }
    .into()
}

/// Fetches the deck, then its profile and command zone cards, which fall back quietly: the zone from the deck's own profile, nothing for a card that fails.
async fn load(account: &Account, ticket: Ticket, deck_id: Uuid) -> screens::Update {
    let client = &account.client;
    let fetched = async {
        let session = account.fresh().await?;
        let deck = client.fetch_deck(deck_id, &session).await?;
        Ok((session, deck))
    }
    .await;
    let (session, deck) = match fetched {
        Ok(fetched) => fetched,
        Err(e) => return failed(ticket, &e),
    };
    let profile = client.fetch_deck_profile(deck_id, &session).await.ok();
    let mut zone = Zone::default();
    for (slot, card_id) in deck_cards::zone_requests(profile.as_ref().unwrap_or(&deck.deck_profile))
    {
        *zone.slot_mut(slot) = client.fetch_card(card_id).await.ok();
    }
    Update::Loaded {
        ticket,
        profile: profile.map(Box::new),
        deck: Box::new(deck),
        zone: Box::new(zone),
    }
    .into()
}

/// Writes a burst the view left behind: a delete or the burst's absolute quantity. Only a failure reports back, as a toast.
async fn flush(
    account: &Account,
    deck_id: Uuid,
    card_id: Uuid,
    write: Write,
) -> Option<screens::Update> {
    let client = &account.client;
    let result = async {
        let session = account.fresh().await?;
        match write {
            Write::Nothing => Ok(()),
            Write::Delete => client.delete_deck_card(deck_id, card_id, &session).await,
            Write::Set(quantity) => client
                .update_deck_card(
                    deck_id,
                    card_id,
                    &HttpPatchDeckCard::new(Some(quantity), None),
                    &session,
                )
                .await
                .map(drop),
        }
    }
    .await;
    result.err().map(|e| {
        screens::failure(&e, |message| {
            screens::Update::Toast(ToastKind::Error, message)
        })
    })
}

/// Hands a reply to the model. A failure raises its toast even when the view has closed.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let action = match update {
        Update::Loaded {
            ticket,
            profile,
            deck,
            zone,
        } => {
            let Some(view) = nav.find_mut(pick) else {
                return;
            };
            let profile = profile.unwrap_or_else(|| Box::new(deck.deck_profile.clone()));
            dispatch(view, Action::ProfileLoaded { ticket, profile }, cx);
            dispatch(view, Action::Loaded { ticket, deck, zone }, cx);
            return;
        }
        Update::TokensLoaded { ticket, tokens } => Action::TokensLoaded { ticket, tokens },
        Update::Debounced { ticket } => Action::Debounced { ticket },
        Update::Searched { ticket, cards } => Action::Searched { ticket, cards },
        Update::Created { ticket, deck_card } => Action::Created { ticket, deck_card },
        Update::Patched { ticket, deck_card } => Action::Patched { ticket, deck_card },
        Update::Deleted { ticket } => Action::Deleted { ticket },
        Update::ProfilePatched { ticket } => Action::ProfilePatched { ticket },
        Update::Failed { ticket, message } => {
            if let Some(message) = message {
                cx.toast(ToastKind::Error, message);
            }
            Action::Failed { ticket }
        }
    };
    if let Some(view) = nav.find_mut(pick) {
        dispatch(view, action, cx);
    }
}
