//! Add deck cards: swipe through search results or the maybeboard to build a deck.
//!
//! What the screen knows and does is `zwipe_ui::screens::add_cards`; this
//! file draws its state, performs its effects and keeps the gestures, the
//! card's entrance, image easing and the sheets.

use components::{
    Chip, HintBullet, HintBullets, HintDialog, HintKey, HintLine, TOAST_NORMAL, ToastOptions,
    Toasts, use_toast,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::query::CardQuery;
use zwipe_core::domain::deck::Board;
use zwipe_core::domain::user::models::hints::HINT_ADD_DECK_CARDS;
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::add_cards::{
    self, Action, Destination, Effect, Pile, Source, State, UndoClaim,
};
use zwipe_ui::stack;
use zwipe_ui::stores::parked;

use super::components::card_info::{CardDetailsSheet, CardInfoDisplay, CardSkeleton};
use super::components::printing_sheet::PrintingSheet;
use crate::inbound::components::action_dock::{self, DockFilter, DockIcon, EyeIcon};
use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::inbound::components::hint_dialog::{self, HintColored};
use crate::inbound::components::interactions::swipe::SwipeStack;
use crate::inbound::components::interactions::swipe::config::SwipeConfig;
use crate::inbound::components::interactions::swipe::direction::Direction;
use crate::inbound::components::on_bump::use_on_bump;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::flush_loop;
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::components::telemetry::vocabulary::{DeckScreen, Screen, component, screen};
use crate::inbound::components::{guide_link, ui_toast};
use crate::inbound::screens::deck::card::components::action_history::AddAction;
use crate::inbound::screens::deck::card::components::add_stack_cache::{
    AddStackCache, ParkedStack,
};
use crate::inbound::screens::deck::card::components::filter_store::{FilterScope, FilterStore};
use crate::inbound::screens::deck::card::components::flippable_card_image;
use crate::inbound::screens::deck::card::components::undo_log::{UndoAction, UndoStore};
use crate::inbound::screens::deck::card::filter::card_filter_sheet::CardFilterSheet;
use crate::inbound::screens::deck::missing_deck;

/// Pixels a swipe travels to commit.
const SWIPE_DISTANCE_PX: f64 = 90.0;
/// Pixels per millisecond a flick needs to commit.
const SWIPE_SPEED_PX_PER_MS: f64 = 2.5;
/// Pixels a downward pull on the end card travels to undo the last swipe.
const BACK_PULL_PX: f64 = 60.0;

/// The screen's model with the handles that perform its effects. Copy, so
/// every handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    deck_id: Uuid,
    navigator: Navigator,
    toast: Toasts,
    authed: Authed,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    usage_buffer: Signal<UsageBuffer>,
    undo_store: UndoStore,
    filter_store: FilterStore,
    /// The filter the sheet edits; the model reads it on every rerun.
    filter_builder: Signal<CardQueryBuilder>,
    /// Bumped to rerun the filter; the filter sheet's sections reset on it.
    filter_reset_counter: Signal<u32>,
    /// The side the Search card on top enters from after an undo; the swipe
    /// stack clears it when the entrance ends.
    search_entering: Signal<Option<Direction>>,
    /// The same for the maybeboard pile.
    maybeboard_entering: Signal<Option<Direction>>,
    hint_open: Signal<bool>,
}

