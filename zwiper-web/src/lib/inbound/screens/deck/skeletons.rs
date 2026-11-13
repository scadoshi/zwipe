//! The deck screens' loading ghosts, drawn with the app's skeleton classes
//! so each keeps the app's shapes. Skeletons hold no real text or controls.

use dioxus::prelude::*;

use crate::inbound::chrome::ghost::{GhostChip, GhostHint, GhostText};

/// A chip row while its screen loads: the label as a tinted bare chip, then
/// bare chips of the given sizes (`sm`, `md`, `lg`, `xl`).
#[component]
pub fn ChipRowSkeleton(label: &'static str, chips: Vec<&'static str>) -> Element {
    rsx! {
        div { class: "chip-row",
            div { class: "skeleton-bar skeleton-chip skeleton-chip-{label} stat-chip-p1 skeleton-tinted" }
            for (i, size) in chips.iter().enumerate() {
                div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-{size}" }
            }
        }
    }
}

/// The Decks list ghost's tiles, each its chips' widths, varied so the ghost reads like mixed
/// decks. Enough to fill a wide list: web.css shows three on a phone, five in one wide column
/// and all ten in two columns from 1400px.
const DECK_LIST_GHOST_TILES: [&[&str]; 10] = [
    &["md", "lg", "xl", "sm", "md", "sm"],
    &["md", "xl", "sm"],
    &["lg", "md", "sm", "md"],
    &["xl", "sm", "md"],
    &["md", "md", "lg", "sm", "sm"],
    &["lg", "xl", "sm"],
    &["sm", "md", "xl", "md"],
    &["md", "lg", "sm"],
    &["xl", "md", "sm", "lg", "sm"],
    &["lg", "sm", "md"],
];

