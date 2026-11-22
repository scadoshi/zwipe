//! The app's 404 names itself once: the dead end's card holds the h1, so
//! the frame adds no hidden one of its own.

#[test]
fn the_not_found_frame_leaves_the_h1_to_its_card() {
    let screen = include_str!("../src/lib/inbound/screens/not_found.rs");
    assert!(
        screen.contains(r#"ScreenFrame { title: "Page not found", titled: true,"#),
        "the 404's frame is told its card carries the title"
    );
}
