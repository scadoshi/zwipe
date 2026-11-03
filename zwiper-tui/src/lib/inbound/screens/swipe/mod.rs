//! Add deck cards: swipe search results or the maybeboard into a deck, with the filter sheet, Synergy, undo that reconciles with the deck's undo log, and the land and budget signals.
//!
//! What the screen knows and decides (the deck as Add sees it, both stacks and their cursors, paging, the adds in flight, every undo and its reconciling with the undo log, the land and budget crossings, and every toast but a request's failure) is zwipe-ui's [`State`]; this view adds the client's card filter the sheet edits, the filter, details and printings sheets, the face shown, the images, and the card motion with each pile's entrance mark.
//!
//! Add opens on the deck's id and fetches the deck quietly, as the app does. Leaving parks the filter per source and the search stack per deck, so coming back resumes.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
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
use zwipe_core::domain::deck::{Board, Deck, DeckCard};
use zwipe_core::domain::user::hints::HINT_ADD_DECK_CARDS;
use zwipe_core::http::contracts::deck::HttpDeckCardSearch;
use zwipe_ui::deck_signals::Level;
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::filter::picker::Sources;
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::add_cards::{self, Action, Destination, Effect, Source, State};
use zwipe_ui::stack::Direction;
use zwipe_ui::stores::filters::FilterScope;

use super::Screen;
use super::card_sheets::details::DetailsSheet;
use super::card_sheets::filter::FilterSheet;
use super::card_sheets::printing::PrintingSheet;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::card_motion::{CardMotion, Snapshot};
use crate::inbound::widgets::hint;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;
use crate::outbound::images;

/// Images fetched ahead of the card on screen.
const IMAGES_AHEAD: usize = 3;

/// The filter store's scope for a source.
fn scope(source: Source) -> FilterScope {
    match source {
        Source::Search => FilterScope::AddSearch,
        Source::Maybeboard => FilterScope::AddMaybeboard,
    }
}

/// The side each pile's top card is coming back from after an undo, the app's two entrance signals. The model sets them with `Enter` and clears them with `StopEnter`; any key clears them too.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Entering {
    pub(crate) search: Option<Direction>,
    pub(crate) maybeboard: Option<Direction>,
}

impl Entering {
    /// The mark on `pile`.
    pub(crate) fn of(self, pile: Source) -> Option<Direction> {
        match pile {
            Source::Search => self.search,
            Source::Maybeboard => self.maybeboard,
        }
    }

    fn set(&mut self, pile: Source, side: Option<Direction>) {
        match pile {
            Source::Search => self.search = side,
            Source::Maybeboard => self.maybeboard = side,
        }
    }
}

/// The Add screen: the model plus what only the terminal keeps.
#[derive(Debug)]
pub(crate) struct SwipeView {
    pub(crate) model: State,
    /// The client's card filter: the sheet edits it, the model writes it with `SetFilter` and reads it back on every rerun.
    pub(crate) filter: CardQueryBuilder,
    pub(crate) entering: Entering,
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

impl Default for SwipeView {
    /// A view before [`Action::Load`]: no deck, no card, Search picked.
    fn default() -> Self {
        Self {
            model: State::new(),
            filter: CardQueryBuilder::default(),
            entering: Entering::default(),
            sheet: None,
            details: None,
            printing: None,
            face: 0,
            images: HashMap::new(),
            show_image: false,
            motion: None,
        }
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
        self.model.window().get(1)
    }

    /// The mark on the live pile.
    pub(crate) fn entering(&self) -> Option<Direction> {
        self.entering.of(self.model.source())
    }

    /// Whether the Filter key shows its dot.
    pub(crate) fn filter_dot(&self) -> bool {
        self.model.shows_dot(&self.filter)
    }

    /// Sends the top card off by `direction`, keeping it and its face to draw it leaving. Replaces any motion still running. A maybeboard card swiped up stays, since it is already there.
    pub(crate) fn exit(&mut self, direction: Direction) {
        if self.model.source() == Source::Maybeboard && direction == Direction::Up {
            return;
        }
        let Some(card) = self.model.current().cloned() else {
            return;
        };
        let snapshot = Snapshot {
            card,
            face: self.face,
        };
        self.motion = Some(CardMotion::exit(direction, snapshot, Instant::now()));
    }

