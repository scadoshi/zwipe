//! The deck screens' loading ghosts, drawn with the app's skeleton classes
//! so each keeps the app's shapes. Skeletons hold no real text or controls.

use dioxus::prelude::*;

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

/// The deck list while it loads: the `All` group with three ghost rows of
/// mixed widths, in the live list's own classes.
#[component]
pub fn DeckListSkeleton() -> Element {
    const TILES: &[&[&str]] = &[
        &["md", "lg", "xl", "sm", "md", "sm"],
        &["md", "xl", "sm"],
        &["lg", "md", "sm", "md"],
    ];
    rsx! {
        div { class: "card-group web-deck-group",
            div { class: "card-group-header group-collapsible expanded skeleton-deck-list-header",
                div { class: "skeleton-bar skeleton-deck-list-header-title" }
            }
            div { class: "collapsible open",
                div { class: "collapsible-inner",
                    for (i, chips) in TILES.iter().enumerate() {
                        div { key: "{i}", class: "card-row",
                            div { class: "deck-list-row",
                                div { class: "skeleton-bar skeleton-deck-list-art" }
                                div { class: "skeleton-bar skeleton-deck-list-title" }
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
