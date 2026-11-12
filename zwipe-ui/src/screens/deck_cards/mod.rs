//! The deck cards view: one deck's cards in groups, with the featured strip, Quick add, quantities, board moves, printings, MVP stars, Undo and Reset filter.
//!
//! The client keeps a [`State`] while the screen is up and sends [`Action::Load`] with the deck's id and its remembered filter when it opens. The filter and group pass and the featured strip come from [`crate::deck_cards`], the land and budget toasts from [`crate::deck_signals`]. The card filter belongs to the client: the shared filter sheet edits it, and the client hands it in with [`Action::Filter`] each time the filter is applied, which is when the model asks for [`Effect::Refilter`] and when the sheet applies. The undo log belongs to the client too: the model records entries with [`Effect::RecordUndo`], and the client pops the newest and hands it in with [`Action::Undo`]. Which row is expanded, the image preview, the detail pane, drag to move, row art, whether the filter or printing sheet is up and the rows' drawing stay in the client.

#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::time::Duration;

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::card_filter::query::CardQuery;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::{
    Board, Deck, DeckCard, DeckEntry, DeckProfile, Quantity, deck_metrics,
};
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_core::http::contracts::deck::HttpUpdateDeckProfile;
use zwipe_core::http::contracts::deck_card::{HttpCreateDeckCard, HttpPatchDeckCard};

use crate::deck::command_zone::Slot;
use crate::deck_cards::keys::{RowInfo, RowKind};
use crate::deck_cards::{self, Boards, Display, Write, Zone};
use crate::deck_signals::{self, Crossing, Level};
use crate::generation::{Generation, Ticket};
use crate::stores::undo::{UndoAction, create_board_field};
use crate::timing;
use crate::toast::{Line, Toast, ToastKind, keep_one};

/// The screen's title.
pub const TITLE: &str = "Cards";

/// The hint's title.
pub const HINT_TITLE: &str = "Browsing your deck";

/// The Quick add row's label.
pub const QUICK_ADD: &str = "Quick add";

/// The Quick add input's placeholder.
pub const QUICK_ADD_PLACEHOLDER: &str = "Search a card to add";

/// The one result chip while a search waits or is out.
pub const SEARCHING: &str = "Searching…";

/// The one result chip when the search found nothing.
pub const NO_RESULTS: &str = "No results";

/// The Group by row's label.
pub const GROUP_BY: &str = "Group by";

/// The Boards row's label.
pub const BOARDS: &str = "Boards";

/// The Show row's label.
pub const SHOW: &str = "Show";

/// The Show chip for the tokens section.
pub const TOKENS: &str = "Tokens";

/// The Show chip for the command zone's pinned groups.
pub const COMMAND_ZONE: &str = "Command zone";

/// The Show chip for row art, which the client keeps.
pub const ART: &str = "Art";

/// The featured strip's role for a starred card.
pub const MVP: &str = "MVP";

/// The line under the chip rows when the main board shows nothing.
pub const NO_CARDS: &str = "No cards";

/// The Back button, to where the user came from.
pub const BACK: &str = "Back";

/// The button that opens Add cards.
pub const ADD: &str = "Add";

/// The button that opens Remove cards.
pub const REMOVE: &str = "Remove";

/// The button that opens the filter sheet.
pub const FILTER: &str = "Filter";

/// The button that walks the undo log back, shown while it holds an entry.
pub const UNDO: &str = "Undo";

/// The button that clears the filter, shown while one is on.
pub const RESET_FILTER: &str = "Reset filter";

/// The toast once a loaded deck shows under a remembered filter.
pub const FILTER_ACTIVE: &str = "Filter is active";

/// The toast after Reset filter.
pub const FILTER_RESET: &str = "Filter reset";

/// The toast when a quantity tap takes a card below one copy.
pub const CARD_REMOVED: &str = "Card removed";

/// The toast for an undo entry the deck has moved past.
pub const ALREADY_CHANGED: &str = "Already changed";

/// The collapse key of the tokens section.
pub const TOKENS_KEY: &str = "Tokens";

/// The collapse key of the lands section.
pub const LANDS_KEY: &str = "Lands";

/// The collapse key of the maybeboard section.
pub const MAYBEBOARD_KEY: &str = "Maybeboard";

/// The collapse key of the sideboard section.
pub const SIDEBOARD_KEY: &str = "Sideboard";

/// The Boards chips in the order they draw.
pub const BOARD_CHIPS: [Board; 3] = [Board::Deck, Board::Maybeboard, Board::Sideboard];

/// A Boards chip's words.
#[must_use]
pub fn board_label(board: Board) -> &'static str {
    match board {
        Board::Deck => "Main",
        Board::Maybeboard => "Maybe",
        Board::Sideboard => "Side",
    }
}

/// The toast after Quick add adds `name`.
#[must_use]
pub fn added(name: &str) -> String {
    format!("Added {name}")
}

/// The toast after a move to the mainboard, from any screen.
pub const MOVED_TO_MAINBOARD: &str = "Moved to mainboard";

/// The toast after a row's move to `board`. The words are the copy's
/// (mainboard), not the board's wire name (deck).
#[must_use]
pub fn moved_to(board: Board) -> String {
    match board {
        Board::Deck => MOVED_TO_MAINBOARD.to_owned(),
        Board::Maybeboard => "Moved to maybeboard".to_owned(),
        Board::Sideboard => "Moved to sideboard".to_owned(),
    }
}

/// A card group's header: `label` and its copy count, or the count alone for a color group, whose pips stand in for the label.
#[must_use]
pub fn group_title(label: &str, copies: i32) -> String {
    if label.is_empty() {
        format!("({copies})")
    } else {
        format!("{label} ({copies})")
    }
}

/// The lands section's header, counting copies.
#[must_use]
pub fn lands_title(copies: i32) -> String {
    format!("Lands ({copies})")
}

/// The tokens section's header, counting tokens.
#[must_use]
pub fn tokens_title(count: usize) -> String {
    format!("Tokens ({count})")
}

/// The maybeboard section's header, counting rows.
#[must_use]
pub fn maybeboard_title(count: usize) -> String {
    format!("Maybeboard ({count})")
}

/// The sideboard section's header, counting rows.
#[must_use]
pub fn sideboard_title(count: usize) -> String {
    format!("Sideboard ({count})")
}

/// One stack of the Stacks view: a section of the List with its rows as the stack's cards.
#[derive(Debug, Clone, PartialEq)]
pub struct StackGroup {
    /// The fold key the List's section uses, so a fold carries between the views.
    pub key: String,
    /// The header, as the List's section reads it.
    pub title: String,
    /// Pips in place of the header, for a color group.
    pub pips: Option<Vec<zwipe_core::domain::card::scryfall_data::colors::Color>>,
    /// The cards, top to bottom, each with its copies and what its details offer.
    pub rows: Vec<RowInfo>,
}

/// The Quick add search for `query`: cards whose name holds the trimmed query, synergy off, skipped cards included, sorted by name, [`timing::QUICK_ADD_LIMIT`] at most. The server leaves out cards already in the deck.
#[must_use]
pub fn quick_add_query(query: &str) -> CardQueryBuilder {
    let mut builder = CardQueryBuilder::with_name_contains(query.trim());
    builder
        .set_synergy(false)
        .set_include_skipped(true)
        .set_sort(CardSortKey::Name)
        .set_limit(timing::QUICK_ADD_LIMIT);
    builder
}

