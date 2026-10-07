//! A one-at-a-time media gallery for the sites: the media with round
//! prev/next buttons on its edges, and a footer with its caption and a
//! counter. Base look in `assets/site.css` ([`SITE_CSS`](crate::SITE_CSS)).
//!
//! The media itself (an image, a video) and where the footer sits (under a
//! rule in the body, in a panel's actions row) stay the host's.

use dioxus::prelude::*;

/// The media with prev/next buttons, which wrap around. The buttons show only
/// when there is more than one item. `noun` names an item for the buttons'
/// labels ("Previous demo"); `class` adds to `gallery-body`.
///
/// Give the media `key: "{index}"` so it remounts on a step, which restarts a
/// video's autoplay rather than keeping the old one.
#[component]
pub fn GalleryFrame(
    index: Signal<usize>,
    total: usize,
    noun: String,
    #[props(default)] class: Option<String>,
    children: Element,
) -> Element {
    let mut index = index;
    let class = match class {
        Some(extra) => format!("gallery-body {extra}"),
        None => "gallery-body".to_string(),
    };
    rsx! {
        div { class,
            {children}
            if total > 1 {
                button {
                    class: "gallery-nav gallery-prev",
                    aria_label: "Previous {noun}",
                    onclick: move |_| {
                        let i = index();
                        index.set(if i == 0 { total - 1 } else { i - 1 });
                    },
                    "\u{2190}"
                }
                button {
                    class: "gallery-nav gallery-next",
                    aria_label: "Next {noun}",
                    onclick: move |_| {
                        let i = index();
                        index.set((i + 1) % total);
                    },
                    "\u{2192}"
                }
            }
        }
    }
}

/// The caption on the left, keyed on `index` so it eases in afresh on each
/// step, and an `n / total` counter on the right when there is more than one
/// item. `index` is the item shown, from zero.
#[component]
pub fn GalleryFooter(
    index: usize,
    total: usize,
    #[props(default)] caption: Option<String>,
) -> Element {
    rsx! {
        div { class: "gallery-footer",
            if let Some(caption) = caption {
                span { key: "{index}", class: "gallery-caption", "{caption}" }
            }
            if total > 1 {
                span { class: "gallery-counter", "{index + 1} / {total}" }
            }
        }
    }
}
