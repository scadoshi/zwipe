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

/// A label over a bordered input with a ghost value.
#[component]
fn InputFieldSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-edit-deck-field",
            div { class: "skeleton-bar skeleton-edit-deck-label" }
            div { class: "skeleton-edit-deck-input",
                div { class: "skeleton-bar skeleton-edit-deck-value" }
            }
        }
    }
}

/// A label over a row of bare chips of the given sizes.
#[component]
fn ChipsFieldSkeleton(chips: Vec<&'static str>) -> Element {
    rsx! {
        div { class: "skeleton-edit-deck-field",
            div { class: "skeleton-bar skeleton-edit-deck-label" }
            div { class: "skeleton-edit-deck-chips",
                for (i, size) in chips.iter().enumerate() {
                    div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-{size}" }
                }
            }
        }
    }
}

/// Edit deck while the deck loads, in the form's two columns: name,
/// format, commander and power level; then the land and price targets, the
/// deck tags' chip box and the other tags.
#[component]
pub fn EditDeckSkeleton() -> Element {
    rsx! {
        div { class: "container-sm web-deck-form",
            div { class: "skeleton-edit-deck web-deck-form-column",
                for i in 0..3 {
                    InputFieldSkeleton { key: "{i}" }
                }
                ChipsFieldSkeleton { chips: vec!["lg", "lg", "lg", "lg", "lg"] }
            }
            div { class: "skeleton-edit-deck web-deck-form-column",
                ChipsFieldSkeleton { chips: vec!["sm", "sm", "sm"] }
                div { class: "skeleton-edit-deck-field",
                    div { class: "skeleton-bar skeleton-edit-deck-label" }
                    div { class: "skeleton-edit-deck-chips",
                        for i in 0..3 {
                            div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                        }
                    }
                    div { class: "skeleton-edit-deck-input",
                        div { class: "skeleton-bar skeleton-edit-deck-value" }
                    }
                }
                div { class: "skeleton-edit-deck-field",
                    div { class: "skeleton-bar skeleton-edit-deck-label" }
                    div { class: "skeleton-edit-deck-chipbox",
                        for (i, size) in ["md", "sm", "md", "sm"].iter().enumerate() {
                            div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-{size}" }
                        }
                    }
                }
                ChipsFieldSkeleton { chips: vec!["md", "sm", "lg", "sm", "xl"] }
            }
        }
    }
}

/// A bordered list ghost: its title bar, then `rows` label and value bars.
#[component]
pub fn SkeletonInfoList(rows: usize) -> Element {
    rsx! {
        div { class: "skeleton-info-list",
            div { class: "skeleton-card-header",
                div { class: "skeleton-bar skeleton-section-label" }
            }
            for i in 0..rows {
                div { key: "{i}", class: "skeleton-info-row",
                    div { class: "skeleton-bar skeleton-info-row-label" }
                    div { class: "skeleton-bar skeleton-info-row-value" }
                }
            }
        }
    }
}

/// The deck view's Profile card while the deck loads.
#[component]
pub fn DeckProfileSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-profile",
            SkeletonInfoList { rows: 3 }
        }
    }
}

/// The deck view's stats while the deck loads: one open list, then three
/// sections closed to their headers.
#[component]
pub fn DeckStatsSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-stats",
            SkeletonInfoList { rows: 5 }
            for i in 0..3 {
                div { key: "{i}", class: "skeleton-chart-box",
                    div { class: "skeleton-bar skeleton-section-label" }
                }
            }
        }
    }
}

/// The deck view's cards column while the deck loads: two groups of ghost
/// rows.
#[component]
pub fn DeckCardsSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-card-list web-deck-cards",
            div { class: "skeleton-card-group",
                div { class: "skeleton-card-group-header",
                    div { class: "skeleton-bar skeleton-card-bar-header" }
                }
                div { class: "skeleton-card-row",
                    div { class: "skeleton-bar skeleton-card-thumb" }
                    div { class: "skeleton-bar skeleton-card-bar-row" }
                }
            }
            div { class: "skeleton-card-group",
                div { class: "skeleton-card-group-header",
                    div { class: "skeleton-bar skeleton-card-bar-header skeleton-card-bar-header-wide" }
                }
                for i in 0..6 {
                    div { key: "{i}", class: "skeleton-card-row",
                        div { class: "skeleton-bar skeleton-card-thumb" }
                        div { class: "skeleton-bar skeleton-card-bar-row" }
                    }
                }
            }
        }
    }
}
