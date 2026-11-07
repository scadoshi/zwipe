//! Full-screen "Swipe select" screen: swipe to pick a command-zone card.
//!
//! Rendered as a sibling overlay above the create/edit form and toggled by
//! `open`: it stays mounted while closed so its filter and swipe position
//! persist, letting you commit a pick and reopen right where you left off. The
//! [`SwipeMode`] decides the always-on pool (commander, partner, background, or
//! signature spell); cards are served in **popularity order** through the
//! full [`CardFilterSheet`]. Swipe **left** to skip, **right** to choose,
//! **down** to undo; **up** saves to the commander maybeboard (commander and
//! partner modes only: the pile advances and selection continues).
//!
//! [`SwipeMode::Suggested`] deals a given ranking instead of a search: the
//! host's held suggestions, read through `suggested_pile` each time the overlay
//! opens. There a left swipe dismisses the commander for a week and the eyeball
//! sheet carries the reasons it was suggested. Its Filter opens a sheet holding
//! the host's `suggested_filter` chips; when they change the pile mid-session,
//! the stack re-deals, keeping what was already swiped behind the top card so
//! undo still walks back through it.
//!
//! What the overlay knows and does is `zwipe_ui::screens::swipe_select`; this
//! file draws its state and performs its effects.

use std::collections::HashSet;

use components::hint::ContentHintDialog;
use components::icons::{self, EyeIcon};
use components::{
    BottomSheet, DockAction, DockFilter, ToastOptions, Toasts, use_overlay_back, use_toast,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::query::CardQuery;
use zwipe_ui::generation::Ticket;
use zwipe_ui::hint::{SwipeSelectMode, one_time};
use zwipe_ui::screens::swipe_select::{self as model, Action, Destination, Effect, State, Status};
pub(crate) use zwipe_ui::screens::swipe_select::{Mode as SwipeMode, SuggestedPile};
use zwipe_ui::stack;

use crate::inbound::components::interactions::swipe::SwipeStack;
use crate::inbound::components::interactions::swipe::config::SwipeConfig;
use crate::inbound::components::interactions::swipe::direction::Direction;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::swipe_confirm::{self, Said, SwipeConfirmChip};
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::components::telemetry::vocabulary::component;
use crate::inbound::components::{hint_dialog, ui_toast};
use crate::inbound::screens::deck::card::components::card_info::{
    CardDetailsSheet, CardSkeleton, use_describe_tag,
};
use crate::inbound::screens::deck::card::components::card_row::OtagExamplesOpen;
use crate::inbound::screens::deck::card::components::printing_sheet::PrintingSheet;
use crate::inbound::screens::deck::card::filter::card_filter_sheet::CardFilterSheet;
use crate::inbound::screens::deck::suggested_commanders::SuggestionReasons;

/// Pixels a drag travels to commit a swipe.
const SWIPE_DISTANCE_PX: f64 = 90.0;
/// Pixels per millisecond a flick needs to commit.
const SWIPE_SPEED_PX_PER_MS: f64 = 2.5;

/// The overlay's model with the handles that perform its effects. Copy, so
/// every handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    usage_buffer: Signal<UsageBuffer>,
    toast: Toasts,
    hint_open: Signal<bool>,
    entering: Signal<Option<Direction>>,
    host_screen: &'static str,
    on_select: EventHandler<Card>,
    on_close: EventHandler<()>,
    /// The swipe confirmations chip's words.
    said: Said,
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
        match effect {
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(
                    hint.key(),
                    self.session,
                    self.client,
                    self.hint_open,
                );
            }
            Effect::SearchCommanders { ticket, query } => {
                if self.session.peek().is_some() {
                    self.usage_buffer.peek().record_search();
                }
                self.search(ticket, *query, "zwipe select search failed");
            }
            Effect::LoadMore { ticket, query } => {
                self.search(ticket, *query, "zwipe select load-more failed");
            }
            Effect::DismissCommander(oracle_id) => {
                // Quiet like a deck skip: the pile already moved on, and a
                // failed post only means it comes back sooner.
                let Some(session) = self.session.peek().clone() else {
                    return;
                };
                let client = self.client;
                spawn(async move {
                    if let Err(e) = client().dismiss_commander(oracle_id, &session).await {
                        tracing::warn!("dismiss commander failed: {e}");
                    }
                });
            }
            Effect::UndismissCommander(oracle_id) => {
                // Quiet like the dismissal itself: the card is already back.
                let Some(session) = self.session.peek().clone() else {
                    return;
                };
                let client = self.client;
                spawn(async move {
                    if let Err(e) = client().undismiss_commander(oracle_id, &session).await {
                        tracing::warn!("undo commander dismissal failed: {e}");
                        self.report("undismiss_commander", &e);
                    }
                });
            }
            Effect::AddMaybeboardCard { ticket, oracle_id } => {
                let Some(session) = self.session.peek().clone() else {
                    return;
                };
                let client = self.client;
                spawn(async move {
                    let reply = client()
                        .add_commander_maybeboard_card(oracle_id, &session)
                        .await;
                    self.settle(
                        ticket,
                        reply,
                        "add to commander maybeboard failed",
                        "add_commander_maybeboard",
                    );
                });
            }
            Effect::RemoveMaybeboardCard { ticket, oracle_id } => {
                let Some(session) = self.session.peek().clone() else {
                    return;
                };
                let client = self.client;
                spawn(async move {
                    let reply = client()
                        .remove_commander_maybeboard_card(oracle_id, &session)
                        .await;
                    self.settle(
                        ticket,
                        reply,
                        "undo commander maybeboard save failed",
                        "undo_commander_maybeboard",
                    );
                });
            }
            Effect::Enter(direction) => {
                let mut entering = self.entering;
                entering.set(Some(match direction {
                    stack::Direction::Left => Direction::Left,
                    stack::Direction::Right => Direction::Right,
                    stack::Direction::Up => Direction::Up,
                }));
            }
            Effect::Pick(card) => self.on_select.call(*card),
            Effect::Navigate(Destination::Previous) => self.on_close.call(()),
            Effect::Toast(toast) => ui_toast::raise(self.toast, toast),
            Effect::Confirm(toast) => {
                swipe_confirm::confirm(self.session, self.toast, self.said, toast);
            }
        }
    }

    /// Runs a page search and answers with [`Action::Found`], or with
    /// [`Action::Failed`] after logging `failure`.
    fn search(self, ticket: Ticket, query: CardQuery, failure: &'static str) {
        let Some(session) = self.session.peek().clone() else {
            self.dispatch(Action::Failed { ticket });
            return;
        };
        let client = self.client;
        spawn(async move {
            let reply = client().search_commanders(&query, &session).await;
            self.dispatch(match reply {
                Ok(cards) => Action::Found { ticket, cards },
                Err(e) => {
                    tracing::warn!("{failure}: {e}");
                    Action::Failed { ticket }
                }
            });
        });
    }

    /// Answers a maybeboard request: [`Action::Landed`], or on a failure a
    /// warning, a report under `op`, the error toast and [`Action::Failed`].
    fn settle(
        self,
        ticket: Ticket,
        reply: Result<(), ClientError>,
        warning: &'static str,
        op: &'static str,
    ) {
        match reply {
            Ok(()) => self.dispatch(Action::Landed { ticket }),
            Err(e) => {
                tracing::warn!("{warning}: {e}");
                self.report(op, &e);
                self.toast
                    .error(e.to_user_message(), ToastOptions::default());
                self.dispatch(Action::Failed { ticket });
            }
        }
    }

    /// Reports a failed request under the host screen and this overlay.
    fn report(self, op: &'static str, e: &ClientError) {
        self.usage_buffer
            .peek()
            .report_error(self.host_screen, component::SWIPE_SELECT, op, e);
    }
}