impl Driver {
    /// Applies `action` to the model and performs what it returns.
    fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    /// Performs one effect, feeding any reply back through [`Self::dispatch`].
    fn perform(self, effect: Effect) {
        let Self {
            deck_id,
            navigator,
            toast,
            session,
            client,
            usage_buffer,
            mut undo_store,
            mut filter_store,
            mut filter_builder,
            mut filter_reset_counter,
            hint_open,
            ..
        } = self;
        match effect {
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(hint.key(), session, client, hint_open);
            }
            Effect::FetchDeck { ticket, deck_id } => self.fetch_deck(ticket, deck_id),
            Effect::RefreshMaybeboard { ticket, deck_id } => {
                self.refresh_maybeboard(ticket, deck_id);
            }
            Effect::Search {
                ticket,
                deck_id,
                query,
            } => self.search(ticket, deck_id, query),
            Effect::LoadMore {
                ticket,
                deck_id,
                query,
            } => self.load_more(ticket, deck_id, query),
            Effect::SkipCard { deck_id, oracle_id } => self.skip_card(deck_id, oracle_id),
            Effect::UnskipCard { deck_id, oracle_id } => self.unskip_card(deck_id, oracle_id),
            Effect::AddCard {
                ticket,
                deck_id,
                card,
                board,
            } => self.add_card(ticket, deck_id, &card, board),
            Effect::DeleteUndoneAdd { deck_id, card_id } => {
                self.delete_undone_add(deck_id, card_id);
            }
            Effect::PromoteCard {
                ticket,
                deck_id,
                card_id,
            } => self.promote_card(ticket, deck_id, card_id),
            Effect::TakeUndo {
                ticket,
                deck_id,
                claim,
            } => {
                let taken = undo_store.take_newest(deck_id, |a| claims(claim, a));
                self.dispatch(Action::Taken {
                    ticket,
                    taken: taken.map(Into::into),
                });
            }
            Effect::UndoAdd {
                ticket,
                deck_id,
                card_id,
                board,
            } => self.undo_add(ticket, deck_id, card_id, board),
            Effect::UndoPromote {
                ticket,
                deck_id,
                card_id,
            } => self.undo_promote(ticket, deck_id, card_id),
            Effect::RecordUndo { deck_id, action } => {
                undo_store.push(deck_id, action.into());
            }
            Effect::Enter { pile, from } => self.entering(pile).set(Some(direction(from))),
            Effect::StopEnter(pile) => self.entering(pile).set(None),
            Effect::SetFilter(filter) => filter_builder.set(filter),
            Effect::Rerun => {
                let current = *filter_reset_counter.peek();
                filter_reset_counter.set(current.wrapping_add(1));
            }
            Effect::ParkFilter { source, filter } => {
                filter_store.park(scope(source), deck_id, filter);
            }
            Effect::EaseImages => flippable_card_image::reset_image_ease(),
            Effect::PileEnd {
                deck_id,
                kind,
                synergy,
            } => usage_buffer.peek().record_pile_end(deck_id, kind, synergy),
            Effect::Navigate(Destination::Previous) => navigator.go_back(),
            Effect::Toast(raised) => ui_toast::raise(toast, raised),
        }
    }

    /// Fetches the deck quietly.
    fn fetch_deck(self, ticket: Ticket, deck_id: Uuid) {
        let Self {
            authed,
            navigator,
            toast,
            ..
        } = self;
        spawn(async move {
            // Quiet: the screen works without deck context; telemetry
            // still records the failure. A missing deck is the exception.
            let deck = authed
                .try_run_quiet("load_deck_context", |c, s| async move {
                    c.fetch_deck(deck_id, &s).await
                })
                .await;
            if let Err(e @ ClientError::NotFound(_)) = &deck {
                toast.error(
                    e.to_user_message(),
                    ToastOptions::default().duration(TOAST_NORMAL),
                );
            }
            self.dispatch(match missing_deck::settle(navigator, deck) {
                Some(deck) => Action::DeckLoaded {
                    ticket,
                    deck: Box::new(deck),
                },
                None => Action::Failed { ticket },
            });
        });
    }

    /// Fetches the deck again for its maybeboard.
    fn refresh_maybeboard(self, ticket: Ticket, deck_id: Uuid) {
        let Self { authed, .. } = self;
        spawn(async move {
            let deck = authed
                .run("search_cards", |c, s| async move {
                    c.fetch_deck(deck_id, &s).await
                })
                .await;
            self.dispatch(match deck {
                Some(deck) => Action::DeckLoaded {
                    ticket,
                    deck: Box::new(deck),
                },
                None => Action::Failed { ticket },
            });
        });
    }

    /// Searches from the top, flushing buffered usage first.
    fn search(self, ticket: Ticket, deck_id: Uuid, query: CardQuery) {
        let Self {
            authed,
            session,
            client,
            usage_buffer,
            ..
        } = self;
        usage_buffer.peek().record_search();
        spawn(async move {
            // Buffered usage feeds synergy ordering, so it goes out
            // before the search.
            flush_loop::flush_once(&usage_buffer(), &client, &session).await;
            let reply = authed
                .try_run("search_cards", |c, s| async move {
                    c.search_deck_cards(deck_id, &query, &s).await
                })
                .await;
            self.dispatch(match reply {
                Ok(page) => Action::Searched { ticket, page },
                Err(_) => Action::Failed { ticket },
            });
        });
    }

    /// Fetches the next page.
    fn load_more(self, ticket: Ticket, deck_id: Uuid, query: CardQuery) {
        let Self {
            authed,
            usage_buffer,
            ..
        } = self;
        usage_buffer.peek().record_search();
        spawn(async move {
            let reply = authed
                .try_run("load_more", |c, s| async move {
                    c.search_deck_cards(deck_id, &query, &s).await
                })
                .await;
            self.dispatch(match reply {
                Ok(page) => Action::Searched { ticket, page },
                Err(_) => Action::Failed { ticket },
            });
        });
    }

    /// Posts a skip quietly.
    fn skip_card(self, deck_id: Uuid, oracle_id: Uuid) {
        let Self { authed, .. } = self;
        spawn(async move {
            // Quiet: fires on every left swipe; a per-swipe toast on a
            // bad connection would bury the screen, and the visual
            // skip already happened.
            let _ = authed
                .run_quiet("skip_card", |c, s| async move {
                    c.skip_deck_card(deck_id, oracle_id, &s).await
                })
                .await;
        });
    }

    /// Deletes a skip.
    fn unskip_card(self, deck_id: Uuid, oracle_id: Uuid) {
        let Self { authed, .. } = self;
        spawn(async move {
            let _ = authed
                .run("undo_skip", |c, s| async move {
                    c.unskip_deck_card(deck_id, oracle_id, &s).await
                })
                .await;
        });
    }

    /// Deletes a card whose add was undone while it was in flight.
    fn delete_undone_add(self, deck_id: Uuid, card_id: Uuid) {
        let Self { authed, .. } = self;
        spawn(async move {
            let _ = authed
                .run("undo_add", |c, s| async move {
                    c.delete_deck_card(deck_id, card_id, &s).await
                })
                .await;
        });
    }

    /// Creates one copy of `card` on `board`.
    fn add_card(self, ticket: Ticket, deck_id: Uuid, card: &Card, board: Board) {
        let Self { authed, .. } = self;
        let request = add_cards::add_request(card, board);
        let operation = if board.is_maybeboard() {
            "add_maybeboard"
        } else {
            "add_card"
        };
        spawn(async move {
            let added = authed
                .run(operation, |c, s| async move {
                    c.create_deck_card(deck_id, &request, &s).await
                })
                .await;
            self.dispatch(match added {
                Some(deck_card) => Action::Added {
                    ticket,
                    deck_card: Box::new(deck_card),
                },
                None => Action::Failed { ticket },
            });
        });
    }

    /// Moves a maybeboard card to the deck.
    fn promote_card(self, ticket: Ticket, deck_id: Uuid, card_id: Uuid) {
        let Self { authed, .. } = self;
        let request = add_cards::board_request(Board::Deck);
        spawn(async move {
            let moved = authed
                .run("promote_maybeboard", |c, s| async move {
                    c.update_deck_card(deck_id, card_id, &request, &s).await
                })
                .await;
            self.dispatch(match moved {
                Some(_) => Action::Landed { ticket },
                None => Action::Failed { ticket },
            });
        });
    }

    /// The entrance signal for `pile`.
    fn entering(self, pile: Source) -> Signal<Option<Direction>> {
        match pile {
            Source::Search => self.search_entering,
            Source::Maybeboard => self.maybeboard_entering,
        }
    }

    /// Undoes an add to `board`: deletes the card.
    fn undo_add(self, ticket: Ticket, deck_id: Uuid, card_id: Uuid, board: Board) {
        let Self { toast, client, .. } = self;
        let operation = if board.is_maybeboard() {
            "undo_maybe"
        } else {
            "undo_add"
        };
        // Facade holdout: staged compensation (a refresh failure rewinds the
        // undo; a failed delete leaves the add standing).
        spawn(async move {
            let Some(session) = self.fresh_session(operation).await else {
                self.dispatch(Action::NotSent { ticket });
                return;
            };
            match client().delete_deck_card(deck_id, card_id, &session).await {
                Ok(()) => self.dispatch(Action::Landed { ticket }),
                Err(e) => {
                    if board.is_maybeboard() {
                        tracing::warn!("undo maybeboard (delete deck card) failed: {e}");
                    } else {
                        tracing::warn!("undo add (delete deck card) failed: {e}");
                    }
                    toast.error(format!("Failed to undo: {e}"), ToastOptions::default());
                    self.dispatch(Action::Failed { ticket });
                }
            }
        });
    }

    /// Undoes a move to the deck: sends the card back to the maybeboard.
    fn undo_promote(self, ticket: Ticket, deck_id: Uuid, card_id: Uuid) {
        let Self { toast, client, .. } = self;
        let request = add_cards::board_request(Board::Maybeboard);
        // Facade holdout: staged compensation (a refresh failure cancels the
        // entrance; a failed move leaves the promote standing).
        spawn(async move {
            let Some(session) = self.fresh_session("undo_maybeboard").await else {
                self.dispatch(Action::NotSent { ticket });
                return;
            };
            match client()
                .update_deck_card(deck_id, card_id, &request, &session)
                .await
            {
                Ok(_) => self.dispatch(Action::Landed { ticket }),
                Err(e) => {
                    tracing::warn!("undo promote failed: {e}");
                    toast.error(format!("Failed to undo: {e}"), ToastOptions::default());
                    self.dispatch(Action::Failed { ticket });
                }
            }
        });
    }

    /// A fresh session for an undo's call. On failure it reports the error
    /// under `operation` and raises its toast.
    async fn fresh_session(self, operation: &'static str) -> Option<Session> {
        let Self {
            toast,
            session,
            client,
            usage_buffer,
            ..
        } = self;
        match session.ensure_fresh(client).await {
            Ok(session) => Some(session),
            Err(e) => {
                usage_buffer.peek().report_error(
                    screen::DECK_CARD_ADD,
                    component::NONE,
                    operation,
                    &e,
                );
                toast.error(e.to_user_message(), ToastOptions::default());
                None
            }
        }
    }
}

