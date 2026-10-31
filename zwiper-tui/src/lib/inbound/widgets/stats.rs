//! The stats strip: figures in a row, each a bold number over its label, with digits that roll while the figure loads.
//!
//! A screen showing a strip with a figure still loading should report itself animating, so the loop redraws the rolling digits on its tick.

use std::time::Duration;

use ratatui::style::Stylize;
use ratatui::text::{Line, Span};

use crate::domain::theme::Palette;
use crate::domain::{decode, home};

/// Digits shown for a figure that is still loading.
const ROLLING_DIGITS: usize = 3;

/// One figure: its label, and its value once loaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Stat<'label> {
    pub(crate) label: &'label str,
    pub(crate) value: Option<i64>,
}

/// Whether any figure is still loading, so the strip is rolling.
pub(crate) fn loading(stats: &[Stat<'_>]) -> bool {
    stats.iter().any(|s| s.value.is_none())
}

/// The digits a loading figure shows `elapsed` after its screen opened, changing every animation tick.
pub(crate) fn rolling(elapsed: Duration, seed: usize) -> String {
    let tick = elapsed
        .as_millis()
        .checked_div(decode::TICK.as_millis())
        .unwrap_or(0);
    let tick = usize::try_from(tick).unwrap_or(usize::MAX);
    (0..ROLLING_DIGITS)
        .map(|place| {
            // A cheap scramble so neighboring places and figures roll out of step.
            let mixed = tick
                .wrapping_mul(7)
                .wrapping_add(place.wrapping_mul(13))
                .wrapping_add(seed.wrapping_mul(31));
            char::from_digit(u32::try_from(mixed % 10).unwrap_or(0), 10).unwrap_or('0')
        })
        .collect()
}

/// The strip as one line: each value grouped with commas, or rolling digits until it loads.
pub(crate) fn line(palette: &Palette, stats: &[Stat<'_>], elapsed: Duration) -> Line<'static> {
    let spans: Vec<Span<'static>> = stats
        .iter()
        .enumerate()
        .flat_map(|(i, stat)| {
            let value = stat
                .value
                .map_or_else(|| rolling(elapsed, i), home::grouped);
            [
                Span::styled(value, palette.accent).bold(),
                Span::styled(format!(" {}   ", stat.label), palette.muted),
            ]
        })
        .collect();
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loaded_figures_group_and_loading_ones_roll() {
        let stats = [
            Stat {
                label: "Decks created",
                value: Some(1_234),
            },
            Stat {
                label: "Searches run",
                value: None,
            },
        ];
        assert!(loading(&stats));
        let early = line(&Palette::terminal(), &stats, Duration::ZERO).to_string();
        assert!(early.starts_with("1,234 Decks created   "));
        let later = line(&Palette::terminal(), &stats, decode::TICK).to_string();
        assert_ne!(early, later, "the loading figure rolls each tick");
        assert!(later.ends_with(" Searches run   "));
    }

    #[test]
    fn rolling_digits_are_digits() {
        for ms in [0, 40, 400, 4_000] {
            let digits = rolling(Duration::from_millis(ms), 2);
            assert_eq!(digits.len(), ROLLING_DIGITS);
            assert!(digits.chars().all(|c| c.is_ascii_digit()));
        }
        assert!(!loading(&[Stat {
            label: "x",
            value: Some(0)
        }]));
    }
}
