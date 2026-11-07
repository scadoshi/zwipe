//! Add cards: swipe through cards to build one deck.
//!
//! Two sources feed the pile. Search serves the deck-aware card search a page at a time: right adds the card, left skips it for a week, up sends it to the maybeboard, and down undoes the last swipe. Maybeboard cycles through the deck's maybeboard: right moves the card to the deck and left cycles past it. Synergy narrows Search to the commander's pool. Each source keeps its own card filter.
//!
//! The model holds both stacks and their cursors, the deck as Add sees it, paging, the land and budget crossings, adds still in flight, and every toast but a request's failure. The card filter is the client's: the shared filter sheet edits it, and the client hands it in with [`Action::Filter`] when it is applied. The undo log the deck cards view walks also belongs to the client, so the model asks for an entry back with [`Effect::TakeUndo`] and records one with [`Effect::RecordUndo`]. Gestures, the card's exit and entrance motion, image loading, the filter sheet, the details and printings sheets stay in the client.

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::query::CardQuery;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::deck::{Board, Deck, DeckCard, DeckEntry, Quantity};
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_core::http::contracts::deck::HttpDeckCardSearch;
use zwipe_core::http::contracts::deck_card::{HttpCreateDeckCard, HttpPatchDeckCard};
use zwipe_core::http::contracts::metrics::PileEndKind;

use crate::add::{self, AddDeck};
use crate::deck_signals::{self, Crossing, Level};
use crate::filter::{self, AddContext};
use crate::generation::{Generation, Ticket};
use crate::stack::{
    AddAction, CARDS_WARNING_THRESHOLD, CardStack, Direction, MAX_CARDS_IN_STACK, MaybeboardAction,
    NO_CARDS_LEFT, STACK_WARNING_EVERY, StackAction,
};
use crate::stores::in_flight::{AddOutcome, InFlightAdds};
use crate::stores::parked::ParkedStack;
use crate::stores::undo::{self, UndoAction};
use crate::timing;
use crate::toast::{Line, Toast, ToastKind, keep_one};

/// The screen's title.
pub const TITLE: &str = "Add";

/// The hint's title.
pub const HINT_TITLE: &str = "Swipe to build";

/// The label in front of the source chips.
pub const FROM: &str = "From";

/// The chip that narrows Search to the commander's synergy pool.
pub const SYNERGY: &str = "Synergy";

/// The Back button, to where the user came from.
pub const BACK: &str = "Back";

/// The button that opens the filter sheet.
pub const FILTER: &str = "Filter";

/// The button that searches again, or reloads the maybeboard.
pub const REFRESH: &str = "Refresh";

/// The printings sheet's toast once a printing is picked.
pub const PRINTING_SWAPPED: &str = "Printing swapped";

/// The toast for a left swipe.
pub const SKIPPED: &str = "Skipped";

/// The toast for a right swipe on Search.
pub const ADDED_TO_DECK: &str = "Added to deck";

/// The toast for an up swipe on Search.
pub const ADDED_TO_MAYBEBOARD: &str = "Added to maybeboard";

/// The toast for a right swipe on Maybeboard.
pub const MOVED_TO_DECK: &str = "Moved to deck";

/// The toast for an up swipe on Maybeboard.
pub const ALREADY_ON_MAYBEBOARD: &str = "Already on maybeboard";

/// The toast for an undo with no swipe to undo.
pub const NOTHING_TO_UNDO: &str = "Nothing to undo";

/// The toast for an undo on the first card.
pub const NO_PREVIOUS_CARD: &str = "No previous card";

/// The toast for undoing a skip.
pub const UNDID_SKIP: &str = "Undid skip";

/// The toast for undoing an add.
pub const UNDID_ADD: &str = "Undid add";

/// The toast for undoing an up swipe on Search.
pub const UNDID_MAYBEBOARD: &str = "Undid maybeboard";

/// The toast once an undone move to the deck lands.
pub const UNDID_MOVE_TO_DECK: &str = "Undid move to deck";

/// The toast for undoing a move the deck cards view's Undo already reversed.
pub const ALREADY_CHANGED: &str = "Already changed";

/// The toast when paging stops at the stack's cap.
pub const STACK_LIMIT_REACHED: &str = "Card stack limit reached, start a new search to continue";

/// The toast as the stack nears its cap.
pub const STACK_LIMIT_NEAR: &str =
    "Approaching the card stack limit, start a new search to continue";

/// The toast when Search has nothing to serve without a filter.
pub const TRY_A_FILTER: &str = "Try a filter to start swiping";

/// The toast when the Synergy toggle turns synergy on.
pub const SYNERGY_ON: &str = "Synergy on";

/// The toast when the Synergy toggle turns synergy off.
pub const SYNERGY_OFF: &str = "Synergy off";

/// The toast when the search falls back to the full pool while synergy warms up.
pub const SYNERGY_WARMING: &str = "Synergy warming up; showing all cards for now";

/// The toast when the first card past the synergy pool reaches the top.
pub const SYNERGY_SPENT: &str = "That's every synergy pick. Showing cards like your deck";

/// The toast for a Reset with nothing to reset.
pub const FILTER_AT_DEFAULT: &str = "Filter already at default";

/// The toast for a Reset that staged the default.
pub const FILTER_RESET: &str = "Filter reset";

/// The toast for Refresh on Search with a filter that can't search.
pub const FILTER_EMPTY: &str = "Filter is empty";

/// The toast for Refresh on Search.
pub const STACK_REFRESHED: &str = "Stack refreshed";

/// The toast for Refresh on Maybeboard.
pub const MAYBEBOARD_REFRESHED: &str = "Maybeboard refreshed";

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// Wherever the user came from, from Back.
    Previous,
    /// The deck list, in place of a screen whose deck is gone.
    DeckList,
}

/// Which pile the screen swipes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Source {
    /// The deck-aware card search, the pick on entry.
    #[default]
    Search,
    /// The deck's maybeboard.
    Maybeboard,
}

impl Source {
    /// The chips in the order they draw.
    pub const ALL: [Self; 2] = [Self::Search, Self::Maybeboard];

    /// The chip's words.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Search => "Search",
            Self::Maybeboard => "Maybe",
        }
    }

    /// The other source: what a tap on the one-chip source toggle switches to.
    #[must_use]
    pub fn toggled(self) -> Self {
        match self {
            Self::Search => Self::Maybeboard,
            Self::Maybeboard => Self::Search,
        }
    }
}

/// What the pile area draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pile {
    /// Cards to swipe: [`State::window`] and [`State::current`].
    Cards,
    /// A loading skeleton: a search or the next page is on its way.
    Loading,
    /// The cursor sits past the last card; a down pull on the skeleton undoes.
    End,
    /// A still skeleton: nothing served.
    Empty,
}

