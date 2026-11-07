//! Playstyle while it loads: the loaded page's own panels and grid, each
//! text a ghost bar in the element that holds it, so the swap to the real
//! page moves nothing.

use dioxus::prelude::*;

use crate::inbound::chrome::ghost::{
    GhostChip, GhostChipBars, GhostChipRow, GhostFold, GhostHint, GhostSection, GhostShowRow,
    GhostText,
};

/// The Card roles chart's ghost chips, as wide as the roles' short labels.
const ROLE_CHIPS: [u8; 10] = [4, 4, 5, 5, 5, 5, 4, 5, 5, 5];
/// Ghost rows in the Oracle tags table, its `TAG_TABLE_ROWS`.
const TAG_ROWS: usize = 10;
/// The Oracle tags table's ghost tag chips, as wide as typical tag slugs.
const TAG_CHIPS: [u8; 5] = [6, 9, 12, 8, 15];
/// Ghost rows in each card list.
const LIST_ROWS: usize = 6;

/// One profile panel: a fold on a wide window, all open, else a section of
/// the accordion, open when `open`. Its title is `chars` wide, with a `?`
/// ghost at the header's end when `hint`.
#[component]
fn GhostPanel(
    wide: bool,
    chars: u8,
    open: bool,
    #[props(default = true)] hint: bool,
    children: Element,
) -> Element {
    if wide {
        rsx! {
            GhostFold { chars, hint,
                div { class: "web-playstyle-fold-body", {children} }
            }
        }
    } else {
        rsx! {
            GhostSection { chars, open, hint, {children} }
        }
    }
}

/// Summary's rows: a label and a value each.
#[component]
fn SummaryGhost() -> Element {
    rsx! {
        div { class: "flex-col",
            for (i, (label, value)) in [(5, 2), (10, 8), (9, 10)].into_iter().enumerate() {
                div { key: "{i}", class: "settings-row",
                    span { class: "settings-row-label", GhostText { chars: label } }
                    span { class: "info-row-value", GhostText { chars: value } }
                }
            }
        }
    }
}

/// The Oracle tags table: its four heads, then a tag chip and three counts
/// per row.
#[component]
fn TagTableGhost() -> Element {
    rsx! {
        div { class: "tag-table",
            div { class: "tag-table-head",
                span { class: "settings-row-label", GhostText { chars: 3 } }
            }
            for (tone, chars) in [("p3", 5), ("p5", 5), ("p2", 6)] {
                div { class: "tag-table-head tag-table-head-{tone}",
                    span { class: "settings-row-label", GhostText { chars } }
                    GhostHint {}
                }
            }
            for chars in TAG_CHIPS.iter().copied().cycle().take(TAG_ROWS) {
                div { class: "tag-table-tag",
                    GhostChip { chars, button: true }
                }
                for tone in ["p3", "p5", "p2"] {
                    span { class: "tag-table-num",
                        span { class: "skeleton-bar skeleton-chip web-ghost-count-chip stat-chip-{tone} skeleton-tinted" }
                    }
                }
            }
        }
    }
}

/// A card list: its header (the title and `actions` ghosts), then `rows`
/// below the filter rows `filters`, which sit inside the panel below the
/// breakpoint.
#[component]
fn ListGhost(title: u8, swipe: bool, filters: Option<Element>) -> Element {
    rsx! {
        div { class: "maybeboard-list card-group",
            div { class: "card-group-header card-group-header-with-hint",
                span { GhostText { chars: title } }
                div { class: "card-group-header-actions",
                    if swipe {
                        span { class: "skeleton-bar skeleton-chip web-ghost-action" }
                    }
                    GhostHint {}
                }
            }
            if let Some(filters) = filters {
                div { class: "web-card-group-filters", {filters} }
            }
            for i in 0..LIST_ROWS {
                div { key: "{i}", class: "skeleton-card-row web-ghost-card-row",
                    div { class: "skeleton-bar skeleton-card-thumb" }
                    div { class: "skeleton-bar skeleton-card-bar-row" }
                }
            }
        }
    }
}

/// The page while it loads, in the loaded page's layout: on a wide window
/// Summary beside Colors, Card roles beside Oracle tags, then the
/// suggestions beside Staples with their chip rows above them; below the
/// breakpoint the accordion with Colors open, then the lists with their
/// chip rows inside.
#[component]
pub fn PlaystyleSkeleton(wide: bool) -> Element {
    let show = rsx! {
        GhostShowRow { pips: 3 }
    };
    let staples = rsx! {
        GhostChipRow { label: 4, chips: vec![3, 8, 4] }
        GhostShowRow { label: 5, pips: 3, art: false }
    };
    rsx! {
        div { class: "web-playstyle web-playstyle-grid web-page",
            div { class: "web-hero web-playstyle-summary-hero web-playstyle-cell",
                GhostPanel { wide, chars: 7, open: true, hint: false, SummaryGhost {} }
            }
            div { class: "web-columns",
                div { class: "web-playstyle-profile",
                    div { class: "web-playstyle-cell web-playstyle-colors",
                        GhostPanel { wide, chars: 6, open: true, components::ColorBarsGhost {} }
                    }
                    div { class: "web-playstyle-summary-inline",
                        GhostPanel { wide, chars: 7, open: false, hint: false, SummaryGhost {} }
                    }
                    div { class: "web-playstyle-cell web-playstyle-roles",
                        GhostPanel { wide, chars: 10, open: false, GhostChipBars { chips: ROLE_CHIPS.to_vec() } }
                    }
                    div { class: "web-playstyle-cell web-playstyle-tags",
                        GhostPanel { wide, chars: 11, open: false, TagTableGhost {} }
                    }
                }
                div { class: "web-playstyle-list",
                    div { class: "web-playstyle-cell web-playstyle-suggested",
                        if wide {
                            div { class: "deck-controls", {show.clone()} }
                        }
                        ListGhost { title: 20, swipe: true, filters: (!wide).then_some(show) }
                    }
                    div { class: "web-playstyle-cell web-playstyle-staples",
                        if wide {
                            div { class: "deck-controls", {staples.clone()} }
                        }
                        ListGhost { title: 7, swipe: false, filters: (!wide).then_some(staples) }
                    }
                    div { class: "web-playstyle-cell web-playstyle-danger",
                        div { class: "settings-list danger-zone",
                            div { class: "settings-list-header",
                                span { class: "settings-list-title", GhostText { chars: 11 } }
                            }
                            div { class: "settings-row",
                                span { class: "settings-row-label", GhostText { chars: 11 } }
                                div { class: "settings-row-value",
                                    span { class: "skeleton-bar skeleton-chip web-ghost-action" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
