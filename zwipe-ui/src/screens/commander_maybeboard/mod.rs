//! The commander maybeboard: the user's saved commanders narrowed by the Show pips and the card filter, Quick add by name, Create deck through the create gate, Remove, Clear maybeboard and the screen's one-time hint.
//!
//! The client keeps a [`State`] while the screen is up and sends [`Action::Load`] when it opens. The pip and Quick add rules come from [`crate::commander::maybeboard`] and the Create check from [`crate::deck::create_gate`]. The Swipe overlay stays the client's: its up swipes save on their own, and the client sends [`Action::SwipeClosed`] when it closes and [`Action::Create`] for a right swipe. The card filter sheet, the Art chip, the expanded row and whether the Clear dialog is up stay in the client too.

#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::time::Duration;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::query::CardQuery;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::deck::Format;
use zwipe_core::domain::user::hints::OneTimeHint;

use crate::commander::maybeboard::{self, Sort, Step};
pub use crate::commander::maybeboard::{LIST_TITLE, LOAD_FAILED, NO_COMMANDERS, NO_MATCH, SortBy};
use crate::generation::{Generation, Ticket};
use crate::toast::{Toast, ToastKind};
use crate::{deck, timing};

/// The screen's title, also the hint's.
pub const TITLE: &str = "Commander maybeboard";

/// The Quick add row's label.
pub const QUICK_ADD: &str = "Quick add";

/// The Quick add input's placeholder.
pub const QUICK_ADD_PLACEHOLDER: &str = "Search a commander to add";

/// The one result chip while a search waits or is out.
pub const SEARCHING: &str = "Searching…";

/// The one result chip when the search found nothing that isn't saved yet.
pub const NO_RESULTS: &str = "No results";

/// The Show row's label.
pub const SHOW: &str = "Show";

/// The Show chip that unpicks every color.
pub const SHOW_ALL: &str = "All";

/// The Show chip that toggles the rows' art.
pub const ART: &str = "Art";

/// A row's button that starts a deck with its commander.
pub const CREATE_DECK: &str = "Create deck";

/// A row's button that takes its commander off the list.
pub const REMOVE: &str = "Remove";

/// The button that opens the card filter.
pub const FILTER: &str = "Filter";

/// The button that opens the commander swipe.
pub const SWIPE: &str = "Swipe";

/// The button that asks to clear the list, and the title of the dialog it opens.
pub const CLEAR_MAYBEBOARD: &str = "Clear maybeboard";

/// The Clear maybeboard dialog's line.
pub const CLEAR_BODY: &str =
    "Every commander on your maybeboard will be removed. This can't be undone.";

/// The Clear maybeboard dialog's button that closes it without clearing.
pub const CANCEL: &str = "Cancel";

/// The Clear maybeboard dialog's button that clears.
pub const CLEAR: &str = "Clear";

/// The toast after a remove.
pub const REMOVED: &str = "Removed";

/// The toast after Clear maybeboard.
pub const CLEARED: &str = "Maybeboard cleared";

/// The toast after Quick add saves `name`.
#[must_use]
pub fn added(name: &str) -> String {
    format!("Added {name}")
}

/// The Quick add search for `query`: commanders in the Commander format whose name holds the trimmed query, no tokens, the closest names first and the most played first within each (`CardSortKey::BestMatch`), [`timing::MAYBEBOARD_QUICK_ADD_LIMIT`] at most.
#[must_use]
pub fn search_query(query: &str) -> CardQueryBuilder {
    let mut builder = CardQueryBuilder::with_name_contains(query.trim());
    builder
        .set_is_commander_in_format(Format::Commander)
        .set_is_token(false)
        .set_sort(CardSortKey::BestMatch)
        .set_limit(timing::MAYBEBOARD_QUICK_ADD_LIMIT);
    builder
}

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// The deck list, from Back.
    DeckList,
    /// The Create deck form, after [`Effect::SeedCommander`].
    CreateDeck,
}

