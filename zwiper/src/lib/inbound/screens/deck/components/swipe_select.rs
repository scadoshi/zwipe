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
//! [`SwipeMode::Suggested`] deals the merged commander ranking a page at a time
//! instead of a search, starting from the host's filter (`start_filter`). There a
//! left swipe dismisses the commander for a week and the eyeball sheet carries the
//! reasons it was suggested.
//!
//! What the overlay knows and does is `zwipe_ui::screens::swipe_select`; this
//! file draws its state and performs its effects.

use std::collections::{HashMap, HashSet};

use components::hint::ContentHintDialog;
use components::icons::{self, EyeIcon};
use components::{
    Chip, DockAction, DockFilter, LoadFailed, ToastOptions, Toasts, spawn_in_order,
    use_overlay_back, use_toast,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::query::CardQuery;
use zwipe_core::http::contracts::suggestions::HttpCommanderDeal;
use zwipe_ui::generation::Ticket;
use zwipe_ui::hint::{SwipeSelectMode, one_time};
use zwipe_ui::screens::add_cards;
pub(crate) use zwipe_ui::screens::swipe_select::Mode as SwipeMode;
use zwipe_ui::screens::swipe_select::{self as model, Action, Destination, Effect, State, Status};
use zwipe_ui::stack;

use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::inbound::components::interactions::swipe::SwipeStack;
use crate::inbound::components::interactions::swipe::config::SwipeConfig;
use crate::inbound::components::interactions::swipe::direction::Direction;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::swipe_confirm::{self, Said, SwipeConfirmChip};
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::components::telemetry::vocabulary::component;
use crate::inbound::components::{held_open, hint_dialog, ui_toast};
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
            Effect::LoadMaybeboard { ticket } => self.load_maybeboard(ticket),
            Effect::DealCommanders { ticket, request } => self.deal(ticket, *request),
            Effect::DismissCommander(oracle_id) => {
                // Quiet like a deck skip: the pile already moved on, and a
                // failed post only means it comes back sooner.
                let client = self.client;
                spawn_in_order(oracle_id, async move {
                    let reply = match self.fresh().await {
                        Ok(session) => client().dismiss_commander(oracle_id, &session).await,
                        Err(e) => Err(e),
                    };
                    if let Err(e) = reply {
                        tracing::warn!("dismiss commander failed: {e}");
                    }
                });
            }
            Effect::UndismissCommander(oracle_id) => {
                // Quiet like the dismissal itself: the card is already back.
                let client = self.client;
                spawn_in_order(oracle_id, async move {
                    let reply = match self.fresh().await {
                        Ok(session) => client().undismiss_commander(oracle_id, &session).await,
                        Err(e) => Err(e),
                    };
                    if let Err(e) = reply {
                        tracing::warn!("undo commander dismissal failed: {e}");
                        self.report("undismiss_commander", &e);
                    }
                });
            }
            Effect::AddMaybeboardCard { ticket, oracle_id } => {
                let client = self.client;
                spawn_in_order(oracle_id, async move {
                    let reply = match self.fresh().await {
                        Ok(session) => {
                            client()
                                .add_commander_maybeboard_card(oracle_id, &session)
                                .await
                        }
                        Err(e) => Err(e),
                    };
                    self.settle(
                        ticket,
                        reply,
                        "add to commander maybeboard failed",
                        "add_commander_maybeboard",
                    );
                });
            }
            Effect::RemoveMaybeboardCard { ticket, oracle_id } => {
                let client = self.client;
                spawn_in_order(oracle_id, async move {
                    let reply = match self.fresh().await {
                        Ok(session) => {
                            client()
                                .remove_commander_maybeboard_card(oracle_id, &session)
                                .await
                        }
                        Err(e) => Err(e),
                    };
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

    /// Asks the merged ranking for `request` and answers with [`Action::Dealt`], logging a
    /// failure.
    fn deal(self, ticket: Ticket, request: HttpCommanderDeal) {
        let client = self.client;
        spawn(async move {
            let reply = match self.fresh().await {
                Ok(session) => client().deal_commanders(request, &session).await,
                Err(e) => Err(e),
            };
            let reply = reply.map_err(|e| tracing::warn!("swipe select deal failed: {e}"));
            self.dispatch(Action::Dealt { ticket, reply });
        });
    }

    /// Loads the user's commander maybeboard and answers with [`Action::MaybeboardLoaded`], or
    /// with [`Action::Failed`] after logging the failure.
    fn load_maybeboard(self, ticket: Ticket) {
        let client = self.client;
        spawn(async move {
            let reply = match self.fresh().await {
                Ok(session) => client().fetch_commander_maybeboard(&session).await,
                Err(e) => Err(e),
            };
            self.dispatch(match reply {
                Ok(saved) => Action::MaybeboardLoaded { ticket, saved },
                Err(e) => {
                    tracing::warn!("swipe select maybeboard load failed: {e}");
                    Action::Failed { ticket }
                }
            });
        });
    }

    /// The session with an access token that is still good, refreshed first when it is due.
    async fn fresh(self) -> Result<Session, ClientError> {
        self.session.ensure_fresh(self.client).await
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
                self.toast.error(
                    components::carry_title(&e.to_user_message()),
                    ToastOptions::default(),
                );
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
    /// deal); deck create/edit pass the cards already in the command zone, so
    /// the partner stack never offers the commander.
    #[props(default)]
    exclude_oracle_ids: Option<ReadSignal<HashSet<Uuid>>>,
    /// The host's filter, which the pile starts from: an open that finds it
    /// changed since the last deals again under it, on Search.
    #[props(default)]
    start_filter: Option<ReadSignal<CardQueryBuilder>>,
    /// Decks the suggested reasons count against, as the host knows it.
    #[props(default)]
    reasons_deck_count: Option<ReadSignal<u32>>,
    /// Whether a pick starts a new deck, as from the commander maybeboard,
    /// which the hint then says.
    #[props(default)]
    starts_deck: bool,
) -> Element {
    // OS back gesture closes this overlay before touching the router.
    let layer = use_overlay_back(open);
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
    // sortless: the server's popularity ordering is the default. Persists across open/close
    // because this component stays mounted.
    let mut filter_builder = use_signal(CardQueryBuilder::new);
    use_context_provider(|| filter_builder);
    let mut filter_reset_counter = use_signal(|| 0u32);
    use_context_provider(|| filter_reset_counter);
    // The counter value the pile was last searched for: the sheet's Apply
    // bumps the counter, and Refresh bumps it and searches itself.
    let mut searched_counter = use_signal(|| 0u32);

    let mut filters_overlay_open = use_signal(|| false);
    let mut show_details = use_signal(|| false);
    // The face each card shows, the front where it has none: its stack image and its details
    // drawer show it, and either's flip turns it, so the image and the details turn together.
    let faces: Signal<HashMap<Uuid, usize>> = use_signal(HashMap::new);
    let mut printing_open = use_signal(|| false);

    // Each open hands the model the mode, the filter, the reasons' deck count
    // and the host's exclusions; the model keeps the pile or deals afresh, and
    // offers the hint on the first open. A host filter that changed since deals
    // again under it.
    use_effect(move || {
        if !open() {
            return;
        }
        let mode = mode.peek().clone();
        let filter = filter_builder.peek().clone();
        let deck_count = reasons_deck_count.map_or(0, |count| *count.peek());
        let excluded = exclude_oracle_ids.map_or_default(|ids| ids.peek().clone());
        driver.dispatch(Action::Open {
            mode,
            filter,
            deck_count,
            excluded,
            from: starts_deck,
        });
        let on_search = driver.model.peek().source() != Some(add_cards::Source::Maybeboard);
        if let Some(start) = start_filter.filter(|_| on_search) {
            let start = start.peek().clone();
            if *driver.model.peek().filter() != start {
                driver.dispatch(Action::Filter(start));
            }
        }
        // An open back on Search takes Search's own filter: the sheet shows it, its sections reset.
        let dealt_under = driver.model.peek().filter().clone();
        if *filter_builder.peek() != dealt_under {
            filter_builder.set(dealt_under);
            let counter = filter_reset_counter.peek().wrapping_add(1);
            searched_counter.set(counter);
            filter_reset_counter.set(counter);
        }
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

    // Bumps the counter so the filter sections reset with the pile, and
    // searches again from the top.
    let mut refresh = move || {
        let counter = filter_reset_counter().wrapping_add(1);
        searched_counter.set(counter);
        filter_reset_counter.set(counter);
        let filter = filter_builder.peek().clone();
        driver.dispatch(Action::Refresh(filter));
    };

    // The From chip: the model parks this source's filter and deals the other, and the filter
    // sheet shows the filter it took back, its sections reset to it.
    let mut pick_source = move || {
        driver.dispatch(Action::PickSource);
        let taken = driver.model.peek().filter().clone();
        filter_builder.set(taken);
        let counter = filter_reset_counter().wrapping_add(1);
        searched_counter.set(counter);
        filter_reset_counter.set(counter);
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
    let backdrop = held_open::use_held_open(open.into());
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
    let current_reasons = state.reasons().cloned();
    let deck_count = state.deck_count();
    drop(state);
    // The reasons behind the card on top, for the details sheet's body. Its
    // shared-tag chips take the sheet's own description lookup and the host's
    // examples opener as props: the suggested commanders screen provides the
    // opener, and its examples overlay draws over this screen and the sheet.
    let describe_tag = use_describe_tag();
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let hint_mode = if suggested {
        SwipeSelectMode::Suggested { has_filter: true }
    } else if starts_deck {
        SwipeSelectMode::StartsDeck
    } else {
        SwipeSelectMode::Picking(noun)
    };
    let title = hint_mode.title();
    let source = driver.model.read().source();
    let mut directions = vec![Direction::Left, Direction::Right, Direction::Down];
    if maybe_enabled {
        directions.push(Direction::Up);
    }
    let swipe_config = SwipeConfig::new(directions, SWIPE_DISTANCE_PX, SWIPE_SPEED_PX_PER_MS);

    let dock = rsx! {
        div { class: if source.is_some() { "action-dock" } else { "action-dock action-dock-right" },
            // Popularity order is this screen's default (no sort), so any user
            // filter or explicit sort lights the dot.
            DockFilter {
                label: model::FILTER,
                dot: model::filter_lit(&filter_builder.read()),
                onclick: move |()| filters_overlay_open.set(true),
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
    };

    rsx! {
        div { class: "{screen_class}", style: layer().map(|z| format!("--overlay-z: {z};")),
            // A screen over another paints its own backdrop, since .screen is
            // clear and the one beneath would show through.
            if backdrop {
                ::components::GridCells { still: true }
            }
            if open() {
                ScreenHeader { title, hint: hint_open, back: move |()| driver.dispatch(Action::Back), search: false }

                div { class: "screen-content card-swipe content-enter",
                    SwipeConfirmChip { said: driver.said, at_end: status == Status::Empty }
                    if has_cards {
                        SwipeStack {
                            faces: Some(faces),
                            ghost: true,
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
                    } else if status == Status::Failed {
                        LoadFailed { message: model::LOAD_FAILED, on_retry: move |()| driver.dispatch(Action::Retry) }
                    } else if let Some(line) = empty_line {
                        // Why there is no card, inside the card's own box, as deck Add says it.
                        CardSkeleton { note: line }
                    } else {
                        CardSkeleton {}
                    }
                }

                // Filter, refresh and card details as the action dock. The
                // suggested ranking filters by the host's Show chips alone; a
                // chosen identity lights the dot.
                // The Commander maybeboard's swipe has From on the dock's left, as Add does: one
                // chip naming the pile, a tap switching to the other.
                if let Some(source) = source {
                    div { class: "swipe-bottom-row",
                        div { class: "swipe-bottom-chips",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "{add_cards::FROM}" }
                            Chip { selected: true, onclick: move |_| pick_source(), "{source.label()}" }
                        }
                        {dock}
                    }
                } else {
                    {dock}
                }

                // Root-mounted (siblings of the ActionBar) so the bottom sheets
                // cover the footer, the printing sheet after the details sheet
                // so it draws over it.
                if let Some(card) = current_card {
                    CardDetailsSheet {
                        faces: Some(faces),
                        open: show_details,
                        card,
                        on_printings: move |()| printing_open.set(true),
                        // Folded under the row's chip at the body's end, so the foot keeps
                        // only its actions.
                        body_end: current_reasons.map(|reasons| rsx! {
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

                ContentHintDialog {
                    open: hint_open,
                    actions: components::hint::guide_for_swipe_select(hint_mode),
                    content: one_time::swipe_select(hint_mode),
                }
            }
        }
    }
}
