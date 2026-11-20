//! A data table's bar: a value drawn as a thin bar in its column's color, its length the value's
//! share of the column's most, with the value inside it.

use dioxus::prelude::*;

use crate::charts::bar_delay;

/// `value` as a bar `percent` of its cell long, in the palette slot `tone` (`p3`, `p5`, `p2`
/// and so on), the value in the text color inside it. The bar grows in and the value fades in
/// as a chart's do, staggered by `index`, the row's place.
#[component]
pub fn DataBar(
    percent: u32,
    tone: &'static str,
    value: String,
    #[props(default)] index: usize,
) -> Element {
    let delay = bar_delay(index);
    rsx! {
        span { class: "data-bar data-bar-{tone}",
            span { class: "data-bar-fill chart-bar", style: "width: {percent}%; {delay}" }
            span { class: "data-bar-value chart-value", style: "{delay}", "{value}" }
        }
    }
}

/// A [`DataBar`] while its value loads: the same bar in the same color, `percent` long, pulsing,
/// with nothing in it.
#[component]
pub fn DataBarGhost(percent: u32, tone: &'static str) -> Element {
    rsx! {
        span { class: "skeleton-bar data-bar data-bar-{tone} data-bar-ghost",
            span { class: "data-bar-fill", style: "width: {percent}%;" }
        }
    }
}
