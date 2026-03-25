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
use uuid::Uuid;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::Card,
        deck::models::{
            deck::{copy_max::CopyMax, deck_profile::DeckProfile},
            deck_metrics::ComputeMetrics,
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
    let mut load_error = use_signal(|| None::<String>);

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
    let deck_resource: Resource<Result<Vec<Card>, ApiError>> = use_resource(move || async move {
        session.upkeep(client);
        let Some(session) = session() else {
            return Err(ApiError::Unauthorized("session expired".to_string()));
        };
        client()
            .get_deck(deck_id, &session)
            .await
            .map(|d| d.entries.into_iter().map(|e| e.card).collect::<Vec<_>>())
    });
    use_effect(move || match commander_resource() {
        Some(Ok(Some(original_commander))) => {
            commander.set(Some(original_commander));
        }
        Some(Err(e)) => {
            load_error.set(Some(e.to_string()));
        }
        Some(Ok(None)) | None => (),
    });

    let mut show_delete_dialog = use_signal(|| false);
    let mut delete_error = use_signal(|| None::<String>);
    let mut attempt_delete = move || {
        session.upkeep(client);
        let Some(session) = session() else {
            delete_error.set(Some("session expired".to_string()));
            return;
        };

        spawn(async move {
            match client().delete_deck(deck_id, &session).await {
                Ok(_) => {
                    navigator.push(Router::DeckList {});
                }
                Err(e) => {
                    delete_error.set(Some(e.to_string()));
                }
            }
        });
    };

    // pre-compute metrics and chart data before rsx!
    let metrics = deck_resource()
        .and_then(|r| r.ok())
        .filter(|c| !c.is_empty())
        .map(|c| c.compute_metrics());

    let bar_heights: Option<[usize; 7]> = metrics.as_ref().map(|m| {
        const MAX_HEIGHT: usize = 8;
        let max_count = m.cmc_histogram.iter().copied().max().unwrap_or(0);
        std::array::from_fn(|i| {
            let opt = m.cmc_histogram.get(i);
            if opt.is_some_and(|c| *c != 0 && max_count != 0)
                && let Some(c) = opt
            {
                (c * MAX_HEIGHT / max_count).max(1)
            } else {
                0
            }
        })
    });

    let type_bar_heights: Option<Vec<(&str, usize, usize)>> = metrics.as_ref().map(|m| {
        const MAX_HEIGHT: usize = 8;
        let max_count = m.type_counts.iter().map(|(_, c)| *c).max().unwrap_or(0);
        m.type_counts
            .iter()
            .map(|(label, count)| {
                let height = if *count > 0 && max_count > 0 {
                    (count * MAX_HEIGHT / max_count).max(1)
                } else {
                    0
                };
                (abbreviate_type(label), *count, height)
            })
            .collect()
    });

    let color_bar_heights: Option<Vec<(&str, usize, usize)>> = metrics.as_ref().map(|m| {
        const MAX_HEIGHT: usize = 8;
        let max_count = m.color_counts.iter().map(|(_, c)| *c).max().unwrap_or(0);
        m.color_counts
            .iter()
            .map(|(label, count)| {
                let height = if *count > 0 && max_count > 0 {
                    (count * MAX_HEIGHT / max_count).max(1)
                } else {
                    0
                };
                (abbreviate_color(label), *count, height)
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
                            div { style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
                                if let Some(error) = load_error() {
                                    div { class: "message-error", "{error}" }
                                }

                                div { class: "flex items-center flex-between mb-2 gap-2",
                                    div { class: "flex-1",
                                        label { class: "label", "deck name" }
                                        p { class: "text-base font-light mb-1", "{deck_profile.name}" }
                                    }
                                }

                                div { class: "flex items-center flex-between mb-2 gap-2",
                                    div { class: "flex-1",
                                        label { class: "label", "copy rule" }
                                        p { class: "text-base font-light mb-1",
                                            if deck_profile.copy_max == Some(CopyMax::standard()) {
                                                "standard"
                                            } else if deck_profile.copy_max == Some(CopyMax::singleton()) {
                                                "singleton"
                                            } else {
                                                "none"
                                            }
                                        }
                                    }
                                }

                                div { class: "flex items-center flex-between mb-2 gap-2",
                                    div { class: "flex-1",
                                        label { class: "label", "commander" }
                                        p { class: "text-base font-light mb-1",
                                            if let Some(cmd) = commander() {
                                                { cmd.scryfall_data.name.to_lowercase() }
                                            } else {
                                                "none"
                                            }
                                        }
                                    }
                                }

                                if let (Some(m), Some(heights)) = (metrics.as_ref(), bar_heights.as_ref()) {
                                    // ── stats ──────────────────────────────────────
                                    label { class: "label", "stats" }
                                    div { class: "flex items-center flex-between mb-1",
                                        span { class: "text-sm font-light", "cards" }
                                        span { class: "text-sm font-light opacity-50", "{m.total_cards}" }
                                    }
                                    div { class: "flex items-center flex-between mb-1",
                                        span { class: "text-sm font-light", "avg cmc" }
                                        span { class: "text-sm font-light opacity-50", "{m.avg_cmc:.1}" }
                                    }
                                    div { class: "flex items-center flex-between mb-2",
                                        span { class: "text-sm font-light", "lands" }
                                        span { class: "text-sm font-light opacity-50", "{m.land_count}" }
                                    }

                                    // ── mana curve ─────────────────────────────────
                                    label { class: "label", "mana curve" }
                                    div { style: "width:100%;border:1px solid rgba(255,255,255,0.1);border-radius:0.5rem;padding:0.75rem;display:flex;flex-direction:column;align-items:center;margin-bottom:0.5rem;",
                                        for row in 0..9usize {
                                            div { class: "flex",
                                                for col in 0..7usize {
                                                    span {
                                                        style: "width:4ch;text-align:center;font-family:monospace;font-size:0.75rem;",
                                                        {
                                                            let h = heights[col];
                                                            let c = m.cmc_histogram.get(col).copied().unwrap_or(0);
                                                            if (8 - row) < h {
                                                                "\u{2588}".to_string()
                                                            } else if (8 - row) == h {
                                                                c.to_string()
                                                            } else {
                                                                "\u{00a0}".to_string()
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        // label row
                                        div { class: "flex",
                                            for (i, label) in ["0","1","2","3","4","5","6+"].iter().enumerate() {
                                                span {
                                                    style: if i > 0 { "width:4ch;text-align:center;font-size:0.75rem;font-family:monospace;opacity:0.5;border-left:1px solid rgba(255,255,255,0.15);" } else { "width:4ch;text-align:center;font-size:0.75rem;font-family:monospace;opacity:0.5;" },
                                                    "{label}"
                                                }
                                            }
                                        }
                                    }

                                    // ── types ──────────────────────────────────────
                                    if let Some(type_bars) = type_bar_heights.as_ref() {
                                        label { class: "label", "types" }
                                        div { style: "width:100%;border:1px solid rgba(255,255,255,0.1);border-radius:0.5rem;padding:0.75rem;display:flex;flex-direction:column;align-items:center;margin-bottom:0.5rem;",
                                            for row in 0..9usize {
                                                div { class: "flex",
                                                    for (_label, count, height) in type_bars.iter() {
                                                        span {
                                                            style: "width:6ch;text-align:center;font-family:monospace;font-size:0.75rem;",
                                                            {
                                                                if (8 - row) < *height {
                                                                    "\u{2588}".to_string()
                                                                } else if (8 - row) == *height {
                                                                    count.to_string()
                                                                } else {
                                                                    "\u{00a0}".to_string()
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            // label row (bottom)
                                            div { class: "flex",
                                                for (i, (label, _count, _height)) in type_bars.iter().enumerate() {
                                                    span {
                                                        style: if i > 0 { "width:6ch;text-align:center;font-size:0.75rem;font-family:monospace;opacity:0.5;border-left:1px solid rgba(255,255,255,0.15);" } else { "width:6ch;text-align:center;font-size:0.75rem;font-family:monospace;opacity:0.5;" },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // ── colors ─────────────────────────────────────
                                    if let Some(color_bars) = color_bar_heights.as_ref() {
                                        label { class: "label", "colors" }
                                        div { style: "width:100%;border:1px solid rgba(255,255,255,0.1);border-radius:0.5rem;padding:0.75rem;display:flex;flex-direction:column;align-items:center;margin-bottom:0.5rem;",
                                            for row in 0..9usize {
                                                div { class: "flex",
                                                    for (_label, count, height) in color_bars.iter() {
                                                        span {
                                                            style: "width:6ch;text-align:center;font-family:monospace;font-size:0.75rem;",
                                                            {
                                                                if (8 - row) < *height {
                                                                    "\u{2588}".to_string()
                                                                } else if (8 - row) == *height {
                                                                    count.to_string()
                                                                } else {
                                                                    "\u{00a0}".to_string()
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            // label row (bottom)
                                            div { class: "flex",
                                                for (i, (label, _count, _height)) in color_bars.iter().enumerate() {
                                                    span {
                                                        style: if i > 0 { "width:6ch;text-align:center;font-size:0.75rem;font-family:monospace;opacity:0.5;border-left:1px solid rgba(255,255,255,0.15);" } else { "width:6ch;text-align:center;font-size:0.75rem;font-family:monospace;opacity:0.5;" },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                if let Some(error) = delete_error() {
                                    div { class: "message-error", "{error}" }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! { div { class: "message-error", "{e}" } },
                        None => rsx! { div { class: "spinner" } }
                    }
                }

            div { class: "util-bar",
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
                if metrics.is_some() {
                    button {
                        class : "util-btn",
                        onclick : move |_| {
                            navigator.push(Router::RemoveDeckCard { deck_id });
                        },
                        "remove"
                    }
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
