//! The keyboard's focus ring: a control that drops the browser's ring (`all: unset`, or an
//! outline taken away) draws the kit's own on `:focus-visible`, so a keyboard user can see where
//! focus is while a mouse or a tap looks as it always did.

const FILTER: &str = include_str!("../assets/filter.css");

/// The ring the kit draws on a bare control: a 2px accent outline, 2px out.
const RING: [&str; 2] = [
    "outline: 2px solid var(--accent-primary);",
    "outline-offset: 2px;",
];

/// The body of the rule for `selector` in `sheet`.
fn rule(sheet: &'static str, selector: &str) -> &'static str {
    let Some(at) = sheet.find(&format!("\n{selector} {{")) else {
        return "";
    };
    sheet
        .get(at..)
        .and_then(|rest| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn the_filters_x_buttons_and_accordion_triggers_ring_on_keyboard_focus() {
    for selector in [
        ".chip-remove:focus-visible",
        ".clear-btn:focus-visible",
        ".accordion-trigger:focus-visible",
    ] {
        let body = rule(FILTER, selector);
        for line in RING {
            assert!(body.contains(line), "{selector}: {body:?}");
        }
    }
}

#[test]
fn every_unset_control_has_a_focus_ring() {
    let mut selector = String::new();
    for line in FILTER.lines() {
        if let Some(head) = line.strip_suffix(" {") {
            selector = head.trim().to_owned();
        }
        if line.trim() == "all: unset;" {
            let ring = format!("{selector}:focus-visible");
            assert!(
                !rule(FILTER, &ring).is_empty(),
                "`{selector}` unsets the browser's ring but has no `{ring}` rule"
            );
        }
    }
}
