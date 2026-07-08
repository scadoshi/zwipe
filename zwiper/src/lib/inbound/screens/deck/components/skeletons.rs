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
    rsx! {
        div { class: "skeleton-card-list",
            div { class: "skeleton-card-group",
                div { class: "skeleton-card-group-header",
                    div { class: "skeleton-bar skeleton-card-bar-header" }
                }
                div { class: "skeleton-card-row",
                    div { class: "skeleton-bar skeleton-card-bar-row" }
                }
            }
            div { class: "skeleton-card-group",
                div { class: "skeleton-card-group-header",
                    div { class: "skeleton-bar skeleton-card-bar-header skeleton-card-bar-header-wide" }
                }
                for i in 0..6 {
                    div { key: "{i}", class: "skeleton-card-row",
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
        div { class: "skeleton-deck-list",
            for (i , chips) in TILES.iter().enumerate() {
                div { key: "{i}", class: "skeleton-deck-list-item",
                    div { class: "skeleton-deck-list-head",
                        div { class: "skeleton-bar skeleton-deck-list-title" }
                        div { class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                    }
                    div { class: "skeleton-deck-list-tags",
                        for (j , size) in chips.iter().enumerate() {
                            div { key: "{j}", class: "skeleton-bar skeleton-chip skeleton-chip-{size}" }
                        }
                    }
                }
            }
        }
    }
}
