//! The deck view: the Profile card with Edit, Clone and Share, the Budget,
//! Tags, stats and Warnings sections, the Data and Danger zone panels at
//! the foot, and Cards floating in the dock.
//!
//! On a wide window the Profile card is the hero at the top of zite's page;
//! under it Budget, Tags and the stats share the left column, and Warnings
//! over the cards column for reading share the right, with Data and Danger
//! zone in a row under both. Every section is open at once. Below the
//! breakpoint it is the app's accordion with `Budget` open and no cards
//! column. A deck the server no longer has leaves for Decks, as the app's
//! model does with `Action::Gone`.

pub mod cards;
pub mod dialogs;
pub mod panels;
pub mod sections;

use components::{Button, ButtonVariant, TOAST_NORMAL, TOAST_QUICK};
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
use zwipe_ui::screens::deck_view::CARDS;

use self::cards::CardsColumn;
use self::dialogs::{DeckDialogsHost, use_deck_dialogs};
use self::panels::{DangerZonePanel, DataPanel};
use self::sections::{
    BudgetSection, ProfileCard, StatsSections, TagsSection, WarningFixes, WarningsSection,
};
use super::skeletons::{DeckCardsSkeleton, DeckProfileSkeleton, DeckStatsSkeleton};
use crate::domain::deck::{has_budget, metrics_possible, tag_count};
use crate::inbound::cards::card_row::{CardOverlaysHost, use_card_overlays};
use crate::inbound::cards::otag::{OtagExamplesHost, use_otag_reveals};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::router::Route;
use crate::inbound::shell::{HeadPlace, PageHead, ScreenFrame};

/// The screen's title, in the tab, the folded nav and the page head.
const TITLE: &str = "Deck";
/// The page head's line under the title.
const LINE: &str = "The deck's profile, stats, warnings and cards";

/// The deck view for `deck_id`.
#[component]
pub fn ViewDeck(deck_id: Uuid) -> Element {
    let navigator = use_navigator();
    let client: Signal<ZwipeClient> = use_context();
    let session: Signal<Option<Session>> = use_context();
    let authed = use_authed();
    let toast = use_toast();
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

    let mut profile: Resource<Result<DeckProfile, ClientError>> =
        use_resource(use_reactive!(|deck_id| async move {
            authed
                .try_run("load_deck", |c, s| async move {
                    c.fetch_deck_profile(deck_id, &s).await
                })
                .await
        }));
    let mut deck: Resource<Result<Deck, ClientError>> =
        use_resource(use_reactive!(|deck_id| async move {
            authed
                .try_run("load_deck", |c, s| async move {
                    c.fetch_deck(deck_id, &s).await
                })
                .await
        }));

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
                    }
                    Err(e) => {
                        tracing::warn!("remove_flagged_card failed: {e}");
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

    rsx! {
        ScreenFrame {
            title: TITLE,
            head_place: HeadPlace::Content,
            on_hint,
            div { class: "screen-content",
                match loaded_profile {
                    Some(Ok(p)) => {
                        let cards = metrics
                            .as_ref()
                            .map_or_else(|| usize::try_from(p.card_count).unwrap_or(0), |m| m.total_cards);
                        rsx! {
                            // The head beside the profile on a wide window,
                            // then the stats under the head and the cards under
                            // the profile.
                            div { class: "content-enter web-deck-view web-page web-sections-open",
                                div { class: "web-columns",
                                    div { class: "web-column",
                                        PageHead { title: TITLE, line: LINE, on_hint }
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
                                            DeckStatsSkeleton {}
                                        } else {
                                            StatsSections { metrics: metrics.clone(), land_target: p.land_target, open_section }
                                        }
                                    }
                                    div { class: "web-column",
                                        div { class: "web-deck-profile",
                                            ProfileCard { profile: p, commander: commander(), cards, dialogs }
                                        }
                                        div { class: "web-deck-view-cards",
                                            if let Some(loaded) = loaded_deck {
                                                CardsColumn { key: "{deck_id}", deck: loaded }
                                            } else if deck_loading {
                                                DeckCardsSkeleton {}
                                            }
                                        }
                                    }
                                }
                                // The warnings and the less used actions at the
                                // foot of the page, Danger zone last; the content's
                                // trailing room keeps the floating Cards off Delete.
                                div { class: "web-columns",
                                    div { class: "web-column",
                                        if !warnings.is_empty() {
                                            WarningsSection { deck_id, warnings, fixes, open_section }
                                        }
                                        DataPanel { deck_id, has_cards: metrics.is_some() }
                                    }
                                    div { class: "web-column",
                                        DangerZonePanel { dialogs }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(())) => rsx! {
                        p { class: "text-muted", "Could not load deck" }
                    },
                    None => rsx! {
                        div { class: "content-enter web-deck-view web-page",
                            div { class: "web-columns",
                                div { class: "web-column",
                                    PageHead { title: TITLE, line: LINE, on_hint }
                                    DeckStatsSkeleton {}
                                }
                                div { class: "web-column",
                                    div { class: "web-deck-profile",
                                        DeckProfileSkeleton {}
                                    }
                                    div { class: "web-deck-view-cards",
                                        DeckCardsSkeleton {}
                                    }
                                }
                            }
                        }
                    },
                }
            }
            // Cards, the screen's main action, floats at the bottom right.
            if shows_cards {
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
        DeckDialogsHost { deck_id, dialogs, share_token, clone_name }
        OneTimeHintDialog { hint: OneTimeHint::FirstDeck, open: hint_open }
        CardOverlaysHost { overlays, on_printing_save: move |_: Card| {}, read_only: true }
        OtagExamplesHost { examples }
    }
}
