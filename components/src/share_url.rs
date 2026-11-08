//! A shared deck's public link, shown in the Share dialog.

use dioxus::prelude::*;

/// `url` on one line, cut short with an ellipsis when it runs past the
/// dialog, the whole of it in the tooltip. A tap selects all of it, so it can
/// be copied by hand as well as with Copy link.
#[component]
pub fn ShareUrl(url: String) -> Element {
    rsx! {
        div { class: "share-url", title: "{url}", "{url}" }
    }
}
