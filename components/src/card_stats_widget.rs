//! One of the card page's stats widgets, drawn in Playstyle's Oracle tags table grammar.

use std::collections::HashSet;

use dioxus::prelude::*;
use zwipe_ui::screens::card_lookup::stats::{Cell, Widget};

use crate::DeckFold;

/// `widget` as a fold: its column labels, then each row's label and a chip per figure, the last
/// column's chips in the rate color. `folded` holds the folds the user shut. While the stats
/// load, each chip holds `ghost`, when the host draws one, and takes `ghost_class` too, the host's loading look.
#[component]
pub fn CardStatsWidget(
    widget: Widget,
    folded: Signal<HashSet<String>>,
    #[props(default)] ghost: Option<Element>,
    #[props(default)] ghost_class: &'static str,
) -> Element {
    let Widget { title, heads, rows } = widget;
    let figures = heads.len().saturating_sub(1);
    let columns = format!("grid-template-columns: minmax(0, 1fr) repeat({figures}, auto);");
    let chip = move |column: usize| {
        if column.saturating_add(1) == figures {
            "stat-chip stat-chip-p2"
        } else {
            "stat-chip stat-chip-p3"
        }
    };
    rsx! {
        DeckFold { fold_key: title, title, folded,
            div { class: "tag-table card-stats-table", style: "{columns}",
                for (column, head) in heads.iter().enumerate() {
                    div {
                        key: "{head}",
                        class: match column {
                            0 => "tag-table-head",
                            c if c == figures => "tag-table-head tag-table-head-p2",
                            _ => "tag-table-head tag-table-head-p3",
                        },
                        span { class: "settings-row-label", "{head}" }
                    }
                }
                for row in rows {
                    div { key: "{row.label}", class: "tag-table-tag",
                        span { class: "settings-row-label", "{row.label}" }
                    }
                    for (column, cell) in row.cells.into_iter().enumerate() {
                        span { class: "tag-table-num",
                            match cell {
                                Cell::Loading => rsx! {
                                    span { class: "{chip(column)} {ghost_class}", {ghost.clone()} }
                                },
                                Cell::Figure(figure) => rsx! {
                                    span { class: chip(column), "{figure}" }
                                },
                                Cell::Blank => rsx! {},
                            }
                        }
                    }
                }
            }
        }
    }
}
