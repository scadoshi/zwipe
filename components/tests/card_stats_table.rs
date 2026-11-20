//! The card page's stats widgets are regular tables in the shared data table style: a header row
//! of plain column names in the muted ink, then a row of cells per stat under a thin rule, the
//! numbers right-aligned as plain text, compact enough that every panel fits a wide window.

use std::collections::HashSet;

use components::CardStatsWidget;
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use zwipe_ui::screens::card_lookup::stats::{Cell, Row, Widget};

const MAGIC: &str = include_str!("../assets/magic.css");

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    dioxus_ssr::render(&dom)
}

fn widget(cells: Vec<Cell>) -> Widget {
    Widget {
        title: "Swipes",
        heads: &["Stat", "Count", "Rate"],
        rows: vec![Row {
            label: "Add",
            cells,
        }],
    }
}

/// The declarations of the rule whose selector is exactly `selector`, or "" when there is none.
fn rule(selector: &str) -> &'static str {
    MAGIC
        .split_once(&format!("\n{selector} {{"))
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn a_real_table_with_a_header_row_and_a_row_of_cells_per_stat() {
    let html = render(|| {
        let folded = use_signal(HashSet::new);
        rsx! {
            CardStatsWidget {
                widget: widget(vec![Cell::Figure("312".to_owned()), Cell::Figure("25%".to_owned())]),
                folded,
            }
        }
    });
    assert!(html.contains(r#"<table class="data-table">"#), "{html}");
    assert!(
        html.contains(r#"<thead><tr><th scope="col">Stat</th><th scope="col" class="data-table-num">Count</th><th scope="col" class="data-table-num">Rate</th></tr></thead>"#),
        "plain column names: {html}"
    );
    assert!(
        html.contains(r#"<tbody><tr><th scope="row">Add</th><td class="data-table-num">312</td><td class="data-table-num">25%</td></tr></tbody>"#),
        "a row of cells: {html}"
    );
    for colored in ["stat-chip", "tag-table", "palette"] {
        assert!(!html.contains(colored), "no {colored}: {html}");
    }
}

#[test]
fn while_the_stats_load_the_table_is_all_ghost_with_the_same_rows_and_columns() {
    let html = render(|| {
        let folded = use_signal(HashSet::new);
        rsx! {
            CardStatsWidget { widget: widget(vec![Cell::Loading, Cell::Loading]), folded }
        }
    });
    for text in [">Stat<", ">Count<", ">Rate<", ">Add<", "…"] {
        assert!(
            !html.contains(text),
            "no real text in the skeleton, {text}: {html}"
        );
    }
    assert!(!html.contains("chip"), "no chip ghosts: {html}");
    assert!(
        html.contains(r#"<thead><tr><th scope="col"><span class="skeleton-bar data-table-ghost" style="width: 4ch;"></span></th>"#),
        "the header row ghosts each name at its length: {html}"
    );
    assert!(
        html.contains(r#"<tbody><tr><th scope="row"><span class="skeleton-bar data-table-ghost" style="width: 3ch;"></span></th><td class="data-table-num"><span class="skeleton-bar data-table-ghost" style="width: 3ch;"></span></td>"#),
        "each row ghosts its label and every number cell: {html}"
    );
    assert_eq!(
        html.matches("skeleton-bar data-table-ghost").count(),
        6,
        "3 heads, a label, 2 figures: {html}"
    );
}

#[test]
fn the_shared_table_style_reads_as_a_plain_table() {
    let head = rule(".data-table thead th");
    assert!(
        head.contains("color: var(--ink-text-muted, var(--text-muted));"),
        "muted names: {head}"
    );
    let rows = rule(".data-table tbody tr");
    assert!(
        rows.contains("border-top: 1px solid var(--border-secondary);"),
        "a thin rule between rows: {rows}"
    );
    let num = rule(".data-table .data-table-num");
    assert!(num.contains("text-align: right;"), "{num}");
    assert!(
        num.contains("width: 4.5rem;"),
        "stacked tables line their columns up: {num}"
    );
    let cells = rule(".data-table :is(th, td)");
    assert!(
        cells.contains("padding: var(--space-1) var(--space-2);"),
        "compact rows: {cells}"
    );
    assert!(
        !cells.contains("border-left") && !num.contains("border-left"),
        "no column rules"
    );
    let ghost = rule(".data-table .data-table-ghost");
    assert!(
        ghost.contains("display: inline-block;"),
        "a ghost sits on its cell's line: {ghost}"
    );
}
