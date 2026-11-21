//! The Commanders screen's Suggested list, under the Maybeboard: the merged commander ranking dealt
//! ten at a time under the screen's one filter, with its Show row, Swipe, the list's `?`, Show
//! more once under the panel, and Clear skips in its Danger zone.
//!
//! What the list knows and does is `zwipe_ui::screens::suggested_commanders`, which the web's
//! Commanders screen runs on too; this file draws its state and performs its effects.

use components::{
    AlertDialog, Button, ButtonVariant, CardListFold, Chip, DialogAction, ListMore, LoadFailed,
    MoreGhostRows, Toasts, use_toast,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_ui::commander::more_ghosts::{GHOSTS_KEY, MoreItem};
use zwipe_ui::screens::commander_maybeboard as maybeboard_model;
use zwipe_ui::screens::suggested_commanders::{self as model, Action, Effect, Page, State};

use super::components::danger_zone::DangerZoneAction;
use super::components::skeletons::{ChipGhost, ChipRowGhost};
use super::components::swipe_select::{SwipeMode, SwipeSelect};
use super::suggested_commanders::SuggestionRow;
use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::InfoButton;
use crate::inbound::components::telemetry::vocabulary::{Screen, screen};
use crate::inbound::components::ui_toast;
use crate::inbound::screens::deck::card::components::card_row::ShowRowArt;

/// How deep the suggestions route is asked for: no list, only the deck count the reasons count
/// against, since the list deals from the merged ranking.
const HELD_LIMIT: u32 = 0;

/// The list's model with the handles that perform its effects. Copy, so every handler and task
/// holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    authed: Authed,
    toast: Toasts,
    /// Called once a row's save landed, so the Maybeboard above loads it.
    on_saved: Callback<()>,
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
        let authed = self.authed;
        match effect {
            Effect::Deal {
                ticket,
                request,
                page,
            } => {
                let op = match page {
                    Page::First => "deal_suggestions",
                    Page::More => "deal_more_suggestions",
                };
                spawn(async move {
                    let request = *request;
                    // try_run reports a failure and raises its error toast.
                    let reply = authed
                        .try_run(
                            op,
                            |c, s| async move { c.deal_commanders(request, &s).await },
                        )
                        .await
                        .map_err(|_| ());
                    self.dispatch(Action::Page { ticket, reply });
                });
            }
            Effect::AddMaybeboard(oracle_id) => {
                spawn(async move {
                    if authed
                        .run("add_commander_maybeboard", |c, s| async move {
                            c.add_commander_maybeboard_card(oracle_id, &s).await
                        })
                        .await
                        .is_some()
                    {
                        self.on_saved.call(());
                        self.dispatch(Action::Saved(oracle_id));
                    }
                });
            }
            Effect::Dismiss(oracle_id) => {
                spawn(async move {
                    if authed
                        .run("dismiss_commander", |c, s| async move {
                            c.dismiss_commander(oracle_id, &s).await
                        })
                        .await
                        .is_some()
                    {
                        self.dispatch(Action::Dismissed(oracle_id));
                    }
                });
            }
            Effect::ClearDismissals => {
                spawn(async move {
                    if authed
                        .run("clear_commander_dismissals", |c, s| async move {
                            c.clear_commander_dismissals(&s).await
                        })
                        .await
                        .is_some()
                    {
                        self.dispatch(Action::Cleared);
                    }
                });
            }
            Effect::Toast(toast) => ui_toast::raise(self.toast, toast),
        }
    }
}

