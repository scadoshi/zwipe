//! A card's icon buttons: Image, Flip, Meld and the Star toggle wherever a card offers them, its icon never a
//! word, each the height of the text buttons beside it and named for a screen reader and a
//! tooltip.

use dioxus::prelude::*;

use crate::data_action;
use crate::icons::DockIcon;

/// The image icon: a square frame with a sun over a hill, Lucide's `image` drawn as one path
/// (ISC, `components/LICENSE-LUCIDE`).
pub const IMAGE_ICON: &str = "M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zM9 7a2 2 0 1 0 0 4 2 2 0 1 0 0-4zM21 15l-3.086-3.086a2 2 0 0 0-2.828 0L6 21";

/// The meld icon: two lines merging into one that points on, Lucide's `merge` drawn as one path
/// (ISC, `components/LICENSE-LUCIDE`).
pub const MELD: &str = "M8 6l4-4 4 4M12 2v10.3a4 4 0 0 1-1.17 2.83L4 22M20 22l-5-5";

/// The star icon: an outline, which [`StarButton`] fills while the card is starred. Our own
/// drawing, as is Flip's, which is the kit's (orangui `icons::FLIP`).
pub const STAR: &str =
    "M12 2.5l2.9 5.9 6.5.9-4.7 4.6 1.1 6.5-5.8-3.1-5.8 3.1 1.1-6.5-4.7-4.6 6.5-.9z";

/// What the Image button is called for a screen reader and in its tooltip.
pub const IMAGE_LABEL: &str = "Image";

/// One of a card's icon buttons: a text button's box (`card-action-btn`) holding the icon `path`
/// at the text's size, named `label`, carrying `action` for the page's keys
/// ([`data_action`]). With `pressed` it toggles, drawn selected while on. It keeps its click from
/// the row or panel under it.
#[component]
pub fn CardIconButton(
    label: &'static str,
    path: &'static str,
    action: &'static str,
    #[props(default)] pressed: Option<bool>,
    on_click: EventHandler<()>,
) -> Element {
    let class = if pressed == Some(true) {
        "card-action-btn card-action-icon selected"
    } else {
        "card-action-btn card-action-icon"
    };
    rsx! {
        button {
            r#type: "button",
            class,
            "data-action": action,
            "aria-label": label,
            "aria-pressed": pressed.map(|on| on.to_string()),
            title: label,
            onclick: move |evt| {
                evt.stop_propagation();
                on_click.call(());
            },
            DockIcon { path }
        }
    }
}

/// The button that opens a card's image, the image icon named [`IMAGE_LABEL`].
#[component]
pub fn ImageButton(on_click: EventHandler<()>) -> Element {
    rsx! {
        CardIconButton { label: IMAGE_LABEL, path: IMAGE_ICON, action: data_action::IMAGE, on_click }
    }
}

/// A deck card's MVP toggle: the star, an outline while the card isn't `starred` and filled
/// while it is, named Star or Unstar to match, pressed while starred.
#[component]
pub fn StarButton(starred: bool, on_toggle: EventHandler<()>) -> Element {
    let label: &'static str = if starred { "Unstar" } else { "Star" };
    rsx! {
        CardIconButton {
            label,
            path: STAR,
            action: data_action::STAR,
            pressed: Some(starred),
            on_click: on_toggle,
        }
    }
}