    /// Opens the filter sheet for the source in use. Its Reset goes to the model's `ResetFilter` (see [`reset_filter`]), so the sheet keeps no Reset rule of its own.
    pub(crate) fn open_filter(&mut self, cx: &mut Context<'_>) {
        let config = SheetConfig {
            show_format: true,
            validate: self.model.validates_apply(),
            sources: Sources::Catalog,
            ..SheetConfig::default()
        };
        self.sheet = Some(FilterSheet::open(cx, &self.filter, config));
    }

    /// Opens the details sheet over the top card, with Printings on Search.
    pub(crate) fn open_details(&mut self) {
        if let Some(card) = self.model.current().cloned() {
            self.details = Some(DetailsSheet::new(
                card,
                self.model.source() == Source::Search,
            ));
        }
    }

    /// Parks the filter under its source and the search stack under the deck.
    pub(crate) fn park(&self, cx: &mut Context<'_>) {
        let Some(deck_id) = self.model.deck_id() else {
            return;
        };
        let stores = &mut cx.shared.cards;
        stores
            .filters
            .park(scope(self.model.source()), deck_id, self.filter.clone());
        if let Some(parked) = self.model.parked() {
            stores.stacks.park(deck_id, parked);
        }
    }

    /// The images to have: the top card's face and the next few Search cards' fronts. Does nothing when card images are off.
    pub(crate) fn sync_images(&mut self, cx: &mut Context<'_>) {
        if cx.shared.images.is_none() {
            return;
        }
        let mut wanted: Vec<String> = Vec::new();
        if let Some(card) = self.model.current()
            && let Some(url) = card
                .scryfall_data
                .face_image_url(self.face, ImageSize::Normal)
        {
            wanted.push(url.to_owned());
        }
        let ahead: Vec<&Card> = match self.model.source() {
            Source::Search => self
                .model
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

/// Where a `SetFilter` effect writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterSink {
    /// The applied filter, which every action but the sheet's Reset writes.
    Applied,
    /// The open sheet's draft, which the app's Reset writes since its sheet edits the filter in place.
    Sheet,
}

/// Applies `action` to the model and performs the effects it returns in order, then the actions those answer at once (`Taken` for `TakeUndo`, `Filter` for `Rerun`). A new card on top shows its front, and the images follow the pile.
pub(crate) fn dispatch(view: &mut SwipeView, action: Action, cx: &mut Context<'_>) {
    run(view, action, cx, FilterSink::Applied);
}

/// The filter sheet's Reset: the model stages the source's default in the sheet's `draft`, which Apply then commits and Back drops.
pub(crate) fn reset_filter(view: &mut SwipeView, draft: CardQueryBuilder, cx: &mut Context<'_>) {
    run(view, Action::ResetFilter(draft), cx, FilterSink::Sheet);
}

/// [`dispatch`], with `SetFilter` writing to `sink`.
fn run(view: &mut SwipeView, action: Action, cx: &mut Context<'_>, sink: FilterSink) {
    let top = |view: &SwipeView| view.model.current().map(|c| c.scryfall_data.id);
    let before = top(view);
    let mut queue = VecDeque::from([action]);
    while let Some(action) = queue.pop_front() {
        for effect in view.model.update(action) {
            if let Some(answer) = perform_effect(view, effect, cx, sink) {
                queue.push_back(answer);
            }
        }
    }
    if top(view) != before {
        view.face = 0;
    }
    view.sync_images(cx);
}

/// Performs one effect, returning the action that answers it at once, if any.
#[expect(
    clippy::too_many_lines,
    reason = "one arm per effect of the model, in one match as the recipe has it"
)]
fn perform_effect(
    view: &mut SwipeView,
    effect: Effect,
    cx: &mut Context<'_>,
    sink: FilterSink,
) -> Option<Action> {
    match effect {
        Effect::OfferHint(offer) => {
            hint::once(cx, offer.key());
        }
        Effect::FetchDeck { ticket, deck_id } => {
            cx.request(Request::Deck { ticket, deck_id }.into());
        }
        Effect::RefreshMaybeboard { ticket, deck_id } => {
            cx.request(Request::ReloadMaybeboard { ticket, deck_id }.into());
        }
        // The terminal's usage batch carries no search counts, so nothing is recorded or flushed first.
        Effect::Search {
            ticket,
            deck_id,
            query,
        } => cx.request(
            Request::Search {
                ticket,
                deck_id,
                query: Box::new(query),
            }
            .into(),
        ),
        Effect::LoadMore {
            ticket,
            deck_id,
            query,
        } => cx.request(
            Request::LoadMore {
                ticket,
                deck_id,
                query: Box::new(query),
            }
            .into(),
        ),
        Effect::SkipCard { deck_id, oracle_id } => {
            cx.request(Request::Skip { deck_id, oracle_id }.into());
        }
        Effect::UnskipCard { deck_id, oracle_id } => {
            cx.request(Request::Unskip { deck_id, oracle_id }.into());
        }
        Effect::AddCard {
            ticket,
            deck_id,
            card,
            board,
        } => cx.request(
            Request::Create {
                ticket,
                deck_id,
                card,
                board,
            }
            .into(),
        ),
        Effect::DeleteUndoneAdd { deck_id, card_id } => {
            cx.request(Request::DeleteUndone { deck_id, card_id }.into());
        }
        Effect::PromoteCard {
            ticket,
            deck_id,
            card_id,
        } => cx.request(
            Request::Promote {
                ticket,
                deck_id,
                card_id,
            }
            .into(),
        ),
        Effect::TakeUndo {
            ticket,
            deck_id,
            claim,
        } => {
            let taken = cx
                .shared
                .cards
                .undo
                .take_newest(deck_id, |a| claim.matches(a));
            return Some(Action::Taken { ticket, taken });
        }
        Effect::UndoAdd {
            ticket,
            deck_id,
            card_id,
            board,
        } => cx.request(
            Request::UndoAdd {
                ticket,
                deck_id,
                card_id,
                board,
            }
            .into(),
        ),
        Effect::UndoPromote {
            ticket,
            deck_id,
            card_id,
        } => cx.request(
            Request::Demote {
                ticket,
                deck_id,
                card_id,
            }
            .into(),
        ),
        Effect::RecordUndo { deck_id, action } => cx.shared.cards.undo.push(deck_id, action),
        Effect::Enter { pile, from } => view.entering.set(pile, Some(from)),
        Effect::StopEnter(pile) => view.entering.set(pile, None),
        Effect::SetFilter(filter) => match sink {
            FilterSink::Applied => view.filter = filter,
            FilterSink::Sheet => {
                if let Some(sheet) = &mut view.sheet {
                    sheet.replace_draft(filter, cx);
                }
            }
        },
        Effect::Rerun => return Some(Action::Filter(view.filter.clone())),
        Effect::ParkFilter { source, filter } => {
            if let Some(deck_id) = view.model.deck_id() {
                cx.shared.cards.filters.park(scope(source), deck_id, filter);
            }
        }
        // The terminal draws an image whole as soon as it arrives, so there is no ease to reset, and its usage batch carries only error reports, so pile ends go unrecorded.
        Effect::EaseImages | Effect::PileEnd { .. } => {}
        Effect::Navigate(Destination::Previous) => cx.back(),
        Effect::Toast(toast) => cx.show(toast),
    }
    None
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

/// How an undo's reversing call failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UndoFailure {
    /// The session could not be refreshed: the undo is rolled back.
    Session(String),
    /// The call itself failed: "Failed to undo".
    Call(String),
}

/// The Add screen's work, each request with a reply carrying its ticket.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    /// The deck, quietly, for the deck context.
    Deck {
        ticket: Ticket,
        deck_id: Uuid,
    },
    /// The deck again, for Refresh on the maybeboard.
    ReloadMaybeboard {
        ticket: Ticket,
        deck_id: Uuid,
    },
    /// A search from the top.
    Search {
        ticket: Ticket,
        deck_id: Uuid,
        query: Box<CardQuery>,
    },
    /// The next page of the stack.
    LoadMore {
        ticket: Ticket,
        deck_id: Uuid,
        query: Box<CardQuery>,
    },
    /// Adds one copy to `board`.
    Create {
        ticket: Ticket,
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
        ticket: Ticket,
        deck_id: Uuid,
        card_id: Uuid,
        board: Board,
    },
    /// Moves a maybeboard card to the deck.
    Promote {
        ticket: Ticket,
        deck_id: Uuid,
        card_id: Uuid,
    },
    /// Moves a promoted card back to the maybeboard.
    Demote {
        ticket: Ticket,
        deck_id: Uuid,
        card_id: Uuid,
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
            Self::Deck { .. } => "deck",
            Self::ReloadMaybeboard { .. } => "reload_maybeboard",
            Self::Search { .. } => "search",
            Self::LoadMore { .. } => "load_more",
            Self::Create { .. } => "create",
            Self::DeleteUndone { .. } => "delete_undone",
            Self::Skip { .. } => "skip",
            Self::Unskip { .. } => "unskip",
            Self::UndoAdd { .. } => "undo_add",
            Self::Promote { .. } => "promote",
            Self::Demote { .. } => "demote",
            Self::Image { .. } => "image",
        }
    }
}