/// The filter store's scope for a source.
fn scope(source: Source) -> FilterScope {
    match source {
        Source::Search => FilterScope::AddSearch,
        Source::Maybeboard => FilterScope::AddMaybeboard,
    }
}

/// The app's swipe direction for a side the model names.
fn direction(side: stack::Direction) -> Direction {
    match side {
        stack::Direction::Left => Direction::Left,
        stack::Direction::Right => Direction::Right,
        stack::Direction::Up => Direction::Up,
    }
}

/// The model's side for a swipe that commits a card.
fn side(direction: Direction) -> Option<stack::Direction> {
    match direction {
        Direction::Left => Some(stack::Direction::Left),
        Direction::Right => Some(stack::Direction::Right),
        Direction::Up => Some(stack::Direction::Up),
        Direction::Down => None,
    }
}

/// Whether `action` is the undo log entry `claim` takes back.
fn claims(claim: UndoClaim, action: &UndoAction) -> bool {
    match (claim, action) {
        (UndoClaim::Added { card_id }, UndoAction::Added { card_id: added, .. }) => {
            *added == card_id
        }
        (
            UndoClaim::MovedBoard { card_id, from },
            UndoAction::MovedBoard {
                card_id: moved,
                from: left,
                ..
            },
        ) => *moved == card_id && *left == from,
        _ => false,
    }
}

