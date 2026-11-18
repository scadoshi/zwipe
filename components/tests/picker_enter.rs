//! Enter in a picker's search acts on its top match, which shows as the Enter target while the
//! search has text.

use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    dioxus_ssr::render(&dom)
}

#[test]
fn only_the_target_choice_is_marked() {
    fn grid() -> Element {
        rsx! {
            components::PickerChoice { target: true, span { "Commander" } }
            components::PickerChoice { target: false, span { "Pauper Commander" } }
        }
    }
    let html = render(grid);
    assert!(
        html.contains(r#"<span class="picker-choice enter-target"><span>Commander</span>"#),
        "{html}"
    );
    assert!(
        html.contains(r#"<span class="picker-choice"><span>Pauper Commander</span>"#),
        "{html}"
    );
}

#[test]
fn the_target_wears_a_chip_s_hover_look_while_the_search_has_focus() {
    let css = include_str!("../assets/screens/screens-b.css");
    assert!(
        css.contains(":is(.picker-body, .tag-screen):has(.chip-row > .input:focus, .chip-row .input-clear-wrap > .input:focus) .enter-target > .chip"),
        "the highlight shows while Enter is the search's, in the web's pickers and the app's"
    );
    assert!(
        css.contains(".picker-choice {\n    display: contents;\n}"),
        "the wrapper leaves the grid's layout alone"
    );
}