/// Where the saved list stands, for the client to pick what it draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// The first load is out: the whole page ghosts.
    Loading,
    /// A load failed with nothing saved on screen: [`LOAD_FAILED`] under the chip rows, until a load lands. A Quick add in the meantime leaves it up, as the app does.
    Failed,
    /// Nothing is saved: [`NO_COMMANDERS`] under the chip rows.
    Empty,
    /// At least one commander is saved: the list, or [`NO_MATCH`] when nothing passes.
    Loaded,
}

/// What floats under the Quick add input once the query is long enough to search.
#[derive(Debug, Clone, PartialEq)]
pub enum QuickAddResults {
    /// The search waits out its debounce or is out: one [`SEARCHING`] chip.
    Searching,
    /// The search found nothing that isn't saved yet: one [`NO_RESULTS`] chip.
    Empty,
    /// The commanders that can still be added, in the search's order, each a chip.
    Found(Vec<Card>),
}

/// What a user does on the commander maybeboard, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Fetches the saved list and the user's decks and offers the hint, as the screen does when it opens.
    Load,
    /// The saved list arrived for the fetch holding `ticket`.
    Loaded {
        /// The ticket from the [`Effect::FetchMaybeboard`] this answers.
        ticket: Ticket,
        /// The saved commanders, newest save first.
        cards: Vec<Card>,
    },
    /// The user's decks arrived, for the create gate. A failure sends nothing, and the gate counts no deck.
    DeckProfilesLoaded {
        /// How many decks the user owns.
        count: usize,
    },
    /// The request holding `ticket` failed. The client has already reported it, and raised its error toast unless the request was a search.
    Failed {
        /// The ticket from the effect this answers.
        ticket: Ticket,
    },
    /// The Quick add input reads `query`, from typing or its clear button. A trimmed query of [`timing::QUICK_ADD_MIN_CHARS`] or more waits out the debounce and searches; a shorter one clears the results.
    Type(String),
    /// The debounce for the query edit holding `ticket` is over.
    Debounced {
        /// The ticket from the [`Effect::Debounce`] this answers.
        ticket: Ticket,
    },
    /// The search holding `ticket` found these commanders.
    Searched {
        /// The ticket from the [`Effect::SearchCommanders`] this answers.
        ticket: Ticket,
        /// The commanders found, saved ones included.
        cards: Vec<Card>,
    },
    /// Moves the highlighted Quick add result one step, for a client with arrow keys.
    StepHighlight(Step),
    /// Saves the Quick add result with this oracle id. It leaves the results at once and joins the top of the list when the server confirms.
    Add(Uuid),
    /// Saves the highlighted Quick add result, as [`Action::Add`]. Nothing highlighted does nothing.
    AddHighlighted,
    /// The add holding `ticket` went through.
    Added {
        /// The ticket from the [`Effect::AddCommander`] this answers.
        ticket: Ticket,
    },
    /// Unpicks every Show color.
    ShowAll,
    /// Picks or unpicks a Show color.
    ToggleColor(Color),
    /// Orders the list by this sort, from the Sort row.
    Sort(Sort),
    /// Starts a deck with this commander, through the create gate: a row's Create deck or a right swipe.
    Create {
        /// The commander, in the printing on screen.
        card: Box<Card>,
        /// Whether the user's email is verified. A client with no session passes `true`, so only the server can refuse.
        verified: bool,
    },
    /// Takes the commander with this oracle id off the list.
    Remove(Uuid),
    /// The remove holding `ticket` went through.
    Removed {
        /// The ticket from the [`Effect::RemoveCommander`] this answers.
        ticket: Ticket,
    },
    /// Clears the list, from the Clear maybeboard dialog.
    Clear,
    /// The clear holding `ticket` went through.
    Cleared {
        /// The ticket from the [`Effect::ClearMaybeboard`] this answers.
        ticket: Ticket,
    },
    /// Swaps in a printing picked from a row, on the entry sharing its oracle id. A card with no oracle id changes nothing. The client keeps the row expanded under the new printing's id.
    PickPrinting(Box<Card>),
    /// The Swipe overlay closed. Its up swipes may have saved commanders, so the list fetches again.
    SwipeClosed,
    /// Goes back to the deck list.
    Back,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Fetches the saved list; answers with [`Action::Loaded`] or [`Action::Failed`] carrying this ticket.
    FetchMaybeboard(Ticket),
    /// Fetches the user's decks, quietly; answers with [`Action::DeckProfilesLoaded`], or nothing on failure.
    FetchDeckProfiles,
    /// Waits `wait`, then answers with [`Action::Debounced`] carrying `ticket`.
    Debounce {
        /// The ticket to hand back.
        ticket: Ticket,
        /// How long to wait: [`timing::QUICK_ADD_DEBOUNCE`].
        wait: Duration,
    },
    /// Searches commanders, quietly; answers with [`Action::Searched`] or [`Action::Failed`] carrying `ticket`.
    SearchCommanders {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The search, from [`search_query`].
        query: Box<CardQuery>,
    },
    /// Saves a commander; answers with [`Action::Added`] or [`Action::Failed`].
    AddCommander {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The commander's oracle id.
        oracle_id: Uuid,
    },
    /// Takes a commander off; answers with [`Action::Removed`] or [`Action::Failed`].
    RemoveCommander {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The commander's oracle id.
        oracle_id: Uuid,
    },
    /// Takes every commander off; answers with [`Action::Cleared`] or [`Action::Failed`].
    ClearMaybeboard(Ticket),
    /// Hands this commander to Create deck, which opens with it in the commander slot.
    SeedCommander(Box<Card>),
    /// Goes to this screen.
    Navigate(Destination),
    /// Raises this toast.
    Toast(Toast),
    /// Stores this sort, so the screen opens in it next time.
    RememberSort(Sort),
}

