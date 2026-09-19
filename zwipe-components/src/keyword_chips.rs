//! Keyword chips with an inline reminder reveal.
//!
//! Renders a card's keywords as chips. Every keyword has a reminder
//! ([`keyword_reminder`] always returns one), so every chip is tappable:
//! tapping eases its explanation open below the row; tapping it again (or
//! another chip) swaps or closes the reveal.

use dioxus::prelude::*;
use std::collections::HashMap;
use zwipe_core::domain::card::keyword::keyword_reminder;

/// Context: the served keyword-reminder catalog (name → reminder). A host
/// that fetches the live catalog provides it so definition fixes land without
/// an app update; where it's absent (or a name is missing), chips fall back
/// to the compiled-in table.
#[derive(Clone, Copy)]
pub struct KeywordReminders(pub Signal<Option<HashMap<String, String>>>);

/// Tappable keyword chips with a shared inline reminder area.
#[component]
pub fn KeywordChips(keywords: Vec<String>) -> Element {
    if keywords.is_empty() {
        return rsx! {};
    }
    let mut open = use_signal(|| None::<usize>);
    // The revealed content follows `shown`, which holds the last-opened index and
    // is NOT cleared on close, so the content stays mounted while the container
    // animates collapsing. Clearing it (like `open`) would yank the DOM node
    // instantly and the close would snap shut instead of easing out.
    let mut shown = use_signal(|| None::<usize>);

    let served = try_use_context::<KeywordReminders>();
    let items: Vec<(String, String)> = keywords
        .iter()
        .map(|k| {
            // The served map is keyed lowercase (the catalog query normalizes
            // names); the compiled fallback lowercases internally too.
            let reminder = served
                .and_then(|s| {
                    (s.0)()
                        .as_ref()
                        .and_then(|map| map.get(&k.trim().to_ascii_lowercase()).cloned())
                })
                .unwrap_or_else(|| keyword_reminder(k).to_string());
            (k.clone(), reminder)
        })
        .collect();

    let open_idx = open();
    let reveal_text = shown().and_then(|i| items.get(i)).map(|(_, r)| r.clone());
    let reveal_class = if open_idx.is_some() {
        "keyword-reveal open"
    } else {
        "keyword-reveal"
    };

    rsx! {
        div { class: "keyword-section",
            span { class: "chips-label", "Keywords" }
            div { class: "keyword-chips",
                for (i , (name , _)) in items.iter().enumerate() {
                    button {
                        key: "{i}",
                        class: if open_idx == Some(i) { "keyword-chip active" } else { "keyword-chip" },
                        onclick: move |evt| {
                            evt.stop_propagation();
                            if open() == Some(i) {
                                open.set(None);
                            } else {
                                open.set(Some(i));
                                shown.set(Some(i));
                            }
                        },
                        "{name}"
                    }
                }
            }
            div { class: "{reveal_class}",
                div { class: "keyword-reveal-inner",
                    if let Some(text) = reveal_text {
                        p { class: "keyword-reveal-text", "{text}" }
                    }
                }
            }
        }
    }
}
