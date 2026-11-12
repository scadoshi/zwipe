//! A label row's `×` takes no room in the row, so picking a chip that brings it in, or clearing
//! one that takes it away, never moves the centered label. Its width is fixed and its right
//! margin gives the row back that width, its left margin and the row's gap.

/// The filter sheet's stylesheet, which the app, app.zwipe.net and the deck form load.
const CSS: &str = include_str!("../assets/filter.css");

/// The body of the first rule whose selector starts a line as `selector`.
fn rule<'a>(selector: &str) -> Option<&'a str> {
    let at = CSS.find(&format!("\n{selector} {{"))?;
    let body = CSS.get(at..)?;
    body.get(body.find('{')?.saturating_add(1)..body.find('}')?)
}

#[test]
fn a_label_row_s_clear_takes_no_room() {
    let row = rule(".label-row").expect("the label row rule");
    assert!(row.contains("gap: 0.25rem"), "{row}");
    let clear = rule(".clear-btn").expect("the clear rule");
    assert!(clear.contains("margin-left: 0.25rem"), "{clear}");
    let held = rule(".label-row > .clear-btn").expect("a label row's clear");
    assert!(held.contains("box-sizing: border-box"), "{held}");
    assert!(held.contains("width: var(--label-clear-w)"), "{held}");
    assert!(
        held.contains("margin-right: calc(-1 * (var(--label-clear-w) + 0.5rem))"),
        "the width, its 0.25rem left margin and the row's 0.25rem gap: {held}"
    );
}
