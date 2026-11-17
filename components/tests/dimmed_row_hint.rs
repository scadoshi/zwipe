//! A grayed settings row takes no taps, except on its "?", which opens the row's hint either way.

const SCREENS_B: &str = include_str!("../assets/screens/screens-b.css");

#[test]
fn a_grayed_row_s_hint_button_still_takes_a_tap() {
    assert!(
        SCREENS_B
            .contains("\n.ub-chips-disabled {\n    opacity: 0.45;\n    pointer-events: none;\n}")
    );
    assert!(
        SCREENS_B.contains("\n.ub-chips-disabled .info-button {\n    pointer-events: auto;\n}")
    );
}
