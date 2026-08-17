use dioxus::prelude::*;

/// One bordered "info-list"-shaped skeleton with `rows` row placeholders inside.
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
            // Mana, Draw odds) start collapsed — header-only boxes.
            SkeletonInfoList { rows: 5 }
            for i in 0..3 {
                div { key: "{i}", class: "skeleton-chart-box",
                    div { class: "skeleton-bar skeleton-section-label" }
                }
            }
        }
    }
}

#[component]
pub(crate) fn DeckCardListSkeleton() -> Element {
    // Grouped rows only: the identity header and featured-cards ghosts render
    // in place in the view, and the quick add + chip rows are static chrome
    // the live screen renders real from the first frame.
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
    // Three varied ghost tiles — different tag counts and widths so the list
    // reads like real mixed decks (the xl chip stands in for a commander name).
    const TILES: &[&[&str]] = &[
        &["md", "lg", "xl", "sm", "md", "sm"],
        &["md", "xl", "sm"],
        &["lg", "md", "sm", "md"],
    ];
    rsx! {
        // The same DOM the loaded list renders: the "All" group container, its
        // collapsible header, and deck rows inside — built from the live
        // classes (`card-group`, `card-group-header`, `collapsible`,
        // `card-row`, `deck-list-row`) rather than skeleton lookalikes, so the
        // two can't drift apart. Only the per-deck content is ghosted; the
        // chip rows above and this header are chrome the live screen also
        // renders real from the first frame.
        div { class: "card-group",
            div { class: "card-group-header group-collapsible expanded skeleton-deck-list-header",
                span { class: "card-row-arrow", "▸" }
                "All"
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