/// The app's parked stack as zwipe-ui's.
fn into_shared(parked: ParkedStack) -> parked::ParkedStack {
    parked::ParkedStack {
        cards: parked.cards,
        index: parked.index,
        history: parked.history.into_iter().map(shared_action).collect(),
        filter: parked.filter,
        exhausted: parked.exhausted,
    }
}

/// zwipe-ui's parked stack as the app's.
fn from_shared(parked: parked::ParkedStack) -> ParkedStack {
    ParkedStack {
        cards: parked.cards,
        index: parked.index,
        history: parked.history.into_iter().map(app_action).collect(),
        filter: parked.filter,
        exhausted: parked.exhausted,
    }
}

/// The app's search swipe as zwipe-ui's.
fn shared_action(action: AddAction) -> stack::AddAction {
    match action {
        AddAction::Skip => stack::AddAction::Skip,
        AddAction::Add => stack::AddAction::Add,
        AddAction::Maybe => stack::AddAction::Maybe,
    }
}

/// zwipe-ui's search swipe as the app's.
fn app_action(action: stack::AddAction) -> AddAction {
    match action {
        stack::AddAction::Skip => AddAction::Skip,
        stack::AddAction::Add => AddAction::Add,
        stack::AddAction::Maybe => AddAction::Maybe,
    }
}

