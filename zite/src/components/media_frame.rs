//! A video or screenshot held in its own box while it loads.
//!
//! The frame takes the media's shape from its known width and height, so the
//! page never moves when the media arrives, and shows the site's loading
//! skeleton (components' `.skeleton-bar`, the app's ghost fill and pulse)
//! inside it. `index.html`'s script marks the media
//! `data-ready` once it can paint its first frame (an image's `load`, a
//! video's `loadeddata`, or an `error` so a broken file never pulses
//! forever); until then the stylesheet keeps the media invisible, then fades
//! it in over the skeleton, which stops pulsing. Without script nothing is held and the media
//! shows as it loads.

use dioxus::prelude::*;

/// A media's width and height in pixels, which fix its frame's shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MediaSize {
    /// Width in pixels.
    pub(crate) width: u32,
    /// Height in pixels.
    pub(crate) height: u32,
}

/// `children` (one `img` or `video`) in a frame of `size`'s shape, with the
/// loading skeleton under it until it can paint. `class` adds the place's
/// own class, which sets the frame's height cap through `--media-max-h`.
#[component]
pub(crate) fn MediaFrame(
    size: MediaSize,
    #[props(default)] class: Option<String>,
    children: Element,
) -> Element {
    let class = match class {
        Some(extra) => format!("media-frame {extra}"),
        None => "media-frame".to_owned(),
    };
    let MediaSize { width, height } = size;
    rsx! {
        div { class, style: "--media-w: {width}; --media-h: {height};",
            {children}
            div { class: "skeleton-bar media-frame-sk", "aria-hidden": "true" }
        }
    }
}
