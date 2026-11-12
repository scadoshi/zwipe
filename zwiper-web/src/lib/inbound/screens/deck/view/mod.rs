//! The deck view. On a wide window it is the deck page both sites share
//! (`page`): its facts and featured cards, the cards as
//! bands, the stats folded under them, and Warnings, Data and Danger zone
//! at the foot. Below the breakpoint it is the app's stack: the Profile
//! card, the accordion with `Budget` open, Data and Danger zone, and Cards
//! floating in the dock. A deck the server no longer has leaves for Decks,
//! as the app's model does with `Action::Gone`.

pub mod cards;
pub mod dialogs;
mod ghost;
mod group_buy;
pub mod page;
pub mod panels;
pub mod sections;
pub(crate) mod stacks;

use std::iter;

use components::{Button, ButtonVariant, LoadFailed, TOAST_NORMAL, TOAST_QUICK, use_back_online};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::{Deck, DeckProfile};
use zwipe_core::http::contracts::deck_card::HttpPatchDeckCard;
use zwipe_ui::buy;
use zwipe_ui::deck::form::Slot;
use zwipe_ui::screens::deck_view::{CARDS, LOAD_FAILED, delete_body};
use zwipe_ui::telemetry::vocabulary::{DeckScreen, Screen, component};

use self::dialogs::{DeckDialogsHost, use_deck_dialogs};
use self::ghost::{DeckViewGhost, StatsSectionsGhost};
use self::page::DeckWidePage;
use self::panels::{DangerZonePanel, DataPanel};
use self::sections::{
    BudgetSection, ProfileCard, StatsSections, TagsSection, WarningFixes, WarningsSection,
};
use crate::domain::deck::{has_budget, metrics_possible, tag_count};
use crate::inbound::cards::card_row::{CardOverlaysHost, use_card_overlays};
use crate::inbound::cards::otag::{OtagExamplesHost, use_otag_reveals};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::media::use_wide;
use crate::inbound::chrome::one_time_hint::{FirstDeckHintDialog, OneTimeHint, use_one_time_hint};
use crate::inbound::router::Route;
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;

/// The screen's title, in the tab and the folded nav.
const TITLE: &str = "Deck";

/// The deck view for `deck_id`, mounted afresh for each deck, so a deck that
/// opens another (a clone) or the browser's back and forward between two
/// decks never shows one deck's state under the other's address.
#[component]
pub fn ViewDeck(deck_id: Uuid) -> Element {
    // A key takes effect only among a list's items, so the body renders as a
    // list of one.
    rsx! {
        for deck_id in iter::once(deck_id) {
            DeckViewScreen { key: "{deck_id}", deck_id }
        }
    }
}

