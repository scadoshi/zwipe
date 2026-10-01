//! Boxes and arrows for an inline SVG diagram, styled by the theme.
//!
//! A page lays the shapes out in its own `svg` and gives each box a tone, so a
//! diagram reads in every theme and its text is real text. [`DiagramDefs`] goes
//! first inside the `svg`: it defines the arrowhead the arrows reference.

use dioxus::prelude::*;

/// Which accent outlines a box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiagramTone {
    /// The muted text color, for things outside the system.
    #[default]
    Muted,
    /// The primary accent.
    Primary,
    /// The secondary accent.
    Secondary,
    /// The tertiary accent.
    Tertiary,
    /// The success color.
    Success,
}

impl DiagramTone {
    fn class(self) -> &'static str {
        match self {
            Self::Muted => "diagram-muted",
            Self::Primary => "diagram-primary",
            Self::Secondary => "diagram-secondary",
            Self::Tertiary => "diagram-tertiary",
            Self::Success => "diagram-success",
        }
    }
}

/// Height of every box, so rows line up.
pub const DIAGRAM_NODE_HEIGHT: f64 = 52.0;

/// The arrowhead marker. Place it first inside the `svg`.
#[component]
pub fn DiagramDefs() -> Element {
    rsx! {
        defs {
            marker {
                id: "diagram-head",
                view_box: "0 0 10 10",
                ref_x: "9",
                ref_y: "5",
                marker_width: "7",
                marker_height: "7",
                orient: "auto-start-reverse",
                path { d: "M 0 0 L 10 5 L 0 10 z" }
            }
        }
    }
}

/// A labeled box: a title and a line under it.
#[component]
pub fn DiagramNode(
    x: f64,
    y: f64,
    #[props(default = 150.0)] w: f64,
    title: String,
    #[props(default)] sub: String,
    #[props(default)] tone: DiagramTone,
) -> Element {
    let cx = x + w / 2.0;
    rsx! {
        g { class: "diagram-node {tone.class()}",
            rect { x: "{x}", y: "{y}", width: "{w}", height: "{DIAGRAM_NODE_HEIGHT}", rx: "8" }
            text { class: "diagram-title", x: "{cx}", y: "{y + 22.0}", text_anchor: "middle", "{title}" }
            if !sub.is_empty() {
                text { class: "diagram-sub", x: "{cx}", y: "{y + 40.0}", text_anchor: "middle", "{sub}" }
            }
        }
    }
}

/// An arrow between two points, with its label beside the line: above a level
/// arrow, to the right of a vertical one, pushed off to the side of a diagonal
/// one. `both` puts a head on each end.
#[component]
pub fn DiagramArrow(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    #[props(default)] label: String,
    #[props(default)] both: bool,
) -> Element {
    let vertical = (x2 - x1).abs() <= 1.0;
    let diagonal = !vertical && (y2 - y1).abs() > 1.0;
    // A label on a rising arrow sits left of the midpoint, on a falling one right.
    let shift = if diagonal {
        -30.0 * (y2 - y1).signum()
    } else {
        0.0
    };
    let (mx, anchor) = if vertical {
        (x1 + 8.0, "start")
    } else {
        (f64::midpoint(x1, x2) + shift, "middle")
    };
    let my = if vertical {
        f64::midpoint(y1, y2) + 4.0
    } else {
        f64::midpoint(y1, y2) - if diagonal { 14.0 } else { 6.0 }
    };
    rsx! {
        g { class: "diagram-arrow",
            line {
                x1: "{x1}", y1: "{y1}", x2: "{x2}", y2: "{y2}",
                marker_end: "url(#diagram-head)",
                marker_start: if both { "url(#diagram-head)" } else { "" },
            }
            if !label.is_empty() {
                text { x: "{mx}", y: "{my}", text_anchor: "{anchor}", "{label}" }
            }
        }
    }
}
