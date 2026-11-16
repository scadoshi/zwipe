//! The web app's About is its Changelog alone, at every width: the page footer under every
//! screen carries the Privacy policy link and the credits, so About draws neither. The mobile
//! apps have no footer, so their About keeps both.

/// The web app's About screen.
const ABOUT: &str = include_str!("../src/lib/inbound/screens/about.rs");

/// The footer every web screen ends with.
const FRAME: &str = include_str!("../src/lib/inbound/shell/frame.rs");

/// The mobile apps' About screen.
const APP_ABOUT: &str = include_str!("../../zwiper/src/lib/inbound/screens/about.rs");

/// The web app's own sheet.
const WEB_CSS: &str = include_str!("../assets/web.css");

#[test]
fn the_web_about_draws_no_privacy_row_and_no_credits() {
    for gone in [
        "Row::PrivacyPolicy",
        "Action::PrivacyPolicy",
        "about::CREDITS",
        "SCRYFALL_CREDIT_HTML",
        "FAN_CONTENT_HTML",
        "web-about-privacy",
        "about-credits",
    ] {
        assert!(!ABOUT.contains(gone), "the web About still has {gone}");
    }
    assert!(
        ABOUT.contains("ChangelogView {"),
        "the web About draws the Changelog"
    );
}

#[test]
fn the_footer_carries_what_about_left() {
    assert!(
        FRAME.contains(
            r#"Link { to: Route::PrivacyPolicy {}, class: "nav-cta", "Privacy policy" }"#
        ),
        "the web footer links the Privacy policy"
    );
    assert!(
        FRAME.contains("SiteFooter {"),
        "the web footer is the kit's, which draws the Scryfall credit and the Fan Content notice"
    );
}

#[test]
fn the_apps_about_keeps_both() {
    assert!(
        APP_ABOUT.contains("PrivacyPolicy"),
        "the apps have no footer, so their About keeps the Privacy policy"
    );
    assert!(
        APP_ABOUT.contains("SCRYFALL_CREDIT_HTML") || APP_ABOUT.contains("CREDITS"),
        "the apps' About keeps the credits"
    );
}

#[test]
fn the_web_sheet_hides_no_about_panel() {
    assert!(
        !WEB_CSS.contains(".web-about > .web-columns"),
        "no rule hides About panels that are no longer drawn"
    );
}