/// The Suggested list over `model`, dealt under the screen's `filter`, again each time the filter
/// sheet's Apply bumps `filter_applied`. Its rows open into `expanded_card` and their image and
/// Printings into the screen's overlays; Create deck and a right swipe go through `on_create`, and
/// a row's save calls `on_saved`. Its Show row's Art chip shows or hides this list's art only.
#[component]
pub(crate) fn SuggestedSection(
    model: Signal<State>,
    filter: Signal<CardQueryBuilder>,
    filter_applied: Signal<u32>,
    expanded_card: Signal<Option<Uuid>>,
    preview_card: Signal<Option<(ScryfallData, usize)>>,
    preview_dismissing: Signal<bool>,
    on_printing: EventHandler<Card>,
    on_create: Callback<Card>,
    on_saved: Callback<()>,
) -> Element {
    let driver = Driver {
        model,
        authed: use_authed(Screen::CommanderMaybeboard),
        toast: use_toast(),
        on_saved,
    };
    let authed = driver.authed;
    let mut show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));
    let mut swipe_open = use_signal(|| false);
    let swipe_mode = use_memo(|| Some(SwipeMode::Suggested));
    let mut clear_open = use_signal(|| false);
    // The open state of the list's fold, so Show more under it hides while it is shut.
    let fold_open = use_signal(|| true);
    let mut deck_count = use_signal(|| 0u32);

    // The first page on mount, and again whenever the filter sheet applies.
    use_effect(move || {
        let _applied = filter_applied();
        let applied = filter.peek().clone();
        driver.dispatch(Action::Load(Box::new(applied)));
    });

    // The deck count the reasons count against, once.
    use_hook(move || {
        spawn(async move {
            let found = authed
                .run_quiet("load_suggestions", |c, s| async move {
                    c.commander_suggestions(HELD_LIMIT, &s).await
                })
                .await;
            if let Some(found) = found {
                deck_count.set(found.deck_count);
            }
        });
    });

    // The overlay's swipes saved and skipped commanders the list must not show any more, so
    // closing it deals again.
    let mut swipe_was_open = use_signal(|| false);
    use_effect(move || {
        if swipe_open() {
            swipe_was_open.set(true);
        } else if *swipe_was_open.peek() {
            swipe_was_open.set(false);
            driver.dispatch(Action::Redeal);
        }
    });

    let clear_skips = use_callback(move |()| driver.dispatch(Action::ClearSkips));
    let on_maybe = move |oracle_id: Uuid| driver.dispatch(Action::Maybeboard(oracle_id));
    let on_dismiss = move |oracle_id: Uuid| driver.dispatch(Action::Dismiss(oracle_id));

    let state = model.read();
    let is_loading = state.is_loading();
    let load_failed = state.load_failed();
    let items = state.layout();
    let ghosts = state.ghosts().clone();
    let line = state.ranked_line();
    let empty = state.empty_line();
    let offers = state.offers_more();
    drop(state);

    rsx! {
        // Its own column, a panel gap under the Maybeboard's Danger zone.
        div { class: "flex-col commanders-suggested",
            if is_loading {
                ChipRowGhost {
                    label: maybeboard_model::SHOW,
                    chips: vec![ChipGhost::Text(maybeboard_model::ART.to_string())],
                }
                div { class: "maybeboard-list card-group row-enter",
                    div { class: "card-group-header card-group-header-with-hint",
                        div { class: "skeleton-bar skeleton-card-bar-header" }
                        span { class: "card-list-fold-hint",
                            div { class: "skeleton-bar skeleton-chip skeleton-info-button" }
                        }
                    }
                    for i in 0..3 {
                        div { key: "{i}", class: "skeleton-card-row",
                            div { class: "skeleton-bar skeleton-card-thumb" }
                            div { class: "skeleton-bar skeleton-card-bar-row" }
                        }
                    }
                }
            } else if load_failed {
                LoadFailed { message: model::LOAD_FAILED, on_retry: move |()| driver.dispatch(Action::Redeal) }
            } else {
                // The console's Show row, Art alone: colors come from the screen's filter.
                div { class: "chip-row",
                    span { class: "stat-chip stat-chip-p1 chip-row-label", {maybeboard_model::SHOW} }
                    Chip {
                        selected: show_row_art(),
                        onclick: move |_| show_row_art.set(!show_row_art()),
                        {maybeboard_model::ART}
                    }
                }
                // Swipe sits in the list's header, so it says what gets swiped.
                CardListFold {
                    title: model::LIST_TITLE,
                    open: fold_open,
                    hint: rsx! {
                        InfoButton { topic: HintTopic::SuggestionList }
                    },
                    actions: rsx! {
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| swipe_open.set(true),
                            {maybeboard_model::SWIPE}
                        }
                    },
                    if let Some(line) = line {
                        div { class: "card-row card-row-note", {line} }
                    }
                    if let Some(empty) = empty {
                        div { class: "card-row card-row-note", {empty} }
                    }
                    // One keyed run, so rows a later page joins after never change parent.
                    for item in items {
                        {match item {
                            MoreItem::Row(suggestion) => rsx! {
                                SuggestionRow {
                                    key: "{suggestion.card.scryfall_data.id}",
                                    suggestion,
                                    deck_count: deck_count(),
                                    expanded_card,
                                    preview_card,
                                    preview_dismissing,
                                    on_start: move |card: Card| on_create.call(card),
                                    on_maybe,
                                    on_dismiss,
                                    on_printing: move |card: Card| on_printing.call(card),
                                }
                            },
                            // Show more's ghost rows answer its press while its page is out, then
                            // lie under the rows it dealt as they ease in.
                            MoreItem::Ghosts => rsx! {
                                MoreGhostRows {
                                    key: "{GHOSTS_KEY}",
                                    ghosts: ghosts.clone(),
                                    on_animation_end: move |name: String| driver.dispatch(Action::GhostsEnded(name)),
                                }
                            },
                        }}
                    }
                }
                // Show more sits under the panel, as a chip row sits over one, and deals one page.
                // Once per deal, so it leaves on its exit while the page loads.
                ListMore {
                    offers,
                    open: fold_open(),
                    leaves: true,
                    onclick: move |()| driver.dispatch(Action::ShowMore),
                }
                DangerZoneAction {
                    label: model::CLEAR_SKIPS,
                    button: maybeboard_model::CLEAR,
                    onclick: move |()| clear_open.set(true),
                }
            }
        }

        AlertDialog {
            open: clear_open,
            title: model::CLEAR_SKIPS,
            actions: vec![
                DialogAction::Dismiss(maybeboard_model::CANCEL.into()),
                DialogAction::Confirm {
                    label: maybeboard_model::CLEAR.into(),
                    danger: true,
                    on_confirm: clear_skips,
                },
            ],
            {model::CLEAR_SKIPS_LINE}
        }

        // The suggested deal, from the screen's filter: right starts a deck with the pick, up
        // saves it, left skips it for a week.
        SwipeSelect {
            host_screen: screen::COMMANDER_MAYBEBOARD,
            open: swipe_open,
            mode: swipe_mode,
            start_filter: Some(filter.into()),
            reasons_deck_count: Some(deck_count.into()),
            on_select: move |card: Card| {
                swipe_open.set(false);
                on_create.call(card);
            },
            on_close: move |()| swipe_open.set(false),
        }
    }
}