/// The Add screen's results, each carrying its request's ticket back to the model.
#[derive(Debug)]
pub(crate) enum Update {
    DeckLoaded {
        ticket: Ticket,
        deck: Box<Deck>,
    },
    Searched {
        ticket: Ticket,
        page: HttpDeckCardSearch,
    },
    Added {
        ticket: Ticket,
        deck_card: Box<DeckCard>,
    },
    /// A promote or an undo went through.
    Landed {
        ticket: Ticket,
    },
    /// An undo never went out because the session could not refresh; `message` is the error toast's text.
    NotSent {
        ticket: Ticket,
        message: String,
    },
    /// A request failed; `message` is the error toast's text, `None` for the quiet deck fetch.
    Failed {
        ticket: Ticket,
        message: Option<String>,
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

/// Opens Add over the current screen for the deck with `deck_id`: the model offers the hint, fetches the deck, and resumes the stack parked with the remembered filter, else searches.
pub(crate) fn open(cx: &mut Context<'_>, deck_id: Uuid) {
    let filter = cx
        .shared
        .cards
        .filters
        .restore(FilterScope::AddSearch, deck_id);
    let parked = cx.shared.cards.stacks.take(deck_id).map(Box::new);
    let mut view = SwipeView {
        filter: filter.clone().unwrap_or_default(),
        ..SwipeView::default()
    };
    dispatch(
        &mut view,
        Action::Load {
            deck_id,
            filter,
            parked,
        },
        cx,
    );
    cx.shared.status.clear();
    cx.push(Screen::Swipe(Box::new(view)));
}

/// Shows the hint again, for `?`.
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    hint::reopen(cx, HINT_ADD_DECK_CARDS);
}

/// A failed request's update, its message the error toast's text.
fn failed(ticket: Ticket, error: &ClientError) -> screens::Update {
    screens::failure(error, |message| {
        Update::Failed {
            ticket,
            message: Some(message),
        }
        .into()
    })
}

/// A failed fire-and-forget request's update: its toast.
fn toasted(error: &ClientError) -> screens::Update {
    screens::Update::Toast(ToastKind::Error, screens::shown(error))
}

#[expect(
    clippy::too_many_lines,
    reason = "one arm per request, as every screen's perform has it"
)]
pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let update = match request {
        Request::Deck { ticket, deck_id } => {
            match async { client.fetch_deck(deck_id, &account.fresh().await?).await }.await {
                Ok(deck) => Update::DeckLoaded {
                    ticket,
                    deck: Box::new(deck),
                },
                // Quiet, as the app's `run_quiet`: the screen works without the deck context.
                Err(e) => {
                    return Some(screens::failure(&e, |_| {
                        Update::Failed {
                            ticket,
                            message: None,
                        }
                        .into()
                    }));
                }
            }
        }
        Request::ReloadMaybeboard { ticket, deck_id } => {
            match async { client.fetch_deck(deck_id, &account.fresh().await?).await }.await {
                Ok(deck) => Update::DeckLoaded {
                    ticket,
                    deck: Box::new(deck),
                },
                Err(e) => return Some(failed(ticket, &e)),
            }
        }
        Request::Search {
            ticket,
            deck_id,
            query,
        }
        | Request::LoadMore {
            ticket,
            deck_id,
            query,
        } => {
            let reply = async {
                let session = account.fresh().await?;
                client.search_deck_cards(deck_id, &query, &session).await
            }
            .await;
            match reply {
                Ok(page) => Update::Searched { ticket, page },
                Err(e) => return Some(failed(ticket, &e)),
            }
        }
        Request::Create {
            ticket,
            deck_id,
            card,
            board,
        } => {
            let request = add_cards::add_request(&card, board);
            let reply = async {
                let session = account.fresh().await?;
                client.create_deck_card(deck_id, &request, &session).await
            }
            .await;
            match reply {
                Ok(deck_card) => Update::Added {
                    ticket,
                    deck_card: Box::new(deck_card),
                },
                Err(e) => return Some(failed(ticket, &e)),
            }
        }
        Request::DeleteUndone { deck_id, card_id } => {
            let reply = async {
                let session = account.fresh().await?;
                client.delete_deck_card(deck_id, card_id, &session).await
            }
            .await;
            return reply.err().map(|e| toasted(&e));
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
            let reply = async {
                let session = account.fresh().await?;
                client.unskip_deck_card(deck_id, oracle_id, &session).await
            }
            .await;
            return reply.err().map(|e| toasted(&e));
        }
        Request::Promote {
            ticket,
            deck_id,
            card_id,
        } => {
            let request = add_cards::board_request(Board::Deck);
            let reply = async {
                let session = account.fresh().await?;
                client
                    .update_deck_card(deck_id, card_id, &request, &session)
                    .await
            }
            .await;
            match reply {
                Ok(_) => Update::Landed { ticket },
                Err(e) => return Some(failed(ticket, &e)),
            }
        }
        Request::UndoAdd {
            ticket,
            deck_id,
            card_id,
            ..
        } => {
            let session = match account.fresh().await {
                Ok(session) => session,
                Err(e) => return Some(not_sent(ticket, &e)),
            };
            match client.delete_deck_card(deck_id, card_id, &session).await {
                Ok(()) => Update::Landed { ticket },
                Err(e) => undo_failed(ticket, &e),
            }
        }
        Request::Demote {
            ticket,
            deck_id,
            card_id,
        } => {
            let session = match account.fresh().await {
                Ok(session) => session,
                Err(e) => return Some(not_sent(ticket, &e)),
            };
            let request = add_cards::board_request(Board::Maybeboard);
            match client
                .update_deck_card(deck_id, card_id, &request, &session)
                .await
            {
                Ok(_) => Update::Landed { ticket },
                Err(e) => undo_failed(ticket, &e),
            }
        }
        Request::Image { url } => Update::Image {
            image: images::fetch(&client.client, &url).await.ok().map(Box::new),
            url,
        },
    };
    Some(update.into())
}

