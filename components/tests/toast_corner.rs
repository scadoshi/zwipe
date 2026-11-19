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

/// Every sheet the app and the web draw with, the kit's included.
fn every_sheet() -> Vec<(&'static str, &'static str)> {
    let kit = [
        ("orangui COMPONENTS_CSS", components::COMPONENTS_CSS),
        ("orangui APP_CSS", components::APP_CSS),
        ("components SCREENS_CSS", components::SCREENS_CSS),
        ("components MAGIC_CSS", components::MAGIC_CSS),
        ("components FILTER_CSS", components::FILTER_CSS),
        ("components DECK_PAGE_CSS", components::DECK_PAGE_CSS),
    ];
    SHEETS.into_iter().chain(kit).collect()
}

/// A toast floats in front of whatever is under it: no rule keys on a live toast to pad, size or
/// move anything else out of its way, a sheet or a dock included.
#[test]
fn nothing_makes_room_for_a_toast() {
    for (name, css) in every_sheet() {
        assert!(
            !css.contains("--toast-band-top"),
            "{name} keeps a band for content to clear the toasts"
        );
        for block in css.split('}') {
            let Some((selector, _)) = block.rsplit_once('{') else {
                continue;
            };
            let selector = selector.lines().last().unwrap_or_default().trim();
            assert!(
                !selector.contains(":has(.toast-container"),
                "{name}: `{selector}` changes other content while a toast is up"
            );
        }
    }
}

/// The kit's toast sheet, as the web copies it.
const TOAST: &str = include_str!("../../zwiper-web/assets/sheets/08-toast.css");

/// Every `z-index` a sheet sets to a number.
fn z_indexes(css: &str) -> impl Iterator<Item = i64> + '_ {
    css.split("z-index:")
        .skip(1)
        .filter_map(|rest| rest.split(';').next()?.trim().parse().ok())
}

/// The stack stays in front of everything else: sheets, docks and dialogs.
#[test]
fn the_toast_stack_sits_in_front_of_every_sheet_and_dock() {
    let stack = TOAST
        .split_once(".toast-container {")
        .and_then(|(_, body)| z_indexes(body.split('}').next()?).next())
        .expect("the stack sets a z-index");
    for (name, css) in every_sheet() {
        for z in z_indexes(css) {
            assert!(
                z < stack,
                "{name} sets z-index {z}, not under the toasts' {stack}"
            );
        }
    }
}
