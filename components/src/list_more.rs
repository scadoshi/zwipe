//! A card list's Show more, centered under its panel: one chip that adds the list's next rows.
//! Suggested and the Maybeboard draw it on the app and the web.

use dioxus::prelude::*;
use zwipe_ui::commander::more_page::SHOW_MORE;

/// Whether `animation` is Show more's exit ending: the fold away, or the fade alone under reduced
/// motion.
#[must_use]
pub fn ends_list_more_leave(animation: &str) -> bool {
    matches!(animation, "card-list-more-leave" | "card-list-more-fade")
}

/// Show more under a card list's panel, while the list `offers` more rows and its panel is
/// `open`. A press calls `onclick`, which answers whether the list took it; a press it took that
/// `leaves` (it shows the last rows) folds the chip away (fades under reduced motion), and the chip
/// stays up until that exit ends. The chip sits in a bare box, since its own padding would floor
/// the row it folds.
#[component]
pub fn ListMore(offers: bool, open: bool, leaves: bool, onclick: Callback<(), bool>) -> Element {
    let mut leaving = use_signal(|| false);
    if !(open && (leaving() || offers)) {
        return rsx! {};
    }
    rsx! {
        div {
            class: if leaving() { "card-list-more card-list-more-leaving" } else { "card-list-more" },
            onanimationend: move |event: AnimationEvent| {
                if ends_list_more_leave(&event.animation_name()) {
                    leaving.set(false);
                }
            },
            div {
                button {
                    r#type: "button",
                    class: "chip",
                    onclick: move |_| {
                        if onclick.call(()) && leaves {
                            leaving.set(true);
                        }
                    },
                    {SHOW_MORE}
                }
            }
        }
    }
}
