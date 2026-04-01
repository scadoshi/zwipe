use crate::inbound::components::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
};
use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        router::Router,
    },
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
    let deck_resource: Resource<Result<(Vec<DeckEntry>, Vec<DeckWarning>), ApiError>> =
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

    // (short_name, consumed, produced, bar_fill_pct, is_surplus)
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
                    (*label, *consumed, *produced, fill_pct, is_surplus)
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
                                label { class: "label", "profile" }
                                div { class: "info-list",
                                    div { class: "info-row",
                                        span { class: "info-row-label", "name" }
                                        span { class: "info-row-value", "{deck_profile.name}" }
                                    }
                                    div { class: "info-row",
                                        span { class: "info-row-label", "format" }
                                        span { class: "info-row-value",
                                            if let Some(fmt) = deck_profile.format {
                                                { fmt.display_name().to_lowercase() }
                                            } else {
                                                "none"
                                            }
                                        }
                                    }
                                    if deck_profile.format.is_some_and(|f| f.has_commander()) {
                                        div { class: "info-row",
                                            span { class: "info-row-label", "commander" }
                                            span { class: "info-row-value",
                                                if let Some(cmd) = commander() {
                                                    { cmd.scryfall_data.name.to_lowercase() }
                                                } else {
                                                    "none"
                                                }
                                            }
                                        }
                                    }
                                }

                                if !warnings.is_empty() {
                                    label { class: "label", "warnings" }
                                    div { class: "info-list",
                                        for warning in warnings.iter() {
                                            div { class: "info-row",
                                                style: "justify-content: flex-start; gap: 0.5rem;",
                                                span { class: "text-muted", "{warning.to_lowercase()}" }
                                            }
                                        }
                                    }
                                }

                                if let (Some(m), Some(mana_curve_bars)) = (metrics.as_ref(), mana_curve_bars.as_ref()) {
                                  div { class: "content-enter",
                                    label { class: "label", "stats" }
                                    div { class: "info-list",
                                        div { class: "info-row",
                                            span { class: "info-row-label", "cards" }
                                            span { class: "info-row-value", "{m.total_cards}" }
                                        }
                                        div { class: "info-row",
                                            span { class: "info-row-label", "avg cmc" }
                                            span { class: "info-row-value", "{m.avg_cmc:.1}" }
                                        }
                                        div { class: "info-row",
                                            span { class: "info-row-label", "lands" }
                                            span { class: "info-row-value", "{m.land_count}" }
                                        }
                                    }

                                    // ── mana curve ─────────────────────────────────
                                    label { class: "label", "mana curve" }
                                    div { style: "width:100%;border:1px solid var(--border-secondary);border-radius:0.5rem;padding:0.75rem;margin-bottom:0.5rem;",
                                        div { style: "display:flex;align-items:flex-end;gap:0.25rem;height:6rem;",
                                            for (count, pct) in mana_curve_bars.iter() {
                                                div { style: "flex:1;display:flex;flex-direction:column;align-items:center;justify-content:flex-end;height:100%;gap:0.15rem;",
                                                    if *count > 0 {
                                                        span { style: "font-size:0.6rem;font-family:monospace;opacity:0.5;line-height:1;", "{count}" }
                                                    }
                                                    div { style: format!("width:100%;height:{pct}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem 0.15rem 0 0;") }
                                                }
                                            }
                                        }
                                        div { style: "display:flex;gap:0.25rem;margin-top:0.35rem;",
                                            for label in ["0","1","2","3","4","5","6+"] {
                                                span { style: "flex:1;text-align:center;font-size:0.65rem;font-family:monospace;opacity:0.5;", "{label}" }
                                            }
                                        }
                                    }

                                    // ── types ──────────────────────────────────────
                                    if let Some(type_bars) = type_bars.as_ref() {
                                        label { class: "label", "basic type distribution" }
                                        div { style: "width:100%;border:1px solid var(--border-secondary);border-radius:0.5rem;padding:0.75rem;margin-bottom:0.5rem;",
                                            div { style: "display:flex;align-items:flex-end;gap:0.25rem;height:6rem;",
                                                for (_label, count, pct) in type_bars.iter() {
                                                    div { style: "flex:1;display:flex;flex-direction:column;align-items:center;justify-content:flex-end;height:100%;gap:0.15rem;",
                                                        if *count > 0 {
                                                            span { style: "font-size:0.6rem;font-family:monospace;opacity:0.5;line-height:1;", "{count}" }
                                                        }
                                                        div { style: format!("width:100%;height:{pct}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem 0.15rem 0 0;") }
                                                    }
                                                }
                                            }
                                            div { style: "display:flex;gap:0.25rem;margin-top:0.35rem;",
                                                for (label, _count, _pct) in type_bars.iter() {
                                                    span { style: "flex:1;text-align:center;font-size:0.65rem;font-family:monospace;opacity:0.5;", "{label}" }
                                                }
                                            }
                                        }
                                    }

                                    // ── colors ─────────────────────────────────────
                                    if let Some(color_bars) = color_bars.as_ref() {
                                        label { class: "label", "color distribution" }
                                        div { style: "width:100%;border:1px solid var(--border-secondary);border-radius:0.5rem;padding:0.75rem;margin-bottom:0.5rem;",
                                            div { style: "display:flex;align-items:flex-end;gap:0.25rem;height:6rem;",
                                                for (_label, count, pct) in color_bars.iter() {
                                                    div { style: "flex:1;display:flex;flex-direction:column;align-items:center;justify-content:flex-end;height:100%;gap:0.15rem;",
                                                        if *count > 0 {
                                                            span { style: "font-size:0.6rem;font-family:monospace;opacity:0.5;line-height:1;", "{count}" }
                                                        }
                                                        div { style: format!("width:100%;height:{pct}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem 0.15rem 0 0;") }
                                                    }
                                                }
                                            }
                                            div { style: "display:flex;gap:0.25rem;margin-top:0.35rem;",
                                                for (label, _count, _pct) in color_bars.iter() {
                                                    span { style: "flex:1;text-align:center;font-size:0.65rem;font-family:monospace;opacity:0.5;", "{label}" }
                                                }
                                            }
                                        }
                                    }

                                    // ── mana balance ───────────────────────────────
                                    if let Some(rows) = mana_balance_rows.as_ref() {
                                        if !rows.is_empty() {
                                            label { class: "label", "mana cost fulfillment" }
                                            div { style: "width:100%;border:1px solid var(--border-secondary);border-radius:0.5rem;padding:0.75rem;margin-bottom:0.5rem;display:flex;flex-direction:column;gap:0.4rem;",
                                                for (color_label, consumed, produced, fill_pct, is_surplus) in rows {
                                                    div { style: "display:flex;align-items:center;gap:0.5rem;",
                                                        // Color initial
                                                        span { style: "width:1ch;font-family:monospace;font-size:0.75rem;opacity:0.8;",
                                                            "{color_label}"
                                                        }
                                                        // Bar track
                                                        div { style: "flex:1;height:1rem;background:var(--border-secondary);border-radius:0.15rem;overflow:hidden;",
                                                            div {
                                                                style: format!(
                                                                    "height:100%;width:{fill_pct}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem;"
                                                                ),
                                                            }
                                                        }
                                                        // Surplus indicator
                                                        span { style: "font-family:monospace;font-size:0.75rem;opacity:0.5;width:1.25rem;text-align:center;flex-shrink:0;",
                                                            if *is_surplus { "✔" } else { "" }
                                                        }
                                                        // Counts
                                                        span { style: "font-family:monospace;font-size:0.75rem;opacity:0.5;white-space:nowrap;width:6ch;text-align:right;flex-shrink:0;",
                                                            "{produced}/{consumed}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
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
                    class : "util-btn",
                    onclick : move |_| {
                        navigator.push(Router::ViewDeckCard { deck_id });
                    },
                    "view"
                }
                button {
                    class : "util-btn",
                    onclick : move |_| {
                        navigator.push(Router::AddDeckCard { deck_id });
                    },
                    "add"
                }
                button {
                    class: "util-btn",
                    disabled: metrics.is_none(),
                    onclick: move |_| {
                        navigator.push(Router::RemoveDeckCard { deck_id });
                    },
                    "remove"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::ImportDeck { deck_id });
                    },
                    "import"
                }
                button {
                    class: "util-btn",
                    disabled: metrics.is_none(),
                    onclick: move |_| {
                        navigator.push(Router::ExportDeck { deck_id });
                    },
                    "export"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| show_delete_dialog.set(true),
                    "delete"
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
            }
        }
    }
}

fn abbreviate_type(label: &str) -> &str {
    match label {
        "lands" => "lands",
        "creatures" => "creat",
        "planeswalkers" => "plnsw",
        "artifacts" => "artif",
        "enchantments" => "enchn",
        "instants" => "instn",
        "sorceries" => "sorcr",
        "other" => "other",
        _ => label,
    }
}

fn abbreviate_color(label: &str) -> &str {
    match label {
        "white" => "white",
        "blue" => "blue",
        "black" => "black",
        "red" => "red",
        "green" => "green",
        "multicolor" => "multi",
        "colorless" => "clrls",
        _ => label,
    }
}
