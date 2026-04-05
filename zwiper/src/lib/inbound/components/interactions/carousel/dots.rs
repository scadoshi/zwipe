//! Page indicator dots for the carousel.

use dioxus::prelude::*;

/// A row of small circles indicating the current page position.
#[component]
pub fn CarouselDots(current: usize, total: usize) -> Element {
    rsx! {
        div { class: "carousel-dots",
            for i in 0..total {
                div {
                    key: "{i}",
                    class: if i == current { "carousel-dot active" } else { "carousel-dot" },
                }
            }
        }
    }
}
