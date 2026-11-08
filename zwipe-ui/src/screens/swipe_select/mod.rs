//! Swipe select: the overlay that swipes through cards to fill a command zone slot, or through the user's suggested commanders.
//!
//! Left skips the card on top, right picks it, up saves it to the commander maybeboard and down undoes the last swipe. A [`Mode`] decides the always-on constraint on the search: legal commanders for a format, partners, backgrounds, or signature spells within an identity. [`Mode::Suggested`] deals the host's ranking instead of searching, and there a left swipe dismisses the commander for a week.
//!
//! The overlay stays mounted while it is closed, so a reopen with the same mode keeps the pile and its place. The client sends [`Action::Open`] each time the overlay opens, [`Action::Filter`] when the card filter sheet applies, and [`Action::Redeal`] when the host's suggested pile changes while it is open. A right swipe hands the card to the host with [`Effect::Pick`], and Back goes to [`Destination::Previous`], which the host takes as the overlay closing. Gestures, the card's exit and entrance motion, the filter sheet's draft, the details and printings sheets and whether the overlay is up stay in the client.

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::{Color, Colors};
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::query::CardQuery;
use zwipe_core::domain::deck::Format;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_core::http::contracts::suggestions::{HttpCommanderSuggestion, HttpSuggestionReasons};

use crate::commander::swipe_select;
use crate::deck::form;
use crate::generation::{Generation, Ticket};
use crate::playstyle;
use crate::stack::{CardStack, Direction, NO_CARDS_LEFT, StackAction};
use crate::timing::{LOAD_MORE_AT, MAYBEBOARD_TOAST, PAGE_SIZE};
use crate::toast::{Line, Toast, ToastKind, keep_one};

/// The overlay's title in every mode but [`Mode::Suggested`].
pub const TITLE: &str = "Swipe select";

/// The overlay's title in [`Mode::Suggested`].
pub const SUGGESTED_TITLE: &str = "Suggested commanders";

/// The button that closes the overlay.
pub const BACK: &str = "Back";

/// The button that opens the filter sheet.
pub const FILTER: &str = "Filter";

/// The button that searches again from the top.
pub const REFRESH: &str = "Refresh";

/// The toast for a left swipe.
pub const SKIPPED: &str = "Skipped";

/// The toast for an up swipe on a card with no oracle id.
pub const CANT_SAVE: &str = "This card can't be saved";

/// The toast once an up swipe's save lands.
pub const SAVED: &str = "Added to your commander maybeboard";

/// The toast once an undone save's removal lands.
pub const UNSAVED: &str = "Removed from your commander maybeboard";

/// The toast for an undo with no swipe to undo.
pub const NOTHING_TO_UNDO: &str = "Nothing to undo";

/// The toast for undoing a skip or a dismissal.
pub const UNDID_SKIP: &str = "Undid skip";

/// The toast for Refresh.
pub const STACK_REFRESHED: &str = "Stack refreshed";

/// The noun the toast and hint use before a mode is known.
pub const FALLBACK_NOUN: &str = "card";

/// Most pages one load more pulls while the host's exclusions or the pile's own cards empty every page it gets.
const PAGE_PULLS: usize = 4;

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// Back to the host, closing the overlay.
    Previous,
}

/// Which command zone slot the overlay fills, which decides the always-on constraint on the search.
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    /// Legal commanders for the deck's format.
    Commander(Format),
    /// Partners for the commander, when one is chosen: only the cards it can pair with are dealt. With none, every card with a partner ability.
    Partner(Option<Box<Card>>),
    /// "Choose a Background" backgrounds.
    Background,
    /// Signature spells within the oathbreaker's color identity.
    SignatureSpell(Colors),
    /// The user's suggested commanders, a fixed ranking rather than a search.
    Suggested,
}

