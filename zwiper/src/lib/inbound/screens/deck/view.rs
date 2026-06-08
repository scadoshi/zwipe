use super::components::clone_deck_dialog::CloneDeckDialog;
use super::components::deck_charts::{DeckCharts, ManaBalanceRow};
use super::components::deck_profile::DeckProfileSection;
use super::components::deck_stats::DeckStats;
use super::components::deck_warnings::DeckWarnings;
use super::components::more_buttons::MoreButtons;
use super::components::skeletons::{DeckProfileSkeleton, DeckStatsSkeleton};
use crate::inbound::components::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
};
use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        router::Router,
    },
    outbound::buy_links,
    outbound::client::{
        ZwipeClient,
        card::get_card::ClientGetCard,
        deck::{
            delete_deck::ClientDeleteDeck, get_deck::ClientGetDeck,
            get_deck_profile::ClientGetDeckProfile, update_deck_profile::ClientUpdateDeckProfile,
        },
        deck_card::update_deck_card::ClientUpdateDeckCard,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use uuid::Uuid;
use zwipe::inbound::http::ApiError;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::{
    DeckEntry, deck_metrics::DeckMetrics, deck_profile::DeckProfile, deck_warning::DeckWarning,
};
use zwipe_core::http::contracts::deck::HttpUpdateDeckProfile;
use zwipe_core::http::contracts::deck_card::HttpUpdateDeckCard;
use zwipe_core::http::helpers::Opdate;

type DeckResult = Result<(Vec<DeckEntry>, Vec<DeckWarning>), ApiError>;

#[component]
pub fn ViewDeck(deck_id: Uuid) -> Element {
    // config
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    // original deck information
    let mut commander: Signal<Option<Card>> = use_signal(|| None);
    let toast = use_toast();

    let mut deck_profile_resource: Resource<Result<DeckProfile, ApiError>> =
        use_resource(move || async move {
            session.upkeep(client);
            let Some(session) = session() else {
                return Err(ApiError::Unauthorized("Session expired".to_string()));
            };
            client().get_deck_profile(deck_id, &session).await
        });
    let commander_resource: Resource<Result<Option<Card>, ApiError>> =
        use_resource(move || async move {
            let Some(Ok(DeckProfile {
                commander_id: Some(original_commander_id),
                ..
            })) = deck_profile_resource()
            else {
                return Ok(None);
            };
            client().get_card(original_commander_id).await.map(Some)
        });
    let mut deck_resource: Resource<DeckResult> = use_resource(move || async move {
        session.upkeep(client);
        let Some(session) = session() else {
            return Err(ApiError::Unauthorized("Session expired".to_string()));
        };
        client()
            .get_deck(deck_id, &session)
            .await
            .map(|d| (d.entries, d.warnings))
    });
    use_effect(move || {
        if let Some(Err(e)) = &*deck_profile_resource.read() {
            toast.error(
                e.to_string(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
        }
    });

    use_effect(move || match commander_resource() {
        Some(Ok(Some(original_commander))) => {
            commander.set(Some(original_commander));
        }
        Some(Err(e)) => {
            toast.error(
                e.to_string(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
        }
        Some(Ok(None)) | None => (),
    });

    let show_buy_sheet = use_signal(|| false);
    let mut show_more_sheet = use_signal(|| false);
    let mut show_delete_dialog = use_signal(|| false);
    let show_clone_dialog = use_signal(|| false);
    let attempt_delete = move || {
        session.upkeep(client);
        let Some(session) = session() else {
            toast.error(
                "Session expired".to_string(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
            return;
        };

        spawn(async move {
            match client().delete_deck(deck_id, &session).await {
                Ok(_) => {
                    navigator.push(Router::DeckList {});
                }
                Err(e) => {
                    toast.error(
                        e.to_string(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                }
            }
        });
    };

    // pre-compute metrics and chart data before rsx!
    let deck_data = deck_resource().and_then(|r| r.ok());
    let warnings: Vec<DeckWarning> = deck_data
        .as_ref()
        .map(|(_, w)| w.clone())
        .unwrap_or_default();
    let metrics = deck_data
        .as_ref()
        .filter(|(entries, _)| !entries.is_empty())
        .map(|(entries, _)| {
            let mut m = DeckMetrics::from_entries(entries);

            // Count variant cards (commander, partner, etc.) stored on the profile
            // but not in deck_cards — mirrors check_card_count in validate_deck.
            if let Some(Ok(p)) = deck_profile_resource() {
                let in_entries = |id: Uuid| entries.iter().any(|e| e.card.scryfall_data.id == id);
                let has_commander_format = p.format.as_ref().is_some_and(|f| f.has_commander());
                if has_commander_format && p.commander_id.is_some_and(|id| !in_entries(id)) {
                    m.total_cards += 1;
                }
                if p.partner_commander_id.is_some_and(|id| !in_entries(id)) {
                    m.total_cards += 1;
                }
                if p.background_id.is_some_and(|id| !in_entries(id)) {
                    m.total_cards += 1;
                }
                if p.signature_spell_id.is_some_and(|id| !in_entries(id)) {
                    m.total_cards += 1;
                }
            }

            m
        });

    let command_zone_names: Vec<String> = deck_profile_resource()
        .and_then(|r| r.ok())
        .map(|p| {
            [
                p.commander_name.as_deref(),
                p.partner_commander_name.as_deref(),
                p.background_name.as_deref(),
                p.signature_spell_name.as_deref(),
            ]
            .into_iter()
            .flatten()
            .map(|s| s.to_string())
            .collect()
        })
        .unwrap_or_default();
    let cz_refs: Vec<&str> = command_zone_names.iter().map(|s| s.as_str()).collect();

    let tcg_url = deck_data.as_ref().map(|(entries, _)| {
        let active: Vec<_> = entries
            .iter()
            .filter(|e| e.deck_card.board.is_active() || e.deck_card.board.is_sideboard())
            .cloned()
            .collect();
        buy_links::tcgplayer_url(&active, &cz_refs)
    });
    let ck_url = deck_data.as_ref().map(|(entries, _)| {
        let active: Vec<_> = entries
            .iter()
            .filter(|e| e.deck_card.board.is_active() || e.deck_card.board.is_sideboard())
            .cloned()
            .collect();
        buy_links::cardkingdom_url(&active, &cz_refs)
    });

    let mana_curve_bars: Option<[(usize, u32); 7]> = metrics.as_ref().map(|m| {
        let max_count = m.cmc_histogram.iter().copied().max().unwrap_or(0);
        std::array::from_fn(|i| {
            let count = m.cmc_histogram.get(i).copied().unwrap_or(0);
            let pct = if max_count > 0 && count > 0 {
                ((count * 100) / max_count).max(4) as u32
            } else {
                0
            };
            (count, pct)
        })
    });

    let type_bars: Option<Vec<(&str, usize, u32)>> = metrics.as_ref().map(|m| {
        let max_count = m.type_counts.iter().map(|(_, c)| *c).max().unwrap_or(0);
        m.type_counts
            .iter()
            .map(|(label, count)| {
                let pct = if max_count > 0 && *count > 0 {
                    ((count * 100) / max_count).max(4) as u32
                } else {
                    0
                };
                (DeckMetrics::abbreviate_type(label), *count, pct)
            })
            .collect()
    });

    let color_bars: Option<Vec<(&str, usize, u32)>> = metrics.as_ref().map(|m| {
        let max_count = m.color_counts.iter().map(|(_, c)| *c).max().unwrap_or(0);
        m.color_counts
            .iter()
            .map(|(label, count)| {
                let pct = if max_count > 0 && *count > 0 {
                    ((count * 100) / max_count).max(4) as u32
                } else {
                    0
                };
                (DeckMetrics::abbreviate_color(label), *count, pct)
            })
            .collect()
    });

    let category_bars: Option<Vec<(&str, usize, u32)>> = metrics.as_ref().and_then(|m| {
        if m.mechanical_category_counts.is_empty() {
            return None;
        }
        let max_count = m
            .mechanical_category_counts
            .iter()
            .map(|(_, c)| *c)
            .max()
            .unwrap_or(0);
        Some(
            m.mechanical_category_counts
                .iter()
                .map(|(label, count)| {
                    let pct = if max_count > 0 && *count > 0 {
                        ((count * 100) / max_count).max(4) as u32
                    } else {
                        0
                    };
                    (*label, *count, pct)
                })
                .collect(),
        )
    });

    let mana_balance_rows = metrics.as_ref().map(|m| -> Vec<_> {
        let labels = ["W", "U", "B", "R", "G"];
        labels
            .iter()
            .zip(m.mana_balance.iter())
            .filter(|(_, (consumed, _produced))| *consumed > 0)
            .map(|(label, (consumed, produced))| {
                let bar_max = (*consumed).max(*produced);
                // Floor a nonzero share at 4% so a barely-produced color still
                // shows a visible sliver, matching the curve/type/color bars.
                let fill_pct = if *produced > 0 {
                    (produced * 100).checked_div(bar_max).unwrap_or(0).max(4) as u32
                } else {
                    0
                };
                let is_surplus = produced >= consumed;
                ManaBalanceRow {
                    label,
                    consumed: *consumed,
                    produced: *produced,
                    fill_pct,
                    is_surplus,
                }
            })
            .collect()
    });

    rsx! {
        Bouncer {
            div { class: "screen",
                div {
                    class : "page-header",
                    h2 { "Deck" }
                }

                div { class: "screen-content",
                    match deck_profile_resource() {
                        Some(Ok(deck_profile)) => {
                            let metrics_possible = deck_profile.card_count > 0
                                || deck_profile.commander_id.is_some()
                                || deck_profile.partner_commander_id.is_some()
                                || deck_profile.background_id.is_some()
                                || deck_profile.signature_spell_id.is_some();
                            rsx! {
                            div { class: "content-enter",
                                  style: "width: calc(100% - 4rem); display: flex; flex-direction: column; gap: 1rem; padding: 1rem 0;",
                                DeckProfileSection {
                                    deck_profile: deck_profile,
                                    commander: commander(),
                                }

                                {
                                    rsx! {
                                        if let (Some(m), Some(mana_curve_bars)) = (metrics.as_ref(), mana_curve_bars.as_ref()) {
                                          div { class: "content-enter",
                                                style: "display: flex; flex-direction: column; gap: 1rem;",
                                            DeckStats {
                                                metrics: m.clone(),
                                                show_buy_sheet: show_buy_sheet,
                                            }

                                            DeckCharts {
                                                mana_curve_bars: *mana_curve_bars,
                                                type_bars: type_bars.clone(),
                                                category_bars: category_bars.clone(),
                                                color_bars: color_bars.clone(),
                                                mana_balance_rows: mana_balance_rows,
                                            }
                                          }
                                        } else if metrics_possible {
                                            DeckStatsSkeleton {}
                                        }
                                    }
                                }

                                if !warnings.is_empty() {
                                    DeckWarnings {
                                        warnings: warnings,
                                        deck_id: deck_id,
                                        on_remove: move |_| {
                                            deck_resource.restart();
                                        },
                                        on_fix_quantity: move |(card_id, target_qty): (Uuid, i32)| {
                                            session.upkeep(client);
                                            let Some(session) = session() else { return; };

                                            let current_qty = deck_resource()
                                                .and_then(|r| r.ok())
                                                .and_then(|(entries, _)| {
                                                    entries.iter()
                                                        .find(|e| e.card.scryfall_data.id == card_id)
                                                        .map(|e| *e.deck_card.quantity)
                                                })
                                                .unwrap_or(1);
                                            let delta = target_qty - current_qty;
                                            let request = HttpUpdateDeckCard::new(Some(delta), None);

                                            spawn(async move {
                                                match client().update_deck_card(deck_id, card_id, &request, &session).await {
                                                    Ok(_) => {
                                                        toast.info(
                                                            format!("Quantity set to {target_qty}"),
                                                            ToastOptions::default().duration(Duration::from_millis(1500)),
                                                        );
                                                        deck_resource.restart();
                                                    }
                                                    Err(e) => {
                                                        toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
                                                    }
                                                }
                                            });
                                        },
                                        on_clear_commander: move |_| {
                                            session.upkeep(client);
                                            let Some(session) = session() else { return; };

                                            let request = HttpUpdateDeckProfile::builder()
                                                .commander_id(Opdate::Set(None))
                                                .build();

                                            spawn(async move {
                                                match client().update_deck_profile(deck_id, &request, &session).await {
                                                    Ok(_) => {
                                                        let label = if deck_profile_resource().is_some_and(|r| r.as_ref().ok().is_some_and(|p| p.format.as_ref().is_some_and(|f| f.has_signature_spell()))) {
                                                            "Oathbreaker"
                                                        } else {
                                                            "Commander"
                                                        };
                                                        toast.info(
                                                            format!("{label} cleared"),
                                                            ToastOptions::default().duration(Duration::from_millis(1500)),
                                                        );
                                                        commander.set(None);
                                                        deck_profile_resource.restart();
                                                        deck_resource.restart();
                                                    }
                                                    Err(e) => {
                                                        toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
                                                    }
                                                }
                                            });
                                        },
                                    }
                                }

                            }
                            }
                        },
                        Some(Err(_)) => rsx! { p { class: "text-muted", "Could not load deck" } },
                        None => rsx! {
                            div { class: "content-enter",
                                  style: "width: calc(100% - 4rem); display: flex; flex-direction: column; gap: 1rem; padding: 1rem 0;",
                                DeckProfileSkeleton {}
                                DeckStatsSkeleton {}
                            }
                        }
                    }
                }

            div { class: "util-bar content-enter-delayed",
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::DeckList {});
                    },
                    "Back"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::EditDeck { deck_id });
                    },
                    "Edit"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::ViewDeckCard { deck_id });
                    },
                    "Cards"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| show_more_sheet.set(true),
                    "More"
                }
            }

            AlertDialogRoot {
                open: show_delete_dialog(),
                on_open_change: move |open| show_delete_dialog.set(open),
                AlertDialogContent {
                    AlertDialogTitle { "Delete deck" }
                    AlertDialogDescription { "Are you sure you want to delete this deck?" }
                    AlertDialogActions {
                        AlertDialogCancel {
                            on_click: move |_| show_delete_dialog.set(false),
                            "Cancel"
                        }
                        AlertDialogAction {
                            danger: true,
                            on_click: move |_| attempt_delete(),
                            "Delete"
                        }
                    }
                }
            }

            MoreButtons {
                deck_id: deck_id,
                show_buy_sheet: show_buy_sheet,
                show_more_sheet: show_more_sheet,
                show_delete_dialog: show_delete_dialog,
                show_clone_dialog: show_clone_dialog,
                has_cards: metrics.is_some(),
                tcg_url: tcg_url,
                ck_url: ck_url,
            }

            CloneDeckDialog {
                source_deck_id: deck_id,
                default_name: deck_profile_resource()
                    .and_then(|r| r.ok())
                    .map(|p| format!("{} (clone)", p.name))
                    .unwrap_or_default(),
                open: show_clone_dialog,
            }

            }
        }
    }
}
