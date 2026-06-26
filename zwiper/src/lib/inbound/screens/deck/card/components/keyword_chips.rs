//! Keyword chips with an inline reminder reveal.
//!
//! Renders a card's keywords as chips. Every keyword has a reminder
//! ([`keyword_reminder`] always returns one), so every chip is tappable:
//! tapping eases its explanation open below the row; tapping it again (or
//! another chip) swaps or closes the reveal.

use dioxus::prelude::*;
use zwipe_core::domain::card::keyword::keyword_reminder;

/// Tappable keyword chips with a shared inline reminder area.
#[component]
pub(crate) fn KeywordChips(keywords: Vec<String>) -> Element {
    let mut open = use_signal(|| None::<usize>);

    let items: Vec<(String, &'static str)> = keywords
        .iter()
        .map(|k| (k.clone(), keyword_reminder(k)))
        .collect();

    let open_idx = open();
    let reveal_text = open_idx.and_then(|i| items.get(i)).map(|(_, r)| *r);
    let reveal_class = if reveal_text.is_some() {
        "keyword-reveal open"
    } else {
        "keyword-reveal"
    };

    rsx! {
        div { class: "keyword-section",
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
