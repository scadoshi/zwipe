use dioxus::prelude::*;

/// One bordered "settings-list"-shaped skeleton with `rows` row placeholders inside.
#[component]
fn SkeletonInfoList(rows: usize) -> Element {
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

#[component]
pub(crate) fn DeckStatsSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-stats",
            // Stats opens expanded; the sections below it (Distributions,
            // Mana, Draw odds) start collapsed: header-only boxes.
            SkeletonInfoList { rows: 5 }
            for i in 0..3 {
                div { key: "{i}", class: "skeleton-chart-box",
                    div { class: "skeleton-bar skeleton-section-label" }
                }
            }
        }
    }
}

/// The suggested commanders' profile sections while the first load is in
/// flight, in the deck view's skeleton grammar: the Summary section open on
/// its three rows, then the three chart sections collapsed to their headers.
#[component]
pub(crate) fn BuilderProfileSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-stats",
            SkeletonInfoList { rows: 3 }
            for i in 0..3 {
                div { key: "{i}", class: "skeleton-chart-box",
                    div { class: "skeleton-bar skeleton-section-label" }
                }
            }
        }
    }
}

/// A `chip-row` while its screen loads: the label as a bare chip in the
/// label's p1 tint, then one bare chip per entry of `chips`. `label` and each
/// chip name a `skeleton-chip-*` width (`sm`, `md`, `lg`, `xl`) close to the
/// real text's, so the swap to the live row doesn't jump. `style` carries the
/// live row's inline style, when it has one.
#[component]
pub(crate) fn ChipRowSkeleton(
    label: &'static str,
    chips: Vec<&'static str>,
    #[props(default)] style: String,
) -> Element {
    rsx! {
        div { class: "chip-row", style: "{style}",
            div { class: "skeleton-bar skeleton-chip skeleton-chip-{label} stat-chip-p1 skeleton-tinted" }
            for (i , size) in chips.iter().enumerate() {
                div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-{size}" }
            }
        }
    }
}

/// The Quick add row while its screen loads: the label as a tinted bare chip
/// and the compact input as an empty bordered box.
#[component]
pub(crate) fn QuickAddSkeleton() -> Element {
    rsx! {
        div { class: "chip-row",
            div { class: "skeleton-bar skeleton-chip skeleton-chip-lg stat-chip-p1 skeleton-tinted" }
            div { class: "skeleton-input-compact" }
        }
    }
}

/// The suggested commanders' Show row while the first load is in flight, atop
/// the list section's ghost rows and inset like the live row.
#[component]
pub(crate) fn ShowChipRowSkeleton() -> Element {
    rsx! {
        ChipRowSkeleton {
            label: "sm",
            chips: vec!["sm", "sm", "md", "md", "sm"],
            style: "padding:0 0.75rem;margin-bottom:0;",
        }
    }
}

#[component]
pub(crate) fn DeckCardListSkeleton() -> Element {
    // Grouped rows only: the identity header ghosts in place in the view, and
    // the view swaps its quick add and chip rows for ghosts beside this.
    rsx! {
        div { class: "skeleton-card-list",
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

#[component]
pub(crate) fn EditDeckSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-edit-deck",
            // Deck name / Format / Commander: label + input.
            for i in 0..3 {
                div { key: "{i}", class: "skeleton-edit-deck-field",
                    div { class: "skeleton-bar skeleton-edit-deck-label" }
                    div { class: "skeleton-edit-deck-input",
                        div { class: "skeleton-bar skeleton-edit-deck-value" }
                    }
                }
            }
            // Tags: label + bordered box of chips.
            div { class: "skeleton-edit-deck-field",
                div { class: "skeleton-bar skeleton-edit-deck-label" }
                div { class: "skeleton-edit-deck-chipbox",
                    div { class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                    div { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                    div { class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                    div { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                }
            }
            // Power level: label + the five wrapped picker chips.
            div { class: "skeleton-edit-deck-field",
                div { class: "skeleton-bar skeleton-edit-deck-label" }
                div { class: "skeleton-edit-deck-chips",
                    for i in 0..5 {
                        div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-lg" }
                    }
                }
            }
            // Other tags: label + five chips of varying sizes.
            div { class: "skeleton-edit-deck-field",
                div { class: "skeleton-bar skeleton-edit-deck-label" }
                div { class: "skeleton-edit-deck-chips",
                    div { class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                    div { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                    div { class: "skeleton-bar skeleton-chip skeleton-chip-lg" }
                    div { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                    div { class: "skeleton-bar skeleton-chip skeleton-chip-xl" }
                }
            }
            // Land target: label + the -/value/+ stepper row.
            div { class: "skeleton-edit-deck-field",
                div { class: "skeleton-bar skeleton-edit-deck-label" }
                div { class: "skeleton-edit-deck-chips",
                    for i in 0..3 {
                        div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                    }
                }
            }
            // Price target: label + currency chips + the target input.
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
        }
    }
}

#[component]
pub(crate) fn DeckProfileSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-profile",
            SkeletonInfoList { rows: 3 }
        }
    }
}

#[component]
pub(crate) fn DeckListSkeleton() -> Element {
    // Three varied ghost tiles: different tag counts and widths so the list
    // reads like real mixed decks (the xl chip stands in for a commander name).
    const TILES: &[&[&str]] = &[
        &["md", "lg", "xl", "sm", "md", "sm"],
        &["md", "xl", "sm"],
        &["lg", "md", "sm", "md"],
    ];
    rsx! {
        // The same DOM the loaded list renders: the "All" group container, its
        // collapsible header, and deck rows inside, built from the live
        // classes (`card-group`, `card-group-header`, `collapsible`,
        // `card-row`, `deck-list-row`) rather than skeleton lookalikes, so the
        // two can't drift apart. The header's arrow and "All" ghost as one
        // short bar, and the console above ghosts in the list screen.
        div { class: "card-group",
            div { class: "card-group-header group-collapsible expanded skeleton-deck-list-header",
                div { class: "skeleton-bar skeleton-deck-list-header-title" }
            }
            div { class: "collapsible open",
                div { class: "collapsible-inner",
                    for (i , chips) in TILES.iter().enumerate() {
                        div { key: "{i}", class: "card-row",
                            // One wrapping row, like the live one: art, name,
                            // then chips that wrap underneath the art.
                            div { class: "deck-list-row",
                                div { class: "skeleton-bar skeleton-deck-list-art" }
                                div { class: "skeleton-bar skeleton-deck-list-title" }
                                for (j , size) in chips.iter().enumerate() {
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
