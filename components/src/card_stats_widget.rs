//! One of the card page's stats widgets, a regular table in the shared data table style.

use std::collections::HashSet;

use dioxus::prelude::*;
use zwipe_ui::screens::card_lookup::stats::{Cell, Widget};

use crate::DeckFold;

/// `widget` as a fold holding a regular table in the shared data table style: a header row of
/// the column names, then a row per stat, its label and its figures, right-aligned. `folded`
/// holds the folds the user shut. While the stats load, the table is all ghost: every name,
/// label and figure a skeleton bar at its place.
#[component]
pub fn CardStatsWidget(widget: Widget, folded: Signal<HashSet<String>>) -> Element {
    let Widget { title, heads, rows } = widget;
    let loading = rows
        .iter()
        .any(|row| row.cells.iter().any(|cell| matches!(cell, Cell::Loading)));
    let (first, columns) = heads.split_first().map_or(("", &[][..]), |(f, c)| (*f, c));
    let text = move |text: &'static str| {
        if loading {
            ghost(text.chars().count())
        } else {
            rsx! { "{text}" }
        }
    };
    rsx! {
        DeckFold { fold_key: title, title, folded,
            table { class: "data-table",
                thead {
                    tr {
                        th { scope: "col", {text(first)} }
                        for head in columns.iter() {
                            th { key: "{head}", scope: "col", class: "data-table-num", {text(head)} }
                        }
                    }
                }
                tbody {
                    for row in rows {
                        tr { key: "{row.label}",
                            th { scope: "row", {text(row.label)} }
                            for cell in row.cells {
                                td { class: "data-table-num",
                                    match cell {
                                        Cell::Loading => ghost(FIGURE_GHOST_CHARS),
                                        Cell::Figure(figure) => rsx! { "{figure}" },
                                        Cell::Blank => rsx! {},
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

/// How many characters a figure's ghost spans.
const FIGURE_GHOST_CHARS: usize = 3;

/// A skeleton bar `chars` characters wide, in a data table's cell.
fn ghost(chars: usize) -> Element {
    rsx! {
        span { class: "skeleton-bar data-table-ghost", style: "width: {chars}ch;" }
    }
}
