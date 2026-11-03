//! The commander swipe, the app's `SwipeSelect` overlay in its two commander modes: Suggested deals Playstyle's held ranking narrowed by the same identity filter, and Commander deals `search_commanders` for the Commander format, most played first, leaving out oracle ids the host already has. Playstyle opens it with `s`; the Commander maybeboard opens it with its saved oracle ids excluded.
//!
//! Right picks the card on top: it closes the overlay and starts a deck with it through [`create_deck_with`]. Up saves it to the commander maybeboard, left skips it (in Suggested, hiding it for a week, quietly), and down undoes the last swipe, lifting a save or a dismissal. Closing leaves a [`Closed`] in the handoff, which tells the host to refetch and, from Suggested, which identity the Filter left picked.
//!
//! The app's eye button opens a details sheet over the card; the terminal shows the card's text and, in Suggested, its reasons right under it. Filter in Suggested picks one color identity from the Show chips; in Commander it opens the card filter sheet, whose Apply deals the stack again from the top.
//!
//! The pile, the paging, the swipes, their undo and their toasts are the shared `zwipe_ui::screens::swipe_select` model; this screen keeps the host's held list and its Filter, the card filter, the motion and the create gate, and performs the model's effects through [`dispatch`].

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::time::Instant;

use crossterm::event::KeyCode;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::query::CardQuery;
use zwipe_core::domain::deck::format::Format;
use zwipe_core::http::contracts::suggestions::{HttpCommanderSuggestion, HttpSuggestionReasons};
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::generation::Ticket;
use zwipe_ui::hint::{HintContent, SwipeSelectMode, one_time};
use zwipe_ui::playstyle;
use zwipe_ui::screens::swipe_select::{self as model, Action, Destination, Effect, State};
use zwipe_ui::stack::Direction;

use super::card_sheets::filter::{FilterSheet, SheetOutcome};
use super::playstyle::{DeckLimit, create_deck_with, suggestion_colors};
use super::{Screen, card_sheets};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::card_motion::CardMotion;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The ranking a Suggested swipe deals: the host's whole held list, the deck count its reasons count against, and the identity its Show row picked (`None` is All). The swipe narrows it before the model deals it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SuggestedPile {
    pub(crate) held: Vec<HttpCommanderSuggestion>,
    pub(crate) deck_count: u32,
    pub(crate) identity: Option<Vec<Color>>,
}

impl SuggestedPile {
    /// The pile as the model deals it: the held list narrowed to the picked identity.
    fn dealt(&self) -> model::SuggestedPile {
        model::SuggestedPile {
            suggestions: playstyle::of_identity(
                &self.held,
                suggestion_colors,
                self.identity.as_deref(),
            ),
            deck_count: self.deck_count,
            identity: self.identity.clone(),
        }
    }
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

/// The Suggested Filter sheet, open with its chip cursor (0 is All).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Filter {
    pub(crate) cursor: usize,
}

/// The commander swipe: the shared `swipe_select` model, plus what only the terminal keeps.
#[derive(Debug)]
pub(crate) struct CommanderSwipeView {
    pub(crate) model: State,
    /// Suggested: the host's held list and the identity the Filter picked. `None` in Commander.
    pub(crate) suggested: Option<SuggestedPile>,
    /// Commander: the card filter as last applied, which each search starts from.
    pub(crate) card_filter: CardQueryBuilder,
    /// The create gate, as the host saw it when it opened the swipe.
    pub(crate) limit: DeckLimit,
    pub(crate) filter: Option<Filter>,
    /// Commander mode's card filter sheet, while it is open over the stack.
    pub(crate) sheet: Option<FilterSheet>,
    /// The swiped card flying off, or the undone one coming back. A pick closes the swipe, so it has no exit.
    pub(crate) motion: Option<CardMotion<Card>>,
}

impl CommanderSwipeView {
    pub(crate) fn new(deal: &Deal, limit: DeckLimit) -> Self {
        Self {
            model: State::new(),
            suggested: match deal {
                Deal::Suggested(pile) => Some(pile.clone()),
                Deal::Commander { .. } => None,
            },
            card_filter: CardQueryBuilder::new(),
            limit,
            filter: None,
            sheet: None,
            motion: None,
        }
    }

    /// The app's `swipe_select` hint for this mode: the suggested deal, or picking a commander.
    pub(crate) fn hint(&self) -> HintContent {
        let mode = if self.suggested.is_some() {
            SwipeSelectMode::Suggested { has_filter: true }
        } else {
            SwipeSelectMode::Picking("commander")
        };
        one_time::swipe_select(mode)
    }

    /// The card on top.
    pub(crate) fn current(&self) -> Option<&Card> {
        self.model.current()
    }

