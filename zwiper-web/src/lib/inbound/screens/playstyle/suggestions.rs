//! The suggested commanders, which the Commanders screen lists:
//! [`use_suggestions`] drives zwipe-ui's [`suggested_commanders`](zwipe_ui::screens::suggested_commanders)
//! model, which deals the list a page at a time from the merged
//! commander ranking under the screen's filter and edits it, [`SuggestedFold`]
//! draws it with its Swipe and its rows, and [`SuggestedOverlays`] holds its
//! Clear skips dialog and its Swipe select. [`use_held`] loads the builder
//! profile Playstyle draws and the deck count the reasons count against.

use components::{
    AlertDialog, Button, ButtonVariant, CardListFold, DialogAction, ListMore, MoreGhostRows,
    Mounted, TOAST_LONG, spawn_write, use_mounted,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, Toasts, use_toast};
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::http::contracts::suggestions::HttpBuilderProfile;
use zwipe_ui::commander::deal::Door;
use zwipe_ui::commander::more_ghosts::MoreItem;
use zwipe_ui::generation::Generation;
use zwipe_ui::screens::playstyle as suggested_model;
use zwipe_ui::screens::suggested_commanders::{self as suggested, Action, Effect, Page, State};

use super::rows::SuggestionRow;
use crate::inbound::caches::CreateDeckSeed;
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::swipe_select::{SwipeMode, SwipeSelect};
use crate::inbound::chrome::authed::{Authed, use_authed};
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::chrome::toast;
use crate::inbound::router::Route;

/// What [`use_held`] asks the suggestions route for: no list, only the profile and deck count,
/// since the list deals from the merged ranking.
const HELD_LIMIT: u32 = 0;

/// The suggested commanders list: its [`State`] and what its rows, Swipe and Clear skips do.
/// Cheap to copy; every field is a signal or a callback.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Suggestions {
    /// The list's model.
    model: Signal<State>,
    /// The user's deck count, which the reasons count against; 0 before it loads.
    pub(crate) deck_count: Signal<u32>,
    /// The user's decks, for the create gate; `None` until loaded.
    pub(crate) deck_profiles: Signal<Option<Vec<DeckProfile>>>,
    filter: Signal<CardQueryBuilder>,
    swipe_open: Signal<bool>,
    clear_open: Signal<bool>,
    dispatch: Callback<Action>,
    create_with: Callback<Card>,
}

impl Suggestions {
    /// Opens the Clear skips dialog.
    pub(crate) fn ask_clear(mut self) {
        self.clear_open.set(true);
    }

    /// Deals the first page again, for the failed load's Retry.
    pub(crate) fn retry(self) {
        self.dispatch.call(Action::Redeal);
    }

    /// Whether the first page is still out.
    pub(crate) fn is_loading(self) -> bool {
        self.model.read().is_loading()
    }

    /// Whether a first page failed with nothing to show.
    pub(crate) fn load_failed(self) -> bool {
        self.model.read().load_failed()
    }

    /// The rows' cards, in order.
    pub(crate) fn cards(self) -> Vec<Card> {
        self.model.read().cards()
    }

    /// Whether a row's commander takes a picked printing of `card`.
    pub(crate) fn picks_printing(self, card: &Card) -> bool {
        suggested_model::picks_printing(self.model.read().entries(), card)
    }

    /// Swaps `picked` into its commander's row.
    pub(crate) fn swap_printing(self, picked: &Card) {
        let mut model = self.model;
        model.write().swap_printing(picked);
    }
}

/// The held ranking's load: what it holds and the tickets that keep an old reply from landing.
#[derive(Clone, Copy)]
pub(crate) struct Held {
    /// The user's deck count; 0 before the first load.
    pub(crate) deck_count: Signal<u32>,
    /// The builder profile the load carries.
    pub(crate) profile: Signal<Option<HttpBuilderProfile>>,
    /// Whether the first load is still out.
    pub(crate) is_loading: Signal<bool>,
    /// Whether a load failed with nothing to show.
    pub(crate) load_failed: Signal<bool>,
    reload: Signal<u32>,
    loads: Signal<Generation>,
    /// Whether the newest load's reply is still out.
    load_out: Signal<bool>,
}

