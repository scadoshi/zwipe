//! The contribution heatmap's grid: a cell per day, a column per week, each
//! shaded by its level, the year's peaks lit, the columns sweeping in one
//! after another. The host owns the SVG and draws its labels and charts
//! around the grid; it hands over the cells and the grid's origin and gets
//! back a [`HeatHit`] per pointer event to place its own chip. The look is
//! the `.heat-*` rules in `assets/components.css`.

use dioxus::prelude::*;

use crate::entrance::sleep_ms;

/// A cell's side and the gap to the next, in SVG units.
pub const HEAT_CELL: f64 = 11.0;
pub const HEAT_GAP: f64 = 2.0;
/// From one cell's edge to the next cell's.
pub const HEAT_STEP: f64 = HEAT_CELL + HEAT_GAP;
/// Rows in the grid, one per weekday.
pub const HEAT_ROWS: usize = 7;
/// How far apart the columns arrive in the sweep. The sweep itself is in the
/// stylesheet.
const SWEEP_STEP_MS: usize = 12;
/// How far the halo reaches past its cell on each side.
const HALO_REACH: f64 = 2.5;

/// One day on the grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeatCell {
    /// Week, oldest first.
    pub column: usize,
    /// Weekday, from the top.
    pub row: usize,
    /// 0 for an empty day up to 4 for the fullest shade; anything past 4
    /// draws as 4.
    pub level: u8,
    /// One of the year's outlier days: a lit edge and a glow behind.
    pub peak: bool,
    /// Tells the cell apart from the others, such as its date.
    pub key: String,
}

/// A cell the pointer reached: its index among the grid's cells and its
/// top-left corner in SVG units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeatHit {
    pub index: usize,
    pub x: f64,
    pub y: f64,
}

/// How far `n` columns or rows reach, in SVG units: a grid `columns` wide is
/// `heat_span(columns)` past its origin, and `heat_span(HEAT_ROWS)` tall.
#[allow(clippy::cast_precision_loss)]
pub fn heat_span(n: usize) -> f64 {
    n as f64 * HEAT_STEP
}

/// The grid's cells, inside the host's `svg`, with their top-left corner at
/// (`left`, `top`). `on_enter` fires as the pointer reaches a cell and
/// `on_tap` on a click or tap; a host without one leaves it out.
#[component]
pub fn HeatGrid(
    cells: Vec<HeatCell>,
    left: f64,
    top: f64,
    #[props(default)] on_enter: Option<EventHandler<HeatHit>>,
    #[props(default)] on_tap: Option<EventHandler<HeatHit>>,
) -> Element {
    rsx! {
        for (index, cell) in cells.iter().enumerate() {
            {
                let x = left + heat_span(cell.column);
                let y = top + heat_span(cell.row);
                let hit = HeatHit { index, x, y };
                let delay = cell.column * SWEEP_STEP_MS;
                let level = cell.level.min(4);
                rsx! {
                    g { key: "{cell.key}",
                        // The glow is a shape of its own rather than a filter:
                        // iOS Safari applies no CSS filter to an SVG child.
                        if cell.peak {
                            rect {
                                class: "heat-halo",
                                style: "animation-delay: {delay}ms",
                                x: "{x - HALO_REACH}",
                                y: "{y - HALO_REACH}",
                                width: "{HEAT_CELL + 2.0 * HALO_REACH}",
                                height: "{HEAT_CELL + 2.0 * HALO_REACH}",
                                rx: "4",
                            }
                        }
                        rect {
                            class: if cell.peak { "heat-cell heat-{level} heat-peak" } else { "heat-cell heat-{level}" },
                            style: "animation-delay: {delay}ms",
                            x: "{x}",
                            y: "{y}",
                            width: "{HEAT_CELL}",
                            height: "{HEAT_CELL}",
                            rx: "2",
                            onmouseenter: move |_| {
                                if let Some(on_enter) = on_enter {
                                    on_enter.call(hit);
                                }
                            },
                            onclick: move |_| {
                                if let Some(on_tap) = on_tap {
                                    on_tap.call(hit);
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

/// Scrolls every `.scroll-end` box to its right edge once the page is up, so
/// a chart wider than a phone opens on its newest columns. Waits a beat
/// after mount, for the app's WebView to have laid the box out.
pub fn use_scroll_to_end() {
    use_effect(|| {
        spawn(async {
            sleep_ms(60).await;
            let _ = document::eval(
                "for (const el of document.querySelectorAll('.scroll-end')) el.scrollLeft = el.scrollWidth;",
            )
            .await;
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_span_counts_cells_and_gaps() {
        assert!((heat_span(0)).abs() < f64::EPSILON);
        assert!((heat_span(1) - 13.0).abs() < f64::EPSILON);
        assert!((heat_span(HEAT_ROWS) - 91.0).abs() < f64::EPSILON);
    }
}
