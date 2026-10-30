//! Page indicator dots for the carousel.

use dioxus::prelude::*;

/// Distance between dot centers in rem: a 0.5rem dot plus a 0.375rem gap.
const DOT_PITCH_REM: f64 = 0.875;
/// Center of the first dot from the strip's left edge in rem: half a dot.
const FIRST_DOT_CENTER_REM: f64 = 0.25;

/// A row of small circles indicating the current page position.
///
/// When there are many pages the dot strip is translated so the active
/// dot stays centered and the edges fade out via a CSS mask.
#[component]
pub fn CarouselDots(current: usize, total: usize) -> Element {
    // Center of dot `i` from the strip's left edge.
    let offset_rem = current as f64 * DOT_PITCH_REM + FIRST_DOT_CENTER_REM;

    rsx! {
        div { class: "carousel-dots",
            div {
                class: "carousel-dots-track",
                style: "margin-left: 50%; transform: translateX(-{offset_rem}rem);",
                for i in 0..total {
                    div {
                        key: "{i}",
                        class: if i == current { "carousel-dot active" } else { "carousel-dot" },
                    }
                }
            }
        }
    }
}