    /// Why the card on top was suggested, in Suggested.
    pub(crate) fn reasons(&self) -> Option<&HttpSuggestionReasons> {
        self.model.reasons()
    }

    /// The identities the Filter offers: those among the held list plus the picked one.
    pub(crate) fn identities(&self) -> Vec<Vec<Color>> {
        self.suggested.as_ref().map_or_else(Vec::new, |pile| {
            colors::identities_present(
                pile.held
                    .iter()
                    .map(suggestion_colors)
                    .chain(pile.identity.as_deref()),
            )
        })
    }

    /// Picks the Filter chip at `index` (0 is All; the picked identity again is All), and the model deals the narrowed pile behind the cards already swiped.
    pub(crate) fn pick_identity(&mut self, index: usize, cx: &mut Context<'_>) {
        let identities = self.identities();
        let Some(pile) = &mut self.suggested else {
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
        let dealt = pile.dealt();
        if let Some(filter) = &mut self.filter
            && dealt.identity.is_none()
        {
            filter.cursor = 0;
        }
        dispatch(self, Action::Redeal(dealt), cx);
    }

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

    /// Sends the card on top off by `direction`, then hands the swipe to the model.
    pub(crate) fn swipe(&mut self, direction: Direction, action: Action, cx: &mut Context<'_>) {
        if let Some(card) = self.current().cloned() {
            self.motion = Some(CardMotion::exit(direction, card, Instant::now()));
        }
        dispatch(self, action, cx);
    }

    /// Up: saves the card on top to the commander maybeboard, where the mode saves.
    pub(crate) fn save(&mut self, cx: &mut Context<'_>) {
        if self.model.can_save() {
            self.swipe(Direction::Up, Action::Save, cx);
        }
    }

    /// Deals afresh from the top: the host's pile again, or a new search.
    pub(crate) fn refresh(&mut self, cx: &mut Context<'_>) {
        self.motion = None;
        let filter = self.card_filter.clone();
        dispatch(self, Action::Refresh(filter), cx);
    }

    /// Whether Filter carries its dot: an identity picked in Suggested, a criterion or sort in Commander.
    pub(crate) fn filter_dot(&self) -> bool {
        match &self.suggested {
            Some(pile) => pile.identity.is_some(),
            None => model::filter_lit(&self.card_filter),
        }
    }

    /// Opens the Filter: the identity chips in Suggested, the card filter sheet without its format section in Commander.
    pub(crate) fn open_filter(&mut self, cx: &mut Context<'_>) {
        if let Some(pile) = &self.suggested {
            let cursor = pile
                .identity
                .as_ref()
                .and_then(|picked| self.identities().iter().position(|i| i == picked))
                .map_or(0, |i| i.saturating_add(1));
            self.filter = Some(Filter { cursor });
        } else {
            let config = SheetConfig {
                show_format: false,
                ..SheetConfig::default()
            };
            self.sheet = Some(FilterSheet::open(cx, &self.card_filter, config));
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
                self.motion = None;
                self.card_filter = *applied;
                let filter = self.card_filter.clone();
                dispatch(self, Action::Filter(filter), cx);
            }
        }
    }

    /// Closes the swipe, leaving its host the identity picked and the word to refetch.
    pub(crate) fn close(&self, cx: &mut Context<'_>) {
        let identity = self
            .suggested
            .as_ref()
            .and_then(|pile| pile.identity.clone());
        cx.shared.caches.handoff.commander_swipe = Some(Closed { identity });
        cx.back();
    }
}

/// Applies `action` to the model and performs what it returns.
pub(crate) fn dispatch(view: &mut CommanderSwipeView, action: Action, cx: &mut Context<'_>) {
    for effect in view.model.update(action) {
        match effect {
            Effect::OfferHint(offer) => {
                hint::once_with(cx, offer.key(), view.hint());
            }
            Effect::SearchCommanders { ticket, query } | Effect::LoadMore { ticket, query } => {
                cx.request(Request::Search { ticket, query }.into());
            }
            Effect::DismissCommander(oracle_id) => cx.request(Request::Dismiss(oracle_id).into()),
            Effect::UndismissCommander(oracle_id) => {
                cx.request(Request::Undismiss(oracle_id).into());
            }
            Effect::AddMaybeboardCard { ticket, oracle_id } => {
                cx.request(Request::Save { ticket, oracle_id }.into());
            }
            Effect::RemoveMaybeboardCard { ticket, oracle_id } => {
                cx.request(Request::Unsave { ticket, oracle_id }.into());
            }
            Effect::Enter(direction) => {
                view.motion = Some(CardMotion::enter(direction, Instant::now()));
            }
            Effect::Pick(card) => {
                view.close(cx);
                create_deck_with(cx, &card, view.limit);
            }
            Effect::Navigate(Destination::Previous) => view.close(cx),
            Effect::Toast(toast) => cx.show(toast),
        }
    }
}

