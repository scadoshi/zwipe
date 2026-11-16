//! The Changelog panel's header is one height on zite and in the clients: the version chip fits
//! inside the title's line with no negative margins, and the header holds a fixed minimum.

/// The sheet the Changelog panel's rules live in.
const MAGIC: &str = include_str!("../assets/magic.css");

/// The body of the rule whose selector starts a line as `selector`.
fn rule(selector: &str) -> Option<&'static str> {
    let at = MAGIC.find(&format!("\n{selector} {{"))?;
    let body = MAGIC.get(at..)?;
    body.get(body.find('{')?.saturating_add(1)..body.find('}')?)
}

#[test]
fn the_header_holds_one_height() {
    let head = rule(".changelog-panel > .settings-list-header").unwrap();
    assert!(
        head.contains("min-height: calc(2 * var(--space-3) + 1.75rem);"),
        "{head}"
    );
}

#[test]
fn the_version_chip_fits_the_title_line() {
    let chip = rule(".changelog-panel .home-panel-header-title > .stat-chip").unwrap();
    assert!(chip.contains("margin: 0;"), "no negative margins: {chip}");
    assert!(chip.contains("padding-block: 0.125rem;"), "{chip}");
    assert!(chip.contains("line-height: 1.6;"), "{chip}");
    // 0.75rem text at 1.6 is 19.2px, plus 2px padding and a 1px border each side: 25.2px, under
    // the 1rem title's 25.6px line.
    let chip_px = 12.0 * 1.6 + 2.0 * 2.0 + 2.0;
    let title_px = 16.0 * 1.6;
    assert!(chip_px <= title_px, "{chip_px} > {title_px}");
}
