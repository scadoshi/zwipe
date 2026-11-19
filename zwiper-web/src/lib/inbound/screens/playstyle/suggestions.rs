//! The suggested commanders, which the Commanders screen lists:
//! [`use_suggestions`] loads and edits the list, [`SuggestedFold`] draws it
//! with its Swipe and its rows, and [`SuggestedOverlays`] holds its Clear
//! skips dialog and its Swipe select. The same load carries the builder
//! profile Playstyle draws, which reads it through [`use_held`] alone.

use std::collections::HashSet;

use components::{
    AlertDialog, Button, ButtonVariant, CardListFold, DialogAction, TOAST_LONG, TOAST_QUICK,
    spawn_write, use_mounted,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::{self, Color};
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::http::contracts::suggestions::{HttpBuilderProfile, HttpCommanderSuggestion};
use zwipe_ui::generation::Generation;
use zwipe_ui::playstyle::{HELD_SIZE, LIST_SIZE, of_identity, shown, suggested_empty_line};

use super::chips::ShowIdentityChips;
use super::rows::SuggestionRow;
use crate::inbound::caches::CreateDeckSeed;
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::swipe_select::{SuggestedPile, SwipeMode, SwipeSelect};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::router::Route;

/// A suggestion's colors, the identity the Show chips read.
fn suggestion_colors(suggestion: &HttpCommanderSuggestion) -> &[Color] {
    &suggestion.card.scryfall_data.color_identity
}

/// `suggestions` the applied `filter` keeps, in their order. An empty filter, or one that
/// doesn't build, keeps them all.
fn passing(
    suggestions: Vec<HttpCommanderSuggestion>,
    filter: &CardQueryBuilder,
) -> Vec<HttpCommanderSuggestion> {
    if filter.is_empty() {
        return suggestions;
    }
    let Ok(criteria) = filter.build_criteria() else {
        return suggestions;
    };
    let cards = Cards::from(
        suggestions
            .iter()
            .map(|s| s.card.clone())
            .collect::<Vec<Card>>(),
    );
    let kept: HashSet<Uuid> = Vec::<Card>::from(cards.matching(&criteria))
        .iter()
        .map(|c| c.scryfall_data.id)
        .collect();
    suggestions
        .into_iter()
        .filter(|s| kept.contains(&s.card.scryfall_data.id))
        .collect()
}

/// The suggested commanders list: its load state, its Show chip, what it shows and what its
/// rows, Swipe and Clear skips do. Cheap to copy; every field is a signal or a callback.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Suggestions {
    /// The held ranking, 100 deep.
    pub(crate) entries: Signal<Vec<HttpCommanderSuggestion>>,
    /// The user's deck count, which the reasons count against; 0 before the first load.
    pub(crate) deck_count: Signal<u32>,
    /// The builder profile the same load carries, for Playstyle's own sections.
    pub(crate) profile: Signal<Option<HttpBuilderProfile>>,
    /// Whether the first load is still out.
    pub(crate) is_loading: Signal<bool>,
    /// Whether a load failed with nothing to show.
    pub(crate) load_failed: Signal<bool>,
    /// The Show chip in effect, `None` being All.
    pub(crate) active_identity: Signal<Option<Vec<Color>>>,
    /// The Show row's chips: the identities in the held list plus the chosen one.
    pub(crate) identities: Memo<Vec<Vec<Color>>>,
    /// The held list narrowed by the Show chip and the screen's filter.
    pub(crate) filtered: Memo<Vec<HttpCommanderSuggestion>>,
    /// The user's decks, for the create gate and Staples' deck chips; `None` until loaded.
    pub(crate) deck_profiles: Signal<Option<Vec<DeckProfile>>>,
    swipe_open: Signal<bool>,
    clear_open: Signal<bool>,
    pile: Memo<SuggestedPile>,
    create_with: Callback<Card>,
    on_maybe: Callback<Uuid>,
    on_dismiss: Callback<Uuid>,
    clear_dismissals: Callback<()>,
}

impl Suggestions {
    /// Opens the Clear skips dialog.
    pub(crate) fn ask_clear(mut self) {
        self.clear_open.set(true);
    }
}

