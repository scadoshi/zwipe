//! A panel of card rows that folds, as every card group does: the header's
//! disclosure arrow and title, with the panel's own actions at its end, over
//! the rows.

use dioxus::prelude::*;

use crate::is_press;

/// A card list panel (Suggested commanders, Staples, Most skipped, the
/// Commander maybeboard): a header of the arrow and `title`, then `actions`
/// at its end, over `children`, which ease shut and open with the
/// `.collapsible` the card groups use. A click on the header folds it; a
/// click among `actions` does not. Starts open.
#[component]
pub fn CardListFold(
    #[props(into)] title: String,
    /// The header's buttons (Swipe, the `?`), at its end.
    #[props(default)]
    actions: Option<Element>,
    children: Element,
) -> Element {
    let mut open = use_signal(|| true);
    rsx! {
        div { class: if open() { "maybeboard-list card-group row-enter" } else { "maybeboard-list card-group row-enter collapsed" },
            div {
                class: if open() { "card-group-header card-group-header-with-hint group-collapsible expanded" } else { "card-group-header card-group-header-with-hint group-collapsible" },
                onclick: move |_| open.toggle(),
                // The keyboard's toggle, beside the actions rather than
                // around them, so their own buttons stay separate.
                span {
                    class: "card-list-fold-title",
                    role: "button",
                    tabindex: "0",
                    "aria-expanded": "{open}",
                    onkeydown: move |evt: KeyboardEvent| {
                        if is_press(&evt) {
                            open.toggle();
                        }
                    },
                    span { class: "card-row-arrow", "\u{25b8}" }
                    span { "{title}" }
                }
                if let Some(actions) = actions {
                    div {
                        class: "card-group-header-actions",
                        onclick: move |evt: MouseEvent| evt.stop_propagation(),
                        {actions}
                    }
                }
            }
            // A shut panel is inert, so Tab and screen readers pass over it.
            div {
                class: if open() { "collapsible open" } else { "collapsible" },
                inert: (!open()).then_some(true),
                div { class: "collapsible-inner", {children} }
            }
        }
    }
}