/// The command zone slots a profile names, in the order the client fetches their cards: commander, signature spell, partner, background.
#[must_use]
pub fn zone_requests(profile: &DeckProfile) -> Vec<(Slot, Uuid)> {
    [
        (Slot::Commander, profile.commander_id),
        (Slot::SignatureSpell, profile.signature_spell_id),
        (Slot::Partner, profile.partner_commander_id),
        (Slot::Background, profile.background_id),
    ]
    .into_iter()
    .filter_map(|(slot, id)| id.map(|id| (slot, id)))
    .collect()
}

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// Wherever the user came from, from Back.
    Previous,
    /// Add cards for this deck.
    AddCards(Uuid),
    /// Remove cards for this deck.
    RemoveCards(Uuid),
    /// The deck list, in place of a screen whose deck is gone.
    DeckList,
}

/// Which write a request is, for the client's telemetry name and its reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    /// A quantity burst's write.
    ChangeQuantity,
    /// A row's move to another board.
    MoveBoard,
    /// A star or unstar.
    ToggleMvp,
    /// A printing picked from the printing sheet.
    ChangePrinting,
    /// The reversing call of an Undo.
    Undo,
    /// A Quick add result's add, reported under the Quick add component.
    QuickAddCard,
}

impl Op {
    /// The telemetry name the client reports a failure under.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::ChangeQuantity => "change_quantity",
            Self::MoveBoard => "move_board",
            Self::ToggleMvp => "toggle_mvp",
            Self::ChangePrinting => "change_printing",
            Self::Undo => "undo",
            Self::QuickAddCard => "quick_add_card",
        }
    }
}

/// What floats over the Quick add input while the query is long enough to search.
#[derive(Debug, Clone, PartialEq)]
pub enum QuickAddResults {
    /// The search waits out its debounce or is out: one [`SEARCHING`] chip.
    Searching,
    /// The search found nothing: one [`NO_RESULTS`] chip.
    Empty,
    /// The cards found, in the search's order, each a chip.
    Found(Vec<Card>),
}

/// What a row draws from its entry: copies, board and star.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Row {
    /// Its copies.
    pub quantity: i32,
    /// Its board.
    pub board: Board,
    /// Whether it is starred.
    pub mvp: bool,
}

/// What a user does on the deck cards view, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Opens the screen for `deck_id` under the filter the client remembered for it, fetching the deck and its tokens.
    Load {
        /// The deck to show.
        deck_id: Uuid,
        /// The filter the client holds for this deck's cards.
        filter: Box<CardQueryBuilder>,
    },
    /// The profile arrived partway through the [`Effect::FetchDeck`] holding `ticket`, for the header.
    ProfileLoaded {
        /// The ticket from the effect this answers.
        ticket: Ticket,
        /// The deck's profile.
        profile: Box<DeckProfile>,
    },
    /// The [`Effect::FetchDeck`] holding `ticket` finished, with the command zone cards it resolved from [`zone_requests`].
    Loaded {
        /// The ticket from the effect this answers.
        ticket: Ticket,
        /// The deck.
        deck: Box<Deck>,
        /// The command zone cards it could fetch.
        zone: Box<Zone>,
    },
    /// The deck's tokens arrived for the [`Effect::FetchTokens`] holding `ticket`. A failed fetch hands back an empty list.
    TokensLoaded {
        /// The ticket from the effect this answers.
        ticket: Ticket,
        /// The tokens the deck's cards make.
        tokens: Vec<Card>,
    },
    /// The request holding `ticket` failed. The client has already reported it, and raised its error toast unless the request was quiet.
    Failed {
        /// The ticket from the request this answers.
        ticket: Ticket,
    },
    /// The request holding `ticket` found no deck: the server no longer has it. The client has already reported it and raised its toast.
    Gone {
        /// The ticket from the request this answers.
        ticket: Ticket,
    },
    /// The client's filter as it stands after an apply or an [`Effect::Refilter`]; the main board's groups follow it.
    Filter(Box<CardQueryBuilder>),
    /// Picks how the main board groups.
    PickGroupBy(GroupByOption),
    /// Turns a Boards chip on or off; turning the last one off turns Main back on.
    ToggleBoard(Board),
    /// Turns the tokens section on or off.
    ToggleTokens,
    /// Turns the command zone's pinned groups on or off.
    ToggleCommandZone,
    /// Opens or collapses the section with this key.
    ToggleSection(String),
    /// The fold-all button: collapses every section under `keys`, or opens every one when `collapsed` is false.
    SetSections {
        /// The sections on screen.
        keys: Vec<String>,
        /// Whether they end up collapsed.
        collapsed: bool,
    },
    /// The Quick add input reads `query`. A trimmed query of [`timing::QUICK_ADD_MIN_CHARS`] bytes or more waits out the debounce and searches; a shorter one closes the results.
    Type(String),
    /// The Quick add clear button: empties the input and the results.
    ClearQuickAdd,
    /// The debounce holding `ticket` is over.
    Debounced {
        /// The ticket from the [`Effect::Debounce`] this answers.
        ticket: Ticket,
    },
    /// The Quick add search holding `ticket` found these cards.
    Searched {
        /// The ticket from the [`Effect::SearchCards`] this answers.
        ticket: Ticket,
        /// The cards found.
        cards: Vec<Card>,
    },
    /// Adds a Quick add result to the main board. It clears the search and closes the results at once, and joins the deck when the server confirms.
    AddCard(Box<Card>),
    /// The [`Effect::CreateCard`] holding `ticket` created this deck card.
    Created {
        /// The ticket from the effect this answers.
        ticket: Ticket,
        /// The deck card the server created.
        deck_card: Box<DeckCard>,
    },
    /// The [`Effect::PatchCard`] holding `ticket` went through.
    Patched {
        /// The ticket from the effect this answers.
        ticket: Ticket,
        /// The deck card as the server holds it.
        deck_card: Box<DeckCard>,
    },
    /// The [`Effect::DeleteCard`] holding `ticket` went through.
    Deleted {
        /// The ticket from the effect this answers.
        ticket: Ticket,
    },
    /// The [`Effect::PatchProfile`] holding `ticket` went through.
    ProfilePatched {
        /// The ticket from the effect this answers.
        ticket: Ticket,
    },
    /// A row's quantity tap: `delta` copies on the card with this printing id. The row changes at once; the server hears the burst's net once taps stop for [`timing::QUANTITY_FLUSH`].
    ChangeQuantity {
        /// The card's printing id.
        card_id: Uuid,
        /// The copies to add, negative to take away.
        delta: i32,
    },
    /// Moves a row to another board.
    MoveTo {
        /// The card's printing id.
        card_id: Uuid,
        /// The board it moves to.
        board: Board,
    },
    /// Stars or unstars a row. `now` is the optimistic star time until the server's lands.
    ToggleMvp {
        /// The card's printing id.
        card_id: Uuid,
        /// The time now.
        now: DateTime<Utc>,
    },
    /// A row's printings button: the printing sheet opens on `card`, in the command zone `slot` or, with `None`, as a deck card.
    OpenPrintings {
        /// The card whose printings to list.
        card: Box<Card>,
        /// The command zone slot the card sits in, if any.
        slot: Option<Slot>,
    },
    /// The printing sheet saved this printing for the card it was opened on.
    PickPrinting(Box<Card>),
    /// The client popped this entry off the deck's undo log; Undo reverses it, or says [`ALREADY_CHANGED`] when the deck has moved past it.
    Undo(Box<UndoAction>),
    /// Clears the filter.
    ResetFilter,
    /// Goes back to where the user came from.
    Back,
    /// Opens Add cards.
    OpenAdd,
    /// Opens Remove cards.
    OpenRemove,
    /// The screen is going away: every burst still waiting writes at once.
    Leave,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone)]