/// The held ranking's load: what it holds and the tickets that keep an old reply from landing.
#[derive(Clone, Copy)]
pub(crate) struct Held {
    entries: Signal<Vec<HttpCommanderSuggestion>>,
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

impl Held {
    /// Drops any load still out and fetches again. Silent while rows show.
    fn refetch(mut self) {
        self.loads.write().next_ticket();
        let tick = *self.reload.peek();
        self.reload.set(tick.wrapping_add(1));
    }
}

/// Loads the held ranking, 100 deep, again on every [`Held::refetch`]. Each load takes a ticket
/// and only the newest one's reply lands.
pub(crate) fn use_held() -> Held {
    let authed = use_authed();
    let load = Held {
        entries: use_signal(Vec::new),
        deck_count: use_signal(|| 0u32),
        profile: use_signal(|| None),
        is_loading: use_signal(|| true),
        load_failed: use_signal(|| false),
        reload: use_signal(|| 0u32),
        loads: use_signal(Generation::default),
        load_out: use_signal(|| false),
    };
    let Held {
        mut entries,
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
                    c.commander_suggestions(HELD_SIZE, &s).await
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
                    entries.set(found.suggestions);
                    load_failed.set(false);
                }
                Err(_) => load_failed.set(entries.peek().is_empty()),
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
                "Verify your email to create more than one deck".to_string(),
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

/// A row's Maybeboard and Remove, which drop it through `drop_row`, and Clear skips, which
/// refetches through `load`.
fn use_writes(
    drop_row: Callback<Uuid>,
    load: Held,
) -> (Callback<Uuid>, Callback<Uuid>, Callback<()>) {
    let toast = use_toast();
    // A save or skip lands even after the screen goes; its row update waits on this.
    let mounted = use_mounted();
    let stores = try_use_context::<CardStores>();
    let authed = use_authed();
    let on_maybe = use_callback(move |oracle_id: Uuid| {
        spawn_write(async move {
            if authed
                .run("add_commander_maybeboard", |c, s| async move {
                    c.add_commander_maybeboard_card(oracle_id, &s).await
                })
                .await
                .is_some()
            {
                if let Some(stores) = stores {
                    stores.maybeboard_written();
                }
                if mounted.get() {
                    drop_row.call(oracle_id);
                }
                toast.success(
                    "Added to your commander maybeboard".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    });
    let on_dismiss = use_callback(move |oracle_id: Uuid| {
        spawn_write(async move {
            if authed
                .run("dismiss_commander", |c, s| async move {
                    c.dismiss_commander(oracle_id, &s).await
                })
                .await
                .is_some()
            {
                if mounted.get() {
                    drop_row.call(oracle_id);
                }
                toast.info(
                    "Skipped for a week".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    });
    let clear_dismissals = use_callback(move |()| {
        spawn_write(async move {
            if authed
                .run("clear_commander_dismissals", |c, s| async move {
                    c.clear_commander_dismissals(&s).await
                })
                .await
                .is_some()
            {
                toast.info(
                    "Skips cleared".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
                if mounted.get() {
                    load.refetch();
                }
            }
        });
    });
    (on_maybe, on_dismiss, clear_dismissals)
}

/// Loads the suggested commanders and holds what their list does, narrowed by `filter` where a
/// screen filters them. The list refetches when Swipe select closes, after a drop leaves the
/// filtered list short and after Clear skips; a refetch with rows showing keeps them. A dropped
/// row restarts a load still out, since its reply predates the drop.
pub(crate) fn use_suggestions(filter: Option<Signal<CardQueryBuilder>>) -> Suggestions {
    let swipe_open = use_signal(|| false);
    let clear_open = use_signal(|| false);
    let load = use_held();
    let mut entries = load.entries;
    let deck_count = load.deck_count;

    // The overlay's swipes dismissed and saved commanders the list must not
    // show any more, so closing it refetches.
    let mut swipe_was_open = use_signal(|| false);
    use_effect(move || {
        if swipe_open() {
            swipe_was_open.set(true);
        } else if *swipe_was_open.peek() {
            swipe_was_open.set(false);
            load.refetch();
        }
    });

    // The chips are the identities in the held list plus the chosen one, so
    // a chip stays after its last commander goes and the empty line says so.
    let active_identity: Signal<Option<Vec<Color>>> = use_signal(|| None);
    let identities = use_memo(move || {
        let held = entries();
        let chosen = active_identity();
        colors::identities_present(held.iter().map(suggestion_colors).chain(chosen.as_deref()))
    });
    let filtered = use_memo(move || {
        let of_one = of_identity(&entries(), suggestion_colors, active_identity().as_deref());
        match filter {
            Some(filter) => passing(of_one, &filter()),
            None => of_one,
        }
    });
    let pile = use_memo(move || SuggestedPile {
        suggestions: filtered(),
        deck_count: deck_count(),
        identity: active_identity(),
    });

    // Drops a row; the next held suggestion of that identity moves up. The
    // server is asked again only when the filtered list runs under ten or a
    // load from before the drop is still out.
    let drop_row = use_callback(move |oracle_id: Uuid| {
        entries
            .write()
            .retain(|s| s.card.scryfall_data.oracle_id != Some(oracle_id));
        let remaining = of_identity(
            &entries.peek(),
            suggestion_colors,
            active_identity.peek().as_deref(),
        )
        .len();
        if remaining < LIST_SIZE || *load.load_out.peek() {
            load.refetch();
        }
    });

    let (deck_profiles, create_with) = use_create_gate();
    let (on_maybe, on_dismiss, clear_dismissals) = use_writes(drop_row, load);

    Suggestions {
        entries,
        deck_count,
        profile: load.profile,
        is_loading: load.is_loading,
        load_failed: load.load_failed,
        active_identity,
        identities,
        filtered,
        deck_profiles,
        swipe_open,
        clear_open,
        pile,
        create_with,
        on_maybe,
        on_dismiss,
        clear_dismissals,
    }
}

/// The list's fold, titled `title`: Swipe and the list's `?` in its header, the empty lines and
/// the first ten rows, each expanding into `expanded_card`. Its controls sit above it, in the
/// screen's console.
#[component]
pub(crate) fn SuggestedFold(
    list: Suggestions,
    title: &'static str,
    expanded_card: Signal<Option<Uuid>>,
) -> Element {
    let Suggestions {
        entries,
        deck_count,
        active_identity,
        filtered,
        mut swipe_open,
        create_with,
        on_maybe,
        on_dismiss,
        ..
    } = list;
    let has_entries = !entries().is_empty();
    let rows = shown(&filtered());
    rsx! {
        // Swipe sits in the list's header, so it says
        // what gets swiped.
        CardListFold {
            title,
            actions: rsx! {
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| swipe_open.set(true),
                    "Swipe"
                }
                InfoButton { topic: HintTopic::SuggestionList }
            },
            if !has_entries {
                div { class: "activity-total activity-total-group",
                    span { class: "chip-note", "No suggestions right now" }
                }
            } else if rows.is_empty() {
                div { class: "activity-total activity-total-group",
                    span { class: "chip-note",
                        {
                            let identity = active_identity();
                            let of_one = of_identity(&entries(), suggestion_colors, identity.as_deref());
                            suggested_empty_line(&of_one, identity.as_deref())
                        }
                    }
                }
            }
            for suggestion in rows {
                SuggestionRow {
                    key: "{suggestion.card.scryfall_data.id}",
                    suggestion,
                    deck_count: deck_count(),
                    expanded_card,
                    on_start: create_with,
                    on_maybe,
                    on_dismiss,
                }
            }
        }
    }
}

/// The list's Clear skips dialog and its Swipe select, whose filter is the same Show row.
#[component]
pub(crate) fn SuggestedOverlays(list: Suggestions) -> Element {
    let Suggestions {
        active_identity,
        identities,
        mut swipe_open,
        clear_open,
        pile,
        create_with,
        clear_dismissals,
        ..
    } = list;
    rsx! {
        AlertDialog {
            open: clear_open,
            title: "Clear skips",
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Confirm {
                    label: "Clear".into(),
                    danger: true,
                    on_confirm: clear_dismissals,
                },
            ],
            "Commanders you skipped will be suggested again. This can't be undone."
        }
        SwipeSelect {
            open: swipe_open,
            mode: SwipeMode::Suggested,
            suggested_pile: Some(pile.into()),
            suggested_filter: rsx! {
                ShowIdentityChips { active_identity, identities: identities() }
            },
            on_select: move |card: Card| {
                swipe_open.set(false);
                create_with.call(card);
            },
            on_close: move |()| swipe_open.set(false),
        }
    }
}
