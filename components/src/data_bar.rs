//! A data table's bar: a value drawn as a thin bar in its column's color, its length the value's
//! share of the column's most, with the value inside it.

use dioxus::prelude::*;

/// `value` as a bar `percent` of its cell long, in the palette slot `tone` (`p3`, `p5`, `p2`
/// and so on), the value in the text color inside it.
#[component]
pub fn DataBar(percent: u32, tone: &'static str, value: String) -> Element {
    rsx! {
        span {
            class: "data-bar data-bar-{tone}",
            style: "--data-bar-fill: {percent}%;",
            span { class: "data-bar-value", "{value}" }
        }
    }
}

/// A [`DataBar`] while its value loads: the same bar in the same color, `percent` long, pulsing,
/// with nothing in it.
#[component]
pub fn DataBarGhost(percent: u32, tone: &'static str) -> Element {
    rsx! {
        span {
            class: "skeleton-bar data-bar data-bar-{tone} data-bar-ghost",
            style: "--data-bar-fill: {percent}%;",
        }
    }
}
