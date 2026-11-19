//! One of the card screen's stats widgets, drawn in Playstyle's Oracle tags table grammar.

use std::collections::HashSet;

use components::DeckFold;
use dioxus::prelude::*;
use zwipe_ui::screens::card_lookup::stats::{Cell, Widget};

use crate::inbound::chrome::ghost::GhostText;

/// `widget` as a fold: its column labels, then each row's label and a chip per figure, the last
/// column's chips in the rate color, a ghost chip while the stats load. `folded` holds the folds
/// the user shut.
#[component]
pub(super) fn StatsWidget(widget: Widget, folded: Signal<HashSet<String>>) -> Element {
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
            div { class: "tag-table web-card-stats-table", style: "{columns}",
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
                                    span { class: "{chip(column)} web-ghost-chip", GhostText { chars: 3 } }
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