/// The undo log entry an undo asks the client to take back, found by the swipe it reverses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UndoClaim {
    /// The entry recorded when this card was added, to the deck or the maybeboard.
    Added {
        /// The added card's printing ID.
        card_id: Uuid,
    },
    /// The entry recorded when this card moved off `from`.
    MovedBoard {
        /// The moved card's printing ID.
        card_id: Uuid,
        /// The board it left.
        from: Board,
    },
}

impl UndoClaim {
    /// Whether `action` is the entry this claim takes, for [`crate::stores::undo::UndoStore::take_newest`].
    #[must_use]
    pub fn matches(&self, action: &UndoAction) -> bool {
        match (self, action) {
            (Self::Added { card_id }, UndoAction::Added { card_id: added, .. }) => added == card_id,
            (
                Self::MovedBoard { card_id, from },
                UndoAction::MovedBoard {
                    card_id: moved,
                    from: left,
                    ..
                },
            ) => moved == card_id && left == from,
            _ => false,
        }
    }
}

/// What a user does on Add cards, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Opens the screen for `deck_id`, offering the hint once per visit. `filter` is the Search filter the client remembered for this deck, and `parked` the stack it parked here last, which resumes when its filter still matches.
    Load {
        /// The deck being built.
        deck_id: Uuid,
        /// The remembered Search filter, if any.
        filter: Option<CardQueryBuilder>,
        /// The stack parked for this deck, if any.
        parked: Option<Box<ParkedStack>>,
    },
    /// The deck arrived for the [`Effect::FetchDeck`] or [`Effect::RefreshMaybeboard`] holding `ticket`.
    DeckLoaded {
        /// The ticket from the effect this answers.
        ticket: Ticket,
        /// The deck.
        deck: Box<Deck>,
    },
    /// A page arrived for the [`Effect::Search`] or [`Effect::LoadMore`] holding `ticket`.
    Searched {
        /// The ticket from the effect this answers.
        ticket: Ticket,
        /// The page.
        page: HttpDeckCardSearch,
    },
    /// The [`Effect::AddCard`] holding `ticket` created the card.
    Added {
        /// The ticket from the effect this answers.
        ticket: Ticket,
        /// The deck card the server created.
        deck_card: Box<DeckCard>,
    },
    /// The [`Effect::PromoteCard`], [`Effect::UndoAdd`] or [`Effect::UndoPromote`] holding `ticket` went through.
    Landed {
        /// The ticket from the effect this answers.
        ticket: Ticket,
    },
    /// The client took the entry an [`Effect::TakeUndo`] asked for, or found none.
    Taken {
        /// The ticket from the [`Effect::TakeUndo`] this answers.
        ticket: Ticket,
        /// The entry taken off the undo log.
        taken: Option<UndoAction>,
    },
    /// The undo holding `ticket` never went out: the session could not refresh. The client has already reported it and raised its error toast.
    NotSent {
        /// The ticket from the [`Effect::UndoAdd`] or [`Effect::UndoPromote`] this answers.
        ticket: Ticket,
    },
    /// The request holding `ticket` failed. The client has already reported it and raised its error toast.
    Failed {
        /// The ticket from the request this answers.
        ticket: Ticket,
    },
    /// The request holding `ticket` found no deck: the server no longer has it. The client has already reported it and raised its toast.
    Gone {
        /// The ticket from the request this answers.
        ticket: Ticket,
    },
    /// The client's card filter, as it stands after an apply or an [`Effect::Rerun`]: Search starts over with it and Maybeboard filters again.
    Filter(CardQueryBuilder),
    /// The filter sheet's Reset, with the sheet's draft: stages the source's default.
    ResetFilter(CardQueryBuilder),
    /// Picks the source, with the filter the client remembered for it on this deck, if any.
    PickSource {
        /// The source to swipe.
        source: Source,
        /// The source's remembered filter.
        remembered: Option<CardQueryBuilder>,
    },
    /// Turns Synergy on or off.
    ToggleSynergy,
    /// The top card's exit to this side committed.
    Swipe(Direction),
    /// Undoes the newest swipe, from a down swipe or a down pull on the end skeleton.
    Undo,
    /// The printings sheet picked another printing of the Search card on top.
    SwapPrinting(Box<Card>),
    /// Searches again from the top, or reloads the maybeboard.
    Refresh,
    /// Goes back to where the user came from.
    Back,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Fetches the deck quietly, since the screen works without it, and answers with [`Action::DeckLoaded`] or [`Action::Failed`].
    FetchDeck {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The deck to fetch.
        deck_id: Uuid,
    },
    /// Fetches the deck again for its maybeboard and answers with [`Action::DeckLoaded`] or [`Action::Failed`].
    RefreshMaybeboard {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The deck to fetch.
        deck_id: Uuid,
    },
    /// Records a search, flushes buffered usage so synergy ordering sees it, then searches and answers with [`Action::Searched`] or [`Action::Failed`].
    Search {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The deck searched for.
        deck_id: Uuid,
        /// The query.
        query: CardQuery,
    },
    /// Records a search, then fetches the next page and answers with [`Action::Searched`] or [`Action::Failed`].
    LoadMore {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The deck searched for.
        deck_id: Uuid,
        /// The query.
        query: CardQuery,
    },
    /// Posts a skip quietly, with no reply.
    SkipCard {
        /// The deck.
        deck_id: Uuid,
        /// The skipped card's oracle ID.
        oracle_id: Uuid,
    },
    /// Deletes a skip, with no reply.
    UnskipCard {
        /// The deck.
        deck_id: Uuid,
        /// The card's oracle ID.
        oracle_id: Uuid,
    },
    /// Creates one copy of the card on `board` with [`add_request`] and answers with [`Action::Added`] or [`Action::Failed`].
    AddCard {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The deck.
        deck_id: Uuid,
        /// The card.
        card: Box<Card>,
        /// The main deck or the maybeboard.
        board: Board,
    },
    /// Deletes a card whose add the user undid while it was in flight, with no reply.
    DeleteUndoneAdd {
        /// The deck.
        deck_id: Uuid,
        /// The card's printing ID.
        card_id: Uuid,
    },
    /// Moves a maybeboard card to the deck with [`board_request`] and answers with [`Action::Landed`] or [`Action::Failed`].
    PromoteCard {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The deck.
        deck_id: Uuid,
        /// The card's printing ID.
        card_id: Uuid,
    },
    /// Takes the newest undo log entry `claim` matches off the deck's log and answers at once with [`Action::Taken`].
    TakeUndo {
        /// The ticket to hand back with the answer.
        ticket: Ticket,
        /// The deck whose log to search.
        deck_id: Uuid,
        /// Which entry to take.
        claim: UndoClaim,
    },
    /// Undoes an add to `board`: refreshes the session, deletes the card and answers with [`Action::Landed`], [`Action::NotSent`] when the session can't refresh, or [`Action::Failed`]. A failure here is the undo's own: the client words it as one.
    UndoAdd {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The deck.
        deck_id: Uuid,
        /// The card's printing ID.
        card_id: Uuid,
        /// The board it was added to.
        board: Board,
    },
    /// Undoes a move to the deck: refreshes the session, sends the card back to the maybeboard with [`board_request`] and answers like [`Effect::UndoAdd`].
    UndoPromote {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The deck.
        deck_id: Uuid,
        /// The card's printing ID.
        card_id: Uuid,
    },
    /// Pushes `action` onto the deck's undo log.
    RecordUndo {
        /// The deck whose log it joins.
        deck_id: Uuid,
        /// The entry.
        action: UndoAction,
    },
    /// Plays the top card's entrance on `pile` from this side: an undo brought it back.
    Enter {
        /// The pile whose top card enters.
        pile: Source,
        /// The side it enters from.
        from: Direction,
    },
    /// Cancels any entrance primed on this pile.
    StopEnter(Source),
    /// Writes this filter into the client's filter, which the sheet edits.
    SetFilter(CardQueryBuilder),
    /// Bumps the client's filter counter, which resets the sheet's sections; the client answers with [`Action::Filter`] holding its filter.
    Rerun,
    /// Remembers `filter` for `source` on this deck.
    ParkFilter {
        /// The source the filter belongs to.
        source: Source,
        /// The filter.
        filter: CardQueryBuilder,
    },
    /// Lets fresh results ease in again, even images seen before.
    EaseImages,
    /// Records how the pile ran out.
    PileEnd {
        /// The deck.
        deck_id: Uuid,
        /// How it ran out.
        kind: PileEndKind,
        /// Whether synergy was on.
        synergy: bool,
    },
    /// Goes to this screen.
    Navigate(Destination),
    /// Goes to this screen in place of the current one, so Back skips a screen with nothing left to show.
    Replace(Destination),
    /// Raises this toast.
    Toast(Toast),
    /// Confirms what a swipe or its undo just did, as the user's swipe
    /// confirmations setting says: this toast's words in the screen's chip,
    /// the toast itself, or nothing.
    Confirm(Toast),
}

