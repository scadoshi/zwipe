//! A list of preference rows, the theme sheet and the app icon sheet, is spaced by its sheet's own
//! gap alone, as on the sites: nothing in the app's sheets adds a margin under a row.

/// The app's shared screen sheets.
const SCREENS: [&str; 4] = [
    include_str!("../assets/screens/base.css"),
    include_str!("../assets/screens/components.css"),
    include_str!("../assets/screens/screens-a.css"),
    include_str!("../assets/screens/screens-b.css"),
];

#[test]
fn no_screen_sheet_puts_a_margin_under_a_preference_row() {
    for css in SCREENS {
        for (at, _) in css.match_indices(".pref-row") {
            let rest = css.get(at..).unwrap_or_default();
            let body = rest
                .split_once('{')
                .and_then(|(_, after)| after.split_once('}'))
                .map_or("", |(body, _)| body);
            assert!(
                !body.contains("margin"),
                "a row's margin stacks on the sheet's gap: {body}"
            );
        }
    }
}
