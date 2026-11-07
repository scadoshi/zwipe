//! Pieces of a loading skeleton that stand in for the screens' own
//! elements: a line of text, a `?`, a section of an accordion, a fold. Each
//! draws the element's own classes with a ghost bar where the text goes,
//! so it keeps the element's size and place.

use dioxus::prelude::*;

/// A line of text `chars` characters wide, as a bar inside the element
/// that holds the text, so the line keeps its height.
#[component]
pub fn GhostText(chars: u8) -> Element {
    rsx! {
        span { class: "skeleton-bar web-ghost-text", style: "width: {chars}ch;" }
    }
}

/// The `?` beside a title or a column head.
#[component]
pub fn GhostHint() -> Element {
    rsx! {
        span { class: "skeleton-bar skeleton-chip skeleton-info-button web-ghost-box" }
    }
}

/// A [`CollapsibleSection`](super::section::CollapsibleSection): the arrow,
/// a title `chars` wide and, when `hint`, a `?`, over the body, open when
/// `open`.
#[component]
pub fn GhostSection(
    chars: u8,
    open: bool,
    #[props(default = true)] hint: bool,
    children: Element,
) -> Element {
    rsx! {
        div { class: if open { "web-section open" } else { "web-section" },
            div { class: "web-section-head",
                div { class: "web-section-title",
                    span { class: "card-row-arrow web-section-arrow",
                        span { class: "skeleton-bar web-ghost-text web-ghost-arrow" }
                    }
                    span { class: "settings-list-title", GhostText { chars } }
                }
                if hint {
                    div { class: "web-section-accessory", GhostHint {} }
                }
            }
            div { class: if open { "collapsible open" } else { "collapsible" },
                div { class: "collapsible-inner",
                    hr { class: "web-section-rule" }
                    div { class: "web-section-body", {children} }
                }
            }
        }
    }
}

/// An open `DeckFold`: the arrow, a title `chars` wide and, when `hint`, a
/// `?`, over the body.
#[component]
pub fn GhostFold(chars: u8, #[props(default = true)] hint: bool, children: Element) -> Element {
    rsx! {
        div { class: "deck-fold",
            div { class: "deck-fold-head",
                span { class: "card-row-arrow",
                    span { class: "skeleton-bar web-ghost-text web-ghost-arrow" }
                }
                GhostText { chars }
                if hint {
                    div { class: "deck-fold-accessory", GhostHint {} }
                }
            }
            div { class: "deck-fold-body",
                div { class: "deck-fold-inner", {children} }
            }
        }
    }
}

/// A bar chart whose rows lead with a chip (a card role, an oracle tag):
/// `rows` rows of a chip, its bar and its count, in the chart's own grid.
#[component]
pub fn GhostChipBars(rows: usize) -> Element {
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:0.35rem;padding:0 0.75rem;",
            div { style: "display:grid;grid-template-columns:fit-content(50%) minmax(0,1fr) auto;column-gap:0.5rem;align-items:center;",
                for _ in 0..rows {
                    div { style: "padding:0.175rem 0;text-align:right;",
                        span { class: "skeleton-bar web-ghost-keyword-chip" }
                    }
                    div { class: "skeleton-bar chart-track" }
                    span { class: "skeleton-bar web-ghost-count" }
                }
            }
        }
    }
}