/// Which of an action's effects raise a toast, for [`keep_one`].
fn toast_line(effect: &Effect) -> Option<Line<'_>> {
    match effect {
        Effect::Toast(toast) => Some(Line::Toast(toast)),
        Effect::Confirm(toast) => Some(Line::Confirm(toast)),
        _ => None,
    }
}

/// The create request that adds one copy of `card` to `board`.
#[must_use]
pub fn add_request(card: &Card, board: Board) -> HttpCreateDeckCard {
    HttpCreateDeckCard::new(&card.scryfall_data, 1, undo::create_board_field(board))
}

/// The update request that moves a card to `board`.
#[must_use]
pub fn board_request(board: Board) -> HttpPatchDeckCard {
    HttpPatchDeckCard::new(None, Some(board.display_name().to_owned()))
}

/// Whether the visit has offered the one-time hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

/// A request out, waiting for its reply.
#[derive(Debug, Clone, PartialEq)]
enum Pending {
    /// The deck's first fetch.
    Deck,
    /// Refresh's fetch of the deck for its maybeboard.
    Maybeboard,
    /// A search from the top; its ticket starts a new stack. `snapshot` is the filter as sent, recorded once it lands; `explains_empty` toasts an empty reply.
    Search {
        snapshot: Box<CardQueryBuilder>,
        explains_empty: bool,
    },
    /// The next page of the stack the search holding `epoch` started.
    Page { epoch: Ticket, synergy: bool },
    /// An add to `board`.
    Add { card: Box<Card>, board: Board },
    /// A move from the maybeboard to the deck.
    Promote { card_id: Uuid, card_name: String },
    /// An undone add waiting for the client to take its log entry.
    ClaimAdd { card: Box<Card>, board: Board },
    /// An undone add's delete. `taken` goes back on the log if it fails.
    UndoAdd {
        card: Box<Card>,
        board: Board,
        taken: Option<UndoAction>,
    },
    /// An undone move to the deck waiting for the client to take its log entry.
    ClaimPromote { card: Box<Card> },
    /// An undone move to the deck. `taken` goes back on the log if it fails.
    UndoPromote {
        card: Box<Card>,
        taken: Option<UndoAction>,
    },
}

/// Where the Search stack's fetching stands.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct Run {
    /// A search from the top is on its way.
    searching: bool,
    /// The next page is on its way.
    loading_more: bool,
    /// Paging found nothing new.
    exhausted: bool,
}

/// Add cards for one visit. Starts with no card and no deck; send [`Action::Load`] with the deck.
///
/// Every request takes a fresh [`Ticket`] from one [`Generation`] and waits in a list keyed by it, so each reply finds its own request. A search from the top marks its ticket as the stack's epoch; a reply to an older search, or a page of an older stack, changes nothing but the loading flag it held. A new [`Action::Load`] empties the list.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    deck_id: Option<Uuid>,
    /// The deck once its first fetch lands.
    deck: Option<AddDeck>,
    source: Source,
    /// The live source's filter, as the client last handed it in or the model last set it.
    filter: CardQueryBuilder,
    /// Whether the Search filter on entry was remembered, which keeps its Synergy pick.
    remembered: bool,
    stack: CardStack<AddAction>,
    maybeboard: CardStack<MaybeboardAction>,
    /// The ticket of the search that started the stack.
    epoch: Option<Ticket>,
    run: Run,
    /// The filter that produced the stack, which parking records.
    last_search: Option<CardQueryBuilder>,
    warming: bool,
    /// Printing IDs served after the synergy pool ran out.
    related: HashSet<Uuid>,
    related_announced: bool,
    in_flight: InFlightAdds,
    waiting: Vec<(Ticket, Pending)>,
    generation: Generation,
    hint: HintOffer,
}

