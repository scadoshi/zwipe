//! zwipe.net's changelog page names itself with one h1, the panel's title.

#[test]
fn the_changelog_page_s_title_is_its_h1() {
    let page = include_str!("../src/pages/changelog.rs");
    assert!(
        page.contains("title_h1: true"),
        "zite's changelog page asks for its title as the h1"
    );
    let component = include_str!("../../components/src/changelog.rs");
    assert!(
        component.contains(r#"h1 { class: "settings-list-title", {TITLE} }"#),
        "the changelog title renders as an h1 when asked"
    );
}