/// The deck view's body for one deck.
#[component]
fn DeckViewScreen(deck_id: Uuid) -> Element {
    telemetry::use_telemetry_screen(Screen::Deck(DeckScreen::View));
    let navigator = use_navigator();
    let wide = use_wide();
    let client: Signal<ZwipeClient> = use_context();
    let session: Signal<Option<Session>> = use_context();
    let authed = use_authed();
    let toast = use_toast();
    let usage = telemetry::use_usage();
    let hint_open = use_one_time_hint(OneTimeHint::FirstDeck);
    let on_hint = use_callback(move |()| {
        let mut hint_open = hint_open;
        hint_open.set(true);
    });
    let dialogs = use_deck_dialogs();
    let examples = use_otag_reveals();
    let overlays = use_card_overlays();
    let open_section: Signal<Option<String>> = use_signal(|| Some("Budget".to_owned()));
    let mut currency = use_signal(|| PriceCurrency::Usd);
    let mut share_token: Signal<Option<Uuid>> = use_signal(|| None);
    let mut seeded_for: Signal<Option<Uuid>> = use_signal(|| None);
    let mut commander: Signal<Option<Card>> = use_signal(|| None);
    // Counts the warning fixes that have landed, so the wide page reloads
    // its own copy of the deck after each.
    let mut fixed = use_signal(|| 0u32);

    let mut profile: Resource<Result<DeckProfile, ClientError>> =
        use_resource(use_reactive!(|deck_id| async move {
            authed
                .try_run("load_deck", |c, s| async move {
                    c.fetch_deck_profile(deck_id, &s).await
                })
                .await
        }));
    // Quiet: the profile fetch beside it raises the one toast a failed
    // visit gets.
    let mut deck: Resource<Result<Deck, ClientError>> =
        use_resource(use_reactive!(|deck_id| async move {
            authed
                .try_run_quiet("load_deck", |c, s| async move {
                    c.fetch_deck(deck_id, &s).await
                })
                .await
        }));

    // Tries a failed load again, from Retry or the connection coming back.
    let retry = use_callback(move |()| {
        if matches!(&*profile.peek(), Some(Err(_))) {
            profile.restart();
        }
        if matches!(&*deck.peek(), Some(Err(_))) {
            deck.restart();
        }
    });
    use_back_online(retry);

    // A deck the server no longer has leaves for Decks, replacing this
    // screen in history so Back never reopens it; the fetch's toast has
    // already said why.
    use_effect(move || {
        let gone = matches!(&*profile.read(), Some(Err(ClientError::NotFound(_))))
            || matches!(&*deck.read(), Some(Err(ClientError::NotFound(_))));
        if gone {
            navigator.replace(Route::DeckList {});
        }
    });

    // Once per deck: the currency chips start on the target's currency, the
    // share link is the saved one, and the commander card is fetched for
    // its name.
    use_effect(move || {
        let Some(Ok(loaded)) = &*profile.read() else {
            return;
        };
        if *seeded_for.peek() == Some(loaded.id) {
            share_token.set(loaded.share_token);
            return;
        }
        seeded_for.set(Some(loaded.id));
        share_token.set(loaded.share_token);
        if let Some(stored) = loaded.price_target_currency {
            currency.set(stored);
        }
        commander.set(None);
        if let Some(id) = loaded.commander_id {
            spawn(async move {
                let http = client.peek().clone();
                match http.fetch_card(id).await {
                    Ok(card) => commander.set(Some(card)),
                    Err(e) => {
                        tracing::warn!("load_commander failed: {e}");
                        toast.error(
                            e.to_user_message(),
                            ToastOptions::default().duration(TOAST_NORMAL),
                        );
                    }
                }
            });
        }
    });

    let fixes = WarningFixes {
        fix_quantity: use_callback(move |(card, quantity): (Uuid, i32)| {
            let request = HttpPatchDeckCard::new(Some(quantity), None);
            spawn(async move {
                if authed
                    .run("update_card_quantity", |c, s| async move {
                        c.update_deck_card(deck_id, card, &request, &s).await
                    })
                    .await
                    .is_some()
                {
                    toast.info(
                        format!("Quantity set to {quantity}"),
                        ToastOptions::default().duration(TOAST_QUICK),
                    );
                    deck.restart();
                    fixed.with_mut(|n| *n = n.wrapping_add(1));
                }
            });
        }),
        clear_commander: use_callback(move |()| {
            let request = Slot::Commander.update_request(None);
            spawn(async move {
                let cleared = authed
                    .run("remove_commander", |c, s| async move {
                        c.update_deck_profile(deck_id, &request, &s).await
                    })
                    .await;
                let Some(cleared) = cleared else {
                    return;
                };
                let label = Slot::Commander.label(cleared.format);
                toast.info(
                    format!("{label} cleared"),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
                commander.set(None);
                profile.restart();
                deck.restart();
                fixed.with_mut(|n| *n = n.wrapping_add(1));
            });
        }),
        // As in the app, straight to the client: a missing session reads
        // `Session expired`.
        remove: use_callback(move |card: Uuid| {
            spawn(async move {
                let current = session.peek().clone();
                let result = match current {
                    Some(current) => {
                        let http = client.peek().clone();
                        http.delete_deck_card(deck_id, card, &current).await
                    }
                    None => Err(ClientError::Unauthorized("Session expired".to_owned())),
                };
                match result {
                    Ok(()) => {
                        toast.info(
                            "Card removed".to_string(),
                            ToastOptions::default().duration(TOAST_QUICK),
                        );
                        deck.restart();
                        fixed.with_mut(|n| *n = n.wrapping_add(1));
                    }
                    Err(e) => {
                        tracing::warn!("remove_flagged_card failed: {e}");
                        usage.peek().report_error(
                            telemetry::screen_name(),
                            component::DECK_WARNINGS,
                            "remove_flagged_card",
                            &telemetry::failure(&e),
                        );
                        toast.error(
                            e.to_user_message(),
                            ToastOptions::default().duration(TOAST_NORMAL),
                        );
                    }
                }
            });
        }),
    };

    let loaded_deck: Option<Deck> = deck.read().as_ref().and_then(|r| r.as_ref().ok()).cloned();
    let deck_loading = deck.read().is_none();
    let metrics = loaded_deck
        .as_ref()
        .filter(|d| !d.entries.is_empty() || !d.command_zone_cards.is_empty())
        .map(|d| DeckMetrics::from_entries_and_command_zone(&d.entries, &d.command_zone_cards));
    let warnings = loaded_deck
        .as_ref()
        .map(|d| d.warnings.clone())
        .unwrap_or_default();
    let buy_links = loaded_deck.as_ref().map(buy::links);
    let loaded_profile: Option<Result<DeckProfile, ()>> = profile
        .read()
        .as_ref()
        .map(|r| r.as_ref().cloned().map_err(|_| ()));
    let shows_cards = loaded_profile.as_ref().is_some_and(Result::is_ok);
    let clone_name = loaded_profile
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .map(|p| format!("{} (clone)", p.name))
        .unwrap_or_default();
    let deck_name = loaded_profile
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .map(|p| p.name.to_string())
        .unwrap_or_default();
    let delete_body = delete_body(
        loaded_profile
            .as_ref()
            .and_then(|r| r.as_ref().ok())
            .map(|p| &*p.name),
    );

    // The deck's name names the page in the tab and the folded nav, once
    // the profile has loaded.
    let deck_title = loaded_profile
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .map_or_else(|| TITLE.to_owned(), |p| p.name.to_string());

    rsx! {
        ScreenFrame {
            title: deck_title,
            on_hint,
            // The wide page's head names the deck once its profile is in.
            titled: wide() && matches!(loaded_profile, Some(Ok(_))),
            div { class: "screen-content",
                match loaded_profile {
                    Some(Err(())) => rsx! {
                        LoadFailed { message: LOAD_FAILED, on_retry: retry }
                    },
                    _ if wide() => rsx! {
                        DeckWidePage { deck_id, currency, fixes, fixed, dialogs }
                    },
                    Some(Ok(p)) => {
                        let cards = metrics
                            .as_ref()
                            .map_or_else(|| usize::try_from(p.card_count).unwrap_or(0), |m| m.total_cards);
                        rsx! {
                            // The app's stack.
                            div { class: "content-enter web-deck-view",
                                ProfileCard { profile: p.clone(), commander: commander(), cards, dialogs }
                                if has_budget(&p, metrics.is_some()) {
                                    BudgetSection {
                                        profile: p.clone(),
                                        metrics: metrics.clone(),
                                        buy: buy_links,
                                        currency,
                                        open_section,
                                    }
                                }
                                if tag_count(&p) > 0 {
                                    TagsSection { profile: p.clone(), open_section }
                                }
                                if metrics.is_none() && deck_loading && metrics_possible(&p) {
                                    StatsSectionsGhost {}
                                } else {
                                    StatsSections { metrics: metrics.clone(), land_target: p.land_target, open_section }
                                }
                                if !warnings.is_empty() {
                                    WarningsSection { deck_id, warnings, fixes, open_section }
                                }
                                DataPanel { deck_id, has_cards: metrics.is_some() }
                                DangerZonePanel { dialogs }
                            }
                        }
                    }
                    None => rsx! {
                        DeckViewGhost {}
                    },
                }
            }
            // Cards, the screen's main action, floats at the bottom right.
            if shows_cards && !wide() {
                div { class: "action-dock action-dock-right content-enter-delayed",
                    Button {
                        variant: ButtonVariant::Action,
                        onclick: move |_| {
                            navigator.push(Route::ViewDeckCard { deck_id });
                        },
                        {CARDS}
                    }
                }
            }
        }
        DeckDialogsHost { deck_id, dialogs, share_token, deck_name, clone_name, delete_body }
        FirstDeckHintDialog { open: hint_open, wide: wide() }
        CardOverlaysHost { overlays }
        OtagExamplesHost { examples }
    }
}
