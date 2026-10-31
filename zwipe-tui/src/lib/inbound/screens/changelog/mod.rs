//! The Changelog screen: the launch's changelog under chip rows that narrow it from `Latest` to a year, a month and a minor version, with a skeleton while it loads and a note when it could not.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::time::Instant;

use super::Screen;
use crate::domain::changelog::{self, Filter, Listed};
use crate::inbound::app::Context;

/// The load-failure note.
pub(crate) const FAILED: &str = "Could not load the changelog";

/// One chip: its label, and whether it is lit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Chip {
    pub(crate) label: String,
    pub(crate) lit: bool,
}

/// The Changelog screen's state: the filter, the focused chip, and how far the cards are scrolled.
#[derive(Debug)]
pub(crate) struct ChangelogView {
    pub(crate) filter: Filter,
    /// The chip row with the focus: 0 for years, 1 for months, 2 for minor versions.
    pub(crate) row: usize,
    pub(crate) chip: usize,
    pub(crate) scroll: u16,
    /// When the filter last changed, for the cards' entrance.
    pub(crate) filtered_at: Instant,
}

impl Default for ChangelogView {
    fn default() -> Self {
        Self {
            filter: Filter::Latest,
            row: 0,
            chip: 0,
            scroll: 0,
            filtered_at: Instant::now(),
        }
    }
}

impl ChangelogView {
    /// The chip rows the filter opens, each a list of chips with the lit one marked.
    pub(crate) fn rows(&self, listed: &[Listed<'_>]) -> Vec<Vec<Chip>> {
        let (year, month, minor) = match &self.filter {
            Filter::Latest => (None, None, None),
            Filter::Year(year) => (Some(year.as_str()), None, None),
            Filter::Month { year, month } => (Some(year.as_str()), Some(month.as_str()), None),
            Filter::Minor { year, month, minor } => (
                Some(year.as_str()),
                Some(month.as_str()),
                Some(minor.as_str()),
            ),
        };
        let mut first = vec![Chip {
            label: "Latest".to_owned(),
            lit: year.is_none(),
        }];
        first.extend(
            changelog::years(listed)
                .into_iter()
                .map(|(key, label)| Chip {
                    lit: year == Some(key.as_str()),
                    label,
                }),
        );
        let mut rows = vec![first];
        if let Some(year) = year {
            rows.push(
                changelog::months(listed, year)
                    .into_iter()
                    .map(|(key, label)| Chip {
                        lit: month == Some(key.as_str()),
                        label,
                    })
                    .collect(),
            );
        }
        if let Some(month) = month {
            rows.push(
                changelog::minors(listed, month)
                    .into_iter()
                    .map(|m| Chip {
                        lit: minor == Some(m.as_str()),
                        label: m,
                    })
                    .collect(),
            );
        }
        rows
    }

    /// Presses the focused chip: it lights, clearing the rows under it, or when it is the deepest lit chip the filter goes up a level.
    pub(crate) fn press(&mut self, listed: &[Listed<'_>]) {
        let rows = self.rows(listed);
        let Some(chip) = rows.get(self.row).and_then(|r| r.get(self.chip)) else {
            return;
        };
        let deepest = chip.lit && self.row.saturating_add(1) >= rows.len();
        let filter = std::mem::take(&mut self.filter);
        self.filter = match (self.row, filter) {
            (0, _) if self.chip == 0 => Filter::Latest,
            (0, Filter::Year(_)) if deepest => Filter::Latest,
            (0, _) => {
                let years = changelog::years(listed);
                years
                    .get(self.chip.saturating_sub(1))
                    .map_or(Filter::Latest, |(key, _)| Filter::Year(key.clone()))
            }
            (1, Filter::Month { year, .. }) if deepest => Filter::Year(year),
            (1, Filter::Year(year) | Filter::Month { year, .. } | Filter::Minor { year, .. }) => {
                changelog::months(listed, &year).get(self.chip).map_or_else(
                    || Filter::Year(year.clone()),
                    |(key, _)| Filter::Month {
                        year: year.clone(),
                        month: key.clone(),
                    },
                )
            }
            (2, Filter::Minor { year, month, .. }) if deepest => Filter::Month { year, month },
            (2, Filter::Month { year, month } | Filter::Minor { year, month, .. }) => {
                changelog::minors(listed, &month)
                    .get(self.chip)
                    .map_or_else(
                        || Filter::Month {
                            year: year.clone(),
                            month: month.clone(),
                        },
                        |minor| Filter::Minor {
                            year: year.clone(),
                            month: month.clone(),
                            minor: minor.clone(),
                        },
                    )
            }
            (_, filter) => filter,
        };
        self.scroll = 0;
        self.filtered_at = Instant::now();
        let depth = self.rows(listed).len();
        if self.row >= depth {
            self.row = depth.saturating_sub(1);
            self.chip = 0;
        }
    }
}

/// Opens the Changelog on `Latest`.
pub(crate) fn open(cx: &mut Context<'_>) {
    cx.push(Screen::Changelog(Box::default()));
}
