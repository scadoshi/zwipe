//! The toast stack sits where the kit puts it, in the window's bottom-left corner at one inset
//! from both edges, on every screen of the web and the app: no zwipe sheet moves it, lifts it
//! over a dock or a footer, or anchors it anywhere else.

const SHEETS: [(&str, &str); 3] = [
    (
        "components/assets/screens/base.css",
        include_str!("../assets/screens/base.css"),
    ),
    (
        "zwiper-web/assets/web.css",
        include_str!("../../zwiper-web/assets/web.css"),
    ),
    (
        "zwiper/assets/main.css",
        include_str!("../../zwiper/assets/main.css"),
    ),
];

/// Properties that would move the stack off the kit's corner.
const PLACEMENT: [&str; 6] = ["top", "bottom", "left", "right", "translate", "transform"];

#[test]
fn no_sheet_moves_the_toast_stack_off_the_kit_s_corner() {
    for (name, css) in SHEETS {
        assert!(
            !css.contains("--toast-bottom"),
            "{name} lifts the stack with --toast-bottom"
        );
        for block in css.split('}') {
            let Some((selector, body)) = block.rsplit_once('{') else {
                continue;
            };
            let selector = selector.lines().last().unwrap_or_default().trim();
            if !selector.ends_with(".toast-container") {
                continue;
            }
            for line in body.lines().map(str::trim) {
                let property = line.split(':').next().unwrap_or_default().trim();
                assert!(
                    !PLACEMENT.contains(&property),
                    "{name}: `{selector}` sets {line}"
                );
            }
        }
    }
}
