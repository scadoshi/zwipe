//! The app's backdrop of faint tiles, ported from orangui's `grid-cells.js`: every tile a seeded shade of the text color over the background, brightest at the top and fading down.
//!
//! The splash draws it raining in: each tile lands at `row * 18 ms` plus up to 160 ms of seeded scatter, falling three tiles over 350 ms with an ease-out. A tile here is three cells wide on every other row, four cells apart, which reads as the app's 28-pixel squares at a terminal's cell size. Only the background changes, so text drawn over a tile stays as it is; a palette with no colors of its own draws no tiles.

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::domain::theme::Palette;

/// Cells from one tile's left edge to the next.
const PITCH_X: u16 = 4;

/// Rows from one tile to the next.
const PITCH_Y: u16 = 2;

/// A tile's width in cells.
const TILE_WIDTH: u16 = 3;

/// How far apart the rows land, in milliseconds.
const RAIN_STEP_MS: u64 = 18;

/// Each tile's scatter inside its row, in milliseconds.
const JITTER_MS: u64 = 160;

/// One tile's fall, in milliseconds.
const DROP_MS: f64 = 350.0;

/// How many tiles a tile falls.
const DROP_TILES: f64 = 3.0;

/// The tile shades, as shares of the text color.
const SHADES: [f64; 5] = [0.015, 0.03, 0.05, 0.075, 0.1];

/// When the rain is over on a screen `rows` tiles tall: the last row's latest landing.
#[cfg(test)]
pub(crate) fn rain_length(height: u16) -> Duration {
    let rows = u64::from(height.div_ceil(PITCH_Y));
    Duration::from_millis(
        rows.saturating_mul(RAIN_STEP_MS)
            .saturating_add(JITTER_MS)
            .saturating_add(DROP_MS as u64),
    )
}

/// The seeded sequence `grid-cells.js` rolls each tile from.
struct Lcg(u32);

impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        self.0 >> 8
    }
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
    let mut lcg = Lcg(0x9e37_79b9);
    let height = f64::from(area.height.max(1));
    let buffer = f.buffer_mut();
    for row in 0..rows {
        for col in 0..cols {
            let roll = lcg.next();
            let shade = SHADES
                .get(usize::try_from(roll % 5).unwrap_or(0))
                .copied()
                .unwrap_or(0.0);
            let at = u64::from(row)
                .saturating_mul(RAIN_STEP_MS)
                .saturating_add(u64::from(roll >> 4) % JITTER_MS);
            let progress = elapsed.map_or(1.0, |t| {
                let since = t.as_millis().saturating_sub(u128::from(at));
                #[expect(
                    clippy::as_conversions,
                    clippy::cast_precision_loss,
                    reason = "a few seconds of milliseconds is exact in f64"
                )]
                let since = since as f64;
                if t.as_millis() < u128::from(at) {
                    0.0
                } else {
                    (since / DROP_MS).min(1.0)
                }
            });
            if progress <= 0.0 {
                continue;
            }
            let eased = 1.0 - (1.0 - progress).powi(3);
            #[expect(
                clippy::as_conversions,
                clippy::cast_possible_truncation,
                reason = "the fall is at most three tiles of two rows"
            )]
            let lift = ((1.0 - eased) * DROP_TILES * f64::from(PITCH_Y)).round() as u16;
            let Some(y) = row
                .saturating_mul(PITCH_Y)
                .checked_sub(lift)
                .filter(|_| row.saturating_mul(PITCH_Y) >= lift)
            else {
                continue;
            };
            let fade = 1.0 - f64::from(y) / height * 0.75;
            let Some(tile) = mix(base, ink, shade * eased * fade) else {
                continue;
            };
            for dx in 0..TILE_WIDTH {
                let x = area
                    .x
                    .saturating_add(col.saturating_mul(PITCH_X))
                    .saturating_add(dx);
                let y = area.y.saturating_add(y);
                if x < area.right()
                    && y < area.bottom()
                    && let Some(cell) = buffer.cell_mut((x, y))
                {
                    cell.set_bg(tile);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    fn tinted(palette: &Palette, elapsed: Option<Duration>) -> usize {
        let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
        terminal
            .draw(|f| {
                f.render_widget(ratatui::widgets::Block::new().style(palette.base), f.area());
                draw(f, palette, f.area(), elapsed);
            })
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .filter(|c| c.bg != palette.base.bg.unwrap_or_default())
            .count()
    }

    #[test]
    fn the_still_grid_tints_three_cells_of_each_tile() {
        let palette = Palette::for_account("gruvbox", true);
        assert_eq!(tinted(&palette, None), 10 * 6 * 3);
    }

    #[test]
    fn the_rain_starts_empty_and_lands_every_tile() {
        let palette = Palette::for_account("gruvbox", true);
        assert_eq!(tinted(&palette, Some(Duration::ZERO)), 0);
        assert!(tinted(&palette, Some(Duration::from_millis(200))) < 10 * 6 * 3);
        assert_eq!(tinted(&palette, Some(rain_length(12))), 10 * 6 * 3);
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