/// In-place swipe picker for a command-zone slot. Toggled by `open`; `mode` is
/// the (reactive) slot to fill. `on_select` fires with the chosen card;
/// `on_close` cancels. Reads `Session` + `ZwipeClient` from context.
#[component]
pub(crate) fn SwipeSelect(
    open: Signal<bool>,
    mode: ReadSignal<Option<SwipeMode>>,
    on_select: EventHandler<Card>,
    on_close: EventHandler<()>,
    /// Host screen const for error-report breadcrumbs, passed through to the
    /// printing sheet (this overlay serves deck edit and deck create).
    host_screen: &'static str,
    /// Oracle ids kept out of the pile. The commander maybeboard passes its
    /// own saves here (its Swipe is discovery: a saved commander is a wasted
    /// deal); deck create/edit pass nothing, so saves still serve where the
    /// user goes to actually pick one.
    #[props(default)]
    exclude_oracle_ids: Option<ReadSignal<HashSet<Uuid>>>,
    /// The ranking the suggested mode deals, snapshotted on each open so the
    /// pile follows the host's own removes and filters. Without it the
    /// suggested pile is empty.
    #[props(default)]
    suggested_pile: Option<ReadSignal<SuggestedPile>>,
    /// The chips the suggested mode's Filter sheet holds, bound to the same
    /// state that narrows `suggested_pile`. Without it there is no Filter.
    #[props(default)]
    suggested_filter: Option<Element>,
) -> Element {
    // OS back gesture closes this overlay before touching the router.
    use_overlay_back(open);
    let usage_buffer: Signal<UsageBuffer> = use_context();
    let driver = Driver {
        model: use_signal(State::new),
        session: use_context(),
        client: use_context(),
        usage_buffer,
        toast: use_toast(),
        hint_open: use_signal(|| false),
        entering: use_signal(|| None),
        host_screen,
        on_select,
        on_close,
        said: use_signal(|| None),
    };
    let hint_open = driver.hint_open;
    let entering = driver.entering;

    // Local filter context (its own builder) so the shared CardFilterSheet
    // operates here without clobbering the app-wide add-screen filter. Starts
    // sortless: the server's popularity ordering is the default
    // (context/archive/commander_select_ordering.md). Persists across open/close
    // because this component stays mounted.
    let mut filter_builder = use_signal(CardQueryBuilder::new);
    use_context_provider(|| filter_builder);
    let mut filter_reset_counter = use_signal(|| 0u32);
    use_context_provider(|| filter_reset_counter);
    // The counter value the pile was last searched for: the sheet's Apply
    // bumps the counter, and Refresh bumps it and searches itself.
    let mut searched_counter = use_signal(|| 0u32);

    let mut filters_overlay_open = use_signal(|| false);
    // The suggested mode's Filter sheet, holding the host's chips.
    let mut suggested_filter_open = use_signal(|| false);
    let mut show_details = use_signal(|| false);
    let mut printing_open = use_signal(|| false);

    // Each open hands the model the mode, the filter, the host's pile and
    // exclusions; the model keeps the pile or deals afresh, and offers the
    // hint on the first open.
    use_effect(move || {
        if !open() {
            return;
        }
        let mode = mode.peek().clone();
        let filter = filter_builder.peek().clone();
        let pile = suggested_pile
            .map(|pile| pile.peek().clone())
            .unwrap_or_default();
        let excluded = exclude_oracle_ids
            .map(|ids| ids.peek().clone())
            .unwrap_or_default();
        driver.dispatch(Action::Open {
            mode,
            filter,
            pile,
            excluded,
        });
    });

    // The filter sheet's Apply bumped the counter: search again.
    use_effect(move || {
        let counter = filter_reset_counter();
        if counter == *searched_counter.peek() {
            return;
        }
        searched_counter.set(counter);
        let filter = filter_builder.peek().clone();
        driver.dispatch(Action::Filter(filter));
    });

    // The host's Filter chips changed the suggested pile mid-session: the
    // model re-deals it under the cards already swiped. A closed overlay
    // waits for the open above, which deals afresh.
    use_effect(move || {
        let Some(pile) = suggested_pile else {
            return;
        };
        let pile = pile();
        if !*open.peek() {
            return;
        }
        driver.dispatch(Action::Redeal(pile));
    });

    // Bumps the counter so the filter sections reset with the pile, and
    // searches again from the top.
    let mut refresh = move || {
        let counter = filter_reset_counter().wrapping_add(1);
        searched_counter.set(counter);
        filter_reset_counter.set(counter);
        let filter = filter_builder.peek().clone();
        driver.dispatch(Action::Refresh(filter));
    };

    // Stage every user-set filter back to the default (all colors, the
    // backend's popularity order); the sheet's Apply commits and re-searches.
    // The mode constraint is re-applied on the next fetch.
    let mut clear_filter = move || {
        filter_builder.write().clear();
    };

    // The screen stays mounted and fades via CSS (`.show`). The inner content is
    // unmounted whenever closed so a just-committed card can't snap back to
    // center mid-fade; the solid screen simply fades out, easing the form in.
    let screen_class = if open() {
        "screen swipe-select-screen show"
    } else {
        "screen swipe-select-screen"
    };

    let state = driver.model.read();
    let (window, current_card) = if open() {
        (state.window().to_vec(), state.current().cloned())
    } else {
        (Vec::new(), None)
    };
    let has_cards = !window.is_empty();
    let printing_card = current_card.clone();
    let status = state.status();
    let empty_line = (status == Status::Empty).then(|| state.message()).flatten();
    let noun = state.noun();
    let suggested = state.is_suggested();
    // Up-swipe = "maybe this commander" (per-user save), so it's only live in
    // the modes whose pool is commanders.
    let maybe_enabled = state.can_save();
    let title = state.title();
    let current_reasons = state.reasons().cloned();
    let deck_count = state.deck_count();
    let has_suggested_identity = state.identity().is_some();
    drop(state);
    // The reasons behind the card on top, for the details sheet's footer. Its
    // shared-tag chips take the sheet's own description lookup and the host's
    // examples opener as props: the suggested commanders screen provides the
    // opener, and its examples overlay draws over this screen and the sheet.
    let describe_tag = use_describe_tag();
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    // The suggested Filter: the chosen identity lights the button's dot.
    let has_suggested_filter = suggested && suggested_filter.is_some();
    let hint_mode = if suggested {
        SwipeSelectMode::Suggested {
            has_filter: has_suggested_filter,
        }
    } else {
        SwipeSelectMode::Picking(noun)
    };
    let mut directions = vec![Direction::Left, Direction::Right, Direction::Down];
    if maybe_enabled {
        directions.push(Direction::Up);
    }
    let swipe_config = SwipeConfig::new(directions, SWIPE_DISTANCE_PX, SWIPE_SPEED_PX_PER_MS);

    rsx! {
        div { class: "{screen_class}",
            // A screen over another paints its own backdrop, since .screen is
            // clear and the one beneath would show through.
            ::components::GridCells { still: true }
            if open() {
                ScreenHeader { title, hint: hint_open, back: move |()| driver.dispatch(Action::Back) }

                div { class: "screen-content card-swipe content-enter",
                    SwipeConfirmChip { said: driver.said }
                    if has_cards {
                        SwipeStack {
                            cards: window,
                            config: swipe_config,
                            entering,
                            on_swipe_left: move |card: Card| {
                                usage_buffer().record_swipe(Direction::Left);
                                usage_buffer()
                                    .record_select_signal(card.scryfall_data.oracle_id, Direction::Left);
                                driver.dispatch(Action::Skip);
                            },
                            on_swipe_right: move |card: Card| {
                                usage_buffer().record_swipe(Direction::Right);
                                usage_buffer()
                                    .record_select_signal(card.scryfall_data.oracle_id, Direction::Right);
                                driver.dispatch(Action::Choose);
                            },
                            on_swipe_up: move |_card: Card| {
                                // Save is non-committal: the pile advances and
                                // selection continues, exactly like a skip.
                                usage_buffer().record_swipe(Direction::Up);
                                driver.dispatch(Action::Save);
                            },
                            on_swipe_down: move |_card: Card| {
                                usage_buffer().record_swipe(Direction::Down);
                                driver.dispatch(Action::Undo);
                            },
                        }
                    } else if let Some(line) = empty_line {
                        div { class: "message-empty",
                            span { class: "chip-note", "{line}" }
                        }
                    } else {
                        CardSkeleton {}
                    }
                }

                // Filter, refresh and card details as the action dock. The
                // suggested ranking filters by the host's Show chips alone; a
                // chosen identity lights the dot.
                div { class: "action-dock action-dock-right",
                    if has_suggested_filter {
                        DockFilter {
                            label: model::FILTER,
                            dot: has_suggested_identity,
                            onclick: move |()| suggested_filter_open.set(true),
                        }
                    } else if !suggested {
                        // Popularity order is this screen's default (no
                        // sort), so any user filter or explicit sort lights
                        // the dot.
                        DockFilter {
                            label: model::FILTER,
                            dot: model::filter_lit(&filter_builder.read()),
                            onclick: move |()| filters_overlay_open.set(true),
                        }
                    }
                    DockAction {
                        label: model::REFRESH,
                        icon: icons::REFRESH,
                        onclick: move |()| refresh(),
                    }
                    span { class: "action-dock-divider" }
                    button {
                        class: "action-dock-btn",
                        "aria-label": "Card details",
                        disabled: !has_cards,
                        onclick: move |_| {
                            let mut open = show_details;
                            open.toggle();
                        },
                        EyeIcon {}
                    }
                }

                // Root-mounted (siblings of the ActionBar) so the bottom sheets
                // cover the footer, the printing sheet after the details sheet
                // so it draws over it.
                if let Some(card) = current_card {
                    CardDetailsSheet {
                        open: show_details,
                        card,
                        on_printings: move |()| printing_open.set(true),
                        footer: current_reasons.map(|reasons| rsx! {
                            SuggestionReasons {
                                reasons,
                                deck_count,
                                describe_tag,
                                on_examples,
                            }
                        }),
                    }
                }

                // Pick a printing → swap the focused card and drop both sheets;
                // swipe-right selects that printing (persisted as commander_id).
                if let Some(card) = printing_card {
                    PrintingSheet {
                        card,
                        open: printing_open,
                        host_screen,
                        on_save: move |new_card: Card| {
                            driver.dispatch(Action::PickPrinting(Box::new(new_card)));
                            show_details.set(false);
                        },
                    }
                }

                CardFilterSheet {
                    open: filters_overlay_open,
                    show_format_filter: false,
                    show_active_indicators: true,
                    on_clear: move |()| clear_filter(),
                }

                // A chip applies as it is tapped and the stack re-deals behind
                // the sheet, so the sheet only closes.
                if let (true, Some(chips)) = (suggested, suggested_filter) {
                    BottomSheet {
                        open: suggested_filter_open,
                        title: "Filter",
                        closable: true,
                        {chips}
                    }
                }

                ContentHintDialog {
                    open: hint_open,
                    actions: components::hint::guide_for_swipe_select(hint_mode),
                    content: one_time::swipe_select(hint_mode),
                }
            }
        }
    }
}
