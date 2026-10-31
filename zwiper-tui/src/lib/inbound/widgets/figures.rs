//! The stats strip of home, Login and Activity: three figures side by side, each a bold rolling number over its label, in orangui's compact form (`28.0k`). Home and Login box each figure in its own tile, as the app's `StatsStrip` does; Activity keeps the plain strip.

use std::sync::OnceLock;
use std::time::Duration;

use orangui_core::count_up::{self, PLACEHOLDER};
use orangui_core::number;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Text};
use ratatui::widgets::{BorderType, Paragraph};
use zwipe_ui::home::Figures;

use super::count_up as count_up_widget;
use crate::domain::theme::Palette;

/// Each figure's roll seed, handed out once per launch as orangui hands one to each counter, so neighbors never roll in step.
fn seeds() -> [u64; 3] {
    static SEEDS: OnceLock<[u64; 3]> = OnceLock::new();
    *SEEDS.get_or_init(|| [(); 3].map(|()| count_up_widget::next_seed()))
}

/// How long a figure takes to land once it arrives: [`count_up::FRAMES`] ticks of [`count_up::TICK_MS`].
pub(crate) const ROLL: Duration = Duration::from_millis(1_008);

/// The rows the strip takes: the numbers, then the labels.
pub(crate) const HEIGHT: u16 = 2;

/// The three numbers as they read at this moment.
pub(crate) fn numbers(
    figures: Option<Figures>,
    since_open: Duration,
    since_arrival: Duration,
) -> [String; 3] {
    let values = figures.map(Figures::values);
    let mut out: [String; 3] = Default::default();
    for (i, slot) in out.iter_mut().enumerate() {
        let value = values.and_then(|v| v.get(i).copied().flatten());
        let seed = seeds().get(i).copied().unwrap_or_default();
        *slot = match value {
            None => match ticks(since_open) {
                0 => PLACEHOLDER.to_owned(),
                tick => number::short(count_up::waiting(seed, tick)),
            },
            Some(value) => number::short(count_up::count_frame(
                value,
                u32::try_from(ticks(since_arrival)).unwrap_or(u32::MAX),
            )),
        };
    }
    out
}

/// Whole ticks of [`count_up::TICK_MS`] in `elapsed`.
fn ticks(elapsed: Duration) -> u64 {
    let ticks = elapsed
        .as_millis()
        .checked_div(count_up_widget::tick().as_millis())
        .unwrap_or(0);
    u64::try_from(ticks).unwrap_or(u64::MAX)
}

/// Draws the strip across `area`.
pub(crate) fn draw(
    f: &mut Frame<'_>,
    palette: &Palette,
    area: Rect,
    figures: Option<Figures>,
    since_open: Duration,
    since_arrival: Duration,
) {
    let numbers = numbers(figures, since_open, since_arrival);
    plain(f, palette, area, numbers);
}

/// The plain strip: each number over its label in a third of `area`.
fn plain(f: &mut Frame<'_>, palette: &Palette, area: Rect, numbers: [String; 3]) {
    let columns = Layout::horizontal([Constraint::Ratio(1, 3); 3]).split(area);
    for ((number, label), column) in numbers.into_iter().zip(Figures::LABELS).zip(columns.iter()) {
        let text = Text::from(vec![
            Line::styled(number, palette.accent).bold(),
            Line::styled(label, palette.muted),
        ]);
        f.render_widget(Paragraph::new(text).centered(), *column);
    }
}

/// The rows the tiles take: the top border, the number, the label and the bottom border.
pub(crate) const TILE_HEIGHT: u16 = 4;

/// The narrowest a row of tiles draws, in cells: three tiles that each hold the longest label inside their borders, with a cell between them.
pub(crate) const TILES_MIN_WIDTH: u16 = 47;

/// Draws `numbers` over their labels across `area`, each figure in its own rounded tile as the app's `StatsStrip` draws a `.stat`: the number bold in `--palette-5`, the label muted under it. An area shorter than [`TILE_HEIGHT`] or narrower than [`TILES_MIN_WIDTH`] gets the plain strip in its middle rows instead.
pub(crate) fn tiles(f: &mut Frame<'_>, palette: &Palette, area: Rect, numbers: [String; 3]) {
    if area.height < TILE_HEIGHT || area.width < TILES_MIN_WIDTH {
        let top = area.height.saturating_sub(HEIGHT) / 2;
        let strip = Rect {
            y: area.y.saturating_add(top),
            height: area.height.min(HEIGHT),
            ..area
        };
        plain(f, palette, strip, numbers);
        return;
    }
    let number_style = palette.swatches.get(4).copied().unwrap_or(palette.accent);
    let row = Rect {
        height: TILE_HEIGHT,
        ..area
    };
    let columns = Layout::horizontal([Constraint::Fill(1); 3])
        .spacing(1)
        .split(row);
    for ((number, label), column) in numbers.into_iter().zip(Figures::LABELS).zip(columns.iter()) {
        let block = crate::inbound::widgets::panel::bordered()
            .border_type(BorderType::Rounded)
            .border_style(palette.border);
        let inner = block.inner(*column);
        f.render_widget(block, *column);
        let text = Text::from(vec![
            Line::styled(number, number_style).bold(),
            Line::styled(label, palette.muted),
        ]);
        f.render_widget(Paragraph::new(text).centered(), inner);
    }
}

/// Draws home's figures as rolling tiles across `area`, as [`tiles`] does.
pub(crate) fn draw_tiles(
    f: &mut Frame<'_>,
    palette: &Palette,
    area: Rect,
    figures: Option<Figures>,
    since_open: Duration,
    since_arrival: Duration,
) {
    tiles(
        f,
        palette,
        area,
        numbers(figures, since_open, since_arrival),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figures_land_in_compact_form() {
        let figures = Figures {
            decks_created: Some(28_049),
            searches: Some(12),
            cards_swiped: Some(1_234_567),
        };
        let landed = numbers(Some(figures), Duration::ZERO, ROLL);
        assert_eq!(landed, ["28.0k", "12", "1.2m"]);
    }

    #[test]
    fn missing_figures_start_blank_then_roll() {
        let blank = numbers(None, Duration::ZERO, Duration::ZERO);
        assert!(blank.iter().all(|n| n == PLACEHOLDER));
        let rolling = numbers(None, Duration::from_millis(100), Duration::ZERO);
        assert!(
            rolling
                .iter()
                .all(|n| n.chars().any(|c| c.is_ascii_digit()))
        );
        assert_ne!(rolling[0], rolling[1], "each figure rolls its own digits");
    }
}
