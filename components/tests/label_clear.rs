//! A label row's `×` takes no room in the row, so picking a chip that brings it in, or clearing
//! one that takes it away, never moves the centered label. Its width is fixed and its right
//! margin gives the row back that width, its left margin and the row's gap. A mode chip that
//! comes with a pick stays in the row folded to nothing, and opens and folds on the enter settle.

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

#[test]
fn a_chip_that_comes_with_a_pick_eases_in_on_the_enter_settle() {
    let chip = rule(".label-row > .label-chip").expect("a label row's pick chip");
    assert!(
        chip.contains("transition: all var(--enter-duration) var(--ease-enter)"),
        "{chip}"
    );
    assert!(chip.contains("max-width: 4rem"), "{chip}");
    let folded = rule(".label-row > .label-chip-folded").expect("its folded state");
    for line in [
        "max-width: 0",
        "padding-inline: 0",
        "border-inline-width: 0",
        "margin-left: -0.25rem",
    ] {
        assert!(folded.contains(line), "{line} in {folded}");
    }
}
