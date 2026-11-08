//! app.zwipe.net serves a real robots.txt from `public/`, not the app's
//! index.html behind the single-page fallback, so a crawler or link-preview
//! bot reads plain rules: every path allowed, for everyone.

const ROBOTS: &str = include_str!("../public/robots.txt");

#[test]
fn every_bot_may_fetch_every_path() {
    assert!(!ROBOTS.contains('<'), "plain text, not a page");
    let lines: Vec<&str> = ROBOTS.lines().map(str::trim).collect();
    assert_eq!(lines, ["User-agent: *", "Allow: /"]);
}
