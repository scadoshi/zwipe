//! The app's backdrop of faint tiles, orangui's `grid-cells.js` drawn from orangui-core's `grid_rain`: every tile a seeded shade of the text color over the background, brightest at the top and fading down.
//!
//! The splash draws it raining in: each tile lands at `row * 18 ms` plus up to 160 ms of seeded scatter, falling three tiles over 350 ms with an ease-out.
//!
//! A tile fills four cells by two rows, about square since a cell is about twice as tall as it is wide, and the tiles meet with no gap, a mosaic of shades. The shade falls off with the square of the distance down the area, so the top is at full strength, the middle at a quarter and the bottom edge bare. Only the background changes, so text drawn over a tile stays as it is. A palette with no colors of its own draws no tiles.

use std::time::Duration;

use orangui_core::grid_rain;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::domain::theme::Palette;

/// Cells from one tile's left edge to the next.
const PITCH_X: u16 = 4;

/// Rows from one tile to the next.
const PITCH_Y: u16 = 2;

/// When the rain over an area `height` rows tall has landed.
#[cfg(test)]
fn rain_length(height: u16) -> Duration {
    let rows = u64::from(height.div_ceil(PITCH_Y));
    Duration::from_millis(
        rows.saturating_mul(u64::from(grid_rain::RAIN_STEP_MS))
            .saturating_add(u64::from(grid_rain::JITTER_MS))
            .saturating_add(u64::from(grid_rain::DROP_MS)),
    )
}

/// `color` mixed into `base` at `share`, both 24-bit; `None` otherwise.
fn mix(base: Color, color: Color, share: f64) -> Option<Color> {
    let (Color::Rgb(br, bg, bb), Color::Rgb(cr, cg, cb)) = (base, color) else {
        return None;
    };
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a mix of two channels stays within 0 to 255"
    )]
    let channel = |b: u8, c: u8| -> u8 {
        (f64::from(b) + (f64::from(c) - f64::from(b)) * share)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Some(Color::Rgb(
        channel(br, cr),
        channel(bg, cg),
        channel(bb, cb),
    ))
}