pub enum Effect {
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Fetches the deck, then its profile quietly, sending [`Action::ProfileLoaded`] when that lands, then each card [`zone_requests`] names, quietly; answers with [`Action::Loaded`], or [`Action::Failed`] when the deck itself fails.
    FetchDeck {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck to fetch.
        deck_id: Uuid,
    },
    /// Fetches the deck's tokens quietly; answers with [`Action::TokensLoaded`], with an empty list on failure.
    FetchTokens {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck.
        deck_id: Uuid,
    },
    /// Waits `wait`, then answers with [`Action::Debounced`] carrying `ticket`.
    Debounce {
        /// The ticket to hand back.
        ticket: Ticket,
        /// How long to wait.
        wait: Duration,
    },
    /// Records a search for usage, then searches the deck's cards quietly; answers with [`Action::Searched`] or [`Action::Failed`].
    SearchCards {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck searched for.
        deck_id: Uuid,
        /// The search, from [`quick_add_query`].
        query: Box<CardQuery>,
    },
    /// Creates a deck card; answers with [`Action::Created`] or [`Action::Failed`].
    CreateCard {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck.
        deck_id: Uuid,
        /// The request.
        request: Box<HttpCreateDeckCard>,
        /// Which write this is.
        op: Op,
    },
    /// Patches a deck card; answers with [`Action::Patched`] or [`Action::Failed`].
    PatchCard {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck.
        deck_id: Uuid,
        /// The card's printing id.
        card_id: Uuid,
        /// The request.
        request: Box<HttpPatchDeckCard>,
        /// Which write this is.
        op: Op,
    },
    /// Deletes a deck card; answers with [`Action::Deleted`] or [`Action::Failed`].
    DeleteCard {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck.
        deck_id: Uuid,
        /// The card's printing id.
        card_id: Uuid,
        /// Which write this is.
        op: Op,
    },
    /// Patches the deck's profile; answers with [`Action::ProfilePatched`] or [`Action::Failed`].
    PatchProfile {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck.
        deck_id: Uuid,
        /// The request.
        request: Box<HttpUpdateDeckProfile>,
        /// Which write this is.
        op: Op,
    },
    /// Writes a burst the screen leaves behind, detached from the screen, with no reply: a [`Write::Delete`] deletes the card, a [`Write::Set`] patches its quantity. Reported as `flush_quantity`.
    Flush {
        /// The deck.
        deck_id: Uuid,
        /// The card's printing id.
        card_id: Uuid,
        /// What to write.
        write: Write,
    },
    /// Records a deliberate removal for suggestions: the deck and the removed card's oracle id.
    RecordRemoval {
        /// The deck.
        deck_id: Uuid,
        /// The removed card's oracle id, if it had one.
        oracle_id: Option<Uuid>,
    },
    /// Pushes this entry onto the deck's undo log.
    RecordUndo(Box<UndoAction>),
    /// Takes the newest [`UndoAction::Removed`] entry for this card off the undo log, after its delete failed.
    ForgetRemoval {
        /// The card's printing id.
        card_id: Uuid,
    },
    /// Clears the client's filter.
    ClearFilter,
    /// Bumps the client's filter counter: the filter sheet's sections reset their typed searches, and the client hands the filter back in with [`Action::Filter`].
    Refilter,
    /// Collapses the expanded row.
    Collapse,
    /// Opens the printing sheet on [`State::printing`].
    OpenPrintings,
    /// Goes to this screen.
    Navigate(Destination),
    /// Goes to this screen in place of the current one, so Back skips a screen with nothing left to show.
    Replace(Destination),
    /// Raises this toast.
    Toast(Toast),
}

/// Which of an action's effects raise a toast, for [`keep_one`].
fn toast_line(effect: &Effect) -> Option<Line<'_>> {
    match effect {
        Effect::Toast(toast) => Some(Line::Toast(toast)),
        _ => None,
    }
}

/// A burst of quantity taps on one card, waiting out its debounce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Burst {
    card_id: Uuid,
    /// The copies before the burst: the delete threshold and the rollback target.
    baseline: i32,
    /// The taps' net change.
    delta: i32,
    /// The newest tap's debounce.
    ticket: Ticket,
    /// Whether the newest tap records undo; undo-driven taps do not.
    record: bool,
}

impl Burst {
    /// The burst's absolute target, `None` when it leaves `i32`.
    fn target(&self) -> Option<i32> {
        self.baseline.checked_add(self.delta)
    }
}

/// A request waiting for its reply.
#[derive(Debug, Clone, PartialEq)]
enum Pending {
    Deck,
    Tokens,
    Search,
    QuickAdd(Box<Card>),
    /// A burst's delete. `removed` is the row it took off, to put back on failure.
    Delete {
        card_id: Uuid,
        removed: Option<Box<DeckEntry>>,
        baseline: i32,
    },
    /// A burst's quantity write.
    Quantity {
        card_id: Uuid,
        delta: i32,
        baseline: i32,
        record: bool,
    },
    Move {
        card_id: Uuid,
        from: Board,
    },
    Mvp {
        card_id: Uuid,
        before: Option<DateTime<Utc>>,
    },
    Printing {
        old: Box<Card>,
        new: Box<Card>,
    },
    ZonePrinting {
        slot: Slot,
        old: Box<Card>,
        new: Box<Card>,
    },
    /// Undo's re-create of a removed card.
    Recreate(Uuid),
    /// An undo call whose reply changes nothing.
    Quiet,
}

/// The Quick add input and its search.
#[derive(Debug, Default, Clone, PartialEq)]
struct QuickAdd {
    query: String,
    /// The edit whose debounce or search may still land.
    waiting: Option<Ticket>,
    searching: bool,
    open: bool,
    results: Vec<Card>,
}

/// The deck cards view for one visit. Starts loading with no deck; send [`Action::Load`] to start the fetches.
///
/// Every request and debounce takes a fresh [`Ticket`] from one [`Generation`] and waits in a list keyed by it, so each reply finds its own request, and a reply after a newer [`Action::Load`] changes nothing. The deck's entries are the source of truth for every list; edits change them at once and roll back when the server refuses. The main board's groups follow the filter last handed in with [`Action::Filter`]; the maybeboard, sideboard and tokens take the client's filter as it stands, as the sections did when they read it while drawing.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    deck_id: Option<Uuid>,
    loaded: bool,
    profile: Option<DeckProfile>,
    /// Every entry but the command zone's, on every board.
    entries: Vec<DeckEntry>,
    command_zone_cards: Vec<Card>,
    zone: Zone,
    /// The cards the filter sheet picks its values from, as the deck loaded.
    filter_cards: Vec<Card>,
    tokens: Option<Vec<Card>>,
    filter: CardQueryBuilder,
    group_by: GroupByOption,
    boards: Boards,
    show_tokens: bool,
    show_command_zone: bool,
    collapsed: HashSet<String>,
    quick: QuickAdd,
    bursts: Vec<Burst>,
    /// Rows a burst deleted at once, until the burst writes.
    removed_rows: Vec<DeckEntry>,
    printing: Option<(Card, Option<Slot>)>,
    waiting: Vec<(Ticket, Pending)>,
    hint: HintOffer,
    tickets: Generation,
}

/// Whether the visit has offered the one-time hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

