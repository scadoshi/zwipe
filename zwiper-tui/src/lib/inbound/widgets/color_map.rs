//! The color map drawn in cells: each tile two cells wide so it reads square, shaded toward the background by its distance from the middle, sweeping in ring by ring from the middle out.

use std::time::Duration;

use orangui_core::number;
use ratatui::style::{Color as TermColor, Style, Stylize};
use ratatui::text::{Line, Span};
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_ui::color_map::{
    ColorMapCounts, ColorMapModel, FULL_SHADE, SWEEP_RINGS, SWEEP_STEP_MS, Tile, shade_percent,
};

use super::mana;
use crate::domain::theme::Palette;

/// Joins the parts of one key chip so a wrapping key breaks only between chips, as the app's flex-wrapped chips do.
const NBSP: &str = "\u{a0}";

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

/// `top` mixed over `bottom` at `percent`, rounded per channel as CSS `color-mix(in srgb, ..)` does.
pub(crate) fn mix(top: TermColor, bottom: Option<TermColor>, percent: u32) -> TermColor {
    let percent = percent.min(100);
    match (top, bottom) {
        (TermColor::Rgb(r, g, b), Some(TermColor::Rgb(br, bg, bb))) => {
            let channel = |c: u8, under: u8| {
                let mixed = u32::from(c)
                    .saturating_mul(percent)
                    .saturating_add(
                        u32::from(under).saturating_mul(100_u32.saturating_sub(percent)),
                    )
                    .saturating_add(50)
                    / 100;
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

/// How far into its `heat-sweep` a tile of `ring` is at `elapsed`, in percent, eased out as the app's 0.4 s `ease-out`; 100 once settled.
fn swept(ring: u32, elapsed: Duration) -> u32 {
    let delay = Duration::from_millis(u64::from(ring.saturating_mul(SWEEP_STEP_MS)));
    let Some(into) = elapsed.checked_sub(delay) else {
        return 0;
    };
    let linear = into
        .as_millis()
        .saturating_mul(100)
        .checked_div(SWEEP.as_millis())
        .unwrap_or(100);
    let left = 100_u128.saturating_sub(linear.min(100));
    let eased = 100_u128.saturating_sub(left.saturating_mul(left) / 100);
    u32::try_from(eased).unwrap_or(100)
}

/// The full-bleed rule the app draws over the key, `width` cells wide.
pub(crate) fn rule(palette: &Palette, width: usize) -> Line<'static> {
    Line::styled("─".repeat(width), palette.border)
}

/// The window of `count` cells to show out of `total`, centered on the middle.
fn window(total: i32, count: i32) -> (i32, i32) {
    if total <= count {
        return (0, total);
    }
    let start = total.saturating_sub(count) / 2;
    (start, start.saturating_add(count))
}

/// How many cells a tile takes on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scale {
    /// Two cells wide and one row tall, so a tile reads square.
    Full,
    /// One cell wide and half a row tall, two tiles to a cell drawn with half blocks: the same shape at half the size, for a map too big for its room.
    Half,
}

impl Scale {
    /// The scale `model` draws at in `width` by `height` cells: full when the whole map fits, half otherwise.
    pub(crate) fn fitting(model: &ColorMapModel, width: u16, height: u16) -> Self {
        let fits =
            model.columns.saturating_mul(2) <= i32::from(width) && model.rows <= i32::from(height);
        if fits { Self::Full } else { Self::Half }
    }

    /// The rows `model` takes at this scale.
    pub(crate) fn rows(self, model: &ColorMapModel) -> u16 {
        let rows = match self {
            Self::Full => model.rows,
            Self::Half => model.rows.saturating_add(1) / 2,
        };
        u16::try_from(rows.max(0)).unwrap_or(u16::MAX)
    }
}

/// The map drawn at `scale`, at most `width` cells wide and `height` rows tall, cropped around the middle. `paint` gives a tile's style, its color in the foreground, or `None` to leave it out; `full` is a full-scale tile's two cells.
fn grid(
    model: &ColorMapModel,
    scale: Scale,
    width: u16,
    height: u16,
    full: &'static str,
    paint: impl Fn(&Tile) -> Option<Style>,
) -> Vec<Line<'static>> {
    let style_at = |row: i32, column: i32| {
        model
            .tiles
            .iter()
            .find(|t| t.row == row && t.column == column)
            .and_then(&paint)
    };
    match scale {
        Scale::Full => {
            let (left, right) = window(model.columns, i32::from(width / 2));
            let (top, bottom) = window(model.rows, i32::from(height));
            (top..bottom)
                .map(|row| {
                    let spans: Vec<Span<'static>> = (left..right)
                        .map(|column| {
                            style_at(row, column)
                                .map_or_else(|| Span::raw("  "), |style| Span::styled(full, style))
                        })
                        .collect();
                    Line::from(spans)
                })
                .collect()
        }
        Scale::Half => {
            let (left, right) = window(model.columns, i32::from(width));
            let pairs = model.rows.saturating_add(1) / 2;
            let (top, bottom) = window(pairs, i32::from(height));
            (top..bottom)
                .map(|pair| {
                    let upper_row = pair.saturating_mul(2);
                    let spans: Vec<Span<'static>> = (left..right)
                        .map(|column| {
                            let upper = style_at(upper_row, column);
                            let lower = style_at(upper_row.saturating_add(1), column);
                            half_cell(upper, lower)
                        })
                        .collect();
                    Line::from(spans)
                })
                .collect()
        }
    }
}

/// One half-scale cell holding the tile above in its top half and the tile below in its bottom half.
fn half_cell(upper: Option<Style>, lower: Option<Style>) -> Span<'static> {
    match (upper, lower) {
        (Some(upper), Some(lower)) => {
            let style = match lower.fg {
                Some(color) => upper.bg(color),
                None => upper,
            };
            Span::styled("▀", style)
        }
        (Some(upper), None) => Span::styled("▀", upper),
        (None, Some(lower)) => Span::styled("▄", lower),
        (None, None) => Span::raw(" "),
    }
}

