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

/// A chip whose label is `chars` wide: the chip's own box with a ghost bar
/// for its label, so it keeps the chip's size. `button` sets the line
/// height of the kit's `Chip`, a button. It does nothing.
#[component]
pub fn GhostChip(chars: u8, #[props(default)] button: bool) -> Element {
    rsx! {
        div {
            class: if button { "chip web-ghost-chip web-ghost-button" } else { "chip web-ghost-chip" },
            GhostText { chars }
        }
    }
}

/// A [`CollapsibleSection`](super::section::CollapsibleSection): the arrow,
/// a title `chars` wide and at the header's end `accessory`, else a `?`
/// when `hint`, over the body, open when `open`.
#[component]
pub fn GhostSection(
    chars: u8,
    open: bool,
    #[props(default = true)] hint: bool,
    #[props(default)] accessory: Option<Element>,
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
                if let Some(accessory) = accessory {
                    div { class: "web-section-accessory", {accessory} }
                } else if hint {
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

/// A `DeckFold`: the arrow, a title `chars` wide and at the header's end
/// `accessory`, else a `?` when `hint`, over the body, shut when `folded`.
#[component]
pub fn GhostFold(
    chars: u8,
    #[props(default = true)] hint: bool,
    #[props(default)] folded: bool,
    #[props(default)] accessory: Option<Element>,
    children: Element,
) -> Element {
    rsx! {
        div { class: if folded { "deck-fold folded" } else { "deck-fold" },
            div { class: "deck-fold-head",
                span { class: "card-row-arrow",
                    span { class: "skeleton-bar web-ghost-text web-ghost-arrow" }
                }
                GhostText { chars }
                // A zero-width space holds the title's text line.
                span { "\u{200b}" }
                if let Some(accessory) = accessory {
                    div { class: "deck-fold-accessory", {accessory} }
                } else if hint {
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

/// A button whose label is `chars` wide, in the button's own box. It does
/// nothing.
#[component]
pub fn GhostButton(chars: u8) -> Element {
    rsx! {
        span { class: "btn-action web-ghost-chip web-ghost-button", GhostText { chars } }
    }
}

/// An icon button, in the button's own box. It does nothing.
#[component]
pub fn GhostIconButton() -> Element {
    rsx! {
        span { class: "btn-action btn-icon web-ghost-chip",
            span { class: "skeleton-bar web-ghost-icon" }
        }
    }
}

/// A list row: its label `label` wide, then a value `value` wide.
#[component]
pub fn GhostInfoRow(label: u8, value: u8) -> Element {
    rsx! {
        div { class: "settings-row",
            span { class: "settings-row-label", GhostText { chars: label } }
            span { class: "info-row-value", GhostText { chars: value } }
        }
    }
}

/// A list row ending in a button: its label `label` wide, the button's
/// `button`.
#[component]
pub fn GhostActionRow(label: u8, button: u8) -> Element {
    rsx! {
        div { class: "settings-row",
            span { class: "settings-row-label", GhostText { chars: label } }
            div { class: "settings-row-value", GhostButton { chars: button } }
        }
    }
}

/// A list panel: its title `chars` wide, `header` at the title's end, over
/// its rows.
#[component]
pub fn GhostList(
    chars: u8,
    #[props(default)] class: Option<String>,
    #[props(default)] header: Option<Element>,
    children: Element,
) -> Element {
    let class = class.map_or_else(
        || "settings-list".to_owned(),
        |c| format!("settings-list {c}"),
    );
    rsx! {
        div { class,
            div { class: "settings-list-header",
                span { class: "settings-list-title", GhostText { chars } }
                {header}
            }
            {children}
        }
    }
}
