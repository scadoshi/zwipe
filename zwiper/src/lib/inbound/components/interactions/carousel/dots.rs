//! Page indicator dots for the carousel.

use dioxus::prelude::*;

/// A row of small circles indicating the current page position.
///
/// When there are many pages the dot strip is translated so the active
/// dot stays centred and the edges fade out via a CSS mask.
#[component]
pub fn CarouselDots(current: usize, total: usize) -> Element {
    // Each dot is 0.5rem wide with 0.375rem gap.
    // Centre of dot `i` = i * 0.875 + 0.25 rem from the strip's left edge.
    let offset_rem = current as f64 * 0.875 + 0.25;

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
