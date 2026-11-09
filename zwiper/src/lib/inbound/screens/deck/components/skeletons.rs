//! Loading skeletons for the deck screens.

use components::ColorBarsGhost;
use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::colors::Color;

use super::collapsible_section::{HEADER_STYLE, TITLE_GROUP_STYLE, card_style};

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

/// A [`CollapsibleSection`](super::collapsible_section::CollapsibleSection)
/// while its screen loads: the same card and header, the title a ghost bar
/// `chars` wide inside the title's own text line, the "?" a ghost of the
/// button when `hint`, and `children` under the rule when `open`. Built from
/// the section's own styles, so the swap to the live section stays in place.
#[component]
pub(crate) fn SectionGhost(chars: u8, open: bool, hint: bool, children: Element) -> Element {
    rsx! {
        div { style: card_style("var(--border-secondary)"), "aria-hidden": "true",
            div { style: HEADER_STYLE,
                div { style: TITLE_GROUP_STYLE,
                    span { class: "card-row-arrow ghost-box", "▸" }
                    span { class: "settings-list-title",
                        span { class: "skeleton-bar ghost-text", style: "width: {chars}ch;" }
                    }
                }
                if hint {
                    span { class: "info-button ghost-box", style: if open { "" } else { "visibility:hidden;" }, "?" }
                }
            }
            if open {
                div { class: "collapsible open",
                    div { class: "collapsible-inner",
                        hr { class: "collapsible-section-rule" }
                        div { class: "collapsible-section-body", {children} }
                    }
                }
            }
        }
    }
}

/// The suggested commanders' profile sections while the first load is in
/// flight: Colors open on its ghost bars, then Summary and the two chart
/// sections closed, each the live section's box with its title and "?" as
/// ghosts.
#[component]
pub(crate) fn BuilderProfileSkeleton() -> Element {
    rsx! {
        SectionGhost { chars: 6, open: true, hint: true, ColorBarsGhost {} }
        SectionGhost { chars: 7, open: false, hint: false }
        SectionGhost { chars: 10, open: false, hint: true }
        SectionGhost { chars: 11, open: false, hint: true }
    }
}

/// One chip in a [`ChipRowGhost`]: a chip of words, a fold chip (the
/// disclosure arrow, then words), or a chip holding one color's pip.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ChipGhost {
    /// A chip whose label is these words.
    Text(String),
    /// A fold chip whose label is these words.
    Fold(String),
    /// A chip holding this color's pip.
    Pip(Color),
}

/// A `chip-row` with fixed labels while its screen loads: the label chip in
/// its p1 tint, then each of `chips`. Every chip is the live chip's own box
/// with its words or pip kept for their size and drawn invisible, so the
/// ghost row is the live row's size and its chips are empty.
#[component]
pub(crate) fn ChipRowGhost(label: &'static str, chips: Vec<ChipGhost>) -> Element {
    rsx! {
        div { class: "chip-row", "aria-hidden": "true",
            span { class: "stat-chip stat-chip-p1 chip-row-label ghost-box",
                span { {label} }
            }
            for (i , chip) in chips.into_iter().enumerate() {
                button { key: "{i}", r#type: "button", class: "chip ghost-box", tabindex: "-1",
                    match chip {
                        ChipGhost::Text(words) => rsx! { span { {words} } },
                        ChipGhost::Fold(words) => rsx! {
                            span { class: "chip-fold-arrow", "\u{25b8}" }
                            span { {words} }
                        },
                        ChipGhost::Pip(color) => rsx! {
                            i { class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost" }
                        },
                    }
                }
            }
        }
    }
}

/// The Quick add row while its screen loads: the label chip, empty at its
/// live size, and the compact input as an empty bordered box.
#[component]
pub(crate) fn QuickAddSkeleton(label: &'static str) -> Element {
    rsx! {
        div { class: "chip-row",
            span { class: "stat-chip stat-chip-p1 chip-row-label ghost-box",
                span { {label} }
            }
            div { class: "skeleton-input-compact" }
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

/// The create and edit deck form while the deck loads: the Profile, Budget
/// and Tags panels' outlines with their fields ghosted at the live form's
/// heights, so the swap doesn't jump.
#[component]
pub(crate) fn EditDeckSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-edit-deck",
            // Profile: Deck name / Format / Commander as label + input, then
            // Power level as label + the five wrapped picker chips.
            div { class: "form-panel",
                SkeletonPanelHeader {}
                div { class: "skeleton-edit-deck-fields",
                    for i in 0..3 {
                        div { key: "{i}", class: "skeleton-edit-deck-field",
                            div { class: "skeleton-bar skeleton-edit-deck-label" }
                            div { class: "skeleton-edit-deck-input",
                                div { class: "skeleton-bar skeleton-edit-deck-value" }
                            }
                        }
                    }
                    div { class: "skeleton-edit-deck-field",
                        div { class: "skeleton-bar skeleton-edit-deck-label" }
                        div { class: "skeleton-edit-deck-chips",
                            for i in 0..5 {
                                div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-lg" }
                            }
                        }
                    }
                }
            }
            // Budget: Land target as label + the -/value/+ stepper row, then
            // Price target as label + currency chips + the target input.
            div { class: "form-panel",
                SkeletonPanelHeader {}
                div { class: "skeleton-edit-deck-fields",
                    div { class: "skeleton-edit-deck-field",
                        div { class: "skeleton-bar skeleton-edit-deck-label" }
                        div { class: "skeleton-edit-deck-chips",
                            for i in 0..3 {
                                div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                            }
                        }
                    }
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
            // Tags: Deck tags and Oracle tags as label + bordered box of
            // chips, then Other tags as label + five chips of varying sizes.
            div { class: "form-panel",
                SkeletonPanelHeader {}
                div { class: "skeleton-edit-deck-fields",
                    for i in 0..2 {
                        div { key: "{i}", class: "skeleton-edit-deck-field",
                            div { class: "skeleton-bar skeleton-edit-deck-label" }
                            div { class: "skeleton-edit-deck-chipbox",
                                div { class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                                div { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                                div { class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                                div { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                            }
                        }
                    }
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
                }
            }
        }
    }
}

/// A form panel's title bar while it loads: one bar centered on it.
#[component]
fn SkeletonPanelHeader() -> Element {
    rsx! {
        div { class: "skeleton-card-header",
            div { class: "skeleton-bar skeleton-section-label" }
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
        // two can't drift apart. The header is its live box with the arrow
        // and "All" drawn invisible, and the console above ghosts in the
        // list screen.
        div { class: "card-group deck-group",
            div { class: "card-group-header group-collapsible expanded skeleton-deck-list-header ghost-box",
                span { class: "card-row-arrow", "\u{25b8}" }
                span { "All (0)" }
            }
            div { class: "collapsible open",
                div { class: "collapsible-inner",
                    for (i , chips) in TILES.iter().enumerate() {
                        div { key: "{i}", class: "card-row deck-row",
                            // One tile, like the live one: art, then the name
                            // with the chips under it beside the art.
                            div { class: "deck-list-row",
                                div { class: "skeleton-bar skeleton-deck-list-art" }
                                div { class: "deck-row-text",
                                    div { class: "skeleton-bar skeleton-deck-list-title" }
                                    div { class: "deck-row-chips",
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
    }
}