/// An undo whose session could not refresh, so its call never went out.
fn not_sent(ticket: Ticket, error: &ClientError) -> screens::Update {
    Update::NotSent {
        ticket,
        message: screens::shown(error),
    }
    .into()
}

/// An undo whose call failed, worded as the undo's own failure.
fn undo_failed(ticket: Ticket, error: &ClientError) -> Update {
    Update::Failed {
        ticket,
        message: Some(format!("Failed to undo: {error}")),
    }
}

/// Hands a reply to the model. A failure's error toast goes up even when the screen has closed; a reply with no screen to take it does nothing, as the app's task dies with its screen.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    match &update {
        Update::NotSent { message, .. }
        | Update::Failed {
            message: Some(message),
            ..
        } => cx.toast(ToastKind::Error, message.clone()),
        _ => {}
    }
    let Some(view) = nav.find_mut(pick) else {
        return;
    };
    let action = match update {
        Update::DeckLoaded { ticket, deck } => Action::DeckLoaded { ticket, deck },
        Update::Searched { ticket, page } => Action::Searched { ticket, page },
        Update::Added { ticket, deck_card } => Action::Added { ticket, deck_card },
        Update::Landed { ticket } => Action::Landed { ticket },
        Update::NotSent { ticket, .. } => Action::NotSent { ticket },
        Update::Failed { ticket, .. } => Action::Failed { ticket },
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
            return;
        }
    };
    dispatch(view, action, cx);
}
