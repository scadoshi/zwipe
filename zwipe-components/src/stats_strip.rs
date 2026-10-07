//! A row of headline figures, each rolling to its value over a label, with
//! an optional row of chips under it saying where the numbers come from.
//! The base look is in `assets/components.css`; each host sizes the numbers
//! for its own space.

use dioxus::prelude::*;

use crate::CountUp;

/// The figures strip. Each figure is its value, `None` while it has not
/// arrived (it rolls until it does), and its label.
///
/// `source` is the contents of the chip row under the strip, the host's own
/// links and chips; without it there is no row.
#[component]
pub fn StatsStrip(
    figures: Vec<(Option<u64>, &'static str)>,
    /// Print `28.0k` rather than every digit, for a narrow strip.
    #[props(default)]
    compact: bool,
    #[props(default)] source: Option<Element>,
) -> Element {
    rsx! {
        div { class: "hero-figures",
            section { class: "stats-strip",
                for (value, label) in figures {
                    div { key: "{label}", class: "stat",
                        span { class: "stat-num", CountUp { value, compact } }
                        span { class: "stat-label", "{label}" }
                    }
                }
            }
            if let Some(source) = source {
                div { class: "tag-row stats-source", {source} }
            }
        }
    }
}