impl Default for State {
    fn default() -> Self {
        Self {
            deck_id: None,
            loaded: false,
            profile: None,
            entries: Vec::new(),
            command_zone_cards: Vec::new(),
            zone: Zone::default(),
            filter_cards: Vec::new(),
            tokens: None,
            filter: CardQueryBuilder::default(),
            group_by: GroupByOption::CardType,
            boards: Boards::default(),
            show_tokens: false,
            show_command_zone: true,
            collapsed: HashSet::new(),
            quick: QuickAdd::default(),
            bursts: Vec::new(),
            removed_rows: Vec::new(),
            printing: None,
            waiting: Vec::new(),
            hint: HintOffer::Waiting,
            tickets: Generation::default(),
        }
    }
}

impl State {
    /// The screen as it first draws: loading, grouped by card type, Main on, the command zone shown and the tokens hidden.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// The hint is offered once per visit, the first time the deck holds a card. A reply whose request is no longer waiting changes nothing.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        let mut effects = self.apply(action);
        if self.hint == HintOffer::Waiting && !self.entries.is_empty() {
            self.hint = HintOffer::Offered;
            effects.push(Effect::OfferHint(OneTimeHint::DeckCards));
        }
        keep_one(&mut effects, toast_line);
        effects
    }

    fn apply(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load { deck_id, filter } => self.load(deck_id, *filter),
            Action::ProfileLoaded { ticket, profile } => {
                if self.is_waiting(ticket) {
                    self.profile = Some(*profile);
                }
                Vec::new()
            }
            Action::Loaded { ticket, deck, zone } => self.loaded(ticket, *deck, *zone),
            Action::TokensLoaded { ticket, tokens } => {
                if self.settle(ticket).is_some() {
                    self.tokens = Some(tokens);
                }
                Vec::new()
            }
            Action::Failed { ticket } => self.failed(ticket),
            Action::Gone { ticket } => self.gone(ticket),
            Action::Filter(filter) => {
                self.filter = *filter;
                Vec::new()
            }
            Action::PickGroupBy(option) => {
                self.group_by = option;
                vec![Effect::Collapse]
            }
            Action::ToggleBoard(board) => {
                self.boards.toggle(board);
                vec![Effect::Collapse, Effect::Refilter]
            }
            Action::ToggleTokens => {
                self.show_tokens = !self.show_tokens;
                Vec::new()
            }
            Action::ToggleCommandZone => {
                self.show_command_zone = !self.show_command_zone;
                Vec::new()
            }
            Action::ToggleSection(key) => {
                if !self.collapsed.remove(&key) {
                    self.collapsed.insert(key);
                }
                Vec::new()
            }
            Action::SetSections { keys, collapsed } => self.set_sections(keys, collapsed),
            Action::Type(query) => self.type_query(query),
            Action::ClearQuickAdd => {
                self.quick.results.clear();
                self.type_query(String::new())
            }
            Action::Debounced { ticket } => self.debounced(ticket),
            Action::Searched { ticket, cards } => {
                if self.settle(ticket).is_some() && self.quick.waiting == Some(ticket) {
                    self.quick.waiting = None;
                    self.quick.searching = false;
                    self.quick.results = cards;
                    self.quick.open = true;
                }
                Vec::new()
            }
            Action::AddCard(card) => self.add_card(*card),
            Action::Created { ticket, deck_card } => self.created(ticket, *deck_card),
            Action::Patched { ticket, deck_card } => self.patched(ticket, &deck_card),
            Action::Deleted { ticket } => {
                let _ = self.settle(ticket);
                Vec::new()
            }
            Action::ProfilePatched { ticket } => self.profile_patched(ticket),
            Action::ChangeQuantity { card_id, delta } => self.tap(card_id, delta, true),
            Action::MoveTo { card_id, board } => self.move_to(card_id, board, true),
            Action::ToggleMvp { card_id, now } => self.toggle_mvp(card_id, now),
            Action::OpenPrintings { card, slot } => {
                self.printing = Some((*card, slot));
                vec![Effect::OpenPrintings]
            }
            Action::PickPrinting(card) => self.pick_printing(*card),
            Action::Undo(entry) => self.undo(*entry),
            Action::ResetFilter => {
                self.filter.clear();
                vec![
                    Effect::ClearFilter,
                    Effect::Collapse,
                    Effect::Refilter,
                    Effect::Toast(Toast::quick(ToastKind::Info, FILTER_RESET)),
                ]
            }
            Action::Back => vec![Effect::Navigate(Destination::Previous)],
            Action::OpenAdd => self
                .deck_id
                .map(|id| Effect::Navigate(Destination::AddCards(id)))
                .into_iter()
                .collect(),
            Action::OpenRemove => self
                .deck_id
                .map(|id| Effect::Navigate(Destination::RemoveCards(id)))
                .into_iter()
                .collect(),
            Action::Leave => self.leave(),
        }
    }

    /// Collapses or opens every section under `keys`.
    fn set_sections(&mut self, keys: Vec<String>, collapsed: bool) -> Vec<Effect> {
        if collapsed {
            self.collapsed.extend(keys);
        } else {
            for key in &keys {
                self.collapsed.remove(key);
            }
        }
        Vec::new()
    }

    fn load(&mut self, deck_id: Uuid, filter: CardQueryBuilder) -> Vec<Effect> {
        let tickets = self.tickets;
        let hint = self.hint;
        *self = Self {
            deck_id: Some(deck_id),
            filter,
            hint,
            tickets,
            ..Self::default()
        };
        let deck = self.request(Pending::Deck);
        let tokens = self.request(Pending::Tokens);
        vec![
            Effect::FetchDeck {
                ticket: deck,
                deck_id,
            },
            Effect::FetchTokens {
                ticket: tokens,
                deck_id,
            },
        ]
    }

    fn loaded(&mut self, ticket: Ticket, deck: Deck, zone: Zone) -> Vec<Effect> {
        if self.settle(ticket).is_none() {
            return Vec::new();
        }
        let Deck {
            entries,
            command_zone_cards,
            ..
        } = deck;
        self.command_zone_cards = command_zone_cards;
        let mut entries = entries;
        for (_, card) in zone.cards() {
            if let Some(oracle_id) = card.scryfall_data.oracle_id {
                entries.retain(|e| e.card.scryfall_data.oracle_id != Some(oracle_id));
            }
        }
        self.zone = zone;
        self.filter_cards = entries.iter().map(|e| e.card.clone()).collect();
        self.entries = entries;
        self.loaded = true;
        let mut effects = Vec::new();
        if !self.filter.is_empty() {
            effects.push(Effect::Toast(Toast::quick(
                ToastKind::Warning,
                FILTER_ACTIVE,
            )));
        }
        effects.push(Effect::Refilter);
        effects
    }

    fn gone(&mut self, ticket: Ticket) -> Vec<Effect> {
        let mut effects = self.failed(ticket);
        effects.push(Effect::Replace(Destination::DeckList));
        effects
    }

    fn failed(&mut self, ticket: Ticket) -> Vec<Effect> {
        let Some(pending) = self.settle(ticket) else {
            return Vec::new();
        };
        match pending {
            Pending::Tokens => {
                self.tokens = Some(Vec::new());
                Vec::new()
            }
            Pending::Search => {
                if self.quick.waiting == Some(ticket) {
                    self.quick.waiting = None;
                    self.quick.searching = false;
                    self.quick.open = false;
                }
                Vec::new()
            }
            Pending::Delete {
                card_id,
                removed,
                baseline,
            } => {
                let Some(removed) = removed else {
                    return Vec::new();
                };
                if !self.has_entry(card_id) {
                    let mut restored = *removed;
                    if let Ok(quantity) = Quantity::new(baseline) {
                        restored.deck_card.quantity = quantity;
                    }
                    self.entries.push(restored);
                }
                vec![Effect::ForgetRemoval { card_id }, Effect::Refilter]
            }
            Pending::Quantity {
                card_id, baseline, ..
            } => {
                if let Some(entry) = self.entry_mut(card_id)
                    && let Ok(quantity) = Quantity::new(baseline)
                {
                    entry.deck_card.quantity = quantity;
                }
                vec![Effect::Refilter]
            }
            Pending::Move { card_id, from } => {
                if let Some(entry) = self.entry_mut(card_id) {
                    entry.deck_card.board = from;
                }
                vec![Effect::Refilter]
            }
            Pending::Mvp { card_id, before } => {
                if let Some(entry) = self.entry_mut(card_id) {
                    entry.deck_card.mvp_at = before;
                }
                vec![Effect::Refilter]
            }
            Pending::Recreate(card_id) => {
                self.entries.retain(|e| e.card.scryfall_data.id != card_id);
                vec![Effect::Refilter]
            }
            Pending::Deck
            | Pending::QuickAdd(_)
            | Pending::Printing { .. }
            | Pending::ZonePrinting { .. }
            | Pending::Quiet => Vec::new(),
        }
    }

    fn type_query(&mut self, query: String) -> Vec<Effect> {
        let long_enough = query.trim().len() >= timing::QUICK_ADD_MIN_CHARS;
        self.quick.query = query;
        let ticket = self.tickets.next_ticket();
        if !long_enough {
            self.quick.waiting = None;
            self.quick.open = false;
            self.quick.searching = false;
            return Vec::new();
        }
        self.quick.waiting = Some(ticket);
        self.quick.searching = true;
        self.quick.open = true;
        vec![Effect::Debounce {
            ticket,
            wait: timing::QUICK_ADD_DEBOUNCE,
        }]
    }

    fn debounced(&mut self, ticket: Ticket) -> Vec<Effect> {
        if let Some(at) = self.bursts.iter().position(|b| b.ticket == ticket) {
            let burst = self.bursts.remove(at);
            return self.write_burst(burst);
        }
        if self.quick.waiting != Some(ticket) {
            return Vec::new();
        }
        let (Some(deck_id), Ok(query)) = (self.deck_id, quick_add_query(&self.quick.query).build())
        else {
            self.quick.waiting = None;
            self.quick.searching = false;
            self.quick.open = false;
            return Vec::new();
        };
        self.waiting.push((ticket, Pending::Search));
        vec![Effect::SearchCards {
            ticket,
            deck_id,
            query: Box::new(query),
        }]
    }

    /// Empties Quick add's search and closes its results.
    fn close_quick_add(&mut self) {
        self.quick.query.clear();
        self.quick.results.clear();
        self.quick.waiting = None;
        self.quick.searching = false;
        self.quick.open = false;
    }

    fn add_card(&mut self, card: Card) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        self.close_quick_add();
        let request = HttpCreateDeckCard::new(&card.scryfall_data, 1, None);
        let ticket = self.request(Pending::QuickAdd(Box::new(card)));
        vec![Effect::CreateCard {
            ticket,
            deck_id,
            request: Box::new(request),
            op: Op::QuickAddCard,
        }]
    }

    fn created(&mut self, ticket: Ticket, deck_card: DeckCard) -> Vec<Effect> {
        match self.settle(ticket) {
            Some(Pending::QuickAdd(card)) => {
                let card_id = card.scryfall_data.id;
                let card_name = card.scryfall_data.name.clone();
                self.entries.push(DeckEntry {
                    card: *card,
                    deck_card,
                });
                vec![
                    Effect::Refilter,
                    Effect::RecordUndo(Box::new(UndoAction::Added {
                        card_id,
                        card_name: card_name.clone(),
                    })),
                    Effect::Toast(Toast::quick(ToastKind::Info, added(&card_name))),
                ]
            }
            Some(Pending::Recreate(card_id)) => {
                if let Some(entry) = self.entry_mut(card_id) {
                    entry.deck_card = deck_card;
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn patched(&mut self, ticket: Ticket, deck_card: &DeckCard) -> Vec<Effect> {
        match self.settle(ticket) {
            Some(Pending::Quantity {
                card_id,
                delta,
                baseline,
                record: true,
            }) => self
                .entry(card_id)
                .map(|entry| {
                    Effect::RecordUndo(Box::new(UndoAction::QuantityChanged {
                        card_id,
                        card_name: entry.card.scryfall_data.name.clone(),
                        net: delta,
                        baseline,
                    }))
                })
                .into_iter()
                .collect(),
            Some(Pending::Mvp { card_id, .. }) => {
                if let Some(entry) = self.entry_mut(card_id) {
                    entry.deck_card.mvp_at = deck_card.mvp_at;
                }
                Vec::new()
            }
            Some(Pending::Printing { old, new }) => {
                let old_id = old.scryfall_data.id;
                let new_id = new.scryfall_data.id;
                if let Some(entry) = self.entry_mut(old_id) {
                    entry.card = (*new).clone();
                    entry.deck_card.scryfall_data_id = new_id;
                }
                self.printing = Some((*new, None));
                vec![
                    Effect::RecordUndo(Box::new(UndoAction::PrintingChanged {
                        old_card: old,
                        new_id,
                    })),
                    Effect::Refilter,
                ]
            }
            _ => Vec::new(),
        }
    }

    fn profile_patched(&mut self, ticket: Ticket) -> Vec<Effect> {
        let Some(Pending::ZonePrinting { slot, old, new }) = self.settle(ticket) else {
            return Vec::new();
        };
        let new_id = new.scryfall_data.id;
        *self.zone.slot_mut(slot) = Some((*new).clone());
        self.printing = Some((*new, Some(slot)));
        vec![
            Effect::RecordUndo(Box::new(UndoAction::CommandZonePrintingChanged {
                slot,
                old_card: old,
                new_id,
            })),
            Effect::Refilter,
        ]
    }

    /// A quantity tap: the row changes at once and the card's burst re-arms its debounce. `record` is false for undo's inverse tap, which records no entry.
    fn tap(&mut self, card_id: Uuid, delta: i32, record: bool) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let current = self.quantity(card_id);
        let burst_at = self.bursts.iter().position(|b| b.card_id == card_id);
        let net = burst_at
            .and_then(|at| self.bursts.get(at))
            .map_or(0, |b| b.delta);
        let (Some(target), Some(folded)) = (current.checked_add(delta), net.checked_add(delta))
        else {
            tracing::warn!(%card_id, current, delta, "quantity tap leaves i32, dropped");
            return Vec::new();
        };
        let before_lands = deck_signals::land_count(&self.entries);
        let before_price = self.total_price();
        let mut effects = Vec::new();

        if target < 1 {
            let removed = self.entry(card_id).cloned();
            effects.push(Effect::RecordRemoval {
                deck_id,
                oracle_id: removed
                    .as_ref()
                    .and_then(|e| e.card.scryfall_data.oracle_id),
            });
            if let Some(entry) = removed {
                self.removed_rows
                    .retain(|e| e.card.scryfall_data.id != card_id);
                self.removed_rows.push(entry.clone());
                if record {
                    let baseline = burst_at
                        .and_then(|at| self.bursts.get(at))
                        .map_or(current, |b| b.baseline);
                    effects.push(Effect::RecordUndo(Box::new(UndoAction::Removed {
                        entry: Box::new(entry),
                        baseline,
                    })));
                }
            }
            self.entries.retain(|e| e.card.scryfall_data.id != card_id);
            effects.push(Effect::Refilter);
            effects.extend(self.crossings(before_lands, before_price));
            effects.push(Effect::Toast(Toast::quick(ToastKind::Info, CARD_REMOVED)));
        } else {
            if let Some(entry) = self.entry_mut(card_id)
                && let Ok(quantity) = Quantity::new(target)
            {
                entry.deck_card.quantity = quantity;
            }
            effects.push(Effect::Refilter);
            effects.extend(self.crossings(before_lands, before_price));
        }

        let ticket = self.tickets.next_ticket();
        if let Some(burst) = burst_at.and_then(|at| self.bursts.get_mut(at)) {
            burst.delta = folded;
            burst.ticket = ticket;
            burst.record = record;
        } else {
            self.bursts.push(Burst {
                card_id,
                baseline: current,
                delta: folded,
                ticket,
                record,
            });
        }
        effects.push(Effect::Debounce {
            ticket,
            wait: timing::QUANTITY_FLUSH,
        });
        effects
    }

    /// The land and budget toasts for a change from `before_lands` and `before_price` to the deck as it stands.
    fn crossings(&self, before_lands: i32, before_price: f64) -> Vec<Effect> {
        let target = self.profile.as_ref().and_then(|p| p.land_target);
        let budget = self.profile.as_ref().and_then(|p| p.price_target);
        let lands = deck_signals::land_crossing(
            target,
            before_lands,
            deck_signals::land_count(&self.entries),
            Crossing::Both,
        );
        let price = deck_signals::budget_crossing(
            budget,
            self.currency(),
            before_price,
            self.total_price(),
        );
        [lands, price]
            .into_iter()
            .flatten()
            .map(|(level, text)| {
                let kind = match level {
                    Level::Info => ToastKind::Info,
                    Level::Warning => ToastKind::Warning,
                };
                Effect::Toast(Toast::normal(kind, text))
            })
            .collect()
    }

    fn total_price(&self) -> f64 {
        deck_metrics::deck_price(&self.entries, &self.command_zone_cards, self.currency())
    }

    /// The write a burst makes once its debounce is over.
    fn write_burst(&mut self, burst: Burst) -> Vec<Effect> {
        let removed = self
            .removed_rows
            .iter()
            .position(|e| e.card.scryfall_data.id == burst.card_id)
            .map(|at| self.removed_rows.remove(at));
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        if burst.delta == 0 {
            return Vec::new();
        }
        let Some(target) = burst.target() else {
            return Vec::new();
        };
        let card_id = burst.card_id;
        if target < 1 {
            let ticket = self.request(Pending::Delete {
                card_id,
                removed: removed.map(Box::new),
                baseline: burst.baseline,
            });
            vec![Effect::DeleteCard {
                ticket,
                deck_id,
                card_id,
                op: Op::ChangeQuantity,
            }]
        } else {
            let ticket = self.request(Pending::Quantity {
                card_id,
                delta: burst.delta,
                baseline: burst.baseline,
                record: burst.record,
            });
            vec![Effect::PatchCard {
                ticket,
                deck_id,
                card_id,
                request: Box::new(HttpPatchDeckCard::new(Some(target), None)),
                op: Op::ChangeQuantity,
            }]
        }
    }

    fn leave(&mut self) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        self.bursts
            .drain(..)
            .filter(|b| b.delta != 0)
            .filter_map(|b| {
                let target = b.target()?;
                let write = if target < 1 {
                    Write::Delete
                } else {
                    Write::Set(target)
                };
                Some(Effect::Flush {
                    deck_id,
                    card_id: b.card_id,
                    write,
                })
            })
            .collect()
    }

    /// A move to `board`. `record` is false for undo's move back, which records no entry and raises no toast of its own.
    fn move_to(&mut self, card_id: Uuid, board: Board, record: bool) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let (from, card_name) = self
            .entry(card_id)
            .map_or((Board::Deck, String::new()), |e| {
                (e.deck_card.board, e.card.scryfall_data.name.clone())
            });
        let mut effects = Vec::new();
        if record {
            effects.push(Effect::RecordUndo(Box::new(UndoAction::MovedBoard {
                card_id,
                card_name,
                from,
            })));
        }
        if let Some(entry) = self.entry_mut(card_id) {
            entry.deck_card.board = board;
        }
        effects.push(Effect::Refilter);
        let ticket = self.request(Pending::Move { card_id, from });
        effects.push(Effect::PatchCard {
            ticket,
            deck_id,
            card_id,
            request: Box::new(HttpPatchDeckCard::new(
                None,
                Some(board.display_name().to_owned()),
            )),
            op: Op::MoveBoard,
        });
        if record {
            effects.push(Effect::Toast(Toast::quick(
                ToastKind::Info,
                moved_to(board),
            )));
        }
        effects
    }

    fn toggle_mvp(&mut self, card_id: Uuid, now: DateTime<Utc>) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let before = self.entry(card_id).and_then(|e| e.deck_card.mvp_at);
        let star = before.is_none();
        if let Some(entry) = self.entry_mut(card_id) {
            entry.deck_card.mvp_at = star.then_some(now);
        }
        let ticket = self.request(Pending::Mvp { card_id, before });
        vec![
            Effect::Refilter,
            Effect::PatchCard {
                ticket,
                deck_id,
                card_id,
                request: Box::new(HttpPatchDeckCard::with_mvp(star)),
                op: Op::ToggleMvp,
            },
        ]
    }

    fn pick_printing(&mut self, new: Card) -> Vec<Effect> {
        let (Some(deck_id), Some((old, slot))) = (self.deck_id, self.printing.clone()) else {
            return Vec::new();
        };
        let new_id = new.scryfall_data.id;
        if let Some(slot) = slot {
            let ticket = self.request(Pending::ZonePrinting {
                slot,
                old: Box::new(old),
                new: Box::new(new),
            });
            vec![Effect::PatchProfile {
                ticket,
                deck_id,
                request: Box::new(slot.update_request(Some(new_id))),
                op: Op::ChangePrinting,
            }]
        } else {
            let card_id = old.scryfall_data.id;
            let ticket = self.request(Pending::Printing {
                old: Box::new(old),
                new: Box::new(new),
            });
            vec![Effect::PatchCard {
                ticket,
                deck_id,
                card_id,
                request: Box::new(HttpPatchDeckCard::with_printing(&new_id.to_string())),
                op: Op::ChangePrinting,
            }]
        }
    }

    /// Reverses a popped undo entry. A stale entry is consumed with [`ALREADY_CHANGED`] instead of a doomed request.
    fn undo(&mut self, action: UndoAction) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let stale = deck_cards::is_stale(&action, &self.entries, &self.zone)
            || matches!(action, UndoAction::QuantityChanged { net, .. } if net.checked_neg().is_none());
        if stale {
            return vec![Effect::Toast(Toast::quick(
                ToastKind::Info,
                ALREADY_CHANGED,
            ))];
        }
        let toast = Effect::Toast(Toast::quick(
            ToastKind::Info,
            deck_cards::undo_message(&action),
        ));
        match action {
            UndoAction::Added { card_id, .. } => {
                self.entries.retain(|e| e.card.scryfall_data.id != card_id);
                let ticket = self.request(Pending::Quiet);
                vec![
                    Effect::Refilter,
                    toast,
                    Effect::DeleteCard {
                        ticket,
                        deck_id,
                        card_id,
                        op: Op::Undo,
                    },
                ]
            }
            UndoAction::Removed { entry, baseline } => {
                let mut effects = vec![Effect::Refilter, toast];
                effects.push(self.re_add(deck_id, *entry, baseline));
                effects
            }
            UndoAction::QuantityChanged { card_id, net, .. } => {
                let mut effects = net
                    .checked_neg()
                    .map_or_default(|inverse| self.tap(card_id, inverse, false));
                effects.push(toast);
                effects
            }
            UndoAction::MovedBoard { card_id, from, .. } => {
                let mut effects = self.move_to(card_id, from, false);
                effects.push(toast);
                effects
            }
            UndoAction::PrintingChanged { old_card, new_id } => {
                let old_id = old_card.scryfall_data.id;
                if let Some(entry) = self.entry_mut(new_id) {
                    entry.card = *old_card;
                    entry.deck_card.scryfall_data_id = old_id;
                }
                let ticket = self.request(Pending::Quiet);
                vec![
                    Effect::Refilter,
                    toast,
                    Effect::PatchCard {
                        ticket,
                        deck_id,
                        card_id: new_id,
                        request: Box::new(HttpPatchDeckCard::with_printing(&old_id.to_string())),
                        op: Op::Undo,
                    },
                ]
            }
            UndoAction::CommandZonePrintingChanged { slot, old_card, .. } => {
                let request = slot.update_request(Some(old_card.scryfall_data.id));
                *self.zone.slot_mut(slot) = Some(*old_card);
                let ticket = self.request(Pending::Quiet);
                vec![
                    Effect::Refilter,
                    toast,
                    Effect::PatchProfile {
                        ticket,
                        deck_id,
                        request: Box::new(request),
                        op: Op::Undo,
                    },
                ]
            }
        }
    }

    /// Undo's re-add of a removed row at `baseline` copies: the row comes back at once and leaves again if the server refuses.
    fn re_add(&mut self, deck_id: Uuid, entry: DeckEntry, baseline: i32) -> Effect {
        let card_id = entry.card.scryfall_data.id;
        let request = HttpCreateDeckCard::new(
            &entry.card.scryfall_data,
            baseline,
            create_board_field(entry.deck_card.board),
        )
        .with_mvp_at(entry.deck_card.mvp_at);
        let mut restored = entry;
        if let Ok(quantity) = Quantity::new(baseline) {
            restored.deck_card.quantity = quantity;
        }
        self.entries.push(restored);
        let ticket = self.request(Pending::Recreate(card_id));
        Effect::CreateCard {
            ticket,
            deck_id,
            request: Box::new(request),
            op: Op::Undo,
        }
    }

    /// Takes a ticket for `pending` and puts it on the waiting list.
    fn request(&mut self, pending: Pending) -> Ticket {
        let ticket = self.tickets.next_ticket();
        self.waiting.push((ticket, pending));
        ticket
    }

    fn is_waiting(&self, ticket: Ticket) -> bool {
        self.waiting.iter().any(|(t, _)| *t == ticket)
    }

    /// Takes the request holding `ticket` off the waiting list, if it is there.
    fn settle(&mut self, ticket: Ticket) -> Option<Pending> {
        let at = self.waiting.iter().position(|(t, _)| *t == ticket)?;
        Some(self.waiting.remove(at).1)
    }

    fn entry(&self, card_id: Uuid) -> Option<&DeckEntry> {
        self.entries
            .iter()
            .find(|e| e.card.scryfall_data.id == card_id)
    }

    fn entry_mut(&mut self, card_id: Uuid) -> Option<&mut DeckEntry> {
        self.entries
            .iter_mut()
            .find(|e| e.card.scryfall_data.id == card_id)
    }

    fn has_entry(&self, card_id: Uuid) -> bool {
        self.entry(card_id).is_some()
    }

    /// The deck being shown, once [`Action::Load`] has named it.
    #[must_use]
    pub fn deck_id(&self) -> Option<Uuid> {
        self.deck_id
    }

    /// Whether the deck has loaded. Until then the controls and the list draw as skeletons, and a failed load leaves them so.
    #[must_use]
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// The deck's profile, for the header, once it has arrived.
    #[must_use]
    pub fn profile(&self) -> Option<&DeckProfile> {
        self.profile.as_ref()
    }

    /// The currency the deck's budget and the rows' prices use: the budget's, else US dollars.
    #[must_use]
    pub fn currency(&self) -> PriceCurrency {
        self.profile
            .as_ref()
            .and_then(|p| p.price_target_currency)
            .unwrap_or(PriceCurrency::Usd)
    }

    /// The cards the filter sheet picks its values from, as the deck loaded.
    #[must_use]
    pub fn filter_cards(&self) -> &[Card] {
        &self.filter_cards
    }

    /// The featured strip: the command zone with its roles, then the main board's MVPs, oldest star first.
    #[must_use]
    pub fn featured(&self) -> Vec<(Card, &'static str)> {
        let format = self.profile.as_ref().and_then(|p| p.format);
        let mut featured: Vec<(Card, &'static str)> = self
            .zone
            .cards()
            .into_iter()
            .map(|(slot, card)| (card.clone(), slot.label(format)))
            .collect();
        let mut mvps: Vec<&DeckEntry> = self
            .entries
            .iter()
            .filter(|e| e.deck_card.mvp_at.is_some() && e.deck_card.board == Board::Deck)
            .collect();
        mvps.sort_by_key(|e| e.deck_card.mvp_at);
        featured.extend(mvps.into_iter().map(|e| (e.card.clone(), MVP)));
        featured
    }

    /// How the main board groups.
    #[must_use]
    pub fn group_by(&self) -> GroupByOption {
        self.group_by
    }

    /// Which boards list.
    #[must_use]
    pub fn boards(&self) -> Boards {
        self.boards
    }

    /// Whether a Boards chip is on.
    #[must_use]
    pub fn board_on(&self, board: Board) -> bool {
        match board {
            Board::Deck => self.boards.main,
            Board::Maybeboard => self.boards.maybe,
            Board::Sideboard => self.boards.side,
        }
    }

    /// Whether a Boards chip can be tapped: Main always, the others while their board holds a card or the chip is on, so a chip never sticks on with its board empty.
    #[must_use]
    pub fn board_enabled(&self, board: Board) -> bool {
        board == Board::Deck
            || self.board_on(board)
            || self.entries.iter().any(|e| e.deck_card.board == board)
    }

    /// Whether the tokens section shows.
    #[must_use]
    pub fn show_tokens(&self) -> bool {
        self.show_tokens
    }

    /// Whether the command zone's pinned groups show.
    #[must_use]
    pub fn show_command_zone(&self) -> bool {
        self.show_command_zone
    }

    /// Whether the Show row offers the Command zone chip: the deck's format has a command zone, or
    /// the zone holds a card anyway. See [`Zone::offers_chip`].
    #[must_use]
    pub fn offers_command_zone(&self) -> bool {
        self.zone
            .offers_chip(self.profile.as_ref().and_then(|p| p.format))
    }

    /// Whether the section with this key is collapsed.
    #[must_use]
    pub fn is_collapsed(&self, key: &str) -> bool {
        self.collapsed.contains(key)
    }

    /// The keys of the collapsed sections.
    #[must_use]
    pub fn collapsed(&self) -> &HashSet<String> {
        &self.collapsed
    }

    /// The Stacks view's stacks for `display`, the List's sections in its order and under its fold
    /// keys: the tokens, the maybeboard and sideboard, the command zone, then the main board's
    /// groups and lands, each left out where the List leaves it out. `live` sorts the tokens as
    /// [`State::tokens`] does.
    #[must_use]
    pub fn stack_groups(&self, display: &Display, live: &CardQueryBuilder) -> Vec<StackGroup> {
        let mut out: Vec<StackGroup> = self.token_stack(live).into_iter().collect();
        out.extend(self.board_stacks(display));
        if self.show_command_zone {
            out.extend(self.zone_stacks(display));
        }
        if self.board_on(Board::Deck) {
            out.extend(self.main_stacks(display));
        }
        out
    }

    /// The tokens' stack, while they show and there are any.
    fn token_stack(&self, live: &CardQueryBuilder) -> Option<StackGroup> {
        let tokens = self
            .tokens(live)
            .filter(|t| self.show_tokens && !t.is_empty())?;
        Some(StackGroup {
            key: TOKENS_KEY.to_owned(),
            title: tokens_title(tokens.len()),
            pips: None,
            rows: tokens
                .into_iter()
                .map(|card| RowInfo {
                    card,
                    qty: 1,
                    kind: RowKind::Token,
                })
                .collect(),
        })
    }

    /// The maybeboard's and sideboard's stacks, each while its board shows and holds a card.
    fn board_stacks(&self, display: &Display) -> Vec<StackGroup> {
        [
            (Board::Maybeboard, MAYBEBOARD_KEY, &display.maybe),
            (Board::Sideboard, SIDEBOARD_KEY, &display.side),
        ]
        .into_iter()
        .filter(|(board, _, entries)| self.board_on(*board) && !entries.is_empty())
        .map(|(board, key, entries)| StackGroup {
            key: key.to_owned(),
            title: match board {
                Board::Sideboard => sideboard_title(entries.len()),
                _ => maybeboard_title(entries.len()),
            },
            pips: None,
            rows: entries
                .iter()
                .map(|entry| RowInfo {
                    card: entry.card.clone(),
                    qty: *entry.deck_card.quantity,
                    kind: RowKind::Board(board),
                })
                .collect(),
        })
        .collect()
    }

    /// The command zone's stacks: the commander with its partner, the background and the
    /// signature spell, each with a card in it.
    fn zone_stacks(&self, display: &Display) -> Vec<StackGroup> {
        let format = self.profile().and_then(|p| p.format);
        let has_partner = display.zone.iter().any(|(slot, _)| *slot == Slot::Partner);
        [
            (Slot::Commander, &[Slot::Commander, Slot::Partner][..]),
            (Slot::Background, &[Slot::Background][..]),
            (Slot::SignatureSpell, &[Slot::SignatureSpell][..]),
        ]
        .into_iter()
        .filter_map(|(slot, slots)| {
            let rows: Vec<RowInfo> = display
                .zone
                .iter()
                .filter(|(held, _)| slots.contains(held))
                .map(|(held, card)| RowInfo {
                    card: card.clone(),
                    qty: 1,
                    kind: RowKind::Zone(*held),
                })
                .collect();
            (!rows.is_empty()).then(|| StackGroup {
                key: slot.group_key().to_owned(),
                title: slot.group_header(format, has_partner).to_owned(),
                pips: None,
                rows,
            })
        })
        .collect()
    }

    /// The main board's stacks: its groups, then its lands when it has any.
    fn main_stacks(&self, display: &Display) -> Vec<StackGroup> {
        let main_row = |card: &Card| {
            let row = self.row(card.scryfall_data.id);
            RowInfo {
                card: card.clone(),
                qty: row.map_or(1, |r| r.quantity),
                kind: RowKind::Main {
                    mvp: row.is_some_and(|r| r.mvp),
                },
            }
        };
        let mut out: Vec<StackGroup> = display
            .groups
            .iter()
            .map(|group| StackGroup {
                key: group.key.clone(),
                title: group_title(&group.label, self.copies(&group.cards)),
                pips: group.pips.clone(),
                rows: group.cards.iter().map(main_row).collect(),
            })
            .collect();
        if !display.lands.is_empty() {
            out.push(StackGroup {
                key: LANDS_KEY.to_owned(),
                title: lands_title(self.copies(&display.lands)),
                pips: None,
                rows: display.lands.iter().map(main_row).collect(),
            });
        }
        out
    }

    /// Whether every section under `keys` is collapsed, for the fold-all button's state. False with no keys.
    #[must_use]
    pub fn all_collapsed(&self, keys: &[String]) -> bool {
        !keys.is_empty() && keys.iter().all(|k| self.collapsed.contains(k))
    }

    /// The lists after the filter and group pass. The main board's groups and lands and the command zone follow the applied filter; the maybeboard and sideboard follow `live`, the client's filter as it stands.
    #[must_use]
    pub fn display(&self, live: &CardQueryBuilder) -> Display {
        let mut display = deck_cards::pass(&self.entries, &self.zone, &self.filter, self.group_by);
        if *live != self.filter {
            let boards = deck_cards::pass(&self.entries, &Zone::default(), live, self.group_by);
            display.maybe = boards.maybe;
            display.side = boards.side;
        }
        display
    }

    /// The deck's tokens sorted as `live` sorts, Name when unsorted; `None` until they arrive.
    #[must_use]
    pub fn tokens(&self, live: &CardQueryBuilder) -> Option<Vec<Card>> {
        self.tokens.as_ref().map(|tokens| {
            Cards::from(tokens.clone())
                .sorted(live.sort().unwrap_or(CardSortKey::Name), live.direction())
                .into()
        })
    }

    /// What the row for this printing id draws, from the first entry holding it.
    #[must_use]
    pub fn row(&self, card_id: Uuid) -> Option<Row> {
        self.entry(card_id).map(|e| Row {
            quantity: *e.deck_card.quantity,
            board: e.deck_card.board,
            mvp: e.deck_card.mvp_at.is_some(),
        })
    }

    /// The copies of this printing in the deck, one when it has no entry.
    #[must_use]
    pub fn quantity(&self, card_id: Uuid) -> i32 {
        self.row(card_id).map_or(1, |r| r.quantity)
    }

    /// The copies across `cards`, each counted as [`State::quantity`] counts it, for a section's header.
    #[must_use]
    pub fn copies(&self, cards: &[Card]) -> i32 {
        cards.iter().fold(0, |total, c| {
            total.saturating_add(self.quantity(c.scryfall_data.id))
        })
    }

    /// Whether [`NO_CARDS`] shows: Main is on, the deck has loaded and the main board shows nothing.
    #[must_use]
    pub fn no_cards(&self, display: &Display) -> bool {
        self.boards.main && self.loaded && display.groups.is_empty() && display.lands.is_empty()
    }

    /// The Quick add input's text.
    #[must_use]
    pub fn quick_query(&self) -> &str {
        &self.quick.query
    }

    /// What floats over the Quick add input, or `None` while it is closed.
    #[must_use]
    pub fn quick_add(&self) -> Option<QuickAddResults> {
        if !self.quick.open {
            return None;
        }
        Some(if self.quick.searching {
            QuickAddResults::Searching
        } else if self.quick.results.is_empty() {
            QuickAddResults::Empty
        } else {
            QuickAddResults::Found(self.quick.results.clone())
        })
    }

    /// The card the printing sheet was last opened on, or moved to by a save.
    #[must_use]
    pub fn printing(&self) -> Option<&Card> {
        self.printing.as_ref().map(|(card, _)| card)
    }
}
