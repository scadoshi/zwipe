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
