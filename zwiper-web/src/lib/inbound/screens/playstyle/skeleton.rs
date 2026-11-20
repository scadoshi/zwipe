//! Playstyle while it loads: the loaded page's own panels and grid, each
//! text a ghost bar in the element that holds it, so the swap to the real
//! page moves nothing.

use components::DataBarGhost;
use dioxus::prelude::*;

use crate::inbound::chrome::ghost::{
    GhostChipBars, GhostChipRow, GhostFold, GhostHint, GhostSection, GhostShowRow, GhostText,
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

/// The Oracle tags table, all ghost: its four column names (three with their `?`), then a tag
/// and three bars per row, each bar in its column's color, in the shared data table's rows and
/// columns.
#[component]
fn TagTableGhost() -> Element {
    rsx! {
        table { class: "data-table tag-data-table",
            thead {
                tr {
                    th { scope: "col", {table_ghost(3)} }
                    for chars in [5, 5, 6] {
                        th { scope: "col", class: "data-table-bar",
                            {table_ghost(chars)}
                            GhostHint {}
                        }
                    }
                }
            }
            tbody {
                for (i, chars) in TAG_CHIPS.iter().copied().cycle().take(TAG_ROWS).enumerate() {
                    tr { key: "{i}",
                        th { scope: "row", {table_ghost(chars)} }
                        for (tone, percent) in ghost_bars(i) {
                            td { class: "data-table-bar",
                                DataBarGhost { percent, tone }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Row `i`'s three ghost bars, each in its column's color, shortening down the table as the
/// loaded bars do.
fn ghost_bars(i: usize) -> [(&'static str, u32); 3] {
    let step = u32::try_from(i).unwrap_or(0).saturating_mul(7);
    [
        ("p3", 90_u32.saturating_sub(step).max(20)),
        ("p5", 80_u32.saturating_sub(step).max(15)),
        ("p2", 100_u32.saturating_sub(step).max(25)),
    ]
}

/// A data table cell's ghost, `chars` characters wide.
fn table_ghost(chars: u8) -> Element {
    rsx! {
        span { class: "skeleton-bar data-table-ghost", style: "width: {chars}ch;" }
    }
}

/// A card list: its header (the title and `actions` ghosts), then its rows.
#[component]
fn ListGhost(title: u8, swipe: bool) -> Element {
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
/// Summary beside Colors, Card roles beside Oracle tags, then Staples across
/// the page with its chip rows above it; below the breakpoint the accordion
/// with Colors open, then Staples with its chip rows above it too.
#[component]
pub fn PlaystyleSkeleton(wide: bool) -> Element {
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
                    div { class: "web-playstyle-cell web-playstyle-staples",
                        div { class: "deck-controls", {staples} }
                        ListGhost { title: 7, swipe: false }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn the_tag_table_ghosts_every_bar_in_its_own_color_and_no_chips() {
        let html = render(|| rsx! { TagTableGhost {} });
        assert!(
            html.contains(r#"<table class="data-table tag-data-table">"#),
            "{html}"
        );
        for tone in ["p3", "p5", "p2"] {
            assert_eq!(
                html.matches(&format!("data-bar data-bar-{tone} data-bar-ghost"))
                    .count(),
                TAG_ROWS,
                "a {tone} ghost bar in every row: {html}"
            );
        }
        assert!(!html.contains("stat-chip"), "no chip ghosts: {html}");
        assert!(
            !html.contains("skeleton-chip web-ghost-count-chip"),
            "no count chip ghosts: {html}"
        );
    }
}