impl Mode {
    /// The word for the card the mode picks, as the hint and the pick toast say it. The leader is an "oathbreaker" in formats with a signature spell and a "commander" everywhere else.
    #[must_use]
    pub fn noun(&self) -> &'static str {
        match self {
            Self::Commander(format) if format.has_signature_spell() => "oathbreaker",
            Self::Commander(_) | Self::Suggested => "commander",
            Self::Partner(_) => "partner",
            Self::Background => "background",
            Self::SignatureSpell(_) => "signature spell",
        }
    }

    /// Whether an up swipe saves to the commander maybeboard: only where the pile holds commanders.
    #[must_use]
    pub fn saves(&self) -> bool {
        matches!(
            self,
            Self::Commander(_) | Self::Partner(_) | Self::Suggested
        )
    }

    /// Adds this mode's constraint to `builder`. The suggested pile never searches, so it adds none.
    fn apply(&self, builder: &mut CardQueryBuilder) {
        match self {
            Self::Commander(format) => {
                builder.set_is_commander_in_format(*format);
            }
            Self::Partner(commander) => form::narrow_to_partners(builder, commander.as_deref()),
            Self::Background => {
                builder.set_is_background(true);
            }
            Self::SignatureSpell(colors) => {
                builder.set_is_signature_spell(true);
                builder.set_color_identity_within(colors.clone());
            }
            Self::Suggested => {}
        }
    }
}

/// The ranking [`Mode::Suggested`] deals, held by its host.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SuggestedPile {
    /// Best fit first, as the host's list shows them.
    pub suggestions: Vec<HttpCommanderSuggestion>,
    /// Decks the reasons count against.
    pub deck_count: u32,
    /// The color identity the host's Show chip narrows `suggestions` to; `None` is All.
    pub identity: Option<Vec<Color>>,
}

/// Where the overlay stands, for the client to pick what it draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// No card shows and a search or a next page is out.
    Loading,
    /// No card shows and the search failed.
    Failed,
    /// No card shows and nothing is out: the pile is spent or empty. [`State::message`] has the line to show, if any.
    Empty,
    /// Cards show: [`State::window`] and [`State::current`].
    Loaded,
}

/// What a user does on the overlay, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// The overlay opened. A reopen with the same mode and a pile left keeps it; the suggested pile is dealt afresh every time. The hint is offered on the first open.
    Open {
        /// The slot to fill; `None` deals nothing.
        mode: Option<Mode>,
        /// The card filter the search starts from.
        filter: CardQueryBuilder,
        /// The host's ranking, for [`Mode::Suggested`]; the default elsewhere.
        pile: SuggestedPile,
        /// Oracle ids kept out of searched pages: the host's own saves.
        excluded: HashSet<Uuid>,
    },
    /// The filter sheet applied `filter`: the pile is dealt again from the top.
    Filter(CardQueryBuilder),
    /// Refresh: the pile is dealt again from the top with `filter`.
    Refresh(CardQueryBuilder),
    /// The host's suggested pile changed while the overlay is open: the cards already swiped stay behind the top one and the new pile deals after them.
    Redeal(SuggestedPile),
    /// A page arrived for the search holding `ticket`.
    Found {
        /// The ticket from the [`Effect::SearchCommanders`] or [`Effect::LoadMore`] this answers.
        ticket: Ticket,
        /// The page as the server served it.
        cards: Vec<Card>,
    },
    /// The request holding `ticket` failed. The client has already logged or reported it, and raised its error toast where the request has one.
    Failed {
        /// The ticket from the request this answers.
        ticket: Ticket,
    },
    /// The maybeboard request holding `ticket` went through.
    Landed {
        /// The ticket from the [`Effect::AddMaybeboardCard`] or [`Effect::RemoveMaybeboardCard`] this answers.
        ticket: Ticket,
    },
    /// Skips the card on top, as its left exit commits. In [`Mode::Suggested`] it is dismissed for a week.
    Skip,
    /// Picks the card on top, as its right exit commits.
    Choose,
    /// Saves the card on top to the commander maybeboard, as its up exit commits. The pile moves on as for a skip.
    Save,
    /// Undoes the newest swipe.
    Undo,
    /// A printing was picked for the card on top.
    PickPrinting(Box<Card>),
    /// Closes the overlay.
    Back,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Searches the first page of a fresh pile and answers with [`Action::Found`] or [`Action::Failed`]. A failure is logged only.
    SearchCommanders {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The search.
        query: Box<CardQuery>,
    },
    /// Searches the next page of the pile and answers with [`Action::Found`] or [`Action::Failed`]. A failure is logged only.
    LoadMore {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The search.
        query: Box<CardQuery>,
    },
    /// Dismisses this oracle id's commander for a week. Nothing comes back; a failure is logged only.
    DismissCommander(Uuid),
    /// Lifts a dismissal. Nothing comes back; a failure is reported without a toast.
    UndismissCommander(Uuid),
    /// Saves this oracle id to the commander maybeboard and answers with [`Action::Landed`] or [`Action::Failed`].
    AddMaybeboardCard {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The commander's oracle id.
        oracle_id: Uuid,
    },
    /// Removes this oracle id from the commander maybeboard and answers with [`Action::Landed`] or [`Action::Failed`].
    RemoveMaybeboardCard {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The commander's oracle id.
        oracle_id: Uuid,
    },
    /// Plays the top card's entrance from this side: an undo brought it back.
    Enter(Direction),
    /// Hands the picked card to the host.
    Pick(Box<Card>),
    /// Goes to this screen.
    Navigate(Destination),
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

