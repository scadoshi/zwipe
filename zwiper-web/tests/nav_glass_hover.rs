//! The card search's glass is one of the nav's icon buttons, so it answers a mouse as they do.

const WEB: &str = include_str!("../assets/web.css");

#[test]
fn the_glass_lights_with_the_nav_s_icon_buttons() {
    assert!(
        WEB.contains(
            ".web-shell :is(.nav-trailing-row .btn-action, .card-search-label.nav-icon):hover {"
        ),
        "the glass shares the nav icons' hover"
    );
}