/// A request that changes the list, waiting for its reply.
#[derive(Debug, Clone, PartialEq)]
enum Edit {
    /// The commander leaves the Quick add results when the add goes out and joins the list when it lands.
    Add {
        oracle_id: Uuid,
        card: Box<Card>,
    },
    Remove(Uuid),
    Clear,
}

/// A reply that an [`Edit`] went through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Landed {
    Added,
    Removed,
    Cleared,
}

/// Whether the visit has offered the one-time hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

/// The Quick add input and its search.
#[derive(Debug, Default, Clone, PartialEq)]
struct QuickAdd {
    query: String,
    /// The query edit whose debounce or search may still land.
    waiting: Option<Ticket>,
    /// A search for the current query waits out its debounce or is out.
    searching: bool,
    /// The last search's commanders, saved ones included.
    results: Vec<Card>,
    highlight: Option<usize>,
}

/// The commander maybeboard for one visit. Starts loading with nothing saved and no pip picked; send [`Action::Load`] to start the fetches.
///
/// Every request takes a fresh [`Ticket`] from one [`Generation`], so a ticket names exactly one request. A load's reply lands only while it is the newest load, and a load still out when an edit lands is sent again, since its reply left before the edit and could bring a removed row back. A refetch with commanders on screen is silent.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    /// Until the first load's reply lands.
    loading: bool,
    /// A load failed while nothing was saved on screen; only a landed load clears it.
    failed: bool,
    /// The saved commanders, newest save first.
    entries: Vec<Card>,
    /// The newest load, while it is out.
    load_waiting: Option<Ticket>,
    deck_count: Option<usize>,
    picked: HashSet<Color>,
    sort: Sort,
    quick: QuickAdd,
    edits: Vec<(Ticket, Edit)>,
    hint: HintOffer,
    tickets: Generation,
}

impl Default for State {
    fn default() -> Self {
        Self {
            loading: true,
            failed: false,
            entries: Vec::new(),
            load_waiting: None,
            deck_count: None,
            picked: HashSet::new(),
            sort: Sort::default(),
            quick: QuickAdd::default(),
            edits: Vec::new(),
            hint: HintOffer::Waiting,
            tickets: Generation::default(),
        }
    }
}

impl State {
    /// The screen as it first draws: loading, nothing saved, no pip picked, an empty Quick add.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// [`Self::new`] in `sort`, the one the client remembers.
    #[must_use]
    pub fn with_sort(sort: Sort) -> Self {
        Self {
            sort,
            ..Self::default()
        }
    }

