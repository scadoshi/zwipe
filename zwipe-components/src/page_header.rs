//! The app screen header, for the apps. Styled by `assets/app.css`.

use dioxus::prelude::*;

use crate::{Button, ButtonVariant};

/// A centered screen title with a button at each corner: `leading` on the
/// left (the app's own help or about button, carrying the
/// `page-header-support` class), and a faded "?" on the right when
/// `on_hint` is given.
///
/// The hint dialog stays the screen's; `on_hint` only opens it. Without one
/// the corner stays empty rather than offering a dialog with nothing in it.
#[component]
pub fn PageHeader(
    title: String,
    /// The left corner. The host's button, so what it opens stays the host's.
    leading: Element,
    #[props(default)] on_hint: Option<Callback<()>>,
) -> Element {
    rsx! {
        header { class: "page-header",
            {leading}
            h2 { "{title}" }
            if let Some(open) = on_hint {
                Button {
                    variant: ButtonVariant::Util,
                    class: "page-header-corner",
                    onclick: move |_| open.call(()),
                    "?"
                }
            }
        }
    }
}
