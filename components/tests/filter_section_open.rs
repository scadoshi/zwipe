//! An opening filter section shows its rows whole: they stay hidden through the height ease and
//! fade in at its end, and the scroll that brings the section to the top is instant, so nothing
//! slides under the pointer.

use components::{FILTER_CSS, OPEN_SECTION_SCROLL_JS};

/// The declarations of the first rule whose selector is exactly `selector`.
fn rule(selector: &str) -> Option<&'static str> {
    let start = FILTER_CSS.find(&format!("\n{selector} {{"))?;
    let body = FILTER_CSS.get(start..)?;
    let open = body.find('{')?;
    let close = body.find('}')?;
    body.get(open.checked_add(1)?..close)
}

#[test]
fn an_opening_section_fades_its_rows_in_after_the_height_ease() {
    let body = rule(r#".accordion-content[data-open="true"] > *"#).unwrap_or_default();
    assert!(
        body.contains("accordion-content-in 120ms ease 80ms both"),
        "the rows stay hidden through most of the 180ms ease, then fade in: {body}"
    );
    assert!(FILTER_CSS.contains("@keyframes accordion-content-in"));
}

#[test]
fn the_open_section_scrolls_into_view_at_once() {
    assert!(
        OPEN_SECTION_SCROLL_JS.contains("behavior: 'instant'"),
        "{OPEN_SECTION_SCROLL_JS}"
    );
    assert!(
        !OPEN_SECTION_SCROLL_JS.contains("smooth"),
        "{OPEN_SECTION_SCROLL_JS}"
    );
    assert!(
        OPEN_SECTION_SCROLL_JS.contains(r#".accordion-item[data-open="true"]"#),
        "the open item, not a counted position: {OPEN_SECTION_SCROLL_JS}"
    );
}

#[test]
fn a_section_that_closes_for_another_shuts_at_once() {
    let body = rule(
        r#".accordion:has(.accordion-content[data-open="true"]) .accordion-content:not([data-open="true"])"#,
    )
    .unwrap_or_default();
    assert!(body.contains("transition: none"), "{body}");
}