/// Whether the Filter button's dot lights for the client's `filter`: any user filter or explicit sort, since popularity order with no sort is the default.
#[must_use]
pub fn filter_lit(filter: &CardQueryBuilder) -> bool {
    !filter.is_empty() || filter.sort().is_some()
}

/// One swipe that moved the pile on, so undo can reverse it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Swipe {
    /// Left, or an up swipe that saved nothing.
    Skip,
    /// Up: saved this oracle id to the commander maybeboard.
    Save(Uuid),
    /// Left in [`Mode::Suggested`]: dismissed this oracle id for a week.
    Dismiss(Uuid),
}

impl StackAction for Swipe {
    fn exited(&self) -> Direction {
        match self {
            Self::Skip | Self::Dismiss(_) => Direction::Left,
            Self::Save(_) => Direction::Up,
        }
    }
}

/// A request out, waiting for its reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pending {
    /// The first page of the pile.
    Search,
    /// A next page, the `pulls`th of this load more.
    More { pulls: usize },
    /// An up swipe's save.
    Save,
    /// An undone save's removal.
    Unsave,
}

/// Whether the overlay has offered the one-time hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

/// Swipe select for as long as its overlay stays mounted. Starts with no mode and no card; send [`Action::Open`] when it opens.
///
/// Every request takes a fresh [`Ticket`] from one [`Generation`] and waits in a list keyed by it. A fresh deal drops the pile's searches from the list, so a page for an older pile changes nothing; maybeboard requests stay, so their toasts still land.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    mode: Option<Mode>,
    /// The mode the pile was dealt for.
    dealt: Option<Mode>,
    filter: CardQueryBuilder,
    excluded: HashSet<Uuid>,
    pile: SuggestedPile,
    /// The reasons behind every suggested card dealt, kept for the cards behind the top one.
    suggestions: Vec<HttpCommanderSuggestion>,
    stack: CardStack<Swipe>,
    offset: u32,
    exhausted: bool,
    failed: bool,
    waiting: Vec<(Ticket, Pending)>,
    generation: Generation,
    hint: HintOffer,
}