/// The map's rows as lines, at most `width` cells wide and `height` rows tall: full scale when the map fits, half scale when it does not, cropped around the middle if even that is too big. `elapsed` is the time since the map appeared, which drives the sweep.
pub(crate) fn lines(
    palette: &Palette,
    model: &ColorMapModel,
    elapsed: Duration,
    width: u16,
    height: u16,
) -> Vec<Line<'static>> {
    let scale = Scale::fitting(model, width, height);
    grid(model, scale, width, height, TILE, |tile| {
        let progress = swept(tile.ring, elapsed);
        if progress == 0 {
            return None;
        }
        let index = Color::all()
            .iter()
            .position(|c| *c == tile.color)
            .unwrap_or_default();
        let percent = u32::from(shade_percent(tile.shade)).saturating_mul(progress) / 100;
        Some(tile_style(palette, index, percent.max(10)))
    })
}

/// The loading ghost: the ghost model in the muted color, its rings blooming out and folding back on a loop, at the scale [`lines`] would pick.
pub(crate) fn ghost_lines(
    palette: &Palette,
    model: &ColorMapModel,
    elapsed: Duration,
    width: u16,
    height: u16,
) -> Vec<Line<'static>> {
    let phase = elapsed
        .as_millis()
        .checked_rem(BLOOM.as_millis())
        .unwrap_or(0);
    let half = BLOOM.as_millis() / 2;
    let reach = if phase < half {
        phase
    } else {
        BLOOM.as_millis().saturating_sub(phase)
    };
    let rings = u128::from(SWEEP_RINGS)
        .saturating_mul(reach)
        .checked_div(half)
        .unwrap_or(0);
    let scale = Scale::fitting(model, width, height);
    grid(model, scale, width, height, GHOST_TILE, |tile| {
        (u128::from(tile.ring) <= rings).then(|| {
            if tile.shade >= FULL_SHADE.saturating_sub(1) {
                palette.muted
            } else {
                palette.muted.dim()
            }
        })
    })
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
            "{NBSP}{}  ",
            number::with_separators(u64::from(*n))
        )));
    }
    spans.push(Span::styled("C", palette.muted.bold()));
    spans.push(Span::raw(NBSP));
    spans.push(Span::styled("Colorless", palette.muted));
    spans.push(Span::raw(format!(
        "{NBSP}{}",
        number::with_separators(u64::from(counts.colorless))
    )));
    if let Some((letter, label)) = action {
        spans.push(Span::raw("  "));
        spans.push(Span::raw(letter.to_owned()).bold());
        spans.push(Span::styled(format!("{NBSP}{label}"), palette.accent));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use zwipe_ui::color_map::ghost_model;

    use super::*;

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

    /// The tiles `lines` show: two cells of a full-scale tile make one, and a half-scale cell holds one or two.
    fn tiles_in(lines: &[Line<'_>]) -> usize {
        lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .map(|span| match span.content.as_ref() {
                TILE => 1,
                "▀" if span.style.bg.is_some() => 2,
                "▀" | "▄" => 1,
                _ => 0,
            })
            .sum()
    }

    #[test]
    fn a_map_too_tall_for_its_room_draws_whole_at_half_scale() {
        let palette = Palette::for_account("gruvbox", true);
        let model = ColorMapModel::new(&counts(), 7);
        let short = u16::try_from(model.rows - 1).unwrap();
        assert_eq!(Scale::fitting(&model, 200, short), Scale::Half);
        assert_eq!(
            Scale::fitting(&model, 200, u16::try_from(model.rows).unwrap()),
            Scale::Full
        );
        let half = lines(&palette, &model, entrance(), 200, short);
        assert_eq!(
            half.len(),
            usize::from(Scale::Half.rows(&model)),
            "two tile rows to a line"
        );
        assert!(
            half.iter()
                .all(|l| l.width() == usize::try_from(model.columns).unwrap()),
            "one cell a tile"
        );
        assert_eq!(tiles_in(&half), model.tiles.len(), "every tile is there");
        let full = lines(&palette, &model, entrance(), 200, 200);
        assert_eq!(tiles_in(&full), model.tiles.len());
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
            key.to_string().replace(NBSP, " "),
            "W 30  U 12  B 0  R 8  G 50  C Colorless 4  p Playstyle"
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
