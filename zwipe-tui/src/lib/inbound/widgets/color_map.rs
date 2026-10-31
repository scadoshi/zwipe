//! The color map drawn in cells: each tile two cells wide so it reads square, shaded toward the background by its distance from the middle, sweeping in ring by ring from the middle out.

use std::time::Duration;

use ratatui::style::{Color as TermColor, Style, Stylize};
use ratatui::text::{Line, Span};
use zwipe_core::domain::card::scryfall_data::colors::Color;

use super::mana;
use crate::domain::color_map::{
    ColorMapCounts, ColorMapModel, FULL_SHADE, SWEEP_RINGS, SWEEP_STEP_MS, shade_percent,
};
use crate::domain::count_up;
use crate::domain::theme::Palette;

/// One tile's cells.
const TILE: &str = "██";

/// A ghost tile's cells.
const GHOST_TILE: &str = "▒▒";

/// How long one tile takes to sweep in, as the app's `heat-sweep`.
const SWEEP: Duration = Duration::from_millis(400);

/// One bloom of the loading ghost, out and back.
const BLOOM: Duration = Duration::from_millis(2600);

/// How long the whole entrance runs: the last ring's delay plus one sweep.
pub(crate) fn entrance() -> Duration {
    Duration::from_millis(u64::from(SWEEP_STEP_MS.saturating_mul(SWEEP_RINGS)))
        .saturating_add(SWEEP)
}

/// `top` mixed over `bottom` at `percent`.
fn mix(top: TermColor, bottom: Option<TermColor>, percent: u32) -> TermColor {
    let percent = percent.min(100);
    match (top, bottom) {
        (TermColor::Rgb(r, g, b), Some(TermColor::Rgb(br, bg, bb))) => {
            let channel = |c: u8, under: u8| {
                let mixed = (u32::from(c) * percent + u32::from(under) * (100 - percent)) / 100;
                u8::try_from(mixed).unwrap_or(c)
            };
            TermColor::Rgb(channel(r, br), channel(g, bg), channel(b, bb))
        }
        (color, _) => color,
    }
}

/// The style of a tile of the color at `index` drawn at `percent` of its mana color over the background. A terminal palette without true colors dims the fainter shades instead.
pub(crate) fn tile_style(palette: &Palette, index: usize, percent: u32) -> Style {
    let style = mana::style(palette, index);
    let Some(fg) = style.fg else {
        return style;
    };
    let blended = Style::new().fg(mix(fg, palette.base.bg, percent));
    if matches!(fg, TermColor::Rgb(..)) || percent >= 80 {
        blended
    } else {
        blended.dim()
    }
}

/// How far into its sweep a tile of `ring` is at `elapsed`, in percent; 100 once settled.
fn swept(ring: u32, elapsed: Duration) -> u32 {
    let delay = Duration::from_millis(u64::from(ring.saturating_mul(SWEEP_STEP_MS)));
    let Some(into) = elapsed.checked_sub(delay) else {
        return 0;
    };
    let percent = into.as_millis().saturating_mul(100) / SWEEP.as_millis().max(1);
    u32::try_from(percent.min(100)).unwrap_or(100)
}

/// The window of `count` cells to show out of `total`, centered on the middle.
fn window(total: i32, count: i32) -> (i32, i32) {
    if total <= count {
        return (0, total);
    }
    let start = (total - count) / 2;
    (start, start + count)
}

/// The map's rows as lines, at most `width` cells wide and `height` rows tall, cropped around the middle. `elapsed` is the time since the map appeared, which drives the sweep.
pub(crate) fn lines(
    palette: &Palette,
    model: &ColorMapModel,
    elapsed: Duration,
    width: u16,
    height: u16,
) -> Vec<Line<'static>> {
    let (left, right) = window(model.columns, i32::from(width / 2));
    let (top, bottom) = window(model.rows, i32::from(height));
    (top..bottom)
        .map(|row| {
            let spans: Vec<Span<'static>> = (left..right)
                .map(|column| {
                    let tile = model
                        .tiles
                        .iter()
                        .find(|t| t.row == row && t.column == column);
                    let Some(tile) = tile else {
                        return Span::raw("  ");
                    };
                    let progress = swept(tile.ring, elapsed);
                    if progress == 0 {
                        return Span::raw("  ");
                    }
                    let index = Color::all()
                        .iter()
                        .position(|c| *c == tile.color)
                        .unwrap_or_default();
                    let percent = u32::from(shade_percent(tile.shade)) * progress / 100;
                    Span::styled(TILE, tile_style(palette, index, percent.max(10)))
                })
                .collect();
            Line::from(spans)
        })
        .collect()
}

