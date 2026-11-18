//! Activity's heat grid parts both clients draw the same: the month names over the grid, and the
//! parts orangui's open-on-newest script reads, so the grid opens on its newest weeks with no
//! month name cut in half by the pinned weekday strip.

use dioxus::prelude::*;

use crate::{HeatScrollParts, heat_span};

/// The Activity heat grid's parts, as its classes name them: the scroller, the plot that takes
/// the room added at the far end, the pinned weekday strip, and a month name.
pub const ACTIVITY_HEAT_PARTS: HeatScrollParts = HeatScrollParts {
    scroll: ".activity-scroll",
    row: ".activity-plot",
    axis: ".activity-days",
    month: ".heat-month",
};

/// The month names over a heat grid: a `heat-label heat-month` text per month, at its first week's
/// column, `left` in from the grid's left edge, its baseline 4 units above `top`.
#[component]
pub fn HeatMonths(months: Vec<(usize, String)>, left: f64, top: f64) -> Element {
    rsx! {
        for (column, name) in months {
            text {
                key: "m{column}",
                class: "heat-label heat-month",
                x: "{left + heat_span(column)}",
                y: "{top - 4.0}",
                "{name}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_on_newest_js;

    #[test]
    fn the_months_draw_at_their_columns_marked_for_the_open_script() {
        fn app() -> Element {
            rsx! {
                svg {
                    HeatMonths {
                        months: vec![(0, "Jun".to_owned()), (4, "Jul".to_owned())],
                        left: 24.0,
                        top: 14.0,
                    }
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(
            html.matches("class=\"heat-label heat-month\"").count(),
            2,
            "{html}"
        );
        assert!(
            html.contains(&format!("x=\"{}\"", 24.0 + heat_span(4))),
            "{html}"
        );
    }

    #[test]
    fn activity_opens_on_its_newest_weeks_with_no_month_cut_by_the_weekdays() {
        let js = open_on_newest_js(ACTIVITY_HEAT_PARTS);
        for part in [
            "querySelectorAll('.activity-scroll')",
            "querySelector('.activity-plot')",
            "querySelector('.activity-days')",
            "querySelectorAll('.heat-month')",
        ] {
            assert!(js.contains(part), "{part}: {js}");
        }
    }
}