/// Opens the swipe over its host, dealing `deal`, with the host's create gate. Commander mode asks for its first page; both show the swipe hint the first time.
pub(crate) fn open(cx: &mut Context<'_>, deal: Deal, limit: DeckLimit) {
    let mut view = CommanderSwipeView::new(&deal, limit);
    let (mode, pile, excluded) = match deal {
        Deal::Suggested(pile) => (model::Mode::Suggested, pile.dealt(), HashSet::new()),
        Deal::Commander { exclude } => (
            model::Mode::Commander(Format::Commander),
            model::SuggestedPile::default(),
            exclude,
        ),
    };
    dispatch(
        &mut view,
        Action::Open {
            mode: Some(mode),
            filter: CardQueryBuilder::new(),
            pile,
            excluded,
        },
        cx,
    );
    cx.push(Screen::CommanderSwipe(Box::new(view)));
}

/// Shows the hint again, for `?`.
pub(crate) fn reopen_hint(view: &CommanderSwipeView, cx: &mut Context<'_>) {
    hint::open(cx, view.hint());
}

/// The swipe's work, the model's requests carrying their tickets.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    /// A page of commanders, the first of a fresh pile or the next one.
    Search {
        ticket: Ticket,
        query: Box<CardQuery>,
    },
    /// Saves to the commander maybeboard.
    Save { ticket: Ticket, oracle_id: Uuid },
    /// Takes a save back off, for undo.
    Unsave { ticket: Ticket, oracle_id: Uuid },
    /// Hides a suggestion for a week, quietly.
    Dismiss(Uuid),
    /// Lifts a dismissal, for undo; a failure is reported without a toast.
    Undismiss(Uuid),
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Search { .. } => "search",
            Self::Save { .. } => "save",
            Self::Unsave { .. } => "unsave",
            Self::Dismiss(..) => "dismiss",
            Self::Undismiss(..) => "undismiss",
        }
    }
}

/// The swipe's results, each carrying its ticket back.
#[derive(Debug)]
pub(crate) enum Update {
    /// A page arrived.
    Found { ticket: Ticket, cards: Vec<Card> },
    /// A save or its undo went through.
    Landed { ticket: Ticket },
    /// A request failed: `Some` is the error toast a save or its undo raises first, `None` a quiet search.
    Failed {
        ticket: Ticket,
        message: Option<String>,
    },
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
    let failed = |ticket: Ticket, e: &zwipe_client::ClientError| {
        screens::failure(e, |message| {
            Update::Failed {
                ticket,
                message: Some(message),
            }
            .into()
        })
    };
    match request {
        Request::Search { ticket, query } => Some(
            match async {
                client
                    .search_commanders(&query, &account.fresh().await?)
                    .await
            }
            .await
            {
                Ok(cards) => Update::Found { ticket, cards }.into(),
                Err(e) => screens::failure(&e, |_| {
                    Update::Failed {
                        ticket,
                        message: None,
                    }
                    .into()
                }),
            },
        ),
        Request::Save { ticket, oracle_id } => Some(
            match async {
                client
                    .add_commander_maybeboard_card(oracle_id, &account.fresh().await?)
                    .await
            }
            .await
            {
                Ok(()) => Update::Landed { ticket }.into(),
                Err(e) => failed(ticket, &e),
            },
        ),
        Request::Unsave { ticket, oracle_id } => Some(
            match async {
                client
                    .remove_commander_maybeboard_card(oracle_id, &account.fresh().await?)
                    .await
            }
            .await
            {
                Ok(()) => Update::Landed { ticket }.into(),
                Err(e) => failed(ticket, &e),
            },
        ),
        // Quiet like the app's: the pile already moved on, and a failure only means the commander comes back sooner.
        Request::Dismiss(oracle_id) => {
            let _quiet = async {
                client
                    .dismiss_commander(oracle_id, &account.fresh().await?)
                    .await
            }
            .await;
            None
        }
        Request::Undismiss(oracle_id) => {
            if let Err(e) = async {
                client
                    .undismiss_commander(oracle_id, &account.fresh().await?)
                    .await
            }
            .await
            {
                let _reported = screens::shown(&e);
            }
            None
        }
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    // A failed save's toast goes up before the lookup, so it shows even when the swipe has closed.
    if let Update::Failed {
        message: Some(message),
        ..
    } = &update
    {
        cx.toast(ToastKind::Error, message.clone());
    }
    let Some(view) = nav.find_mut(pick) else {
        return;
    };
    let action = match update {
        Update::Found { ticket, cards } => Action::Found { ticket, cards },
        Update::Landed { ticket } => Action::Landed { ticket },
        Update::Failed { ticket, .. } => Action::Failed { ticket },
    };
    dispatch(view, action, cx);
}