/// The loading ghost: the ghost model in the muted color, its rings blooming out and folding back on a loop.
pub(crate) fn ghost_lines(
    palette: &Palette,
    model: &ColorMapModel,
    elapsed: Duration,
    width: u16,
    height: u16,
) -> Vec<Line<'static>> {
    let phase = elapsed.as_millis() % BLOOM.as_millis().max(1);
    let half = BLOOM.as_millis() / 2;
    let reach = if phase < half {
        phase
    } else {
        BLOOM.as_millis() - phase
    };
    let rings = u128::from(SWEEP_RINGS) * reach / half.max(1);
    let (left, right) = window(model.columns, i32::from(width / 2));
    let (top, bottom) = window(model.rows, i32::from(height));
    (top..bottom)
        .map(|row| {
            let spans: Vec<Span<'static>> = (left..right)
                .map(|column| {
                    model
                        .tiles
                        .iter()
                        .find(|t| t.row == row && t.column == column)
                        .filter(|t| u128::from(t.ring) <= rings)
                        .map_or_else(
                            || Span::raw("  "),
                            |t| {
                                let style = if t.shade >= FULL_SHADE.saturating_sub(1) {
                                    palette.muted
                                } else {
                                    palette.muted.dim()
                                };
                                Span::styled(GHOST_TILE, style)
                            },
                        )
                })
                .collect();
            Line::from(spans)
        })
        .collect()
}

/// The key under the map: each color's letter in its mana color and its count, then the colorless count, then `action` when given.
pub(crate) fn key(
    palette: &Palette,
    counts: &ColorMapCounts,
    action: Option<(&str, &str)>,
) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    for (i, n) in counts.cards.iter().enumerate() {
        spans.push(mana::pip(palette, i));
        spans.push(Span::raw(format!(
            " {}  ",
            count_up::with_separators(u64::from(*n))
        )));
    }
    spans.push(Span::styled("Colorless", palette.muted));
    spans.push(Span::raw(format!(
        " {}",
        count_up::with_separators(u64::from(counts.colorless))
    )));
    if let Some((letter, label)) = action {
        spans.push(Span::raw("  "));
        spans.push(Span::raw(letter.to_owned()).bold());
        spans.push(Span::styled(format!(" {label}"), palette.accent));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::color_map::ghost_model;

    fn counts() -> ColorMapCounts {
        ColorMapCounts {
            cards: [30, 12, 0, 8, 50],
            colorless: 4,
        }
    }

    #[test]
    fn a_settled_map_draws_every_tile_two_cells_wide() {
        let model = ColorMapModel::new(&counts(), 7);
        let lines = lines(&Palette::terminal(), &model, entrance(), 200, 200);
        assert_eq!(lines.len(), usize::try_from(model.rows).unwrap());
        let tiles: usize = lines
            .iter()
            .map(|l| l.to_string().matches(TILE).count())
            .sum();
        assert_eq!(tiles, model.tiles.len());
        assert!(
            lines
                .iter()
                .all(|l| l.width() == usize::try_from(model.columns * 2).unwrap())
        );
    }

    #[test]
    fn the_map_sweeps_in_from_the_middle() {
        let model = ColorMapModel::new(&counts(), 7);
        let drawn = |ms| -> usize {
            lines(
                &Palette::terminal(),
                &model,
                Duration::from_millis(ms),
                200,
                200,
            )
            .iter()
            .map(|l| l.to_string().matches(TILE).count())
            .sum()
        };
        assert_eq!(drawn(0), 0);
        assert!(drawn(60) < drawn(200));
        assert!(drawn(200) < model.tiles.len());
        assert_eq!(drawn(1_100), model.tiles.len());
    }

    #[test]
    fn a_large_map_crops_around_its_middle() {
        let model = ColorMapModel::new(&counts(), 7);
        let lines = lines(&Palette::terminal(), &model, entrance(), 10, 3);
        assert_eq!(lines.len(), 3);
        assert!(lines.iter().all(|l| l.width() <= 10));
        assert_eq!(window(9, 4), (2, 6));
        assert_eq!(window(3, 4), (0, 3));
    }

    #[test]
    fn shades_mix_toward_the_background() {
        let palette = Palette::for_account("gruvbox", true);
        let full = tile_style(&palette, 0, 100);
        let faint = tile_style(&palette, 0, 30);
        assert_eq!(full.fg, palette.mana[0].fg);
        assert_ne!(faint.fg, full.fg);
    }

    #[test]
    fn the_key_counts_every_color_and_the_action() {
        let key = key(&Palette::terminal(), &counts(), Some(("p", "Playstyle")));
        assert_eq!(
            key.to_string(),
            "W 30  U 12  B 0  R 8  G 50  Colorless 4  p Playstyle"
        );
    }

    #[test]
    fn the_ghost_blooms_and_folds() {
        let model = ghost_model();
        let count = |ms| -> usize {
            ghost_lines(
                &Palette::terminal(),
                &model,
                Duration::from_millis(ms),
                200,
                200,
            )
            .iter()
            .map(|l| l.to_string().matches(GHOST_TILE).count())
            .sum()
        };
        assert!(count(0) < count(1_300));
        assert_eq!(count(1_300), model.tiles.len());
        assert!(count(2_500) < count(1_300));
    }
}
