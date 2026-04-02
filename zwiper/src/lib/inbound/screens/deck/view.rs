use super::components::deck_charts::{abbreviate_color, abbreviate_type, DeckCharts, ManaBalanceRow};
use super::components::deck_profile::DeckProfileSection;
use super::components::deck_stats::DeckStats;
use super::components::deck_warnings::DeckWarnings;
use super::components::more_buttons::MoreButtons;
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
            get_deck_profile::ClientGetDeckProfile,
        },
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use uuid::Uuid;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::Card,
        deck::models::{
            deck::{deck_profile::DeckProfile, deck_warning::DeckWarning, DeckEntry},
            deck_metrics::DeckMetrics,
        },
    },
    inbound::http::ApiError,
};

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

    let deck_profile_resource: Resource<Result<DeckProfile, ApiError>> =
        use_resource(move || async move {
            session.upkeep(client);
            let Some(session) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
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
            session.upkeep(client);
            let Some(session) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };
            client()
                .get_card(original_commander_id, &session)
                .await
                .map(Some)
        });
    let mut deck_resource: Resource<DeckResult> =
        use_resource(move || async move {
            session.upkeep(client);
            let Some(session) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };
            client()
                .get_deck(deck_id, &session)
                .await
                .map(|d| (d.entries, d.warnings))
        });
    use_effect(move || {
        if let Some(Err(e)) = &*deck_profile_resource.read() {
            toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
        }
    });

    use_effect(move || match commander_resource() {
        Some(Ok(Some(original_commander))) => {
            commander.set(Some(original_commander));
        }
        Some(Err(e)) => {
            toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
        }
        Some(Ok(None)) | None => (),
    });

    let show_buy_sheet = use_signal(|| false);
    let mut show_more_sheet = use_signal(|| false);
    let mut show_delete_dialog = use_signal(|| false);
    let attempt_delete = move || {
        session.upkeep(client);
        let Some(session) = session() else {
            toast.error("session expired".to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
            return;
        };

        spawn(async move {
            match client().delete_deck(deck_id, &session).await {
                Ok(_) => {
                    navigator.push(Router::DeckList {});
                }
                Err(e) => {
                    toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
                }
            }
        });
    };

    // pre-compute metrics and chart data before rsx!
    let deck_data = deck_resource().and_then(|r| r.ok());
    let warnings: Vec<DeckWarning> = deck_data.as_ref().map(|(_, w)| w.clone()).unwrap_or_default();
    let metrics = deck_data
        .as_ref()
        .filter(|(entries, _)| !entries.is_empty())
        .map(|(entries, _)| DeckMetrics::from_entries(entries));

    let tcg_url = deck_data
        .as_ref()
        .map(|(entries, _)| buy_links::tcgplayer_url(entries));
    let ck_url = deck_data
        .as_ref()
        .map(|(entries, _)| buy_links::cardkingdom_url(entries));

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
                (abbreviate_type(label), *count, pct)
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
                (abbreviate_color(label), *count, pct)
            })
            .collect()
    });

    let mana_balance_rows = metrics.as_ref().map(|m| -> Vec<_> {
            let labels = ["W", "U", "B", "R", "G"];
            labels
                .iter()
                .zip(m.mana_balance.iter())
                .filter(|(_, (consumed, _produced))| *consumed > 0)
                .map(|(label, (consumed, produced))| {
                    let bar_max = (*consumed).max(*produced);
                    let fill_pct = if bar_max > 0 {
                        ((produced * 100) / bar_max) as u32
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
                    h2 { "deck" }
                }

                div { class: "screen-content",
                    match deck_profile_resource() {
                        Some(Ok(deck_profile)) => rsx! {
                            div { class: "content-enter",
                                  style: "width: calc(100% - 4rem); display: flex; flex-direction: column; gap: 1rem; padding: 1rem 0;",
                                DeckProfileSection {
                                    deck_profile: deck_profile,
                                    commander: commander(),
                                }

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
                                        color_bars: color_bars.clone(),
                                        mana_balance_rows: mana_balance_rows,
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
                                    }
                                }

                            }
                        },
                        Some(Err(_)) => rsx! { p { class: "text-muted", "could not load deck" } },
                        None => rsx! { div { class: "spinner" } }
                    }
                }

            div { class: "util-bar content-enter-delayed",
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::DeckList {});
                    },
                    "back"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::EditDeck { deck_id });
                    },
                    "edit"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::ViewDeckCard { deck_id });
                    },
                    "cards"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| show_more_sheet.set(true),
                    "more"
                }
            }

            AlertDialogRoot {
                open: show_delete_dialog(),
                on_open_change: move |open| show_delete_dialog.set(open),
                AlertDialogContent {
                    AlertDialogTitle { "delete deck" }
                    AlertDialogDescription { "are you sure you want to delete this deck?" }
                    AlertDialogActions {
                        AlertDialogCancel {
                            on_click: move |_| show_delete_dialog.set(false),
                            "cancel"
                        }
                        AlertDialogAction {
                            on_click: move |_| attempt_delete(),
                            "delete"
                        }
                    }
                }
            }

            MoreButtons {
                deck_id: deck_id,
                show_buy_sheet: show_buy_sheet,
                show_more_sheet: show_more_sheet,
                show_delete_dialog: show_delete_dialog,
                has_cards: metrics.is_some(),
                tcg_url: tcg_url,
                ck_url: ck_url,
            }

            }
        }
    }
}
