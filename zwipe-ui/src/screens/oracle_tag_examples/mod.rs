//! Oracle tag examples: real cards that carry one tag, most played first, browsed one at a time. It opens over the dictionary or any screen that shows a tag's definition.
//!
//! The browse is read only: Next (the app's left swipe) moves to the next card and Back one (its down swipe) steps back. Cards come from `search_cards` in pages of [`timing::PAGE_SIZE`] by EDHREC rank, fetched again when [`timing::LOAD_MORE_AT`] or fewer are left ahead, up to [`timing::EXAMPLES_MAX_CARDS`]. An empty page ends it.
//!
//! The model holds the cards, the cursor and the paging. Gestures, the card's exit and entrance animations, the details and printings sheets stay in the client.

#[cfg(test)]
mod tests;

use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::error::InvalidCardCriteria;
use zwipe_core::domain::card::search_card::card_filter::{CardQuery, CardSortKey};
use zwipe_core::domain::user::hints::OneTimeHint;

use crate::generation::{Generation, Ticket};
use crate::stack::{CardStack, Direction, StackAction};
use crate::timing;
use crate::toast::{Toast, ToastKind};

/// The screen's title.
pub const TITLE: &str = "Examples";

/// The hint's title.
pub const HINT_TITLE: &str = "Example cards";

/// The line past the last card.
pub const END: &str = "No more cards. Swipe down to go back.";

/// The line when the tag has no cards, or the first page failed.
pub const EMPTY: &str = "No example cards for this tag yet.";

/// The toast for stepping back from the first card.
pub const AT_FIRST: &str = "Already at the first card";

/// The Back button, to the screen that opened the examples.
pub const BACK: &str = "Back";

/// Where an action sends the user. The client maps each to its own route or overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// The screen that opened the examples, from Back.
    Previous,
}

/// Where the browse stands, for the client to pick what it draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// A page is out with no card to show: the card ghosts.
    Loading,
    /// A card is up: [`State::current`].
    Showing,
    /// Past the last card with nothing more coming: [`END`].
    End,
    /// No card at all: [`EMPTY`].
    Empty,
}

/// A step forward through the examples. Its card leaves to the left, so a step back brings it in from there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Next;

impl StackAction for Next {
    fn exited(&self) -> Direction {
        Direction::Left
    }
}

/// What a user does on the examples, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Starts the browse for the tag with this slug from its first card, offering the hint once per visit.
    Load {
        /// The tag's slug.
        slug: String,
    },
    /// A page arrived for the fetch holding `ticket`.
    Loaded {
        /// The ticket from the [`Effect::SearchCards`] this answers.
        ticket: Ticket,
        /// The page's cards; empty past the tag's last card.
        cards: Vec<Card>,
    },
    /// The fetch holding `ticket` failed. The client has already reported it and raised its error toast.
    Failed {
        /// The ticket from the [`Effect::SearchCards`] this answers.
        ticket: Ticket,
    },
    /// Moves past the card on screen, as its exit commits.
    Next,
    /// Steps back one card.
    BackOne,
    /// Goes back to the screen that opened the examples.
    Back,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Searches cards with this query and answers with [`Action::Loaded`] or [`Action::Failed`] carrying the ticket.
    SearchCards {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// One page of the tag's cards.
        query: Box<CardQuery>,
    },
    /// Plays the top card's entrance from this side: a step back brought it back.
    Enter(Direction),
    /// Goes to this screen.
    Navigate(Destination),
    /// Raises this toast.
    Toast(Toast),
}

/// Whether the visit has offered the one-time hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

/// The examples for one visit. Starts loading with no card; send [`Action::Load`] with the tag.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    slug: String,
    stack: CardStack<Next>,
    /// Where the next page starts.
    offset: u32,
    /// The first page has not settled.
    first_page: bool,
    /// A page is out.
    paging: bool,
    /// An empty page came back, the offset ran out or the cap was reached.
    exhausted: bool,
    hint: HintOffer,
    generation: Generation,
}

impl Default for State {
    fn default() -> Self {
        Self {
            slug: String::new(),
            stack: CardStack::default(),
            offset: 0,
            first_page: true,
            paging: false,
            exhausted: false,
            hint: HintOffer::Waiting,
            generation: Generation::default(),
        }
    }
}

