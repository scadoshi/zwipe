//! The compact stats strip of home and Activity: three figures side by side, each a bold rolling number over its label, in orangui's compact form (`28.0k`).

use std::sync::OnceLock;
use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Text};
use ratatui::widgets::Paragraph;

use crate::domain::count_up;
use crate::domain::home::Figures;
use crate::domain::theme::Palette;

/// Each figure's roll seed, handed out once per launch as orangui hands one to each counter, so neighbors never roll in step.
fn seeds() -> [u64; 3] {
    static SEEDS: OnceLock<[u64; 3]> = OnceLock::new();
    *SEEDS.get_or_init(|| [(); 3].map(|()| count_up::next_seed()))
}

/// How long a figure takes to land once it arrives: [`count_up::FRAMES`] ticks of [`count_up::TICK`].
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
        let value = values.and_then(|v| v.get(i).copied());
        let seed = seeds().get(i).copied().unwrap_or_default();
        *slot = match value {
            None => match ticks(since_open) {
                0 => count_up::BLANK.to_owned(),
                tick => count_up::compact(count_up::waiting(seed, tick)),
            },
            Some(value) => count_up::compact(count_up::landing(value, ticks(since_arrival))),
        };
    }
    out
}

/// Whole ticks of [`count_up::TICK`] in `elapsed`.
fn ticks(elapsed: Duration) -> u64 {
    let tick = count_up::TICK.as_millis().max(1);
    u64::try_from(elapsed.as_millis() / tick).unwrap_or(u64::MAX)
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
    let columns = Layout::horizontal([Constraint::Ratio(1, 3); 3]).split(area);
    for ((number, label), column) in numbers.into_iter().zip(Figures::LABELS).zip(columns.iter()) {
        let text = Text::from(vec![
            Line::styled(number, palette.accent).bold(),
            Line::styled(label, palette.muted),
        ]);
        f.render_widget(Paragraph::new(text).centered(), *column);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figures_land_in_compact_form() {
        let figures = Figures {
            decks_created: 28_049,
            searches: 12,
            cards_swiped: 1_234_567,
        };
        let landed = numbers(Some(figures), Duration::ZERO, ROLL);
        assert_eq!(landed, ["28.0k", "12", "1.2m"]);
    }

    #[test]
    fn missing_figures_start_blank_then_roll() {
        let blank = numbers(None, Duration::ZERO, Duration::ZERO);
        assert!(blank.iter().all(|n| n == count_up::BLANK));
        let rolling = numbers(None, Duration::from_millis(100), Duration::ZERO);
        assert!(
            rolling
                .iter()
                .all(|n| n.chars().any(|c| c.is_ascii_digit()))
        );
        assert_ne!(rolling[0], rolling[1], "each figure rolls its own digits");
    }
}
