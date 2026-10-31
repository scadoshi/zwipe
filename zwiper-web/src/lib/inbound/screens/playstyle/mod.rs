//! Playstyle (the app's suggested commanders): what the user builds, read
//! from every deck they own, over their top suggested commanders and their
//! staples.
//!
//! On a wide window the profile's Summary is the hero at the top of zite's
//! page; under it the other profile sections sit open in the left column
//! and the Show row, the suggestions and Staples in the right. Below the
//! breakpoint it is the app's single column with the profile as one
//! accordion, Color map open.

mod chips;
mod profile;
mod rows;

use components::{
    ActionBar, AlertDialog, Button, ButtonVariant, Chip, DialogAction, TOAST_LONG, TOAST_QUICK,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::{self, Color};
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::http::contracts::suggestions::{
    HttpBuilderProfile, HttpCommanderSuggestion, HttpStaple,
};
use zwipe_ui::generation::Generation;
use zwipe_ui::playstyle::{HELD_SIZE, LIST_SIZE, empty_identity_line, of_identity, shown};

pub use self::chips::{IdentityPips, ShowIdentityChips};
use self::profile::{
    BuilderProfileSections, BuilderProfileSkeleton, ProfileSummary, ShowChipRowSkeleton,
};
pub use self::rows::SuggestionReasons;
use self::rows::{StaplesPanel, SuggestionRow};
use crate::inbound::caches::CreateDeckSeed;
use crate::inbound::cards::card_row::{CardOverlaysHost, ShowRowArt, use_card_overlays};
use crate::inbound::cards::otag::{OtagExamplesHost, use_otag_reveals};
use crate::inbound::cards::swipe_select::{SuggestedPile, SwipeMode, SwipeSelect};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::chrome::more_menu::{MoreButton, MoreMenu, use_more_menu};
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::router::Route;
use crate::inbound::screens::deck::skeletons::SkeletonInfoList;
use crate::inbound::shell::ScreenFrame;

/// The suggestions list's title.
const LIST_TITLE: &str = "Suggested commanders";

/// A suggestion's colors, the identity the Show chips read.
fn suggestion_colors(suggestion: &HttpCommanderSuggestion) -> &[Color] {
    &suggestion.card.scryfall_data.color_identity
}

/// The Playstyle screen.
#[component]
pub fn Playstyle() -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let toast = use_toast();
    let authed = use_authed();
    let seed: CreateDeckSeed = use_context();
    let hint_open = use_one_time_hint(OneTimeHint::SuggestedCommanders);
    let mut show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));
    let examples = use_otag_reveals();
    let overlays = use_card_overlays();
    let more = use_more_menu();
    let mut more_open = more.open;

    let expanded_card: Signal<Option<Uuid>> = use_signal(|| None);
    let mut swipe_open = use_signal(|| false);
    let mut clear_open = use_signal(|| false);

    // The held ranking, 100 deep. `reload` refetches: when the Swipe overlay
    // closes, after a drop leaves the filtered list short, and after Clear
    // skips. A refetch with rows showing keeps them (no skeleton). Each load
    // takes a ticket and only the newest one's reply lands; a dropped row
    // restarts a load still out, since its reply predates the drop.
    let mut entries: Signal<Vec<HttpCommanderSuggestion>> = use_signal(Vec::new);
    let mut deck_count = use_signal(|| 0u32);
    let mut profile: Signal<Option<HttpBuilderProfile>> = use_signal(|| None);
    // The hero's Summary only shows on a wide window, where it is always open.
    let summary_section: Signal<Option<String>> = use_signal(|| None);
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
                    profile.set(found.profile);
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
    let filtered =
        use_memo(move || of_identity(&entries(), suggestion_colors, active_identity().as_deref()));
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
                "Verify your email to create more than 1 deck".to_string(),
                ToastOptions::default().duration(TOAST_LONG),
            );
        } else {
            let mut slot = seed.0;
            slot.set(Some(card));
            navigator.push(Route::CreateDeck {});
        }
    };

    let on_maybe = move |oracle_id: Uuid| {
        spawn(async move {
            if authed
                .run("add_commander_maybeboard", |c, s| async move {
                    c.add_commander_maybeboard_card(oracle_id, &s).await
                })
                .await
                .is_some()
            {
                drop_row(oracle_id);
                toast.success(
                    "Added to your commander maybeboard".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    };

    let on_dismiss = move |oracle_id: Uuid| {
        spawn(async move {
            if authed
                .run("dismiss_commander", |c, s| async move {
                    c.dismiss_commander(oracle_id, &s).await
                })
                .await
                .is_some()
            {
                drop_row(oracle_id);
                toast.info(
                    "Skipped for a week".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    };

    let clear_dismissals = use_callback(move |()| {
        more_open.set(false);
        spawn(async move {
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
                refetch();
            }
        });
    });

    let cold_start = deck_count() == 0;
    let has_entries = !entries().is_empty();
    let rows = shown(&filtered());
    let staples: Vec<HttpStaple> = profile
        .read()
        .as_ref()
        .map(|p| p.staples.clone())
        .unwrap_or_default();

    rsx! {
        ScreenFrame {
            title: "Playstyle",
            on_hint: move |()| {
                let mut hint_open = hint_open;
                hint_open.set(true);
            },
            div { class: "screen-content",
                if is_loading() {
                    div { class: "web-playstyle web-page",
                        div { class: "web-hero web-hero-narrow web-playstyle-summary-hero",
                            SkeletonInfoList { rows: 3 }
                        }
                        div { class: "web-columns",
                            div { class: "web-playstyle-profile",
                                BuilderProfileSkeleton {}
                            }
                            div { class: "web-playstyle-list",
                                ShowChipRowSkeleton {}
                                div { class: "maybeboard-list card-group row-enter",
                                    div { class: "card-group-header card-group-header-with-hint",
                                        div { class: "skeleton-bar skeleton-card-bar-header-wide" }
                                        div { class: "skeleton-bar skeleton-chip skeleton-info-button" }
                                    }
                                    for i in 0..6 {
                                        div { key: "{i}", class: "skeleton-card-row",
                                            div { class: "skeleton-bar skeleton-card-thumb" }
                                            div { class: "skeleton-bar skeleton-card-bar-row" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if load_failed() {
                    div { class: "message-empty",
                        span { class: "chip-note", "Could not load your suggestions" }
                    }
                } else if cold_start {
                    div { class: "message-empty",
                        span { class: "chip-note", "Build a deck to see your playstyle" }
                    }
                } else {
                    div { class: "web-playstyle web-page content-enter",
                        if let Some(profile) = profile() {
                            div { class: "web-hero web-hero-narrow web-playstyle-summary-hero web-sections-open",
                                ProfileSummary { profile, open_section: summary_section }
                            }
                        }
                        div { class: "web-columns",
                            if let Some(profile) = profile() {
                                div { class: "web-playstyle-profile web-sections-open",
                                    BuilderProfileSections { profile }
                                }
                            }
                            div { class: "web-playstyle-list",
                                ShowIdentityChips { active_identity, identities: identities(),
                                    Chip {
                                        selected: show_row_art(),
                                        onclick: move |_| show_row_art.toggle(),
                                        "Art"
                                    }
                                }
                                if !has_entries {
                                    div { class: "message-empty",
                                        span { class: "chip-note", "No suggestions right now" }
                                    }
                                } else if rows.is_empty() {
                                    div { class: "message-empty",
                                        span { class: "chip-note",
                                            {empty_identity_line(&active_identity().unwrap_or_default())}
                                        }
                                    }
                                } else {
                                    div { class: "maybeboard-list card-group row-enter",
                                        div { class: "card-group-header card-group-header-with-hint",
                                            span { "{LIST_TITLE}" }
                                            InfoButton { topic: HintTopic::SuggestionList }
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
                                if !staples.is_empty() {
                                    StaplesPanel { staples, deck_count: deck_count(), expanded_card }
                                }
                            }
                        }
                    }
                }
            }
            ActionBar {
                Button { variant: ButtonVariant::Action, onclick: move |_| navigator.go_back(), "Back" }
                if !cold_start {
                    Button { variant: ButtonVariant::Action, onclick: move |_| swipe_open.set(true), "Swipe" }
                }
                MoreButton { menu: more }
            }
        }
        OneTimeHintDialog { hint: OneTimeHint::SuggestedCommanders, open: hint_open }
        MoreMenu { menu: more, title: "More actions",
            Button { danger: true, onclick: move |_| clear_open.set(true), "Clear skips" }
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
        CardOverlaysHost {
            overlays,
            on_printing_save: move |picked: Card| {
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
            },
        }
        OtagExamplesHost { examples }
    }
}