/// Draws the tiles over `area`: settled when `elapsed` is `None`, else raining in as of `elapsed`.
pub(crate) fn draw(f: &mut Frame<'_>, palette: &Palette, area: Rect, elapsed: Option<Duration>) {
    let (Some(base), Some(ink)) = (palette.base.bg, palette.base.fg) else {
        return;
    };
    if mix(base, ink, 0.0).is_none() || area.is_empty() {
        return;
    }
    let cols = area.width.div_ceil(PITCH_X);
    let rows = area.height.div_ceil(PITCH_Y);
    let height = f64::from(area.height.max(1));
    #[expect(
        clippy::as_conversions,
        clippy::cast_precision_loss,
        reason = "a few seconds of whole milliseconds is exact in f64"
    )]
    let elapsed_ms = elapsed.map(|t| t.as_millis() as f64);
    let width = usize::from(area.width);
    // Each cell's tile shade.
    let mut cells: Vec<Option<Color>> = vec![None; width.saturating_mul(usize::from(area.height))];
    for cell in grid_rain::cells(u32::from(cols), u32::from(rows)) {
        let (Ok(row), Ok(col)) = (u16::try_from(cell.row), u16::try_from(cell.column)) else {
            continue;
        };
        let progress = elapsed_ms.map_or(1.0, |ms| cell.progress(ms));
        if progress <= 0.0 {
            continue;
        }
        let eased = grid_rain::ease(progress);
        #[expect(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "the fall is at most three tiles of two rows and never below zero"
        )]
        let lift =
            ((1.0 - eased) * f64::from(grid_rain::DROP_CELLS) * f64::from(PITCH_Y)).round() as u16;
        let Some(top) = row.saturating_mul(PITCH_Y).checked_sub(lift) else {
            continue;
        };
        let fade = (1.0 - f64::from(top) / height).max(0.0).powi(2);
        let Some(shade) = mix(base, ink, cell.shade * eased * fade) else {
            continue;
        };
        for dy in 0..PITCH_Y {
            let y = top.saturating_add(dy);
            if y >= area.height {
                continue;
            }
            for dx in 0..PITCH_X {
                let x = col.saturating_mul(PITCH_X).saturating_add(dx);
                if x >= area.width {
                    continue;
                }
                let at = usize::from(y)
                    .saturating_mul(width)
                    .saturating_add(usize::from(x));
                if let Some(slot) = cells.get_mut(at) {
                    *slot = Some(shade);
                }
            }
        }
    }
    let buffer = f.buffer_mut();
    for (y, line) in (area.y..area.bottom()).zip(cells.chunks(width.max(1))) {
        for (x, &tile) in (area.x..area.right()).zip(line) {
            if let (Some(shade), Some(cell)) = (tile, buffer.cell_mut((x, y))) {
                cell.set_bg(shade);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    /// The backgrounds of a 40 by 12 area after the grid draws, row by row.
    fn backgrounds(palette: &Palette, elapsed: Option<Duration>) -> Vec<Vec<Color>> {
        let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
        terminal
            .draw(|f| {
                f.render_widget(ratatui::widgets::Block::new().style(palette.base), f.area());
                draw(f, palette, f.area(), elapsed);
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        (0..12)
            .map(|y| (0..40).map(|x| buffer.cell((x, y)).unwrap().bg).collect())
            .collect()
    }

    /// The cells tiles tint.
    fn tinted(palette: &Palette, elapsed: Option<Duration>) -> usize {
        let base = palette.base.bg.unwrap_or_default();
        backgrounds(palette, elapsed)
            .iter()
            .flatten()
            .filter(|&&bg| bg != base)
            .count()
    }

    /// How far a row's tiles sit from the background, summed over its cells and channels.
    fn strength(row: &[Color], base: Color) -> u32 {
        let Color::Rgb(br, bg, bb) = base else {
            return 0;
        };
        row.iter()
            .map(|&c| match c {
                Color::Rgb(r, g, b) => u32::from(r.abs_diff(br))
                    .saturating_add(u32::from(g.abs_diff(bg)))
                    .saturating_add(u32::from(b.abs_diff(bb))),
                _ => 0,
            })
            .sum()
    }

    #[test]
    fn the_still_grid_fills_the_top_and_fades_to_nothing_at_the_bottom() {
        let palette = Palette::for_account("gruvbox", true);
        let base = palette.base.bg.unwrap();
        let rows = backgrounds(&palette, None);
        assert!(
            rows[0].iter().all(|&bg| bg != base),
            "the top row is all tiles, no gaps"
        );
        let (top, middle, bottom) = (
            strength(&rows[0], base),
            strength(&rows[6], base),
            strength(&rows[11], base),
        );
        assert!(
            top > middle && middle > bottom,
            "{top} > {middle} > {bottom}"
        );
        assert!(
            bottom * 10 < top,
            "the bottom is all but bare: {bottom} of {top}"
        );
    }

    #[test]
    fn the_rain_starts_empty_and_lands_on_the_still_grid() {
        let palette = Palette::for_account("gruvbox", true);
        let still = backgrounds(&palette, None);
        assert_eq!(tinted(&palette, Some(Duration::ZERO)), 0);
        assert!(tinted(&palette, Some(Duration::from_millis(200))) < tinted(&palette, None));
        assert_eq!(backgrounds(&palette, Some(rain_length(12))), still);
    }

    #[test]
    fn the_terminals_own_colors_draw_no_tiles_and_a_tiny_area_is_fine() {
        assert_eq!(tinted(&Palette::terminal(), None), 0);
        let mut terminal = Terminal::new(TestBackend::new(1, 1)).unwrap();
        terminal
            .draw(|f| draw(f, &Palette::for_account("nord", true), f.area(), None))
            .unwrap();
    }

    #[test]
    fn mixing_moves_a_share_of_the_way() {
        assert_eq!(
            mix(Color::Rgb(0, 0, 0), Color::Rgb(200, 100, 0), 0.5),
            Some(Color::Rgb(100, 50, 0))
        );
        assert_eq!(mix(Color::Reset, Color::Rgb(1, 1, 1), 0.5), None);
    }
}