impl State {
    /// The examples as they first draw: loading, no card.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// A reply whose ticket a newer [`Action::Load`] has replaced changes nothing. One page is out at a time.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load { slug } => {
                let hint = self.hint;
                *self = Self {
                    slug,
                    hint: HintOffer::Offered,
                    generation: self.generation,
                    ..Self::default()
                };
                let mut effects = Vec::new();
                if hint == HintOffer::Waiting {
                    effects.push(Effect::OfferHint(OneTimeHint::OtagExamples));
                }
                let ticket = self.generation.next_ticket();
                effects.extend(self.page(ticket));
                effects
            }
            Action::Loaded { ticket, cards } => {
                if self.generation.is_current(ticket) {
                    self.land(cards);
                }
                Vec::new()
            }
            Action::Failed { ticket } => {
                if self.generation.is_current(ticket) {
                    self.settle();
                }
                Vec::new()
            }
            Action::Next => self.next(),
            Action::BackOne => self.back_one(),
            Action::Back => vec![Effect::Navigate(Destination::Previous)],
        }
    }

    /// Asks for the page at the offset unless one is out, the end was reached or the browse holds [`timing::EXAMPLES_MAX_CARDS`].
    fn page(&mut self, ticket: Ticket) -> Option<Effect> {
        if self.paging || self.exhausted {
            return None;
        }
        if self.stack.len() >= timing::EXAMPLES_MAX_CARDS {
            self.exhausted = true;
            self.first_page = false;
            return None;
        }
        let Ok(query) = query_for(&self.slug, self.offset) else {
            self.first_page = false;
            return None;
        };
        self.paging = true;
        Some(Effect::SearchCards {
            ticket,
            query: Box::new(query),
        })
    }

    /// Adds a page's cards, or ends the browse on an empty one.
    fn land(&mut self, cards: Vec<Card>) {
        if cards.is_empty() {
            self.exhausted = true;
        } else {
            match self.offset.checked_add(timing::PAGE_SIZE) {
                Some(next) => self.offset = next,
                None => self.exhausted = true,
            }
            self.stack.append(cards);
        }
        self.settle();
    }

    /// Marks the page out as settled, either way.
    fn settle(&mut self) {
        self.paging = false;
        self.first_page = false;
    }

    /// Moves past the card on screen and asks for more when few are left ahead.
    fn next(&mut self) -> Vec<Effect> {
        let total = self.stack.len();
        self.stack.record(Next);
        if !self.stack.advance() {
            return Vec::new();
        }
        let near_end = self
            .stack
            .index()
            .checked_add(1)
            .is_some_and(|seen| seen >= total.saturating_sub(timing::LOAD_MORE_AT));
        if !near_end {
            return Vec::new();
        }
        let ticket = self.generation.current();
        self.page(ticket).into_iter().collect()
    }

    /// Steps back one card, or says the browse is at the first.
    fn back_one(&mut self) -> Vec<Effect> {
        let Some(action) = self.stack.pop_action() else {
            return vec![Effect::Toast(Toast::quick(ToastKind::Info, AT_FIRST))];
        };
        if !self.stack.step_back(&action) {
            self.stack.record(action);
            return Vec::new();
        }
        self.stack.cancel_entering();
        vec![Effect::Enter(action.exited())]
    }

    /// The tag being browsed.
    #[must_use]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// Where the browse stands.
    #[must_use]
    pub fn status(&self) -> Status {
        if self.current().is_some() {
            Status::Showing
        } else if !self.stack.is_empty() {
            if self.paging {
                Status::Loading
            } else {
                Status::End
            }
        } else if self.first_page {
            Status::Loading
        } else {
            Status::Empty
        }
    }

    /// The line in place of the card: [`END`] or [`EMPTY`]. `None` while loading or with a card up.
    #[must_use]
    pub fn message(&self) -> Option<&'static str> {
        match self.status() {
            Status::End => Some(END),
            Status::Empty => Some(EMPTY),
            Status::Loading | Status::Showing => None,
        }
    }

    /// The card on screen, `None` past the last one or before the first page.
    #[must_use]
    pub fn current(&self) -> Option<&Card> {
        self.stack.current()
    }

    /// The cards a swipe stack draws: up to [`crate::stack::STACK_DEPTH`] from the one on screen.
    #[must_use]
    pub fn window(&self) -> &[Card] {
        self.stack.window()
    }

    /// The cards fetched so far.
    #[must_use]
    pub fn cards(&self) -> &[Card] {
        self.stack.cards()
    }

    /// Where the cursor is: the card on screen, or the count past the last one.
    #[must_use]
    pub fn index(&self) -> usize {
        self.stack.index()
    }
}

/// The search for one page of `slug`'s cards from `offset`: no tokens, by EDHREC rank, [`timing::PAGE_SIZE`] at a time.
///
/// # Errors
///
/// Returns [`InvalidCardCriteria`] when the criteria contradict each other, which these never do.
pub fn query_for(slug: &str, offset: u32) -> Result<CardQuery, InvalidCardCriteria> {
    CardQueryBuilder::new()
        .set_oracle_tags_contains_any(vec![slug.to_owned()])
        .set_is_token(false)
        .set_sort(CardSortKey::EdhrecRank)
        .set_limit(timing::PAGE_SIZE)
        .set_offset(offset)
        .build()
}
