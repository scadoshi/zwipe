//! A shared deck's public link, shown in the Share dialog.

use dioxus::prelude::*;

/// `url` in an accent chip centered under the dialog's text, hugging the
/// link's width. It stays on one line while the dialog fits it and otherwise
/// breaks at any character. A tap selects all of it, so it can be copied by
/// hand as well as with Copy link.
#[component]
pub fn ShareUrl(url: String) -> Element {
    rsx! {
        div { class: "share-url",
            span { class: "stat-chip share-url-chip", "{url}" }
        }
    }
}