    /// The Sort row's order.
    #[must_use]
    pub fn sort(&self) -> Sort {
        self.sort
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// The hint is offered once per visit, on the first [`Action::Load`]. A reply whose ticket a newer request of its kind has replaced changes nothing.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load => self.load(),
            Action::Loaded { ticket, cards } => {
                if self.settle_load(ticket) {
                    self.entries = cards;
                    self.failed = false;
                }
                Vec::new()
            }
            Action::DeckProfilesLoaded { count } => {
                self.deck_count = Some(count);
                Vec::new()
            }
            Action::Failed { ticket } => {
                self.failed_reply(ticket);
                Vec::new()
            }
            Action::Type(query) => self.type_query(query),
            Action::Debounced { ticket } => self.debounced(ticket),
            Action::Searched { ticket, cards } => {
                if self.quick.waiting == Some(ticket) {
                    self.quick.waiting = None;
                    self.quick.searching = false;
                    self.quick.results = cards;
                }
                Vec::new()
            }
            Action::StepHighlight(step) => {
                self.quick.highlight =
                    maybeboard::step_highlight(self.quick.highlight, self.addable().len(), step);
                Vec::new()
            }
            Action::Add(oracle_id) => self.add(oracle_id),
            Action::AddHighlighted => {
                let picked = self.quick.highlight.and_then(|at| {
                    self.addable()
                        .get(at)
                        .and_then(|c| c.scryfall_data.oracle_id)
                });
                picked.map_or_else(Vec::new, |oracle_id| self.add(oracle_id))
            }
            Action::Added { ticket } => self.land(ticket, Landed::Added),
            Action::ShowAll => {
                self.picked.clear();
                Vec::new()
            }
            Action::ToggleColor(color) => {
                if !self.picked.remove(&color) {
                    self.picked.insert(color);
                }
                Vec::new()
            }
            Action::Sort(sort) => {
                self.sort = sort;
                vec![Effect::RememberSort(sort)]
            }
            Action::Create { card, verified } => {
                match deck::create_gate(verified, self.deck_count.unwrap_or(0)) {
                    Ok(()) => vec![
                        Effect::SeedCommander(card),
                        Effect::Navigate(Destination::CreateDeck),
                    ],
                    Err(warning) => vec![Effect::Toast(warning)],
                }
            }
            Action::Remove(oracle_id) => self.edit(Edit::Remove(oracle_id)),
            Action::Removed { ticket } => self.land(ticket, Landed::Removed),
            Action::Clear => self.edit(Edit::Clear),
            Action::Cleared { ticket } => self.land(ticket, Landed::Cleared),
            Action::PickPrinting(card) => {
                let target = card.scryfall_data.oracle_id;
                if target.is_some()
                    && let Some(slot) = self
                        .entries
                        .iter_mut()
                        .find(|c| c.scryfall_data.oracle_id == target)
                {
                    *slot = *card;
                }
                Vec::new()
            }
            Action::SwipeClosed => vec![self.fetch()],
            Action::Back => vec![Effect::Navigate(Destination::DeckList)],
        }
    }

    fn load(&mut self) -> Vec<Effect> {
        let mut effects = Vec::new();
        if self.hint == HintOffer::Waiting {
            self.hint = HintOffer::Offered;
            effects.push(Effect::OfferHint(OneTimeHint::CommanderMaybeboard));
        }
        effects.push(self.fetch());
        effects.push(Effect::FetchDeckProfiles);
        effects
    }

    /// Sends a load whose reply replaces any still out.
    fn fetch(&mut self) -> Effect {
        let ticket = self.tickets.next_ticket();
        self.load_waiting = Some(ticket);
        Effect::FetchMaybeboard(ticket)
    }

    /// Sends the load again when one is still out, after an edit landed.
    fn refetch_stale(&mut self) -> Option<Effect> {
        self.load_waiting.is_some().then(|| self.fetch())
    }

    /// Ends the load holding `ticket` when it is the newest. Returns whether it was.
    fn settle_load(&mut self, ticket: Ticket) -> bool {
        if self.load_waiting != Some(ticket) {
            return false;
        }
        self.load_waiting = None;
        self.loading = false;
        true
    }

    fn failed_reply(&mut self, ticket: Ticket) {
        if self.settle_load(ticket) {
            self.failed = self.entries.is_empty();
        } else if self.quick.waiting == Some(ticket) {
            self.quick.waiting = None;
            self.quick.searching = false;
        } else {
            self.edits.retain(|(t, _)| *t != ticket);
        }
    }

    fn type_query(&mut self, query: String) -> Vec<Effect> {
        self.quick.query = query;
        self.quick.highlight = None;
        let ticket = self.tickets.next_ticket();
        if self.searching_catalog() {
            self.quick.waiting = Some(ticket);
            self.quick.searching = true;
            vec![Effect::Debounce {
                ticket,
                wait: timing::QUICK_ADD_DEBOUNCE,
            }]
        } else {
            self.quick.waiting = None;
            self.quick.searching = false;
            self.quick.results.clear();
            Vec::new()
        }
    }

    fn debounced(&mut self, ticket: Ticket) -> Vec<Effect> {
        if self.quick.waiting != Some(ticket) {
            return Vec::new();
        }
        let Ok(query) = search_query(&self.quick.query).build() else {
            self.quick.waiting = None;
            self.quick.searching = false;
            return Vec::new();
        };
        vec![Effect::SearchCommanders {
            ticket,
            query: Box::new(query),
        }]
    }

    fn add(&mut self, oracle_id: Uuid) -> Vec<Effect> {
        let Some(card) = self
            .quick
            .results
            .iter()
            .find(|c| c.scryfall_data.oracle_id == Some(oracle_id))
            .cloned()
        else {
            return Vec::new();
        };
        self.quick
            .results
            .retain(|c| c.scryfall_data.oracle_id != Some(oracle_id));
        self.quick.highlight = None;
        self.edit(Edit::Add {
            oracle_id,
            card: Box::new(card),
        })
    }

    /// Sends the request for `edit` and waits for its reply.
    fn edit(&mut self, edit: Edit) -> Vec<Effect> {
        let ticket = self.tickets.next_ticket();
        let effect = match &edit {
            Edit::Add { oracle_id, .. } => Effect::AddCommander {
                ticket,
                oracle_id: *oracle_id,
            },
            Edit::Remove(oracle_id) => Effect::RemoveCommander {
                ticket,
                oracle_id: *oracle_id,
            },
            Edit::Clear => Effect::ClearMaybeboard(ticket),
        };
        self.edits.push((ticket, edit));
        vec![effect]
    }

    /// Lands the reply to the edit holding `ticket`, when it is waiting and is the kind `landed` answers.
    fn land(&mut self, ticket: Ticket, landed: Landed) -> Vec<Effect> {
        let Some(at) = self.edits.iter().position(|(t, _)| *t == ticket) else {
            return Vec::new();
        };
        let (_, edit) = self.edits.remove(at);
        match (edit, landed) {
            (Edit::Add { oracle_id, card }, Landed::Added) => {
                let toast = Toast::quick(ToastKind::Info, added(&card.scryfall_data.name));
                if !self
                    .entries
                    .iter()
                    .any(|c| c.scryfall_data.oracle_id == Some(oracle_id))
                {
                    self.entries.insert(0, *card);
                }
                vec![self.fetch(), Effect::Toast(toast)]
            }
            (Edit::Remove(oracle_id), Landed::Removed) => {
                self.entries
                    .retain(|c| c.scryfall_data.oracle_id != Some(oracle_id));
                self.after_edit(REMOVED)
            }
            (Edit::Clear, Landed::Cleared) => {
                self.entries.clear();
                self.after_edit(CLEARED)
            }
            (edit, _) => {
                self.edits.insert(at, (ticket, edit));
                Vec::new()
            }
        }
    }

    /// The load sent again if one is out, then the `text` toast.
    fn after_edit(&mut self, text: &str) -> Vec<Effect> {
        let mut effects: Vec<Effect> = self.refetch_stale().into_iter().collect();
        effects.push(Effect::Toast(Toast::quick(ToastKind::Info, text)));
        effects
    }

    /// The Quick add results that can still be added: an oracle id, not saved yet.
    fn addable(&self) -> Vec<Card> {
        let saved = self.saved_oracles();
        self.quick
            .results
            .iter()
            .filter(|c| maybeboard::is_addable(c.scryfall_data.oracle_id, &saved))
            .cloned()
            .collect()
    }

    /// Where the saved list stands.
    #[must_use]
    pub fn status(&self) -> Status {
        if self.loading {
            Status::Loading
        } else if self.failed {
            Status::Failed
        } else if self.entries.is_empty() {
            Status::Empty
        } else {
            Status::Loaded
        }
    }

    /// The saved commanders, newest save first, unnarrowed.
    #[must_use]
    pub fn entries(&self) -> &[Card] {
        &self.entries
    }

    /// The oracle ids already saved: the Swipe overlay leaves them out of its pile.
    #[must_use]
    pub fn saved_oracles(&self) -> HashSet<Uuid> {
        self.entries
            .iter()
            .filter_map(|c| c.scryfall_data.oracle_id)
            .collect()
    }

    /// The saved commanders the Show pips and `filter` let through, in the Sort row's order, unless `filter` picks a sort of its own.
    ///
    /// `filter` is the card filter sheet's builder, which the client keeps; a builder that fails to build narrows nothing.
    #[must_use]
    pub fn shown(&self, filter: &CardQueryBuilder) -> Vec<Card> {
        let mut shown: Vec<Card> = self
            .entries
            .iter()
            .filter(|c| maybeboard::passes_pips(&c.scryfall_data.color_identity, &self.picked))
            .cloned()
            .collect();
        if !filter.is_empty()
            && let Ok(criteria) = filter.build_criteria()
        {
            shown = Cards::from(shown).matching(&criteria).into();
        }
        shown = self.sort.sorted(shown);
        if let Some(sort) = filter.sort() {
            shown = Cards::from(shown).sorted(sort, filter.direction()).into();
        }
        shown
    }

    /// The line under the chip rows in place of the list: [`LOAD_FAILED`], [`NO_COMMANDERS`], or [`NO_MATCH`] when the pips and `filter` let nothing through. `None` while loading or when commanders show.
    #[must_use]
    pub fn message(&self, filter: &CardQueryBuilder) -> Option<&'static str> {
        match self.status() {
            Status::Loading => None,
            Status::Failed => Some(LOAD_FAILED),
            Status::Empty => Some(NO_COMMANDERS),
            Status::Loaded => self.shown(filter).is_empty().then_some(NO_MATCH),
        }
    }

    /// Whether no Show color is picked, so the All chip is.
    #[must_use]
    pub fn shows_all(&self) -> bool {
        self.picked.is_empty()
    }

    /// Whether the Show chip for `color` is picked.
    #[must_use]
    pub fn is_picked(&self, color: Color) -> bool {
        self.picked.contains(&color)
    }

    /// The Quick add input's text.
    #[must_use]
    pub fn query(&self) -> &str {
        &self.quick.query
    }

    /// Whether the trimmed query is long enough to search, which is also when the results float under the input.
    #[must_use]
    pub fn searching_catalog(&self) -> bool {
        self.quick.query.trim().len() >= timing::QUICK_ADD_MIN_CHARS
    }

    /// What floats under the Quick add input, or `None` while the query is too short to search.
    #[must_use]
    pub fn quick_add(&self) -> Option<QuickAddResults> {
        if !self.searching_catalog() {
            return None;
        }
        if self.quick.searching {
            return Some(QuickAddResults::Searching);
        }
        let found = self.addable();
        Some(if found.is_empty() {
            QuickAddResults::Empty
        } else {
            QuickAddResults::Found(found)
        })
    }

    /// The highlighted Quick add result's place among [`QuickAddResults::Found`]'s, if any. Cleared by every query edit and add.
    #[must_use]
    pub fn highlight(&self) -> Option<usize> {
        self.quick.highlight
    }
}
