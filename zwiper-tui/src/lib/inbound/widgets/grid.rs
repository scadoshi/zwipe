//! The app's backdrop of faint tiles, orangui's `grid-cells.js` drawn from orangui-core's `grid_rain`: every tile a seeded shade of the text color over the background, faded down the screen by the app's mask.
//!
//! The app draws it settled behind every screen, right over the theme's background, and the splash draws it raining in: each tile lands at `row * 18 ms` plus up to 160 ms of seeded scatter, falling three tiles over 350 ms with an ease-out.
//!
//! A tile fills four cells by two rows, about square since a cell is about twice as tall as it is wide, and the tiles meet with no gap, a mosaic of shades. Only the background changes, so text drawn over a tile stays as it is. Every box, on the grid or over it, fills to its outer edge with the theme's background and sits on the tiles as a solid rectangle; loose text draws with no background of its own, so the tiles show up to its letters. A palette with no colors of its own draws no tiles.

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

/// Where orangui's mask (`.grid-cells-canvas` in `assets/components.css`) has faded to [`MASK_MIDDLE`], as a share of the screen's height.
const MASK_KNEE: f64 = 0.45;

/// The mask's strength at [`MASK_KNEE`]; it is full at the top and gone at the bottom.
const MASK_MIDDLE: f64 = 0.45;

/// How much of a tile shows at `fraction` of the way down the screen: orangui's mask, `linear-gradient(to bottom, #000 0, rgba(0, 0, 0, 0.45) 45vh, transparent 100vh)`, falling in a straight line from full to [`MASK_MIDDLE`] at [`MASK_KNEE`], then to nothing at the bottom.
fn mask(fraction: f64) -> f64 {
    let fraction = fraction.clamp(0.0, 1.0);
    if fraction <= MASK_KNEE {
        1.0 - (1.0 - MASK_MIDDLE) * fraction / MASK_KNEE
    } else {
        MASK_MIDDLE * (1.0 - (fraction - MASK_KNEE) / (1.0 - MASK_KNEE))
    }
}

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
    let width = usize::from(area.width);
    let cells = shades(palette, area, elapsed);
    let buffer = f.buffer_mut();
    for (y, line) in (area.y..area.bottom()).zip(cells.chunks(width.max(1))) {
        for (x, &tile) in (area.x..area.right()).zip(line) {
            if let (Some(shade), Some(cell)) = (tile, buffer.cell_mut((x, y))) {
                cell.set_bg(shade);
            }
        }
    }
}

/// Each cell's tile shade over `area`, row by row, settled when `elapsed` is `None`, else as of `elapsed` into the rain. Empty for a palette with no colors of its own.
pub(crate) fn shades(
    palette: &Palette,
    area: Rect,
    elapsed: Option<Duration>,
) -> Vec<Option<Color>> {
    let (Some(base), Some(ink)) = (palette.base.bg, palette.base.fg) else {
        return Vec::new();
    };
    if mix(base, ink, 0.0).is_none() || area.is_empty() {
        return Vec::new();
    }
    let cols = area.width.div_ceil(PITCH_X);
    let rows = area.height.div_ceil(PITCH_Y);
    #[expect(
        clippy::as_conversions,
        clippy::cast_precision_loss,
        reason = "a few seconds of whole milliseconds is exact in f64"
    )]
    let elapsed_ms = elapsed.map(|t| t.as_millis() as f64);
    let width = usize::from(area.width);
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
        for dy in 0..PITCH_Y {
            let y = top.saturating_add(dy);
            if y >= area.height {
                continue;
            }
            // The mask is read at the middle of the cell's row, as the canvas reads it per pixel.
            let down = (f64::from(y) + 0.5) / f64::from(area.height);
            let Some(shade) = mix(base, ink, cell.shade * eased * mask(down)) else {
                continue;
            };
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
    cells
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
    fn the_mask_follows_orangui_full_at_the_top_045_at_45_percent_and_gone_at_the_bottom() {
        assert!((mask(0.0) - 1.0).abs() < 1e-9);
        assert!((mask(MASK_KNEE) - MASK_MIDDLE).abs() < 1e-9);
        assert!(mask(1.0).abs() < 1e-9);
        assert!((mask(0.225) - 0.725).abs() < 1e-9, "halfway to the knee");
        assert!(mask(0.3) > mask(0.6) && mask(0.6) > mask(0.9));
    }

    #[test]
    fn the_still_grid_fills_the_top_and_fades_down_the_screen() {
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

    #[test]
    fn a_box_on_the_grid_is_solid_and_square_while_loose_text_sits_on_its_tile() {
        let palette = Palette::for_account("gruvbox", true);
        let base = palette.base.bg.unwrap();
        let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
        terminal
            .draw(|f| {
                f.render_widget(ratatui::widgets::Block::new().style(palette.base), f.area());
                draw(f, &palette, f.area(), None);
                f.render_widget(
                    ratatui::widgets::Paragraph::new("Decks").block(
                        super::super::panel::bordered(&palette).border_style(palette.border),
                    ),
                    Rect::new(4, 1, 20, 6),
                );
                f.render_widget(
                    ratatui::text::Line::styled("Loose", palette.text()),
                    Rect::new(26, 2, 5, 1),
                );
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        let corners = [(4, 1, "┌"), (23, 1, "┐"), (4, 6, "└"), (23, 6, "┘")];
        for (x, y, glyph) in corners {
            assert_eq!(buffer[(x, y)].symbol(), glyph, "a square corner at {x},{y}");
        }
        for (x, y) in [(4, 1), (10, 1), (4, 4), (23, 6), (5, 2), (12, 4), (22, 5)] {
            assert_eq!(
                buffer[(x, y)].bg,
                base,
                "{x},{y} holds the plain background"
            );
        }
        assert_eq!(buffer[(5, 2)].symbol(), "D");
        assert_eq!(
            buffer[(5, 2)].fg,
            palette.base.fg.unwrap(),
            "text in the text color"
        );
        let tiles = shades(&palette, buffer.area, None);
        let tile = tiles[2 * 40 + 26].unwrap();
        assert_ne!(tile, base, "a tile shows beside the box");
        assert_eq!(buffer[(26, 2)].symbol(), "L");
        assert_eq!(buffer[(26, 2)].bg, tile, "loose text sits on its tile");
    }
}
