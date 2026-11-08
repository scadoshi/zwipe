//! Keyboard access for an element that acts as a button without being one:
//! a row, a group header or a field that opens a picker.

use dioxus::prelude::*;

/// Whether `evt` presses a button: Enter or Space. Cancels the key's default
/// for them, so Space never scrolls the page.
pub fn is_press(evt: &KeyboardEvent) -> bool {
    let pressed = evt.key() == Key::Enter || evt.key() == Key::Character(" ".into());
    if pressed {
        evt.prevent_default();
    }
    pressed
}

/// A `div` that a click, Enter or Space presses, in the tab order as a
/// `role="button"` and ringed while focused. `expanded` sets
/// `aria-expanded` for a header that folds.
///
/// It suits an element with nothing interactive inside. A header holding its
/// own buttons makes only its title the toggle, with [`is_press`].
#[component]
pub fn Pressable(
    #[props(into)] class: String,
    #[props(default)] style: Option<String>,
    #[props(default, into)] expanded: Option<bool>,
    onpress: EventHandler<()>,
    children: Element,
) -> Element {
    rsx! {
        div {
            class,
            style,
            role: "button",
            tabindex: "0",
            "aria-expanded": expanded.map(|open| open.to_string()),
            onclick: move |_| onpress.call(()),
            onkeydown: move |evt: KeyboardEvent| {
                if is_press(&evt) {
                    onpress.call(());
                }
            },
            {children}
        }
    }
}
