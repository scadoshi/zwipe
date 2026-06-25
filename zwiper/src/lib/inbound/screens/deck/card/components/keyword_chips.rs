//! Keyword chips with an inline reminder reveal.
//!
//! Renders a card's keyword abilities as chips. Tapping a chip that has a
//! known reminder ([`keyword_reminder`]) eases its explanation open below the
//! row; tapping it again (or another chip) swaps or closes the reveal. Chips
//! without a reminder render as plain, non-interactive labels.

use dioxus::prelude::*;
use zwipe_core::domain::card::keyword::keyword_reminder;

/// Tappable keyword chips with a shared inline reminder area.
#[component]
pub(crate) fn KeywordChips(keywords: Vec<String>) -> Element {
    let mut open = use_signal(|| None::<usize>);

    // Show every keyword as a chip. Ones we have a reminder for are tappable to
    // reveal it; the rest render as plain labels (still a real keyword, the
    // card's own text explains what it does).
    let items: Vec<(String, Option<&'static str>)> = keywords
        .iter()
        .map(|k| (k.clone(), keyword_reminder(k)))
        .collect();

    let open_idx = open();
    let reveal_text = open_idx.and_then(|i| items.get(i)).and_then(|(_, r)| *r);
    let reveal_class = if reveal_text.is_some() {
        "keyword-reveal open"
    } else {
        "keyword-reveal"
    };

    rsx! {
        div { class: "keyword-section",
            div { class: "keyword-chips",
                for (i , (name , reminder)) in items.iter().enumerate() {
                    if reminder.is_some() {
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
                    } else {
                        span { key: "{i}", class: "detail-chip", "{name}" }
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