impl State {
    /// The overlay as it first mounts: no mode, no card.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order. When the pile turns empty it says so with a toast, unless [`State::message`] already has a line for it or the action leaves the overlay.
    ///
    /// A reply whose request is no longer waiting changes nothing.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        let leaves = matches!(action, Action::Choose | Action::Back);
        let before = self.status();
        let mut effects = self.apply(action);
        if !leaves
            && before != Status::Empty
            && self.status() == Status::Empty
            && self.message().is_none()
        {
            let noun = self.noun();
            // Cards still in the stack were dealt and swiped past; none means the search found nothing.
            let line = if self.stack.is_empty() {
                format!("No {noun}s match your filters")
            } else {
                NO_CARDS_LEFT.to_owned()
            };
            effects.push(Effect::Toast(Toast::quick(ToastKind::Info, line)));
        }
        keep_one(&mut effects, toast_line);
        effects
    }

    fn apply(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Open {
                mode,
                filter,
                pile,
                excluded,
            } => self.open(mode, filter, pile, excluded),
            Action::Filter(filter) => {
                self.filter = filter;
                self.deal()
            }
            Action::Refresh(filter) => {
                self.filter = filter;
                let mut effects = vec![Effect::Toast(Toast::quick(
                    ToastKind::Info,
                    STACK_REFRESHED,
                ))];
                effects.extend(self.deal());
                effects
            }
            Action::Redeal(pile) => {
                self.redeal(pile);
                Vec::new()
            }
            Action::Found { ticket, cards } => self.found(ticket, cards),
            Action::Failed { ticket } => {
                if self.take(ticket) == Some(Pending::Search) {
                    self.failed = true;
                }
                Vec::new()
            }
            Action::Landed { ticket } => match self.take(ticket) {
                Some(Pending::Save) => vec![Effect::Confirm(Toast::new(
                    ToastKind::Success,
                    SAVED,
                    MAYBEBOARD_TOAST,
                ))],
                Some(Pending::Unsave) => {
                    vec![Effect::Confirm(Toast::quick(ToastKind::Info, UNSAVED))]
                }
                _ => Vec::new(),
            },
            Action::Skip => self.skip(),
            Action::Choose => self.choose(),
            Action::Save => self.save(),
            Action::Undo => self.undo(),
            Action::PickPrinting(card) => {
                self.stack.replace_current(*card);
                Vec::new()
            }
            Action::Back => vec![Effect::Navigate(Destination::Previous)],
        }
    }

    /// Opens the overlay, keeping a pile dealt for the same mode.
    fn open(
        &mut self,
        mode: Option<Mode>,
        filter: CardQueryBuilder,
        pile: SuggestedPile,
        excluded: HashSet<Uuid>,
    ) -> Vec<Effect> {
        let mut effects = Vec::new();
        if self.hint == HintOffer::Waiting {
            self.hint = HintOffer::Offered;
            effects.push(Effect::OfferHint(OneTimeHint::SwipeSelect));
        }
        self.mode = mode;
        self.pile = pile;
        self.excluded = excluded;
        let Some(mode) = &self.mode else {
            return effects;
        };
        let keep =
            *mode != Mode::Suggested && self.dealt.as_ref() == Some(mode) && !self.stack.is_empty();
        if !keep {
            self.filter = filter;
            effects.extend(self.deal());
        }
        effects
    }

    /// Deals the pile afresh for the current mode: the suggested ranking, or a search's first page. Searches still out stop counting.
    fn deal(&mut self) -> Vec<Effect> {
        let Some(mode) = self.mode.clone() else {
            return Vec::new();
        };
        self.waiting
            .retain(|(_, p)| matches!(p, Pending::Save | Pending::Unsave));
        self.stack.reset();
        self.offset = 0;
        self.exhausted = false;
        self.failed = false;
        self.dealt = Some(mode.clone());
        if mode == Mode::Suggested {
            self.exhausted = true;
            self.suggestions.clone_from(&self.pile.suggestions);
            self.stack.replace(pile_cards(&self.pile));
            return Vec::new();
        }
        let Some(query) = self.query(&mode, 0) else {
            return Vec::new();
        };
        let ticket = self.wait(Pending::Search);
        vec![Effect::SearchCommanders {
            ticket,
            query: Box::new(query),
        }]
    }

    /// Deals the host's new suggested pile under the cards already swiped.
    fn redeal(&mut self, pile: SuggestedPile) {
        if self.mode != Some(Mode::Suggested) {
            return;
        }
        let fresh = pile_cards(&pile);
        let (cards, index, history) = self.stack.park_state();
        let dealt = swipe_select::redeal(&cards, index, &fresh, swipe_select::card_key);
        self.stack.restore(dealt, index, history);
        for suggestion in &pile.suggestions {
            let id = suggestion.card.scryfall_data.id;
            if !self
                .suggestions
                .iter()
                .any(|s| s.card.scryfall_data.id == id)
            {
                self.suggestions.push(suggestion.clone());
            }
        }
        self.pile = pile;
    }

    /// Lands a page: the first one replaces the pile, a next one joins it. A page the host's exclusions or the pile's own cards empty pulls the next, up to [`PAGE_PULLS`] per load more.
    fn found(&mut self, ticket: Ticket, cards: Vec<Card>) -> Vec<Effect> {
        let raw_empty = cards.is_empty();
        match self.take(ticket) {
            Some(Pending::Search) => {
                let page = self.without_excluded(cards);
                if raw_empty {
                    self.exhausted = true;
                }
                self.offset = PAGE_SIZE;
                let need_more = !raw_empty && page.is_empty();
                self.stack.replace(page);
                if need_more {
                    return self.load_more();
                }
                Vec::new()
            }
            Some(Pending::More { pulls }) => {
                if raw_empty {
                    self.exhausted = true;
                    return Vec::new();
                }
                let seen: HashSet<Uuid> = self
                    .stack
                    .cards()
                    .iter()
                    .map(|c| c.scryfall_data.id)
                    .collect();
                let page: Vec<Card> = self
                    .without_excluded(cards)
                    .into_iter()
                    .filter(|c| !seen.contains(&c.scryfall_data.id))
                    .collect();
                let Some(next) = self.offset.checked_add(PAGE_SIZE) else {
                    self.exhausted = true;
                    self.stack.append(page);
                    return Vec::new();
                };
                self.offset = next;
                if !page.is_empty() {
                    self.stack.append(page);
                    return Vec::new();
                }
                if pulls >= PAGE_PULLS {
                    return Vec::new();
                }
                self.pull(pulls.saturating_add(1))
            }
            _ => Vec::new(),
        }
    }

    /// Asks for the next page unless one is out, the pile is spent or the mode never searches.
    fn load_more(&mut self) -> Vec<Effect> {
        let loading = self
            .waiting
            .iter()
            .any(|(_, p)| matches!(p, Pending::More { .. }));
        if loading || self.exhausted {
            return Vec::new();
        }
        self.pull(1)
    }

    /// Sends the `pulls`th page request of a load more.
    fn pull(&mut self, pulls: usize) -> Vec<Effect> {
        let Some(mode) = self.mode.clone() else {
            return Vec::new();
        };
        if mode == Mode::Suggested {
            return Vec::new();
        }
        let Some(query) = self.query(&mode, self.offset) else {
            return Vec::new();
        };
        let ticket = self.wait(Pending::More { pulls });
        vec![Effect::LoadMore {
            ticket,
            query: Box::new(query),
        }]
    }

    /// Moves past the card on top and asks for the next page when few are left ahead.
    fn advance(&mut self) -> Vec<Effect> {
        let total = self.stack.len();
        self.stack.advance();
        if !self.exhausted && total.saturating_sub(self.stack.index()) <= LOAD_MORE_AT {
            return self.load_more();
        }
        Vec::new()
    }

    /// Skips the card on top; in [`Mode::Suggested`] it is dismissed for a week.
    fn skip(&mut self) -> Vec<Effect> {
        let Some(oracle_id) = self.stack.current().map(|c| c.scryfall_data.oracle_id) else {
            return Vec::new();
        };
        let mut effects = self.advance();
        match oracle_id {
            Some(oracle_id) if self.is_suggested() => {
                self.stack.record(Swipe::Dismiss(oracle_id));
                effects.push(Effect::DismissCommander(oracle_id));
            }
            _ => self.stack.record(Swipe::Skip),
        }
        effects.push(Effect::Confirm(Toast::quick(ToastKind::Info, SKIPPED)));
        effects
    }

    /// Picks the card on top and hands it to the host.
    fn choose(&self) -> Vec<Effect> {
        let Some(card) = self.stack.current() else {
            return Vec::new();
        };
        vec![
            Effect::Confirm(Toast::quick(ToastKind::Success, self.selected())),
            Effect::Pick(Box::new(card.clone())),
        ]
    }

    /// Saves the card on top to the commander maybeboard and moves past it.
    fn save(&mut self) -> Vec<Effect> {
        if !self.can_save() {
            return Vec::new();
        }
        let Some(oracle_id) = self.stack.current().map(|c| c.scryfall_data.oracle_id) else {
            return Vec::new();
        };
        let mut effects = self.advance();
        let Some(oracle_id) = oracle_id else {
            self.stack.record(Swipe::Skip);
            effects.push(Effect::Toast(Toast::normal(ToastKind::Error, CANT_SAVE)));
            return effects;
        };
        self.stack.record(Swipe::Save(oracle_id));
        let ticket = self.wait(Pending::Save);
        effects.push(Effect::AddMaybeboardCard { ticket, oracle_id });
        effects
    }

    /// Brings the last swiped card back from the left, and reverses its save or dismissal.
    fn undo(&mut self) -> Vec<Effect> {
        if self.stack.index() == 0 {
            return vec![Effect::Toast(Toast::normal(
                ToastKind::Info,
                NOTHING_TO_UNDO,
            ))];
        }
        let swipe = self.stack.pop_action().unwrap_or(Swipe::Skip);
        self.stack.step_back(&swipe);
        self.stack.cancel_entering();
        let mut effects = vec![Effect::Enter(Direction::Left)];
        match swipe {
            Swipe::Save(oracle_id) => {
                let ticket = self.wait(Pending::Unsave);
                effects.push(Effect::RemoveMaybeboardCard { ticket, oracle_id });
            }
            Swipe::Dismiss(oracle_id) => {
                effects.push(Effect::UndismissCommander(oracle_id));
                effects.push(Effect::Confirm(Toast::quick(ToastKind::Info, UNDID_SKIP)));
            }
            Swipe::Skip => {
                effects.push(Effect::Confirm(Toast::quick(ToastKind::Info, UNDID_SKIP)));
            }
        }
        effects
    }

    /// The search for `mode` from `offset`: the filter with the mode's constraint, no tokens and one page. No sort is pinned, so a sortless filter gets the server's popularity order.
    fn query(&self, mode: &Mode, offset: u32) -> Option<CardQuery> {
        let mut builder = self.filter.clone();
        mode.apply(&mut builder);
        builder.set_is_token(false);
        builder.set_limit(PAGE_SIZE);
        builder.set_offset(offset);
        builder.build().ok()
    }

    /// `cards` without the host's excluded oracle ids and, on [`Mode::Partner`] with a commander, without the cards it cannot pair with.
    fn without_excluded(&self, cards: Vec<Card>) -> Vec<Card> {
        let kept: Vec<Card> = cards
            .into_iter()
            .filter(|c| {
                !c.scryfall_data
                    .oracle_id
                    .is_some_and(|o| self.excluded.contains(&o))
            })
            .collect();
        match &self.mode {
            Some(Mode::Partner(Some(commander))) => form::legal_partners(commander, kept),
            _ => kept,
        }
    }

    /// The pick toast's words, "Commander selected".
    fn selected(&self) -> String {
        let mut label = self.noun().to_owned();
        if let Some(first) = label.get_mut(..1) {
            first.make_ascii_uppercase();
        }
        format!("{label} selected")
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

    /// Where the overlay stands.
    #[must_use]
    pub fn status(&self) -> Status {
        if !self.stack.window().is_empty() {
            Status::Loaded
        } else if self
            .waiting
            .iter()
            .any(|(_, p)| matches!(p, Pending::Search | Pending::More { .. }))
        {
            Status::Loading
        } else if self.failed {
            Status::Failed
        } else {
            Status::Empty
        }
    }

    /// The line in place of the card when the host's suggested pile has nothing under its Show chip, "No Jeskai commanders in your top 100".
    #[must_use]
    pub fn message(&self) -> Option<String> {
        if !self.is_suggested() || !self.pile.suggestions.is_empty() {
            return None;
        }
        self.pile
            .identity
            .as_deref()
            .map(playstyle::empty_identity_line)
    }

    /// The mode the overlay last opened with.
    #[must_use]
    pub fn mode(&self) -> Option<&Mode> {
        self.mode.as_ref()
    }

    /// Whether the overlay deals the suggested ranking.
    #[must_use]
    pub fn is_suggested(&self) -> bool {
        self.mode == Some(Mode::Suggested)
    }

    /// The overlay's title.
    #[must_use]
    pub fn title(&self) -> &'static str {
        if self.is_suggested() {
            SUGGESTED_TITLE
        } else {
            TITLE
        }
    }

    /// The word for the card the mode picks; [`FALLBACK_NOUN`] before a mode.
    #[must_use]
    pub fn noun(&self) -> &'static str {
        self.mode.as_ref().map_or(FALLBACK_NOUN, Mode::noun)
    }

    /// Whether an up swipe saves, so the client offers it.
    #[must_use]
    pub fn can_save(&self) -> bool {
        self.mode.as_ref().is_some_and(Mode::saves)
    }

    /// The cards a swipe stack draws: up to [`crate::stack::STACK_DEPTH`] from the top one.
    #[must_use]
    pub fn window(&self) -> &[Card] {
        self.stack.window()
    }

    /// The card on top, which the details and printings sheets describe.
    #[must_use]
    pub fn current(&self) -> Option<&Card> {
        self.stack.current()
    }

    /// Why the card on top was suggested, in [`Mode::Suggested`].
    #[must_use]
    pub fn reasons(&self) -> Option<&HttpSuggestionReasons> {
        let card = self.stack.current()?;
        self.suggestions
            .iter()
            .find(|s| s.card.scryfall_data.id == card.scryfall_data.id)
            .map(|s| &s.reasons)
    }

    /// Decks the suggested reasons count against.
    #[must_use]
    pub fn deck_count(&self) -> u32 {
        self.pile.deck_count
    }

    /// The color identity the host's Show chip picked for the suggested pile, which lights its Filter dot.
    #[must_use]
    pub fn identity(&self) -> Option<&[Color]> {
        self.pile.identity.as_deref()
    }
}

/// The cards of `pile`, in its order.
fn pile_cards(pile: &SuggestedPile) -> Vec<Card> {
    pile.suggestions.iter().map(|s| s.card.clone()).collect()
}