#[component]
pub fn Add(deck_id: Uuid) -> Element {
    // This screen's own filter: seeded from the per-(screen, deck) store and
    // provided as context so the filter modules bind to it. Mount always
    // starts on the Search source, so the seed is the AddSearch scope.
    let mut filter_store: FilterStore = use_context();
    let restored_filter = filter_store.restore(FilterScope::AddSearch, deck_id);
    let filter_builder: Signal<CardQueryBuilder> = {
        let restored = restored_filter.clone();
        use_signal(move || restored.unwrap_or_default())
    };
    use_context_provider(|| filter_builder);
    // Bumped to rerun the filter; the filter sheet's sections reset on it.
    let filter_reset_counter: Signal<u32> = use_signal(|| 0);
    use_context_provider(|| filter_reset_counter);
    // Parked stacks per deck, so returning resumes mid-stack.
    let mut stack_cache: AddStackCache = use_context();

    let mut show_details = use_signal(|| false);
    let mut printing_open = use_signal(|| false);
    let mut filters_overlay_open = use_signal(|| false);
    // Gesture start point for the end-of-stack skeleton, which accepts a
    // down-swipe to undo back into the pile (no card, so no SwipeStack).
    let mut end_swipe_start: Signal<Option<(f64, f64)>> = use_signal(|| None);

    let usage_buffer: Signal<UsageBuffer> = use_context();
    let driver = Driver {
        model: use_signal(State::new),
        deck_id,
        navigator: use_navigator(),
        toast: use_toast(),
        authed: use_authed(Screen::Deck(DeckScreen::CardAdd)),
        session: use_context(),
        client: use_context(),
        usage_buffer,
        undo_store: use_context(),
        filter_store,
        filter_builder,
        filter_reset_counter,
        search_entering: use_signal(|| None),
        maybeboard_entering: use_signal(|| None),
        hint_open: use_signal(|| false),
    };

    // Load runs in the first render so the hint opens with the screen.
    use_hook(move || {
        let parked = stack_cache.take(deck_id).map(into_shared).map(Box::new);
        driver.dispatch(Action::Load {
            deck_id,
            filter: restored_filter,
            parked,
        });
    });

    // Every bump after the first render is an applied filter or a rerun the
    // model asked for: hand the filter to the model.
    use_on_bump(filter_reset_counter, move || {
        let filter = filter_builder.peek().clone();
        driver.dispatch(Action::Filter(filter));
    });

    // Park this screen's filter under the live source, and the search stack
    // so returning resumes mid-stack, on leave.
    use_drop(move || {
        let model = driver.model.peek();
        filter_store.park(
            scope(model.source()),
            deck_id,
            filter_builder.peek().clone(),
        );
        if let Some(parked) = model.parked() {
            stack_cache.park(deck_id, from_shared(parked));
        }
    });

    // Swipe config: the stack owns its own SwipeState internally.
    // Thresholds tuned for responsive rapid swiping: a drag past 90px OR a
    // flick faster than 2.5 px/ms (after the 32px minimum) commits.
    let swipe_config = SwipeConfig::new(
        vec![
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ],
        SWIPE_DISTANCE_PX,
        SWIPE_SPEED_PX_PER_MS,
    );

    // A committed swipe: telemetry first, then the model.
    let on_swipe = move |direction: Direction, card: &Card| {
        usage_buffer.peek().record_swipe(direction);
        let committed = side(direction);
        if committed.is_some() && driver.model.peek().source() == Source::Search {
            usage_buffer
                .peek()
                .record_signal(deck_id, card.scryfall_data.oracle_id, direction);
        }
        driver.dispatch(committed.map_or(Action::Undo, Action::Swipe));
    };

    let (source, pile, window, current, printing_card, shows_synergy, validates_apply, shows_dot) = {
        let model = driver.model.read();
        (
            model.source(),
            model.pile(),
            model.window().to_vec(),
            model.current().cloned(),
            model.printing_card().cloned(),
            model.shows_synergy(),
            model.validates_apply(),
            model.shows_dot(&filter_builder.read()),
        )
    };
    let entering = driver.entering(source);
    let end_pull = move |end: (f64, f64), start: (f64, f64)| {
        let (dx, dy) = (end.0 - start.0, end.1 - start.1);
        if dy >= BACK_PULL_PX && dy.abs() > dx.abs() {
            driver.dispatch(Action::Undo);
        }
    };

    let no_card = current.is_none();
    rsx! {
            div { class: "screen",
                ScreenHeader { title: add_cards::TITLE, hint: driver.hint_open, back: move |()| driver.dispatch(Action::Back) }

                div { class: "screen-content card-swipe content-enter",

                div { class : "form-container",
                    if source == Source::Search {
                        if pile == Pile::Cards {
                            SwipeStack {
                                cards: window,
                                config: swipe_config,
                                entering,
                                on_swipe_left: move |card: Card| on_swipe(Direction::Left, &card),
                                on_swipe_right: move |card: Card| on_swipe(Direction::Right, &card),
                                on_swipe_up: move |card: Card| on_swipe(Direction::Up, &card),
                                on_swipe_down: move |card: Card| on_swipe(Direction::Down, &card),
                            }

                            if let Some(card) = current.clone() {
                                CardInfoDisplay { card }
                            }
                        } else if pile == Pile::End {
                            // Cursor sits one past the last card with nothing
                            // more coming: the skeleton stays down-swipeable so
                            // undo leads back into the pile.
                            div {
                                style: "display: flex; flex-direction: column; flex: 1; min-height: 0; width: 100%;",
                                ontouchstart: move |e: Event<TouchData>| {
                                    if let Some(t) = e.touches().into_iter().next() {
                                        let p = t.client_coordinates();
                                        end_swipe_start.set(Some((p.x, p.y)));
                                    }
                                },
                                ontouchend: move |e: Event<TouchData>| {
                                    let Some(start) = end_swipe_start() else {
                                        return;
                                    };
                                    end_swipe_start.set(None);
                                    if let Some(t) = e.touches_changed().into_iter().next() {
                                        let p = t.client_coordinates();
                                        end_pull((p.x, p.y), start);
                                    }
                                },
                                onmousedown: move |e: Event<MouseData>| {
                                    let p = e.client_coordinates();
                                    end_swipe_start.set(Some((p.x, p.y)));
                                },
                                onmouseup: move |e: Event<MouseData>| {
                                    let Some(start) = end_swipe_start() else {
                                        return;
                                    };
                                    end_swipe_start.set(None);
                                    let p = e.client_coordinates();
                                    end_pull((p.x, p.y), start);
                                },
                                CardSkeleton {}
                            }
                        } else {
                            CardSkeleton {}
                        }
                    } else {
                        // Maybeboard mode
                        if pile == Pile::Cards {
                            SwipeStack {
                                cards: window,
                                config: swipe_config,
                                entering,
                                on_swipe_left: move |card: Card| on_swipe(Direction::Left, &card),
                                on_swipe_right: move |card: Card| on_swipe(Direction::Right, &card),
                                on_swipe_up: move |card: Card| on_swipe(Direction::Up, &card),
                                on_swipe_down: move |card: Card| on_swipe(Direction::Down, &card),
                            }

                            if let Some(card) = current.clone() {
                                CardInfoDisplay { card }
                            }
                        } else {
                            CardSkeleton {}
                        }
                    }

                }
            }

            // The bottom row: the source chips on the left, the action dock
            // (synergy, filter, refresh, card details) on the right.
            div { class: "swipe-bottom-row",
                // From, then one chip naming the current source; a tap switches
                // to the other.
                div { class: "swipe-bottom-chips",
                    span { class: "stat-chip stat-chip-p1 chip-row-label", "{add_cards::FROM}" }
                    Chip {
                        selected: true,
                        onclick: move |_| {
                            let next = source.toggled();
                            let remembered = filter_store.restore(scope(next), deck_id);
                            driver.dispatch(Action::PickSource { source: next, remembered });
                        },
                        "{source.label()}"
                    }
                }
                div { class: "action-dock",
                    // Synergy on or off: constrains the stack to the commander's
                    // synergy pool, then sorts within it. Lit while on.
                    if shows_synergy {
                        button {
                            class: if filter_builder.read().synergy() { "action-dock-btn action-dock-on" } else { "action-dock-btn" },
                            "aria-label": add_cards::SYNERGY,
                            "aria-pressed": "{filter_builder.read().synergy()}",
                            onclick: move |_| driver.dispatch(Action::ToggleSynergy),
                            DockIcon { path: action_dock::LINK }
                        }
                    }
                    DockFilter {
                        label: add_cards::FILTER,
                        dot: shows_dot,
                        onclick: move |()| filters_overlay_open.set(true),
                    }
                    button {
                        class: "action-dock-btn",
                        "aria-label": add_cards::REFRESH,
                        onclick: move |_| driver.dispatch(Action::Refresh),
                        DockIcon { path: action_dock::REFRESH }
                    }
                    span { class: "action-dock-divider" }
                    button {
                        class: "action-dock-btn",
                        "aria-label": "Card details",
                        disabled: no_card,
                        onclick: move |_| {
                            let mut open = show_details;
                            open.toggle();
                        },
                        EyeIcon {}
                    }
                }
            }

            // Root-mounted (siblings of the ActionBar), so the bottom sheets'
            // z-index covers the footer instead of stacking under it. The
            // maybeboard pile shows details without Printings.
            if source == Source::Maybeboard {
                if let Some(card) = current {
                    CardDetailsSheet { open: show_details, card }
                }
            } else if let Some(card) = current {
                CardDetailsSheet {
                    open: show_details,
                    card,
                    on_printings: move |()| printing_open.set(true),
                }
            }

            // After the details sheet so it draws over it. Pick a printing →
            // re-skin the focused card and drop both sheets; swipe-right adds
            // that printing.
            if let Some(card) = printing_card {
                PrintingSheet {
                    host_screen: screen::DECK_CARD_ADD,
                    card,
                    open: printing_open,
                    on_save: move |new_card: Card| {
                        driver.dispatch(Action::SwapPrinting(Box::new(new_card)));
                        show_details.set(false);
                    },
                    saved_message: add_cards::PRINTING_SWAPPED,
                }
            }

            HintDialog {
                open: driver.hint_open,
                actions: guide_link::one_time(HINT_ADD_DECK_CARDS),
                title: add_cards::HINT_TITLE,
                HintBullets {
                    HintBullet { "Swipe " HintColored { color: "--color-success", "right" } " adds the card" }
                    HintBullet { "Swipe " HintColored { color: "--color-error", "left" } " skips it for a week" }
                    HintBullet { "Swipe " HintColored { color: "--color-warning", "up" } " sends it to the maybeboard" }
                    HintBullet { "Swipe " HintColored { color: "--accent-tertiary", "down" } " undoes the last swipe" }
                    HintBullet { "The chip after " HintKey { "From" } " switches between Search and your Maybe pile" }
                    HintBullet { "The link icon is " HintKey { "Synergy" } ": on starts with cards that work with your commander, then cards like your deck. Off shows every legal card" }
                }
                HintLine { "Sort only reorders. It never hides cards" }
            }

            CardFilterSheet {
                open: filters_overlay_open,
                show_format_filter: true,
                show_active_indicators: true,
                // Only the server search needs intent: Maybeboard filters
                // client-side and its default is a blank filter, so an empty
                // Apply there (e.g. right after Reset) must commit, not warn.
                validate_before_apply: validates_apply,
                on_clear: move |()| {
                    let draft = filter_builder.peek().clone();
                    driver.dispatch(Action::ResetFilter(draft));
                },
            }
            }
    }
}
