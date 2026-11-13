//! A phone's drawer pulls to the right from anywhere on it, so its content scrolls only
//! vertically and a sideways move pulls instead of panning.

const FILTER: &str = include_str!("../assets/filter.css");

#[test]
fn a_floating_drawer_leaves_sideways_moves_to_its_pull() {
    let media = FILTER.find("@media (max-width: 59.99rem) {");
    let rule = FILTER.find(
        ".sheet-form-drawer > .bottom-sheet.drawer-every-width * {\n        touch-action: pan-y;",
    );
    assert!(
        media.is_some() && rule.is_some(),
        "the phone drawer's pan-y rule reaches its scrolling content"
    );
    assert!(rule > media, "inside the phone's media block");
}

#[test]
fn a_drawer_on_the_left_edge_mirrors_the_right_and_pulls_left() {
    use components::DrawerEdge;
    assert_eq!(DrawerEdge::default(), DrawerEdge::Right);
    assert_eq!(DrawerEdge::Right.class(), "");
    assert_eq!(DrawerEdge::Left.class(), " drawer-left");
    assert_eq!(DrawerEdge::Left.pull(), orangui::Pull::Left);
    assert_eq!(DrawerEdge::Right.pull(), orangui::Pull::Right);
    let rule = ".sheet-form-drawer > .bottom-sheet.drawer-every-width.drawer-left {";
    let at = FILTER
        .find(rule)
        .unwrap_or_else(|| panic!("no rule {rule}"));
    let body = FILTER
        .get(at..)
        .and_then(|s| s.split('}').next())
        .unwrap_or_default();
    for want in [
        "left: round(var(--drawer-float-gap), 1px);",
        "right: auto;",
        "transform: translateX(calc(-100% - var(--drawer-float-gap) - 1px));",
    ] {
        assert!(body.contains(want), "{want} in {body}");
    }
    assert!(
        FILTER.contains(
            ".sheet-form-drawer > .bottom-sheet.drawer-every-width.drawer-left.show {\n        transform: translateX(0);"
        ),
        "the shown left drawer rests in place"
    );
}

#[test]
fn a_left_drawer_sheet_carries_its_edge_class() {
    use components::{DrawerEdge, DrawerSheet};
    use dioxus::prelude::*;
    fn app() -> Element {
        let open = use_signal(|| true);
        rsx! {
            DrawerSheet {
                open,
                on_close: |()| {},
                every_width: true,
                edge: DrawerEdge::Left,
                "body"
            }
        }
    }
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(
        html.contains("bottom-sheet show drawer-every-width drawer-left"),
        "{html}"
    );
}

#[test]
fn a_floating_drawer_s_title_bar_leaves_every_move_to_its_pull() {
    let rule = ".sheet-form-drawer > .bottom-sheet.drawer-every-width > .sheet-header {\n        touch-action: none;";
    let at = FILTER.find(rule);
    let pan = FILTER.find(".sheet-form-drawer > .bottom-sheet.drawer-every-width * {");
    assert!(at.is_some(), "the title bar takes no pan");
    assert!(at > pan, "after the content's pan-y, which it overrides");
}
