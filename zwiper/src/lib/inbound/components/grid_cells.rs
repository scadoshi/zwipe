//! Shaded cells on the backdrop, ported from the portfolio's grid-cells branch: every grid square gets its own shade, so the
//! cells are the grid, with no lines drawn. Fixed like the old grid so they
//! stay put while the page scrolls, brightest at the top and fading out down
//! the viewport. They rain in row by row as soon as the page paints, on every
//! screen, and then hold; a third of them breathe a few percent over half a minute. The
//! shades are seeded, so it is the same field on every load.

use dioxus::prelude::*;

/// The virtual grid, in cells: wide and tall enough to cover a viewport at
/// the stylesheet's cell size. The layer clips whatever falls outside.
const COLUMNS: u32 = 80;
const ROWS: u32 = 44;
/// How far apart the rows arrive, so the field rains down from the top.
const RAIN_STEP_MS: u32 = 18;

struct Cell {
    column: u32,
    row: u32,
    shade: u32,
    rain_ms: u32,
    breathe_ms: Option<u32>,
}

/// The field, from a small linear congruential generator with a fixed
/// seed: each cell's shade, when it lands, and whether it breathes.
fn cells() -> Vec<Cell> {
    let mut state: u32 = 0x9e37_79b9;
    let mut next = move || {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        state >> 8
    };
    let mut out = Vec::with_capacity((ROWS * COLUMNS) as usize);
    for row in 0..ROWS {
        for column in 0..COLUMNS {
            let roll = next();
            out.push(Cell {
                column,
                row,
                shade: roll % 5 + 1,
                rain_ms: row * RAIN_STEP_MS + (roll >> 4) % 160,
                breathe_ms: (roll % 3 == 0).then_some((roll >> 6) % 20_000),
            });
        }
    }
    out
}

/// The field: a fixed layer of shaded cells under everything on screen.
#[component]
pub fn GridCells() -> Element {
    rsx! {
        div { class: "grid-cells", aria_hidden: "true",
            for cell in cells() {
                if let Some(breathe) = cell.breathe_ms {
                    span {
                        class: "grid-cell grid-cell-{cell.shade} grid-cell-breathes",
                        style: "--col: {cell.column}; --row: {cell.row}; animation-delay: {cell.rain_ms}ms, {breathe}ms",
                    }
                } else {
                    span {
                        class: "grid-cell grid-cell-{cell.shade}",
                        style: "--col: {cell.column}; --row: {cell.row}; animation-delay: {cell.rain_ms}ms",
                    }
                }
            }
        }
    }
}