/// Loads the builder profile and deck count from the suggestions route, again on every
/// [`Held::refetch`]. Each load takes a ticket and only the newest one's reply lands.
pub(crate) fn use_held() -> Held {
    let authed = use_authed();
    let load = Held {
        deck_count: use_signal(|| 0u32),
        profile: use_signal(|| None),
        is_loading: use_signal(|| true),
        load_failed: use_signal(|| false),
        reload: use_signal(|| 0u32),
        loads: use_signal(Generation::default),
        load_out: use_signal(|| false),
    };
    let Held {
        mut deck_count,
        mut profile,
        mut is_loading,
        mut load_failed,
        reload,
        mut loads,
        mut load_out,
    } = load;
    use_effect(move || {
        let _tick = reload();
        load_out.set(true);
        let ticket = loads.write().next_ticket();
        spawn(async move {
            let reply = authed
                .try_run("load_suggestions", |c, s| async move {
                    c.commander_suggestions(HELD_LIMIT, &s).await
                })
                .await;
            if !loads.peek().is_current(ticket) {
                return;
            }
            load_out.set(false);
            match reply {
                Ok(found) => {
                    deck_count.set(found.deck_count);
                    profile.set(found.profile);
                    load_failed.set(false);
                }
                Err(_) => load_failed.set(profile.peek().is_none()),
            }
            is_loading.set(false);
        });
    });
    load
}

/// The user's decks, `None` until they load: quiet, so a failure leaves them unknown.
pub(crate) fn use_deck_profiles() -> Signal<Option<Vec<DeckProfile>>> {
    let authed = use_authed();
    let mut deck_profiles: Signal<Option<Vec<DeckProfile>>> = use_signal(|| None);
    use_hook(move || {
        spawn(async move {
            let found = authed
                .run_quiet("load_deck_profiles", |c, s| async move {
                    c.fetch_deck_profiles(&s).await
                })
                .await;
            deck_profiles.set(found);
        });
    });
    deck_profiles
}

/// The user's decks and Create deck from a suggestion: the create gate the deck list's Create
/// shares, an unverified account having one deck at most.
fn use_create_gate() -> (Signal<Option<Vec<DeckProfile>>>, Callback<Card>) {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let toast = use_toast();
    let seed: CreateDeckSeed = use_context();
    let deck_profiles = use_deck_profiles();
    let at_deck_limit = move || {
        session().is_some_and(|s| {
            s.user.email_verified_at.is_none()
                && deck_profiles.read().as_ref().is_some_and(|p| !p.is_empty())
        })
    };
    let create_with = use_callback(move |card: Card| {
        if at_deck_limit() {
            toast.warning(
                components::carry_title("Verify your email to create more than one deck"),
                ToastOptions::default().duration(TOAST_LONG),
            );
        } else {
            let mut slot = seed.0;
            slot.set(Some(card));
            navigator.push(Route::CreateDeck {});
        }
    });
    (deck_profiles, create_with)
}

/// The list's model with the handles that perform its effects. Copy, so every handler and task
/// holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    authed: Authed,
    toasts: Toasts,
    mounted: Mounted,
    stores: Option<CardStores>,
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

    /// Hands a write's reply to the list while the screen is up, else raises only its toast: a
    /// save or skip lands even after the screen goes.
    fn landed(self, reply: Action) {
        if self.mounted.get() {
            self.dispatch(reply);
            return;
        }
        for effect in State::new().update(reply) {
            if let Effect::Toast(t) = effect {
                toast::raise(self.toasts, &t);
            }
        }
    }

    /// Performs one of the list's effects.
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
                spawn_write(async move {
                    if authed
                        .run("add_commander_maybeboard", |c, s| async move {
                            c.add_commander_maybeboard_card(oracle_id, &s).await
                        })
                        .await
                        .is_some()
                    {
                        if let Some(stores) = self.stores {
                            stores.maybeboard_written();
                        }
                        self.landed(Action::Saved(oracle_id));
                    }
                });
            }
            Effect::Dismiss(oracle_id) => {
                spawn_write(async move {
                    if authed
                        .run("dismiss_commander", |c, s| async move {
                            c.dismiss_commander(oracle_id, &s).await
                        })
                        .await
                        .is_some()
                    {
                        self.landed(Action::Dismissed(oracle_id));
                    }
                });
            }
            Effect::ClearDismissals => {
                spawn_write(async move {
                    if authed
                        .run("clear_commander_dismissals", |c, s| async move {
                            c.clear_commander_dismissals(&s).await
                        })
                        .await
                        .is_some()
                    {
                        self.landed(Action::Cleared);
                    }
                });
            }
            Effect::Toast(t) => toast::raise(self.toasts, &t),
        }
    }
}

