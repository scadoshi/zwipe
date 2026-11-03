//! Remove deck cards: swipe through a deck's cards to trim it.
//!
//! What the screen knows and does is `zwipe_ui::screens::remove_cards`; this
//! file draws its state, performs its effects and keeps the gestures, the
//! card's entrance and the sheets.

use components::{
    ActionBar, Button, ButtonVariant, Chip, HintBullet, HintBullets, HintDialog, HintLine,
    ToastOptions, Toasts, use_toast,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::card_sort_key::CardSortKey;
use zwipe_core::domain::deck::Board;
use zwipe_core::domain::user::models::hints::HINT_REMOVE_DECK_CARDS;
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::remove_cards::{
    self, Action, BoardFilter, Destination, Effect, State, UndoClaim,
};
use zwipe_ui::stack;
use zwipe_ui::stores::undo as shared_undo;

use super::components::card_info::{CardDetailsSheet, CardInfoDisplay, CardSkeleton, RulesButton};
use super::components::filter_store::{FilterScope, FilterStore};
use super::components::printing_sheet::PrintingSheet;
use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::inbound::components::hint_dialog::{self, HintColored};
use crate::inbound::components::interactions::swipe::SwipeStack;
use crate::inbound::components::interactions::swipe::config::SwipeConfig;
use crate::inbound::components::interactions::swipe::direction::Direction;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::components::telemetry::vocabulary::{DeckScreen, Screen, component, screen};
use crate::inbound::components::{guide_link, ui_toast};
use crate::inbound::screens::deck::card::components::undo_log::{UndoAction, UndoStore};
use crate::inbound::screens::deck::card::filter::card_filter_sheet::CardFilterSheet;
use crate::inbound::screens::deck::card::filter::deck_cards::DeckCards;

/// Pixels a swipe travels to commit.
const SWIPE_DISTANCE_PX: f64 = 90.0;
/// Pixels per millisecond a flick needs to commit.
const SWIPE_SPEED_PX_PER_MS: f64 = 2.5;

/// The screen's model with the handles that perform its effects. Copy, so
/// every handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    navigator: Navigator,
    toast: Toasts,
    authed: Authed,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    usage_buffer: Signal<UsageBuffer>,
    undo_store: UndoStore,
    /// The side the top card enters from after an undo; the swipe stack
    /// clears it when the entrance ends.
    entering: Signal<Option<Direction>>,
    hint_open: Signal<bool>,
    /// The deck's cards as loaded, for the filter sheet's pickers.
    deck_cards_for_filter: Signal<Vec<Card>>,
    /// Bumped to rerun the filter; the filter sheet's sections reset on it.
    filter_reset_counter: Signal<u32>,
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

    /// Reruns the filter, as the filter sheet does when it applies.
    fn bump_filter(self) {
        let mut counter = self.filter_reset_counter;
        let current = *counter.peek();
        counter.set(current.wrapping_add(1));
    }

    /// Performs one effect, feeding any reply back through [`Self::dispatch`].
    fn perform(self, effect: Effect) {
        let Self {
            navigator,
            toast,
            authed,
            session,
            client,
            mut undo_store,
            mut entering,
            hint_open,
            ..
        } = self;
        match effect {
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(hint.key(), session, client, hint_open);
            }
            Effect::FetchDeck { ticket, deck_id } => self.fetch_deck(ticket, deck_id),
            Effect::DeleteDeckCard {
                ticket,
                deck_id,
                card_id,
            } => {
                spawn(async move {
                    let reply = authed
                        .run("remove_card", |c, s| async move {
                            c.delete_deck_card(deck_id, card_id, &s).await
                        })
                        .await;
                    self.dispatch(match reply {
                        Some(()) => Action::Landed { ticket },
                        None => Action::Failed { ticket },
                    });
                });
            }
            Effect::MoveDeckCard {
                ticket,
                deck_id,
                card_id,
                to,
            } => {
                let request = remove_cards::board_request(to);
                spawn(async move {
                    let reply = authed
                        .run("move_card", |c, s| async move {
                            c.update_deck_card(deck_id, card_id, &request, &s).await
                        })
                        .await;
                    self.dispatch(match reply {
                        Some(_) => Action::Landed { ticket },
                        None => Action::Failed { ticket },
                    });
                });
            }
            Effect::TakeUndo {
                ticket,
                deck_id,
                claim,
            } => {
                let taken = undo_store.take_newest(deck_id, |a| claims(claim, a));
                self.dispatch(Action::Taken {
                    ticket,
                    taken: taken.map(into_shared),
                });
            }
            Effect::RestoreDeckCard {
                ticket,
                deck_id,
                card,
                board,
                quantity,
            } => self.restore_card(ticket, deck_id, &card, board, quantity),
            Effect::RestoreBoard {
                ticket,
                deck_id,
                card_id,
                board,
            } => self.restore_board(ticket, deck_id, card_id, board),
            Effect::RecordUndo { deck_id, action } => {
                undo_store.push(deck_id, from_shared(action));
            }
            Effect::Enter(side) => entering.set(Some(direction(side))),
            Effect::Navigate(Destination::Previous) => navigator.go_back(),
            Effect::Toast(raised) => ui_toast::raise(toast, raised),
        }
    }

    /// Fetches the deck, handing its cards to the filter sheet before the model.
    fn fetch_deck(self, ticket: Ticket, deck_id: Uuid) {
        let Self {
            authed,
            mut deck_cards_for_filter,
            ..
        } = self;
        spawn(async move {
            let deck = authed
                .run("load_deck", |c, s| async move {
                    c.fetch_deck(deck_id, &s).await
                })
                .await;
            let Some(deck) = deck else {
                self.dispatch(Action::Failed { ticket });
                return;
            };
            deck_cards_for_filter.set(deck.entries.iter().map(|e| e.card.clone()).collect());
            self.dispatch(Action::Loaded {
                ticket,
                deck: Box::new(deck),
            });
            self.bump_filter();
        });
    }

    /// Undoes a remove: creates the card again on its board.
    fn restore_card(self, ticket: Ticket, deck_id: Uuid, card: &Card, board: Board, quantity: i32) {
        let Self { toast, client, .. } = self;
        let request = remove_cards::restore_request(card, board, quantity);
        // Facade holdout: staged compensation (refresh failure cancels the
        // entrance; a failed call leaves the removal standing).
        spawn(async move {
            let Some(session) = self.fresh_session().await else {
                self.dispatch(Action::Failed { ticket });
                return;
            };
            match client().create_deck_card(deck_id, &request, &session).await {
                Ok(deck_card) => self.dispatch(Action::Restored {
                    ticket,
                    deck_card: Box::new(deck_card),
                }),
                Err(e) => {
                    tracing::warn!("undo remove (create deck card) failed: {e}");
                    toast.error(format!("Failed to undo: {e}"), ToastOptions::default());
                    self.dispatch(Action::Failed { ticket });
                }
            }
        });
    }

    /// Undoes a move: sends the card back to the board it left.
    fn restore_board(self, ticket: Ticket, deck_id: Uuid, card_id: Uuid, board: Board) {
        let Self { toast, client, .. } = self;
        let request = remove_cards::board_request(board);
        // Facade holdout: staged compensation (refresh failure cancels the
        // entrance; a failed call leaves the move standing).
        spawn(async move {
            let Some(session) = self.fresh_session().await else {
                self.dispatch(Action::Failed { ticket });
                return;
            };
            match client()
                .update_deck_card(deck_id, card_id, &request, &session)
                .await
            {
                Ok(_) => self.dispatch(Action::Landed { ticket }),
                Err(e) => {
                    tracing::warn!("undo board move (update deck card) failed: {e}");
                    toast.error(format!("Failed to undo: {e}"), ToastOptions::default());
                    self.dispatch(Action::Failed { ticket });
                }
            }
        });
    }

    /// A fresh session for an undo's call. On failure it reports the error,
    /// raises its toast and cancels the entrance.
    async fn fresh_session(self) -> Option<Session> {
        let Self {
            toast,
            session,
            client,
            usage_buffer,
            mut entering,
            ..
        } = self;
        match session.ensure_fresh(client).await {
            Ok(session) => Some(session),
            Err(e) => {
                usage_buffer.peek().report_error(
                    screen::DECK_CARD_REMOVE,
                    component::NONE,
                    "undo_remove",
                    &e,
                );
                toast.error(e.to_user_message(), ToastOptions::default());
                entering.set(None);
                None
            }
        }
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

/// Whether `action` is the undo log entry `claim` takes back.
fn claims(claim: UndoClaim, action: &UndoAction) -> bool {
    match (claim, action) {
        (UndoClaim::Removed { card_id }, UndoAction::Removed { entry, .. }) => {
            entry.card.scryfall_data.id == card_id
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

/// The app's undo log entry as zwipe-ui's.
fn into_shared(action: UndoAction) -> shared_undo::UndoAction {
    match action {
        UndoAction::Added { card_id, card_name } => {
            shared_undo::UndoAction::Added { card_id, card_name }
        }
        UndoAction::Removed { entry, baseline } => shared_undo::UndoAction::Removed {
            entry: Box::new(entry),
            baseline,
        },
        UndoAction::QuantityChanged {
            card_id,
            card_name,
            net,
            baseline,
        } => shared_undo::UndoAction::QuantityChanged {
            card_id,
            card_name,
            net,
            baseline,
        },
        UndoAction::MovedBoard {
            card_id,
            card_name,
            from,
        } => shared_undo::UndoAction::MovedBoard {
            card_id,
            card_name,
            from,
        },
        UndoAction::PrintingChanged { old_card, new_id } => {
            shared_undo::UndoAction::PrintingChanged {
                old_card: Box::new(old_card),
                new_id,
            }
        }
        UndoAction::CommandZonePrintingChanged {
            slot,
            old_card,
            new_id,
        } => shared_undo::UndoAction::CommandZonePrintingChanged {
            slot,
            old_card: Box::new(old_card),
            new_id,
        },
    }
}

/// zwipe-ui's undo log entry as the app's.
fn from_shared(action: shared_undo::UndoAction) -> UndoAction {
    match action {
        shared_undo::UndoAction::Added { card_id, card_name } => {
            UndoAction::Added { card_id, card_name }
        }
        shared_undo::UndoAction::Removed { entry, baseline } => UndoAction::Removed {
            entry: *entry,
            baseline,
        },
        shared_undo::UndoAction::QuantityChanged {
            card_id,
            card_name,
            net,
            baseline,
        } => UndoAction::QuantityChanged {
            card_id,
            card_name,
            net,
            baseline,
        },
        shared_undo::UndoAction::MovedBoard {
            card_id,
            card_name,
            from,
        } => UndoAction::MovedBoard {
            card_id,
            card_name,
            from,
        },
        shared_undo::UndoAction::PrintingChanged { old_card, new_id } => {
            UndoAction::PrintingChanged {
                old_card: *old_card,
                new_id,
            }
        }
        shared_undo::UndoAction::CommandZonePrintingChanged {
            slot,
            old_card,
            new_id,
        } => UndoAction::CommandZonePrintingChanged {
            slot,
            old_card: *old_card,
            new_id,
        },
    }
}

#[component]
pub fn Remove(deck_id: Uuid) -> Element {
    // This screen's own filter: seeded from the per-(screen, deck) store,
    // provided as context so the filter modules bind to it, parked on leave.
    let filter_store: FilterStore = use_context();
    let filter_builder: Signal<CardQueryBuilder> = use_signal(|| {
        filter_store
            .restore(FilterScope::Remove, deck_id)
            .unwrap_or_default()
    });
    use_context_provider(|| filter_builder);
    use_drop(move || {
        let mut store = filter_store;
        store.park(FilterScope::Remove, deck_id, filter_builder.peek().clone());
    });

    // Filter overlay state
    let mut filters_overlay_open = use_signal(|| false);

    let show_details = use_signal(|| false);
    let mut printing_open = use_signal(|| false);

    // Incrementing this re-runs the filter. Readers only see that it moved,
    // so bumps wrap.
    let filter_reset_counter: Signal<u32> = use_signal(|| 0);
    use_context_provider(|| filter_reset_counter);

    // Provide Card list context for filter sheet
    let deck_cards_for_filter: Signal<Vec<Card>> = use_signal(Vec::new);
    use_context_provider(|| DeckCards(deck_cards_for_filter));

    let driver = Driver {
        model: use_signal(State::new),
        navigator: use_navigator(),
        toast: use_toast(),
        authed: use_authed(Screen::Deck(DeckScreen::CardRemove)),
        session: use_context(),
        client: use_context(),
        usage_buffer: use_context(),
        // The global per-deck undo stack (walked by the deck cards screen's
        // Undo button). This screen records its deck mutations into the
        // parked stack and takes them back when its own gesture undo
        // reverses one.
        undo_store: use_context(),
        entering: use_signal(|| None),
        // Swipe vocabulary hint: auto-opens on this user's first visit, the
        // grayed "?" in the util bar reopens it on demand.
        hint_open: use_signal(|| false),
        deck_cards_for_filter,
        filter_reset_counter,
    };
    let swipe_hint_open = driver.hint_open;
    let usage_buffer = driver.usage_buffer;
    let mut entering = driver.entering;

    // Offers the hint on the first render and loads the deck, once per mount.
    use_hook(move || {
        let filter = filter_builder.peek().clone();
        driver.dispatch(Action::Load { deck_id, filter });
    });

    // Reruns the filter whenever the counter moves.
    use_effect(move || {
        let _ = filter_reset_counter();
        let filter = filter_builder.peek().clone();
        driver.dispatch(Action::Filter(filter));
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

    // Snapshot the model once per render.
    let state = driver.model.read();
    let stack_empty = state.is_empty();
    let window = state.window().to_vec();
    let current = state.current().cloned();
    let board_filter = state.board();
    drop(state);

    rsx! {
            div { class: "screen",
                ScreenHeader { title: remove_cards::TITLE, hint: swipe_hint_open }

                div { class: "screen-content card-swipe content-enter",

                // Board filter chips
                div { style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
                    div { class: "chip-row",
                        span { class: "stat-chip stat-chip-p1 chip-row-label", {remove_cards::BOARDS} }
                        for variant in BoardFilter::ALL {
                            Chip {
                                selected: board_filter == variant,
                                onclick: move |_| {
                                    driver.dispatch(Action::PickBoard(variant));
                                    driver.bump_filter();
                                },
                                "{variant.label()}"
                            }
                        }
                    }
                }

                div { class: "form-container",
                    if !stack_empty {
                        SwipeStack {
                            cards: window,
                            config: swipe_config,
                            entering,
                            on_swipe_left: move |_card: Card| {
                                usage_buffer().record_swipe(Direction::Left);
                                driver.dispatch(Action::Keep);
                            },
                            on_swipe_right: move |card: Card| {
                                usage_buffer().record_swipe(Direction::Right);
                                // Removal signal (delayed negative) keyed by commander + card.
                                usage_buffer().record_removal(deck_id, card.scryfall_data.oracle_id);
                                driver.dispatch(Action::Remove);
                            },
                            on_swipe_up: move |_card: Card| {
                                usage_buffer().record_swipe(Direction::Up);
                                driver.dispatch(Action::Move);
                            },
                            on_swipe_down: move |_card: Card| {
                                usage_buffer().record_swipe(Direction::Down);
                                driver.dispatch(Action::Undo);
                            },
                        }

                        if let Some(card) = current.clone() {
                            CardInfoDisplay { card }
                        }
                    } else {
                        CardSkeleton {}
                    }

                }
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| driver.dispatch(Action::Back),
                    {remove_cards::BACK}
                }
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| filters_overlay_open.set(true),
                    {remove_cards::FILTER}
                    if !filter_builder.read().is_empty() || filter_builder.read().sort().is_some() {
                        span { class: "filter-dot" }
                    }
                }
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| {
                        // Refresh rewinds the stack, which ends any entrance.
                        entering.set(None);
                        driver.dispatch(Action::Refresh);
                        if filter_builder.peek().sort() == Some(CardSortKey::Random) {
                            driver.bump_filter();
                        }
                    },
                    {remove_cards::REFRESH}
                }
                if current.is_some() {
                    RulesButton { open: show_details }
                }
            }

            // Root-mounted so the bottom sheets cover the footer, the printing
            // sheet after the details sheet so it draws over it. View-only:
            // browsing printings never changes the deck (remove keys on the
            // exact scryfall_data_id).
            if let Some(card) = current {
                CardDetailsSheet {
                    open: show_details,
                    card: card.clone(),
                    on_printings: move |()| printing_open.set(true),
                }
                PrintingSheet {
                    host_screen: screen::DECK_CARD_REMOVE,
                    card,
                    open: printing_open,
                    on_save: move |_: Card| {},
                    read_only: true,
                }
            }

            HintDialog {
                open: swipe_hint_open,
                actions: guide_link::one_time(HINT_REMOVE_DECK_CARDS),
                title: remove_cards::HINT_TITLE,
                HintBullets {
                    HintBullet { "Swipe " HintColored { color: "--color-success", "right" } " removes the card" }
                    HintBullet { "Swipe " HintColored { color: "--color-error", "left" } " keeps it" }
                    HintBullet { "Swipe " HintColored { color: "--color-warning", "up" } " moves it between main and maybeboard" }
                    HintBullet { "Swipe " HintColored { color: "--accent-tertiary", "down" } " undoes the last swipe" }
                }
                HintLine { "The board chips up top pick which board you are trimming" }
            }

            CardFilterSheet {
                open: filters_overlay_open,
                show_format_filter: true,
                show_active_indicators: true,
            }
            }
    }
}