/// The deck list while it loads: the `All` group's header as an empty chip
/// at the loaded one's size, then ten ghost rows of mixed widths, in the live
/// list's own classes; web.css shows as many as the list's width fills.
#[component]
pub fn DeckListSkeleton() -> Element {
    rsx! {
        div { class: "card-group deck-group",
            div { class: "card-group-header group-collapsible expanded skeleton-deck-list-header web-ghost-chip",
                span { class: "card-row-arrow", "\u{25b8}" }
                span { "All (0)" }
            }
            div { class: "collapsible open",
                div { class: "collapsible-inner",
                    for (i, chips) in DECK_LIST_GHOST_TILES.iter().enumerate() {
                        div { key: "{i}", class: "card-row deck-row skeleton-deck-tile",
                            div { class: "deck-list-row",
                                div { class: "skeleton-bar skeleton-deck-list-art" }
                                div { class: "deck-row-text",
                                    div { class: "skeleton-bar skeleton-deck-list-title" }
                                    div { class: "deck-row-chips",
                                        for (j, size) in chips.iter().enumerate() {
                                            div { key: "{j}", class: "skeleton-bar skeleton-chip skeleton-chip-{size}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A form field's label row: its label `chars` wide, its `?`, then
/// `extra` (a count, chips).
#[component]
fn GhostLabelRow(chars: u8, #[props(default)] extra: Option<Element>) -> Element {
    rsx! {
        div { class: "label-row",
            label { class: "label", GhostText { chars } }
            GhostHint {}
            {extra}
        }
    }
}

/// A field's box with a value `chars` wide in it; `input` sets the line
/// height an `input` element draws its text at.
#[component]
fn GhostBox(chars: u8, #[props(default)] input: bool) -> Element {
    rsx! {
        div {
            class: "input-clear-wrap",
            div {
                class: "input input-placeholder",
                style: if input { "line-height: normal;" } else { "" },
                GhostText { chars }
            }
        }
    }
}

/// A panel of the deck form: its title `chars` wide over `children`.
#[component]
fn GhostFormPanel(chars: u8, children: Element) -> Element {
    rsx! {
        div { class: "web-form-panel",
            div { class: "settings-list-header web-form-head",
                span { class: "settings-list-title", GhostText { chars } }
            }
            div { class: "web-form-fields", {children} }
        }
    }
}

/// A centered wrapping row of chips whose labels are the given widths.
#[component]
fn GhostChips(chips: Vec<u8>) -> Element {
    rsx! {
        div { class: "chip-row", style: "flex-wrap: wrap; justify-content: center;",
            for (i, chars) in chips.into_iter().enumerate() {
                GhostChip { key: "{i}", chars }
            }
        }
    }
}

/// Edit deck while the deck loads, in the form's own panels: Profile (the
/// name, format and commander boxes and the power levels) and Budget (the
/// land stepper and the price), then Tags (the two tag boxes and the other
/// tags), on a wide window in its two columns.
#[component]
pub fn EditDeckSkeleton() -> Element {
    rsx! {
        div { class: "container-sm web-deck-form",
            div { class: "flex-col text-center web-deck-form-column",
                GhostFormPanel { chars: 7,
                    div {
                        div { class: "label-hint",
                            span { class: "label-hint-anchor",
                                label { class: "label", GhostText { chars: 9 } }
                                GhostHint {}
                            }
                        }
                        GhostBox { chars: 11, input: true }
                    }
                    div {
                        GhostLabelRow { chars: 6 }
                        GhostBox { chars: 9 }
                    }
                    div { class: "collapsible open",
                        div { class: "collapsible-inner",
                            div {
                                GhostLabelRow {
                                    chars: 9,
                                    extra: rsx! {
                                        div { class: "chip-xs web-ghost-chip", GhostText { chars: 6 } }
                                        div { class: "chip-xs chip-primary web-ghost-chip", GhostText { chars: 5 } }
                                    },
                                }
                                GhostBox { chars: 7, input: true }
                            }
                        }
                    }
                    for i in 0..3 {
                        div { key: "{i}", class: "collapsible",
                            div { class: "collapsible-inner" }
                        }
                    }
                    div { style: "margin-top: 1rem;",
                        GhostLabelRow { chars: 11 }
                        GhostChips { chips: vec![14, 8, 12, 13, 8] }
                    }
                }
                GhostFormPanel { chars: 6,
                    div { style: "margin-top: 1rem;",
                        GhostLabelRow { chars: 11 }
                        div { class: "stepper",
                            span { class: "skeleton-bar skeleton-chip web-ghost-box web-ghost-stepper" }
                            span { class: "stepper-value", style: "width: auto; min-width: 4rem;",
                                GhostText { chars: 7 }
                            }
                            span { class: "skeleton-bar skeleton-chip web-ghost-box web-ghost-stepper" }
                        }
                    }
                    div { style: "margin-top: 1rem;",
                        GhostLabelRow {
                            chars: 12,
                            extra: rsx! {
                                div { class: "label-row-chips",
                                    for i in 0..3 {
                                        GhostChip { key: "{i}", chars: 3 }
                                    }
                                }
                            },
                        }
                        GhostBox { chars: 7, input: true }
                    }
                }
            }
            div { class: "flex-col text-center web-deck-form-column",
                GhostFormPanel { chars: 4,
                    for (i, chars) in [9, 11].into_iter().enumerate() {
                        div { key: "{i}", style: "margin-top: 1rem;",
                            GhostLabelRow { chars, extra: rsx! { span { class: "field-count", GhostText { chars: 4 } } } }
                            div { class: "input input-placeholder", GhostText { chars: 7 } }
                        }
                    }
                    div { style: "margin-top: 1rem;",
                        GhostLabelRow { chars: 10 }
                        GhostChips { chips: vec![6, 4, 4, 6, 15] }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WEB_CSS: &str = include_str!("../../../../../assets/web.css");

    /// The `@media` query that wraps the first rule for `selector` in web.css.
    fn query_for(selector: &str) -> &'static str {
        let at = WEB_CSS
            .find(&format!("    {selector} {{"))
            .unwrap_or_else(|| panic!("no rule for {selector}"));
        let head = WEB_CSS.get(..at).unwrap();
        let query = head.rfind("@media").unwrap();
        let line = head.get(query..).unwrap();
        line.get(..line.find('{').unwrap()).unwrap().trim()
    }

    #[test]
    fn the_decks_ghost_fills_a_wide_list_and_keeps_the_phone_s_three() {
        assert_eq!(
            DECK_LIST_GHOST_TILES.len(),
            10,
            "five rows of two from 1400px"
        );
        assert_eq!(
            query_for(".skeleton-deck-tile:nth-child(n + 4)"),
            "@media (max-width: 899px)",
            "a phone shows three"
        );
        assert!(
            WEB_CSS.contains(
                ".web-scroll-screen:has(.skeleton-deck-tile) > .screen-content {\n    flex: 1 1 0;\n    min-height: 0;\n    overflow: hidden;\n}"
            ),
            "the ghost takes the room above the footer and no more, so the page doesn't scroll while it loads"
        );
        assert_eq!(
            query_for(".skeleton-deck-tile:nth-child(n + 6)"),
            "@media (min-width: 900px) and (max-width: 1399px)",
            "one wide column shows five"
        );
    }
}