/// Deals the suggested commanders a page at a time from the merged ranking under the screen's
/// `filter`, and holds what their list does ([`State`]). A filter change, Swipe select closing
/// and Clear skips deal again from the top.
pub(crate) fn use_suggestions(filter: Signal<CardQueryBuilder>) -> Suggestions {
    let swipe_open = use_signal(|| false);
    let clear_open = use_signal(|| false);
    let held = use_held();
    let model = use_signal(State::new);
    let driver = Driver {
        model,
        authed: use_authed(),
        toasts: use_toast(),
        mounted: use_mounted(),
        stores: try_use_context::<CardStores>(),
    };
    let dispatch = use_callback(move |action: Action| driver.dispatch(action));

    // The first page, on mount and on a filter change.
    use_effect(move || {
        let applied = filter();
        dispatch.call(Action::Load(Box::new(applied)));
    });

    // The overlay's swipes dismissed and saved commanders the list must not
    // show any more, so closing it deals again.
    let mut swipe_was_open = use_signal(|| false);
    use_effect(move || {
        if swipe_open() {
            swipe_was_open.set(true);
        } else if *swipe_was_open.peek() {
            swipe_was_open.set(false);
            dispatch.call(Action::Redeal);
        }
    });

    let (deck_profiles, create_with) = use_create_gate();

    Suggestions {
        model,
        deck_count: held.deck_count,
        deck_profiles,
        filter,
        swipe_open,
        clear_open,
        dispatch,
        create_with,
    }
}

/// The list's fold, titled `title`: Swipe and the list's `?` in its header, the empty lines and
/// the rows, each expanding into `expanded_card`. Show more's ghost rows sit between the rows held
/// before it and the rows its page dealt. Its controls sit above it, in the screen's console.
#[component]
pub(crate) fn SuggestedFold(
    list: Suggestions,
    title: &'static str,
    expanded_card: Signal<Option<Uuid>>,
) -> Element {
    let Suggestions {
        model,
        deck_count,
        mut swipe_open,
        dispatch,
        create_with,
        ..
    } = list;
    // The fold's state, so Show more under the panel hides while it is shut.
    let open = use_signal(|| true);
    let state = model.read();
    let items = state
        .layout()
        .into_iter()
        .map(|item| (item.key(|s| s.card.scryfall_data.id.to_string()), item));
    let ghosts = state.ghosts().clone();
    let offers = state.offers_more();
    let line = state.ranked_line();
    let empty = state.empty_line();
    drop(state);
    let on_maybe =
        use_callback(move |oracle_id: Uuid| dispatch.call(Action::Maybeboard(oracle_id)));
    let on_dismiss = use_callback(move |oracle_id: Uuid| dispatch.call(Action::Dismiss(oracle_id)));
    rsx! {
        // Swipe sits in the list's header, so it says
        // what gets swiped.
        CardListFold {
            title,
            open,
            hint: rsx! {
                InfoButton { topic: HintTopic::SuggestionList }
            },
            actions: rsx! {
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| swipe_open.set(true),
                    "Swipe"
                }
            },
            if let Some(line) = line {
                div { class: "activity-total activity-total-group",
                    span { class: "chip-note", {line} }
                }
            }
            if let Some(empty) = empty {
                div { class: "activity-total activity-total-group",
                    span { class: "chip-note", {empty} }
                }
            }
            // One keyed run, so rows a later page joins after never change parent.
            for (key, item) in items {
                Fragment { key: "{key}",
                    {match item {
                        MoreItem::Row(suggestion) => rsx! {
                            SuggestionRow {
                                suggestion,
                                deck_count: deck_count(),
                                expanded_card,
                                on_start: create_with,
                                on_maybe,
                                on_dismiss,
                            }
                        },
                        MoreItem::Ghosts => rsx! {
                            MoreGhostRows {
                                ghosts: ghosts.clone(),
                                on_animation_end: move |name: String| dispatch.call(Action::GhostsEnded(name)),
                                row_class: "web-ghost-card-row",
                            }
                        },
                    }}
                }
            }
        }
        // Show more sits under the panel and deals one page, once per deal, so it leaves on its
        // exit while the page loads.
        ListMore {
            offers,
            open: open(),
            leaves: true,
            // Taken when its page went out, which spends it; a press while a top-up is out isn't.
            onclick: move |()| {
                dispatch.call(Action::ShowMore);
                !model.peek().offers_more()
            },
        }
    }
}

/// The list's Clear skips dialog and its Swipe select, which starts from the screen's filter.
#[component]
pub(crate) fn SuggestedOverlays(list: Suggestions) -> Element {
    let Suggestions {
        filter,
        deck_count,
        mut swipe_open,
        clear_open,
        dispatch,
        create_with,
        ..
    } = list;
    let clear_dismissals = use_callback(move |()| dispatch.call(Action::ClearSkips));
    rsx! {
        AlertDialog {
            open: clear_open,
            title: suggested::CLEAR_SKIPS,
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Confirm {
                    label: "Clear".into(),
                    danger: true,
                    on_confirm: clear_dismissals,
                },
            ],
            {suggested::CLEAR_SKIPS_LINE}
        }
        SwipeSelect {
            open: swipe_open,
            mode: SwipeMode::Commanders(Door::Suggested),
            start_filter: Some(filter.into()),
            reasons_deck_count: Some(deck_count.into()),
            on_select: move |card: Card| {
                swipe_open.set(false);
                create_with.call(card);
            },
            on_close: move |()| swipe_open.set(false),
        }
    }
}
