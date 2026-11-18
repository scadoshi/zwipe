//! The Commanders screen's Suggested list: the user's top suggested
//! commanders as Playstyle draws them, with the Show row's identity chips,
//! Swipe in the list's header, Create deck, Maybeboard and Remove on each row,
//! and Clear skips under the list. The screen's filter, which it shares with
//! the Maybeboard list, narrows it too. A save to the maybeboard tells the
//! Maybeboard list to reload.

use std::collections::HashSet;

use components::{
    AlertDialog, Button, ButtonVariant, CardListFold, Chip, DangerZoneAction, DialogAction,
    HoverStackHost, TOAST_LONG, TOAST_QUICK, spawn_write, use_hover_stack, use_mounted,
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
use zwipe_core::http::contracts::suggestions::HttpCommanderSuggestion;
use zwipe_ui::generation::Generation;
use zwipe_ui::playstyle::{HELD_SIZE, LIST_SIZE, empty_identity_line, of_identity, shown};
use zwipe_ui::screens::playstyle as model;

use crate::inbound::caches::CreateDeckSeed;
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::card_row::{CardOverlaysHost, ShowRowArt, use_card_overlays};
use crate::inbound::cards::otag::{OtagExamplesHost, use_otag_reveals};
use crate::inbound::cards::swipe_select::{SuggestedPile, SwipeMode, SwipeSelect};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::ghost::{GhostShowRow, GhostText};
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::chrome::media::use_wide;
use crate::inbound::router::Route;
use crate::inbound::screens::playstyle::{ShowIdentityChips, SuggestionRow};

/// The list's header.
const LIST_TITLE: &str = "Suggested";

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

/// The Suggested list, narrowed by the screen's `filter`. Its Show row's Art chip shows or
/// hides this list's art only.
#[component]
pub(super) fn SuggestedPanel(filter: Signal<CardQueryBuilder>) -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let toast = use_toast();
    // A save or skip lands even after the screen goes; its row update waits on this.
    let mounted = use_mounted();
    let stores = try_use_context::<CardStores>();
    let authed = use_authed();
    let seed: CreateDeckSeed = use_context();
    let mut show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));
    let examples = use_otag_reveals();
    let overlays = use_card_overlays();
    let hover_stack = use_hover_stack();
    let wide = use_wide();
    let expanded_card: Signal<Option<Uuid>> = use_signal(|| None);
    let mut swipe_open = use_signal(|| false);
    let mut clear_open = use_signal(|| false);

    // The held ranking, 100 deep, loaded as Playstyle loads it: each load takes a ticket and
    // only the newest one's reply lands; a dropped row restarts a load still out.
    let mut entries: Signal<Vec<HttpCommanderSuggestion>> = use_signal(Vec::new);
    let mut deck_count = use_signal(|| 0u32);
    let mut is_loading = use_signal(|| true);
    let mut load_failed = use_signal(|| false);
    let mut reload = use_signal(|| 0u32);
    let mut loads = use_signal(Generation::default);
    let mut load_out = use_signal(|| false);
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
                    entries.set(found.suggestions);
                    load_failed.set(false);
                }
                Err(_) => load_failed.set(entries.peek().is_empty()),
            }
            is_loading.set(false);
        });
    });
    let mut refetch = move || {
        loads.write().next_ticket();
        let tick = *reload.peek();
        reload.set(tick.wrapping_add(1));
    };

    // The overlay's swipes dismissed and saved commanders the list must not
    // show any more, so closing it refetches.
    let mut swipe_was_open = use_signal(|| false);
    use_effect(move || {
        if swipe_open() {
            swipe_was_open.set(true);
        } else if *swipe_was_open.peek() {
            swipe_was_open.set(false);
            refetch();
        }
    });

    // The Show chip in effect, `None` being All. The chips are the
    // identities in the held list plus the chosen one, so a chip stays after
    // its last commander goes and the empty line says so.
    let active_identity: Signal<Option<Vec<Color>>> = use_signal(|| None);
    let identities = use_memo(move || {
        let held = entries();
        let chosen = active_identity();
        colors::identities_present(held.iter().map(suggestion_colors).chain(chosen.as_deref()))
    });
    let filtered = use_memo(move || {
        let of_one = of_identity(&entries(), suggestion_colors, active_identity().as_deref());
        passing(of_one, &filter())
    });
    let pile = use_memo(move || SuggestedPile {
        suggestions: filtered(),
        deck_count: deck_count(),
        identity: active_identity(),
    });

    // Drops a row; the next held suggestion of that identity moves up. The
    // server is asked again only when the filtered list runs under ten or a
    // load from before the drop is still out.
    let mut drop_row = move |oracle_id: Uuid| {
        entries
            .write()
            .retain(|s| s.card.scryfall_data.oracle_id != Some(oracle_id));
        let remaining = of_identity(
            &entries.peek(),
            suggestion_colors,
            active_identity.peek().as_deref(),
        )
        .len();
        if remaining < LIST_SIZE || *load_out.peek() {
            refetch();
        }
    };

    // The create gate the deck list's Create shares: an unverified account
    // has one deck at most.
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
    let at_deck_limit = move || {
        session().is_some_and(|s| {
            s.user.email_verified_at.is_none()
                && deck_profiles.read().as_ref().is_some_and(|p| !p.is_empty())
        })
    };
    let create_with = move |card: Card| {
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
    };

    let on_maybe = move |oracle_id: Uuid| {
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
                    drop_row(oracle_id);
                }
                toast.success(
                    "Added to your maybeboard".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    };

    let on_dismiss = move |oracle_id: Uuid| {
        spawn_write(async move {
            if authed
                .run("dismiss_commander", |c, s| async move {
                    c.dismiss_commander(oracle_id, &s).await
                })
                .await
                .is_some()
            {
                if mounted.get() {
                    drop_row(oracle_id);
                }
                toast.info(
                    "Skipped for a week".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    };

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
                    refetch();
                }
            }
        });
    });

    // A suggested commander's printing carries into Create deck.
    let picks = overlays
        .printing
        .read()
        .as_ref()
        .is_some_and(|card| model::picks_printing(&entries.read(), card));
    let has_entries = !entries().is_empty();
    let rows = shown(&filtered());
    let show_chips = rsx! {
        ShowIdentityChips { active_identity, identities: identities(),
            Chip {
                selected: show_row_art(),
                onclick: move |_| show_row_art.toggle(),
                "Art"
            }
        }
    };

    rsx! {
        div {
            class: "web-commanders-list web-commanders-suggested",
            "data-cursor-list": "",
            onmousemove: move |e| hover_stack.track(e.client_coordinates().x),
            if is_loading() {
                div { class: "flex-col web-suggested",
                    GhostShowRow {}
                    div { class: "card-group",
                        div { class: "card-group-header", GhostText { chars: 9 } }
                        for i in 0..3 {
                            div { key: "{i}", class: "skeleton-card-row web-ghost-card-row",
                                div { class: "skeleton-bar skeleton-card-thumb" }
                                div { class: "skeleton-bar skeleton-card-bar-row" }
                            }
                        }
                    }
                }
            } else {
                div { class: "flex-col web-suggested",
                    // The Show row sits over the list on a wide window and inside its
                    // panel below the breakpoint, as on Playstyle.
                    if wide() {
                        div { class: "deck-controls", {show_chips.clone()} }
                    }
                    // Swipe sits in the list's header, so it says what gets swiped.
                    CardListFold {
                        title: LIST_TITLE,
                        actions: rsx! {
                            Button {
                                variant: ButtonVariant::Action,
                                onclick: move |_| swipe_open.set(true),
                                "Swipe"
                            }
                            InfoButton { topic: HintTopic::SuggestionList }
                        },
                        if !wide() {
                            div { class: "web-card-group-filters", {show_chips} }
                        }
                        if load_failed() {
                            div { class: "card-row card-row-note", "Couldn't load your suggestions" }
                        } else if deck_count() == 0 {
                            div { class: "card-row card-row-note", "Build a deck to see suggestions" }
                        } else if !has_entries {
                            div { class: "card-row card-row-note", "No suggestions right now" }
                        } else if rows.is_empty() {
                            div { class: "card-row card-row-note",
                                {empty_identity_line(&active_identity().unwrap_or_default())}
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
                    DangerZoneAction {
                        label: "Clear skips",
                        button: "Clear",
                        onclick: move |()| clear_open.set(true),
                    }
                }
            }
        }
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
                create_with(card);
            },
            on_close: move |()| swipe_open.set(false),
        }
        HoverStackHost { stack: hover_stack }
        CardOverlaysHost {
            overlays,
            on_printing_save: picks.then_some(EventHandler::new(move |picked: Card| {
                let target = picked.scryfall_data.oracle_id;
                if target.is_some() {
                    let mut expanded = expanded_card;
                    expanded.set(Some(picked.scryfall_data.id));
                    if let Some(slot) = entries
                        .write()
                        .iter_mut()
                        .find(|s| s.card.scryfall_data.oracle_id == target)
                    {
                        slot.card = picked;
                    }
                }
            })),
        }
        OtagExamplesHost { examples }
    }
}
