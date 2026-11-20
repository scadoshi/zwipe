//! The shared data table (`table.data-table`) takes a tappable chip as a row's label, a header's
//! `?`, and a detail row under a row that eases open with no rule of its own.

const MAGIC: &str = include_str!("../assets/magic.css");

/// The declarations of the rule whose selector is exactly `selector`, or "" when there is none.
fn rule(selector: &str) -> &'static str {
    MAGIC
        .split_once(&format!("\n{selector} {{"))
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn a_detail_row_sits_under_its_row_without_a_rule() {
    let detail = rule(".data-table tbody tr.data-table-detail");
    assert!(detail.contains("border-top: none;"), "{detail}");
    let cell = rule(".data-table .data-table-detail > td");
    assert!(cell.contains("padding: 0 var(--space-4);"), "{cell}");
}

#[test]
fn a_chip_label_wraps_inside_its_cell() {
    let chip = rule(".data-table th .chip");
    for declaration in [
        "max-width: 100%;",
        "white-space: normal;",
        "overflow-wrap: anywhere;",
    ] {
        assert!(chip.contains(declaration), "{declaration} in {chip}");
    }
}

#[test]
fn a_header_s_hint_sits_beside_its_name() {
    let hint = rule(".data-table th .info-button");
    assert!(hint.contains("margin-left: var(--space-1);"), "{hint}");
}