impl State {
    /// The screen as it first draws: no deck, no card, Search picked.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order. When a swipe or a page
    /// reply runs the pile out, it says [`NO_CARDS_LEFT`] in place of the swipe's own line.
    ///
    /// A reply whose request is no longer waiting changes nothing.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        let ran_out_by = matches!(action, Action::Swipe(_) | Action::Searched { .. });
        let before = self.pile();
        let mut effects = match action {
            Action::Load {
                deck_id,
                filter,
                parked,
            } => self.load(deck_id, filter, parked),
            Action::DeckLoaded { ticket, deck } => self.deck_loaded(ticket, &deck),
            Action::Searched { ticket, page } => self.searched(ticket, page),
            Action::Added { ticket, deck_card } => self.added(ticket, *deck_card),
            Action::Landed { ticket } => self.landed(ticket),
            Action::Taken { ticket, taken } => self.taken(ticket, taken),
            Action::NotSent { ticket } => self.not_sent(ticket),
            Action::Failed { ticket } => self.failed(ticket),
            Action::Gone { ticket } => self.gone(ticket),
            Action::Filter(filter) => {
                self.filter = filter;
                let effects = self.fresh_search();
                if self.source == Source::Maybeboard {
                    self.refilter_maybeboard();
                }
                effects
            }
            Action::ResetFilter(draft) => self.reset_filter(draft),
            Action::PickSource { source, remembered } => self.pick_source(source, remembered),
            Action::ToggleSynergy => {
                let on = !self.filter.synergy();
                self.filter.set_synergy(on);
                let said = if on { SYNERGY_ON } else { SYNERGY_OFF };
                vec![
                    Effect::SetFilter(self.filter.clone()),
                    Effect::Rerun,
                    Effect::Toast(Toast::quick(ToastKind::Info, said)),
                ]
            }
            Action::Swipe(direction) => self.swipe(direction),
            Action::Undo => match self.source {
                Source::Search => self.undo_search(),
                Source::Maybeboard => self.undo_maybeboard(),
            },
            Action::SwapPrinting(card) => {
                self.stack.replace_current(*card);
                Vec::new()
            }
            Action::Refresh => self.refresh(),
            Action::Back => vec![Effect::Navigate(Destination::Previous)],
        };
        effects.extend(self.announce_related());
        let after = self.pile();
        let ran_out = match self.source {
            Source::Search => before != Pile::End && after == Pile::End,
            Source::Maybeboard => before == Pile::Cards && after == Pile::Empty,
        };
        if ran_out_by && ran_out {
            effects.push(Effect::Toast(Toast::quick(ToastKind::Info, NO_CARDS_LEFT)));
        }
        keep_one(&mut effects, toast_line);
        effects
    }

    /// Starts the visit for `deck_id`, keeping the generation so replies from an earlier visit stay stale. Resumes the parked stack when its filter still matches, else searches.
    fn load(
        &mut self,
        deck_id: Uuid,
        filter: Option<CardQueryBuilder>,
        parked: Option<Box<ParkedStack>>,
    ) -> Vec<Effect> {
        let hint = self.hint;
        *self = Self {
            deck_id: Some(deck_id),
            remembered: filter.is_some(),
            filter: filter.unwrap_or_default(),
            hint: HintOffer::Offered,
            generation: self.generation,
            ..Self::default()
        };
        let mut effects = Vec::new();
        if hint == HintOffer::Waiting {
            effects.push(Effect::OfferHint(OneTimeHint::AddDeckCards));
        }
        let ticket = self.wait(Pending::Deck);
        effects.push(Effect::FetchDeck { ticket, deck_id });
        if let Some(parked) = parked
            && !parked.cards.is_empty()
            && add::normalized(&parked.filter) == add::normalized(&self.filter)
        {
            let ParkedStack {
                cards,
                index,
                history,
                filter,
                exhausted,
            } = *parked;
            self.run.exhausted = exhausted;
            self.last_search = Some(filter);
            self.stack.restore(cards, index, history);
            self.epoch = Some(self.generation.next_ticket());
            effects.push(Effect::StopEnter(Source::Search));
            return effects;
        }
        effects.extend(self.fresh_search());
        effects
    }

    /// Takes in a fetched deck: the first fills the deck context into the filter and serves an empty stack; a refresh reloads the maybeboard.
    fn deck_loaded(&mut self, ticket: Ticket, deck: &Deck) -> Vec<Effect> {
        match self.take(ticket) {
            Some(Pending::Deck) => {
                let add_deck = AddDeck::from_deck(deck);
                add_deck.seed(&mut self.filter, self.remembered);
                self.deck = Some(add_deck);
                let mut effects = vec![Effect::SetFilter(self.filter.clone())];
                if self.stack.is_empty() {
                    effects.push(Effect::Rerun);
                }
                effects
            }
            Some(Pending::Maybeboard) => {
                if let Some(add_deck) = &mut self.deck {
                    add_deck.maybeboard = AddDeck::from_deck(deck).maybeboard;
                }
                self.maybeboard.rewind();
                vec![Effect::StopEnter(Source::Maybeboard), Effect::Rerun]
            }
            _ => Vec::new(),
        }
    }

    /// Starts a new stack from the top with the live filter. On Search, once the deck is in, a filter Add would not serve asks for one instead.
    fn fresh_search(&mut self) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let ticket = self.generation.next_ticket();
        self.epoch = Some(ticket);
        self.run.searching = false;
        self.stack.reset();
        self.last_search = None;
        self.run.exhausted = false;
        let mut effects = vec![Effect::StopEnter(Source::Search), Effect::EaseImages];
        if self.source == Source::Search
            && let Some(add_deck) = &self.deck
            && !add_deck.context().serves(&self.filter)
        {
            effects.push(Effect::Toast(Toast::normal(
                ToastKind::Warning,
                TRY_A_FILTER,
            )));
            return effects;
        }
        let Ok(query) = add::normalized(&self.filter).build() else {
            return effects;
        };
        self.run.searching = true;
        self.waiting.push((
            ticket,
            Pending::Search {
                snapshot: Box::new(self.filter.clone()),
                explains_empty: true,
            },
        ));
        effects.push(Effect::Search {
            ticket,
            deck_id,
            query,
        });
        effects
    }

    /// Takes in a page: a search's starts the stack, a load-more's tops it up.
    fn searched(&mut self, ticket: Ticket, page: HttpDeckCardSearch) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        match self.take(ticket) {
            Some(Pending::Search {
                snapshot,
                explains_empty,
            }) => {
                if self.epoch != Some(ticket) {
                    return Vec::new();
                }
                let warming = !page.synergy_applied;
                let warned = self.warm(warming);
                self.related = add::related_ids(&page);
                self.related_announced = false;
                let served: Vec<Card> = page.cards.into_iter().filter(|c| !self.holds(c)).collect();
                let mut effects = Vec::new();
                if explains_empty && served.is_empty() {
                    let synergy = snapshot.synergy();
                    effects.push(Effect::PileEnd {
                        deck_id,
                        kind: PileEndKind::Empty,
                        synergy,
                    });
                    effects.push(Effect::Toast(Toast::normal(
                        ToastKind::Warning,
                        add::no_results_message(synergy, warming),
                    )));
                }
                self.stack.replace(served);
                self.last_search = Some(*snapshot);
                self.run.searching = false;
                effects.extend(warned);
                effects
            }
            Some(Pending::Page { epoch, synergy }) => {
                self.run.loading_more = false;
                if self.epoch != Some(epoch) {
                    return Vec::new();
                }
                let warned = self.warm(!page.synergy_applied);
                self.related.extend(add::related_ids(&page));
                let stacked: HashSet<Uuid> = self
                    .stack
                    .cards()
                    .iter()
                    .map(|c| c.card_profile.scryfall_data_id)
                    .collect();
                let fresh: Vec<Card> = page
                    .cards
                    .into_iter()
                    .filter(|c| {
                        !stacked.contains(&c.card_profile.scryfall_data_id) && !self.holds(c)
                    })
                    .collect();
                let mut effects = Vec::new();
                if fresh.is_empty() {
                    self.run.exhausted = true;
                    effects.push(Effect::PileEnd {
                        deck_id,
                        kind: PileEndKind::Empty,
                        synergy,
                    });
                } else {
                    self.stack.append(fresh);
                }
                effects.extend(warned);
                effects
            }
            _ => Vec::new(),
        }
    }

    /// Notes whether synergy is warming, with a toast when it starts.
    fn warm(&mut self, warming: bool) -> Option<Effect> {
        let started = warming && !self.warming;
        self.warming = warming;
        started.then(|| Effect::Toast(Toast::normal(ToastKind::Info, SYNERGY_WARMING)))
    }

    /// Settles an add: one undone in flight is deleted now, else the deck holds it and the undo log records it.
    fn added(&mut self, ticket: Ticket, deck_card: DeckCard) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let Some(Pending::Add { card, board }) = self.take(ticket) else {
            return Vec::new();
        };
        let card_id = card.scryfall_data.id;
        if self.in_flight.finish(card_id) == AddOutcome::Undo {
            return vec![Effect::DeleteUndoneAdd { deck_id, card_id }];
        }
        let card_name = card.scryfall_data.name.clone();
        if let Some(add_deck) = &mut self.deck {
            if let Some(oracle_id) = card.scryfall_data.oracle_id {
                add_deck.in_deck.insert(oracle_id);
            }
            if board.is_maybeboard() {
                add_deck.maybeboard.push(DeckEntry {
                    card: *card,
                    deck_card,
                });
            }
        }
        vec![Effect::RecordUndo {
            deck_id,
            action: UndoAction::Added { card_id, card_name },
        }]
    }

    /// Settles a request that went through with no body.
    fn landed(&mut self, ticket: Ticket) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        match self.take(ticket) {
            Some(Pending::Promote { card_id, card_name }) => vec![Effect::RecordUndo {
                deck_id,
                action: UndoAction::MovedBoard {
                    card_id,
                    card_name,
                    from: Board::Maybeboard,
                },
            }],
            Some(Pending::UndoAdd { card, board, .. }) => {
                let card_id = card.scryfall_data.id;
                if let Some(add_deck) = &mut self.deck
                    && let Some(oracle_id) = card.scryfall_data.oracle_id
                {
                    add_deck.in_deck.remove(&oracle_id);
                }
                if board.is_maybeboard() {
                    if let Some(add_deck) = &mut self.deck {
                        add_deck
                            .maybeboard
                            .retain(|e| e.card.scryfall_data.id != card_id);
                    }
                    vec![confirm(ToastKind::Success, UNDID_MAYBEBOARD)]
                } else {
                    self.rewind_counts(&card);
                    vec![confirm(ToastKind::Success, UNDID_ADD)]
                }
            }
            Some(Pending::UndoPromote { card, .. }) => {
                let card_id = card.scryfall_data.id;
                let oracle_id = card.scryfall_data.oracle_id;
                if let Some(add_deck) = &mut self.deck {
                    if let Some(oracle_id) = oracle_id {
                        add_deck.in_deck.remove(&oracle_id);
                    }
                    add_deck.maybeboard.push(DeckEntry {
                        card: *card,
                        deck_card: DeckCard {
                            deck_id,
                            scryfall_data_id: card_id,
                            oracle_id: oracle_id.unwrap_or_default(),
                            quantity: Quantity::one(),
                            board: Board::Maybeboard,
                            mvp_at: None,
                        },
                    });
                }
                vec![confirm(ToastKind::Success, UNDID_MOVE_TO_DECK)]
            }
            _ => Vec::new(),
        }
    }

    /// Goes on with an undo once the client has taken its log entry. With no entry and the change already reversed, the deck cards view's Undo got there first and nothing is sent.
    fn taken(&mut self, ticket: Ticket, taken: Option<UndoAction>) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        match self.take(ticket) {
            Some(Pending::ClaimAdd { card, board }) => {
                if taken.is_none() && !self.holds(&card) {
                    return vec![confirm(ToastKind::Success, undid_add(board))];
                }
                let card_id = card.scryfall_data.id;
                let ticket = self.wait(Pending::UndoAdd { card, board, taken });
                vec![Effect::UndoAdd {
                    ticket,
                    deck_id,
                    card_id,
                    board,
                }]
            }
            Some(Pending::ClaimPromote { card }) => {
                let card_id = card.scryfall_data.id;
                let back = self.deck.as_ref().is_some_and(|d| {
                    d.maybeboard
                        .iter()
                        .any(|e| e.card.scryfall_data.id == card_id)
                });
                if taken.is_none() && back {
                    return vec![
                        Effect::StopEnter(Source::Maybeboard),
                        quick(ToastKind::Info, ALREADY_CHANGED),
                    ];
                }
                self.maybeboard.insert_current((*card).clone());
                let ticket = self.wait(Pending::UndoPromote { card, taken });
                vec![Effect::UndoPromote {
                    ticket,
                    deck_id,
                    card_id,
                }]
            }
            _ => Vec::new(),
        }
    }

    /// Settles an undo whose call never went out: a Search undo steps forward again, and the taken entry goes back on the log.
    fn not_sent(&mut self, ticket: Ticket) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let (pile, taken) = match self.take(ticket) {
            Some(Pending::UndoAdd { board, taken, .. }) => {
                let action = if board.is_maybeboard() {
                    AddAction::Maybe
                } else {
                    AddAction::Add
                };
                self.stack.unwind_undo(action);
                (Source::Search, taken)
            }
            Some(Pending::UndoPromote { taken, .. }) => (Source::Maybeboard, taken),
            Some(other) => {
                self.waiting.push((ticket, other));
                return self.failed(ticket);
            }
            None => return Vec::new(),
        };
        let mut effects = vec![Effect::StopEnter(pile)];
        effects.extend(taken.map(|action| Effect::RecordUndo { deck_id, action }));
        effects
    }

    /// Settles a failed request.
    fn gone(&mut self, ticket: Ticket) -> Vec<Effect> {
        let mut effects = self.failed(ticket);
        effects.push(Effect::Replace(Destination::DeckList));
        effects
    }

    fn failed(&mut self, ticket: Ticket) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        match self.take(ticket) {
            Some(Pending::Search { .. }) => {
                if self.epoch == Some(ticket) {
                    self.run.searching = false;
                }
                Vec::new()
            }
            Some(Pending::Page { .. }) => {
                self.run.loading_more = false;
                Vec::new()
            }
            Some(Pending::Add { card, .. }) => {
                let _settled = self.in_flight.finish(card.scryfall_data.id);
                Vec::new()
            }
            Some(
                Pending::UndoAdd {
                    taken: Some(action),
                    ..
                }
                | Pending::UndoPromote {
                    taken: Some(action),
                    ..
                },
            ) => vec![Effect::RecordUndo { deck_id, action }],
            _ => Vec::new(),
        }
    }

    /// Stages the source's default in the sheet's draft.
    fn reset_filter(&self, mut draft: CardQueryBuilder) -> Vec<Effect> {
        let reset = match self.source {
            Source::Search => self.context().reset_search(&mut draft),
            Source::Maybeboard => filter::reset_maybeboard(&mut draft),
        };
        match reset {
            Ok(()) => vec![
                Effect::SetFilter(draft),
                quick(ToastKind::Info, FILTER_RESET),
            ],
            Err(_) => vec![quick(ToastKind::Warning, FILTER_AT_DEFAULT)],
        }
    }

    /// Switches source: remembers the leaving filter and takes the entering one, or its default.
    fn pick_source(&mut self, source: Source, remembered: Option<CardQueryBuilder>) -> Vec<Effect> {
        if source == self.source {
            return Vec::new();
        }
        let leaving = std::mem::take(&mut self.filter);
        self.filter = match source {
            Source::Search => remembered.unwrap_or_else(|| self.context().fresh_search()),
            Source::Maybeboard => remembered.unwrap_or_default(),
        };
        let effects = vec![
            Effect::ParkFilter {
                source: self.source,
                filter: leaving,
            },
            Effect::SetFilter(self.filter.clone()),
            Effect::StopEnter(Source::Maybeboard),
            Effect::Rerun,
        ];
        self.source = source;
        self.maybeboard.rewind();
        effects
    }

    /// Builds the maybeboard stack from the deck's maybeboard, then the filter and its sort, from the first card. The swipes stay for undo.
    fn refilter_maybeboard(&mut self) {
        let cards: Vec<Card> = self
            .deck
            .as_ref()
            .map(|d| d.maybeboard.iter().map(|e| e.card.clone()).collect())
            .unwrap_or_default();
        let matching: Vec<Card> = if self.filter.is_empty() {
            cards
        } else {
            match self.filter.build_criteria() {
                Ok(criteria) => Cards::from(cards).matching(&criteria).into(),
                Err(_) => cards,
            }
        };
        let sorted: Vec<Card> = Cards::from(matching)
            .sorted(self.filter.sort(), self.filter.direction())
            .into();
        self.maybeboard.replace(sorted);
    }

    /// Commits the top card's exit to `side`.
    fn swipe(&mut self, side: Direction) -> Vec<Effect> {
        match (self.source, side) {
            (Source::Search, Direction::Left) => self.skip(),
            (Source::Search, Direction::Right) => self.add(Board::Deck),
            (Source::Search, Direction::Up) => self.add(Board::Maybeboard),
            (Source::Maybeboard, Direction::Left) => {
                if self.maybeboard.current_wrapping().is_none() {
                    return Vec::new();
                }
                self.maybeboard.record(MaybeboardAction::Skip);
                self.maybeboard.advance_wrapping();
                vec![confirm(ToastKind::Info, SKIPPED)]
            }
            (Source::Maybeboard, Direction::Right) => self.promote(),
            (Source::Maybeboard, Direction::Up) => {
                vec![confirm(ToastKind::Info, ALREADY_ON_MAYBEBOARD)]
            }
        }
    }

    /// Skips the Search card on top for a week.
    fn skip(&mut self) -> Vec<Effect> {
        let (Some(deck_id), Some(card)) = (self.deck_id, self.stack.current()) else {
            return Vec::new();
        };
        let mut effects = Vec::new();
        if let Some(oracle_id) = card.scryfall_data.oracle_id {
            effects.push(Effect::SkipCard { deck_id, oracle_id });
        }
        self.stack.record(AddAction::Skip);
        effects.push(confirm(ToastKind::Info, SKIPPED));
        effects.extend(self.advance());
        effects
    }

    /// Adds the Search card on top to `board`, counting a main board add toward the land target and the budget.
    fn add(&mut self, board: Board) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let Some(card) = self.stack.current().cloned() else {
            return Vec::new();
        };
        self.stack.record(if board.is_maybeboard() {
            AddAction::Maybe
        } else {
            AddAction::Add
        });
        self.in_flight.begin(card.scryfall_data.id);
        let ticket = self.wait(Pending::Add {
            card: Box::new(card.clone()),
            board,
        });
        let mut effects = vec![Effect::AddCard {
            ticket,
            deck_id,
            card: Box::new(card.clone()),
            board,
        }];
        if board.is_maybeboard() {
            effects.push(confirm(ToastKind::Info, ADDED_TO_MAYBEBOARD));
        } else {
            effects.push(confirm(ToastKind::Success, ADDED_TO_DECK));
            effects.extend(self.count_add(&card));
        }
        effects.extend(self.advance());
        effects
    }

    /// Raises the land count and the running price for a main board add, with their crossing toasts. The price only runs with a budget set.
    fn count_add(&mut self, card: &Card) -> Vec<Effect> {
        let Some(add_deck) = &mut self.deck else {
            return Vec::new();
        };
        let mut signals = Vec::new();
        if card.scryfall_data.is_land() {
            let before = add_deck.land_count;
            add_deck.land_count = before.saturating_add(1);
            signals.extend(deck_signals::land_crossing(
                add_deck.land_target,
                before,
                add_deck.land_count,
                Crossing::Up,
            ));
        }
        if add_deck.budget.is_some_and(|b| b > 0.0) {
            let before = add_deck.total_price;
            add_deck.total_price = before + deck_signals::price_of(card, add_deck.currency);
            signals.extend(deck_signals::budget_crossing(
                add_deck.budget,
                add_deck.currency,
                before,
                add_deck.total_price,
            ));
        }
        signals
            .into_iter()
            .map(|(level, text)| Effect::Toast(Toast::normal(kind_of(level), text)))
            .collect()
    }

    /// Lowers the land count and the running price for an undone main board add.
    fn rewind_counts(&mut self, card: &Card) {
        let Some(add_deck) = &mut self.deck else {
            return;
        };
        if card.scryfall_data.is_land() {
            add_deck.land_count = add_deck.land_count.saturating_sub(1).max(0);
        }
        let price = deck_signals::price_of(card, add_deck.currency);
        add_deck.total_price = (add_deck.total_price - price).max(0.0);
    }

    /// Moves the Search cursor past the committed card, warning near the cap or fetching the next page.
    fn advance(&mut self) -> Vec<Effect> {
        let total = self.stack.len();
        if !self.stack.advance() {
            return Vec::new();
        }
        let mut effects = Vec::new();
        if (CARDS_WARNING_THRESHOLD..MAX_CARDS_IN_STACK).contains(&total)
            && total.is_multiple_of(STACK_WARNING_EVERY)
        {
            effects.push(quick(ToastKind::Info, STACK_LIMIT_NEAR));
        }
        let index = self.stack.index();
        let spent = index >= total && self.run.exhausted;
        if !spent
            && total > 0
            && index
                .checked_add(1)
                .is_some_and(|seen| seen >= total.saturating_sub(timing::LOAD_MORE_AT))
        {
            effects.extend(self.load_more());
        }
        effects
    }

    /// Fetches the next page, unless the stack is at its cap or a page is already on its way.
    fn load_more(&mut self) -> Vec<Effect> {
        let (Some(deck_id), Some(epoch)) = (self.deck_id, self.epoch) else {
            return Vec::new();
        };
        if self.stack.len() >= MAX_CARDS_IN_STACK {
            return vec![Effect::Toast(Toast::normal(
                ToastKind::Warning,
                STACK_LIMIT_REACHED,
            ))];
        }
        if self.run.loading_more {
            return Vec::new();
        }
        let normalized = add::normalized(&self.filter);
        let Ok(query) = normalized.build() else {
            return Vec::new();
        };
        self.run.loading_more = true;
        let ticket = self.wait(Pending::Page {
            epoch,
            synergy: normalized.synergy(),
        });
        vec![Effect::LoadMore {
            ticket,
            deck_id,
            query,
        }]
    }

    /// Moves the maybeboard card on top to the deck, dropping it from the stack and the maybeboard at once.
    fn promote(&mut self) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let Some(card) = self.maybeboard.current_wrapping().cloned() else {
            return Vec::new();
        };
        let card_id = card.scryfall_data.id;
        let card_name = card.scryfall_data.name.clone();
        if let Some(add_deck) = &mut self.deck {
            add_deck
                .maybeboard
                .retain(|e| e.card.scryfall_data.id != card_id);
            if let Some(oracle_id) = card.scryfall_data.oracle_id {
                add_deck.in_deck.insert(oracle_id);
            }
        }
        self.maybeboard.record(MaybeboardAction::Promote {
            card: Box::new(card),
        });
        self.maybeboard.remove_current();
        let ticket = self.wait(Pending::Promote { card_id, card_name });
        vec![
            Effect::PromoteCard {
                ticket,
                deck_id,
                card_id,
            },
            confirm(ToastKind::Success, MOVED_TO_DECK),
        ]
    }

    /// Undoes the newest Search swipe: steps back onto its card, then reverses it. An add still in flight is parked for its reply to delete; a landed one first asks the client for its undo log entry.
    fn undo_search(&mut self) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let Some(action) = self.stack.pop_action() else {
            return vec![Effect::Toast(Toast::normal(
                ToastKind::Info,
                NOTHING_TO_UNDO,
            ))];
        };
        if !self.stack.step_back(&action) {
            self.stack.record(action);
            return vec![Effect::Toast(Toast::normal(
                ToastKind::Warning,
                NO_PREVIOUS_CARD,
            ))];
        }
        let Some(card) = self.stack.current().cloned() else {
            self.stack.unwind_undo(action);
            return vec![Effect::StopEnter(Source::Search)];
        };
        let mut effects = vec![Effect::Enter {
            pile: Source::Search,
            from: action.exited(),
        }];
        let board = match action {
            AddAction::Skip => {
                if let Some(oracle_id) = card.scryfall_data.oracle_id {
                    effects.push(Effect::UnskipCard { deck_id, oracle_id });
                }
                effects.push(confirm(ToastKind::Info, UNDID_SKIP));
                return effects;
            }
            AddAction::Add => Board::Deck,
            AddAction::Maybe => Board::Maybeboard,
        };
        let card_id = card.scryfall_data.id;
        if self.in_flight.undo(card_id) {
            if !board.is_maybeboard() {
                self.rewind_counts(&card);
            }
            effects.push(confirm(ToastKind::Success, undid_add(board)));
            return effects;
        }
        let ticket = self.wait(Pending::ClaimAdd {
            card: Box::new(card),
            board,
        });
        effects.push(Effect::TakeUndo {
            ticket,
            deck_id,
            claim: UndoClaim::Added { card_id },
        });
        effects
    }

    /// Undoes the newest maybeboard swipe: a skip steps back at once; a move to the deck first asks the client for its undo log entry.
    fn undo_maybeboard(&mut self) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let Some(action) = self.maybeboard.pop_action() else {
            return vec![Effect::Toast(Toast::normal(
                ToastKind::Info,
                NOTHING_TO_UNDO,
            ))];
        };
        let enter = Effect::Enter {
            pile: Source::Maybeboard,
            from: action.exited(),
        };
        match action {
            MaybeboardAction::Skip => {
                if !self.maybeboard.retreat_wrapping() {
                    return vec![enter, Effect::StopEnter(Source::Maybeboard)];
                }
                vec![enter, confirm(ToastKind::Info, UNDID_SKIP)]
            }
            MaybeboardAction::Promote { card } => {
                let card_id = card.scryfall_data.id;
                let ticket = self.wait(Pending::ClaimPromote { card });
                vec![
                    enter,
                    Effect::TakeUndo {
                        ticket,
                        deck_id,
                        claim: UndoClaim::MovedBoard {
                            card_id,
                            from: Board::Maybeboard,
                        },
                    },
                ]
            }
        }
    }

    /// Searches again from the top on Search, or reloads the maybeboard.
    fn refresh(&mut self) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        if self.source == Source::Maybeboard {
            let ticket = self.wait(Pending::Maybeboard);
            return vec![
                Effect::RefreshMaybeboard { ticket, deck_id },
                quick(ToastKind::Info, MAYBEBOARD_REFRESHED),
            ];
        }
        let Ok(query) = add::normalized(&self.filter).build() else {
            return vec![quick(ToastKind::Warning, FILTER_EMPTY)];
        };
        let ticket = self.wait(Pending::Search {
            snapshot: Box::new(self.filter.clone()),
            explains_empty: false,
        });
        self.epoch = Some(ticket);
        self.stack.reset();
        self.last_search = None;
        self.run.exhausted = false;
        self.run.searching = true;
        vec![
            Effect::StopEnter(Source::Search),
            Effect::EaseImages,
            Effect::Search {
                ticket,
                deck_id,
                query,
            },
            quick(ToastKind::Info, STACK_REFRESHED),
        ]
    }

    /// Says the synergy pool is spent the first time a card past it reaches the top of the Search stack, once per stack.
    fn announce_related(&mut self) -> Vec<Effect> {
        let (Some(deck_id), Some(card)) = (self.deck_id, self.stack.current()) else {
            return Vec::new();
        };
        if self.related_announced || !self.related.contains(&card.card_profile.scryfall_data_id) {
            return Vec::new();
        }
        self.related_announced = true;
        vec![
            Effect::Toast(Toast::normal(ToastKind::Info, SYNERGY_SPENT)),
            Effect::PileEnd {
                deck_id,
                kind: PileEndKind::SynergyExhausted,
                synergy: true,
            },
        ]
    }

    /// Whether the deck holds a card with this oracle ID; nothing before the deck is in.
    fn holds(&self, card: &Card) -> bool {
        self.deck.as_ref().is_some_and(|d| d.holds(card))
    }

    /// Takes a fresh ticket for `pending` and puts it on the waiting list.
    fn wait(&mut self, pending: Pending) -> Ticket {
        let ticket = self.generation.next_ticket();
        self.waiting.push((ticket, pending));
        ticket
    }

    /// Takes the request holding `ticket` off the waiting list, if it is there.
    fn take(&mut self, ticket: Ticket) -> Option<Pending> {
        let at = self.waiting.iter().position(|(t, _)| *t == ticket)?;
        Some(self.waiting.remove(at).1)
    }

    /// The deck being built, once [`Action::Load`] has named it.
    #[must_use]
    pub fn deck_id(&self) -> Option<Uuid> {
        self.deck_id
    }

    /// Whether the deck has loaded.
    #[must_use]
    pub fn is_deck_loaded(&self) -> bool {
        self.deck.is_some()
    }

    /// The picked source.
    #[must_use]
    pub fn source(&self) -> Source {
        self.source
    }

    /// The live source's filter as the model last saw it.
    #[must_use]
    pub fn filter(&self) -> &CardQueryBuilder {
        &self.filter
    }

    /// The deck facts the Search filter is built from; empty before the deck is in.
    #[must_use]
    pub fn context(&self) -> AddContext {
        self.deck.as_ref().map(AddDeck::context).unwrap_or_default()
    }

    /// Whether the Synergy chip shows: on Search, for a deck with a commander.
    #[must_use]
    pub fn shows_synergy(&self) -> bool {
        self.source == Source::Search && self.deck.as_ref().is_some_and(|d| d.has_commander)
    }

    /// Whether the Filter key shows its dot for the client's `filter`. Deck context, synergy and the automatic land exclusion don't count.
    #[must_use]
    pub fn shows_dot(&self, filter: &CardQueryBuilder) -> bool {
        self.context().shows_dot(filter)
    }

    /// Whether the filter sheet refuses to apply an empty filter: only Search needs one, since Maybeboard filters on the device.
    #[must_use]
    pub fn validates_apply(&self) -> bool {
        self.source == Source::Search
    }

    /// What the pile area draws.
    #[must_use]
    pub fn pile(&self) -> Pile {
        match self.source {
            Source::Search if self.stack.current().is_some() => Pile::Cards,
            Source::Search if !self.stack.is_empty() => {
                if self.run.loading_more {
                    Pile::Loading
                } else {
                    Pile::End
                }
            }
            Source::Search if self.run.searching => Pile::Loading,
            Source::Maybeboard if !self.maybeboard.is_empty() => Pile::Cards,
            Source::Search | Source::Maybeboard => Pile::Empty,
        }
    }

    /// The cards the swipe stack draws: up to [`crate::stack::STACK_DEPTH`] from the cursor.
    #[must_use]
    pub fn window(&self) -> &[Card] {
        match self.source {
            Source::Search => self.stack.window(),
            Source::Maybeboard => self.maybeboard.window(),
        }
    }

    /// The card on top of the live pile, which the card info and the details sheet describe.
    #[must_use]
    pub fn current(&self) -> Option<&Card> {
        match self.source {
            Source::Search => self.stack.current(),
            Source::Maybeboard => self.maybeboard.current_wrapping(),
        }
    }

    /// The Search card on top, which the printings sheet re-skins.
    #[must_use]
    pub fn printing_card(&self) -> Option<&Card> {
        self.stack.current()
    }

    /// The Search stack to park on leave: a served stack with the filter that produced it.
    #[must_use]
    pub fn parked(&self) -> Option<ParkedStack> {
        let filter = self.last_search.clone()?;
        if self.stack.is_empty() {
            return None;
        }
        let (cards, index, history) = self.stack.park_state();
        Some(ParkedStack {
            cards,
            index,
            history,
            filter,
            exhausted: self.run.exhausted,
        })
    }
}

/// A quick toast.
fn quick(kind: ToastKind, text: &str) -> Effect {
    Effect::Toast(Toast::quick(kind, text))
}

/// A swipe's confirmation, in a quick toast's words and kind.
fn confirm(kind: ToastKind, text: &str) -> Effect {
    Effect::Confirm(Toast::quick(kind, text))
}

/// The toast for an undone add to `board`.
fn undid_add(board: Board) -> &'static str {
    if board.is_maybeboard() {
        UNDID_MAYBEBOARD
    } else {
        UNDID_ADD
    }
}

/// The toast kind for a deck signal's level.
fn kind_of(level: Level) -> ToastKind {
    match level {
        Level::Info => ToastKind::Info,
        Level::Warning => ToastKind::Warning,
    }
}
