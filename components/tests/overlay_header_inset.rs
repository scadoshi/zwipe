//! A full-screen overlay that opens with a page header (the phone pickers, Send to among them,
//! and the Oracle tag examples) keeps that header below the status bar, as the app's own
//! header is, and its content doesn't add the status bar's inset a second time.

/// The body of the rule whose whole selector is `selector`.
fn rule<'css>(css: &'css str, selector: &str) -> Option<&'css str> {
    css.split('}').find_map(|block| {
        let (head, body) = block.rsplit_once('{')?;
        let head = head.lines().last()?.trim();
        (head == selector).then_some(body)
    })
}

#[test]
fn an_overlay_s_page_header_sits_below_the_status_bar() {
    let body = rule(components::SCREENS_CSS, ".screen > .page-header")
        .expect("a rule for a screen's own page header");
    let padding = body
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("padding-top:"))
        .expect("the header sets its top padding");
    assert!(
        padding.contains("env(safe-area-inset-top"),
        "the header pads the status bar's inset: {padding}"
    );
}

#[test]
fn content_under_an_overlay_s_page_header_skips_the_inset() {
    let body = rule(
        components::SCREENS_CSS,
        ".screen > .page-header ~ .screen-content",
    )
    .expect("a rule for the content under a screen's page header");
    let padding = body
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("padding-top:"))
        .expect("the content sets its top padding");
    assert!(
        !padding.contains("safe-area-inset-top"),
        "the header already clears the status bar: {padding}"
    );
}

#[test]
fn an_overlay_s_header_corners_center_on_the_title_below_the_status_bar() {
    let body = rule(
        components::SCREENS_CSS,
        ".screen > .page-header .page-header-corner",
    )
    .expect("a rule for the corners of a screen's page header");
    let top = body
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("top:"))
        .expect("the corners set their top");
    assert!(
        top.contains("env(safe-area-inset-top"),
        "the corners move down by the status bar's inset: {top}"
    );
}
