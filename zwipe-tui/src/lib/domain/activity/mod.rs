//! The Activity screen's math, ported from zwiper's `screens/activity/sections.rs` and orangui's `peak_indices`: the year's heatmap layout, its readout tags, the totals, the rate kinds and the role and tag rate charts.

#[cfg(test)]
mod tests;

use std::cmp::Reverse;
use std::collections::HashMap;

use chrono::{Datelike, Duration, NaiveDate};
use zwipe_core::domain::card::card_role;
use zwipe_core::http::contracts::suggestions::{
    ACTIVITY_DAYS, HttpActivity, HttpActivityDay, HttpRoleRates, HttpSwipeOutcomes, HttpTagRates,
};

use crate::domain::count_up;

/// A month label needs this many columns before the next one.
pub(crate) const LABEL_SPAN: usize = 3;

/// A day at or past this many times the median busy day is a peak.
const PEAK_RATIO: u32 = 4;

/// At most this many peaks, the biggest.
const MAX_PEAKS: usize = 12;

/// Rows in a heatmap column, one per weekday.
pub(crate) const DAYS_PER_WEEK: usize = 7;

/// Shades above empty, one per quarter of the busy days.
const SHADES: usize = 4;

/// A tag rate with no other players to go on: even odds.
const EVEN_ODDS: f64 = 0.5;

/// How many cards' worth of the overall rate a tag's rate among other players starts from, so a tag few others have been dealt sits near the overall rate.
const TAG_BASELINE_WEIGHT: f64 = 10.0;

/// Card roles' line before any role has enough swipes to show.
pub(crate) const ROLES_EMPTY: &str = "Swipe more cards to see your card roles";

/// Oracle tags' line before any tag has enough swipes to show.
pub(crate) const TAGS_EMPTY: &str = "Swipe more cards to see your Oracle tags";

/// Most skipped's line before any card qualifies.
pub(crate) const PASSES_EMPTY: &str = "Skipped cards show here";

/// A chart's line on Removed before any card counted there was removed. Only Removed can chart no role: every role sent has a dealt card, but Removed needs an added one.
pub(crate) const NONE_REMOVED: &str = "You haven't removed any cards yet";

/// Oracle tags' line when no tag's share beats other players'.
pub(crate) const NO_TAG_STANDS_OUT: &str = "No tags stand out from other players yet";

/// Rows on the Oracle tags chart.
pub(crate) const TAG_ROWS: usize = 10;

/// The weekday labels down the heatmap's left, by row.
pub(crate) const WEEKDAY_LABELS: [(usize, &str); 3] = [(1, "Mon"), (3, "Wed"), (5, "Fri")];

/// One day on the heatmap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DayCell {
    pub(crate) column: usize,
    pub(crate) row: usize,
    pub(crate) level: u8,
    pub(crate) peak: bool,
    pub(crate) date: NaiveDate,
    pub(crate) swipes: u32,
    pub(crate) searches: u32,
}

/// The heatmap's days in place, its month labels as (column, name), and its width in columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HeatLayout {
    pub(crate) cells: Vec<DayCell>,
    pub(crate) months: Vec<(usize, String)>,
    pub(crate) columns: usize,
}

impl HeatLayout {
    /// The index of the cell at `column` and `row`, if that day is on the grid.
    pub(crate) fn index_at(&self, column: usize, row: usize) -> Option<usize> {
        self.cells
            .iter()
            .position(|c| c.column == column && c.row == row)
    }
}

/// One bar of a chart, as the app's `Bar`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Bar {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) count: u64,
    /// The bar's length, 0 to 100.
    pub(crate) pct: u32,
    /// Whether its label opens a reveal.
    pub(crate) revealable: bool,
}

/// A day's activity, the number its cell is shaded by.
fn activity_of(day: &HttpActivityDay) -> u32 {
    day.swipes.saturating_add(day.searches)
}

/// The indices of the peaks in `counts`: nonzero counts at or past `ratio` times the median nonzero count, the largest `max` of them, biggest first. Orangui's `peak_indices`.
pub(crate) fn peak_indices(counts: &[u32], ratio: u32, max: usize) -> Vec<usize> {
    let mut busy: Vec<u32> = counts.iter().copied().filter(|&n| n > 0).collect();
    if busy.is_empty() {
        return Vec::new();
    }
    busy.sort_unstable();
    let median = busy.get(busy.len() / 2).copied().unwrap_or(0);
    let floor = median.saturating_mul(ratio);
    let mut peaks: Vec<(usize, u32)> = counts
        .iter()
        .copied()
        .enumerate()
        .filter(|&(_, n)| n > 0 && n >= floor)
        .collect();
    peaks.sort_by_key(|&(_, n)| Reverse(n));
    peaks.truncate(max);
    peaks.into_iter().map(|(i, _)| i).collect()
}

/// The shade for `count` given the busy days' counts sorted ascending: 0 for none, then 1 to 4 by which quarter of the busy days it falls in.
pub(crate) fn level(count: u32, busy: &[u32]) -> u8 {
    if count == 0 || busy.is_empty() {
        return 0;
    }
    let quartile = |q: usize| {
        busy.get(busy.len().saturating_sub(1).saturating_mul(q) / SHADES)
            .copied()
            .unwrap_or(0)
    };
    if count <= quartile(1) {
        1
    } else if count <= quartile(2) {
        2
    } else if count <= quartile(3) {
        3
    } else {
        4
    }
}

/// Every day of the year ending `through`, laid out like GitHub's grid: columns are weeks starting on Sunday, rows are weekdays. A day missing from `days` is empty. A month is labeled at the first column whose Sunday falls in it, unless the next label lands within [`LABEL_SPAN`] columns, which leaves a partial first month unlabeled.
pub(crate) fn heat_layout(through: NaiveDate, days: &[HttpActivityDay]) -> HeatLayout {
    // A year reaching back past the first date has no grid to lay out.
    let Some((start, first_sunday)) = through
        .checked_sub_signed(Duration::days(ACTIVITY_DAYS.saturating_sub(1)))
        .and_then(|start| {
            let to_sunday = Duration::days(i64::from(start.weekday().num_days_from_sunday()));
            Some((start, start.checked_sub_signed(to_sunday)?))
        })
    else {
        return HeatLayout {
            cells: Vec::new(),
            months: Vec::new(),
            columns: 0,
        };
    };
    let by_day: HashMap<NaiveDate, &HttpActivityDay> = days.iter().map(|d| (d.day, d)).collect();
    let mut busy: Vec<u32> = days.iter().map(activity_of).filter(|&n| n > 0).collect();
    busy.sort_unstable();

    let mut cells = Vec::new();
    let mut months: Vec<(usize, String)> = Vec::new();
    let mut last_month = None;
    let mut date = start;
    while date <= through {
        let offset =
            usize::try_from(date.signed_duration_since(first_sunday).num_days()).unwrap_or(0);
        let (column, row) = (offset / DAYS_PER_WEEK, offset % DAYS_PER_WEEK);
        let (swipes, searches) = by_day.get(&date).map_or((0, 0), |d| (d.swipes, d.searches));
        if row == 0 && last_month != Some(date.month()) {
            if months
                .last()
                .is_some_and(|(previous, _)| column < previous.saturating_add(LABEL_SPAN))
            {
                months.pop();
            }
            months.push((column, date.format("%b").to_string()));
            last_month = Some(date.month());
        }
        cells.push(DayCell {
            column,
            row,
            level: level(swipes.saturating_add(searches), &busy),
            peak: false,
            date,
            swipes,
            searches,
        });
        let Some(next) = date.succ_opt() else {
            break;
        };
        date = next;
    }
    let counts: Vec<u32> = cells
        .iter()
        .map(|c| c.swipes.saturating_add(c.searches))
        .collect();
    for i in peak_indices(&counts, PEAK_RATIO, MAX_PEAKS) {
        if let Some(cell) = cells.get_mut(i) {
            cell.peak = true;
        }
    }
    let columns = cells.last().map_or(0, |c| c.column.saturating_add(1));
    HeatLayout {
        cells,
        months,
        columns,
    }
}

/// `n` and its noun, singular for one, with thousands separators.
pub(crate) fn counted(n: u32, one: &str, many: &str) -> String {
    let noun = if n == 1 { one } else { many };
    format!("{} {noun}", count_up::with_separators(u64::from(n)))
}

/// A picked day's tags: its swipes, its searches and its date.
pub(crate) fn day_tags(cell: &DayCell) -> [String; 3] {
    [
        counted(cell.swipes, "swipe", "swipes"),
        counted(cell.searches, "search", "searches"),
        cell.date.format("%b %-d").to_string(),
    ]
}

/// The heatmap's resting tags: the year's swipes, searches and span.
pub(crate) fn year_tags(days: &[HttpActivityDay]) -> [String; 3] {
    let swipes = days
        .iter()
        .map(|d| d.swipes)
        .fold(0_u32, u32::saturating_add);
    let searches = days
        .iter()
        .map(|d| d.searches)
        .fold(0_u32, u32::saturating_add);
    [
        counted(swipes, "swipe", "swipes"),
        counted(searches, "search", "searches"),
        "Last year".to_owned(),
    ]
}

/// The four outcomes as columns: added, skipped, saved and removed, each against the largest.
pub(crate) fn outcome_bars(outcomes: &HttpSwipeOutcomes) -> Vec<Bar> {
    let values = [
        ("Added", outcomes.added),
        ("Skipped", outcomes.skipped),
        ("Maybeboard", outcomes.maybed),
        ("Removed", outcomes.removed),
    ];
    let top = values.iter().map(|(_, n)| *n).max().unwrap_or(0).max(1);
    values
        .into_iter()
        .map(|(label, n)| Bar {
            key: label.to_owned(),
            label: label.to_owned(),
            count: n,
            pct: n
                .saturating_mul(100)
                .checked_div(top)
                .and_then(|pct| u32::try_from(pct).ok())
                .unwrap_or(100),
            revealable: false,
        })
        .collect()
}

/// Whether any swipe came to anything.
pub(crate) fn has_outcomes(outcomes: &HttpSwipeOutcomes) -> bool {
    outcomes.added > 0 || outcomes.skipped > 0 || outcomes.maybed > 0 || outcomes.removed > 0
}

/// Whether the activity has anything to draw: a day, an outcome, a rate or a pass.
pub(crate) fn has_anything(activity: &HttpActivity) -> bool {
    !activity.days.is_empty()
        || has_outcomes(&activity.outcomes)
        || !activity.roles.is_empty()
        || !activity.tags.is_empty()
        || !activity.passes.is_empty()
}

/// Which share of dealt cards the rate charts show, picked from the Show chips.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum RateKind {
    /// Added to a deck.
    #[default]
    Added,
    Skipped,
    /// Saved to a maybeboard.
    Maybed,
    /// Added and later taken out.
    Removed,
}

impl RateKind {
    /// The chip row's order.
    pub(crate) const ALL: [Self; 4] = [Self::Added, Self::Skipped, Self::Maybed, Self::Removed];

    /// The chip's label, the Totals column's name.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Added => "Added",
            Self::Skipped => "Skipped",
            Self::Maybed => "Maybeboard",
            Self::Removed => "Removed",
        }
    }

    /// The outcome counted and what it is a share of: the cards dealt, or for Removed the cards added.
    pub(crate) fn parts(self, outcomes: &HttpSwipeOutcomes) -> (u64, u64) {
        let dealt = outcomes
            .added
            .saturating_add(outcomes.skipped)
            .saturating_add(outcomes.maybed);
        match self {
            Self::Added => (outcomes.added, dealt),
            Self::Skipped => (outcomes.skipped, dealt),
            Self::Maybed => (outcomes.maybed, dealt),
            Self::Removed => (outcomes.removed.min(outcomes.added), outcomes.added),
        }
    }

    /// The share as a whole percent, rounded down; `None` with nothing to take a share of.
    pub(crate) fn percent(self, outcomes: &HttpSwipeOutcomes) -> Option<u32> {
        let (count, of) = self.parts(outcomes);
        let pct = count.saturating_mul(100).checked_div(of)?;
        Some(u32::try_from(pct).unwrap_or(100))
    }
}

/// A count as a float, capped at `u32::MAX`, far past any real count.
fn float(n: u64) -> f64 {
    f64::from(u32::try_from(n).unwrap_or(u32::MAX))
}

/// One row per role with a share to take, labeled by the role's short name, its bar and count the share as a percent, highest first and then in the server's order. A role with any of `tags` under it reveals them.
pub(crate) fn role_rate_bars(
    kind: RateKind,
    roles: &[HttpRoleRates],
    tags: &[HttpTagRates],
) -> Vec<Bar> {
    let mut rated: Vec<(&HttpRoleRates, u32)> = roles
        .iter()
        .filter_map(|r| kind.percent(&r.outcomes).map(|rate| (r, rate)))
        .collect();
    rated.sort_by_key(|(_, rate)| Reverse(*rate));
    rated
        .into_iter()
        .map(|(role, rate)| Bar {
            key: role.role.clone(),
            label: card_role::role_short_label(&role.role),
            count: u64::from(rate),
            pct: rate,
            revealable: tags.iter().any(|t| t.role == role.role),
        })
        .collect()
}

/// The user's swiped tags under `role`, in the server's order.
pub(crate) fn tags_under(role: &str, tags: &[HttpTagRates]) -> Vec<String> {
    tags.iter()
        .filter(|t| t.role == role)
        .map(|t| t.tag.clone())
        .collect()
}

/// Other players' share across every tag sent, the rate a tag few of them have been dealt starts from. Even odds with nothing to go on.
pub(crate) fn baseline(kind: RateKind, tags: &[HttpTagRates]) -> f64 {
    let (count, of) = tags.iter().fold((0_u64, 0_u64), |(count, of), t| {
        let (c, o) = kind.parts(&t.everyone);
        (count.saturating_add(c), of.saturating_add(o))
    });
    if of == 0 {
        EVEN_ODDS
    } else {
        float(count) / float(of)
    }
}

/// How much a tag's share stands out for the user: their count times the log of their rate over other players', the others' rate smoothed toward `baseline` by [`TAG_BASELINE_WEIGHT`] cards. `None` when the user's rate is no higher.
pub(crate) fn lift_score(kind: RateKind, tag: &HttpTagRates, baseline: f64) -> Option<f64> {
    let (count, of) = kind.parts(&tag.outcomes);
    if count == 0 || of == 0 {
        return None;
    }
    let (theirs, their_of) = kind.parts(&tag.everyone);
    let mine = float(count) / float(of);
    let others =
        (float(theirs) + TAG_BASELINE_WEIGHT * baseline) / (float(their_of) + TAG_BASELINE_WEIGHT);
    (others > 0.0 && mine > others).then(|| float(count) * (mine / others).ln())
}

/// The [`TAG_ROWS`] tags whose share stands out most from other players', labeled by slug, the bar and count the user's own share as a percent, drawn highest share first.
pub(crate) fn tag_rate_bars(kind: RateKind, tags: &[HttpTagRates]) -> Vec<Bar> {
    let base = baseline(kind, tags);
    let mut scored: Vec<(&HttpTagRates, f64)> = tags
        .iter()
        .filter_map(|t| lift_score(kind, t, base).map(|score| (t, score)))
        .collect();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.tag.cmp(&b.0.tag)));
    let mut bars: Vec<Bar> = scored
        .into_iter()
        .take(TAG_ROWS)
        .filter_map(|(tag, _)| {
            let rate = kind.percent(&tag.outcomes)?;
            Some(Bar {
                key: tag.tag.clone(),
                label: tag.tag.clone(),
                count: u64::from(rate),
                pct: rate,
                revealable: true,
            })
        })
        .collect();
    bars.sort_by_key(|bar| Reverse(bar.pct));
    bars
}

/// Oracle tags' line when the picked share charts no tag: nothing removed under any tag on Removed, otherwise no tag the user beats others on.
pub(crate) fn tag_empty_line(kind: RateKind, tags: &[HttpTagRates]) -> &'static str {
    if kind == RateKind::Removed && tags.iter().all(|t| t.outcomes.removed == 0) {
        NONE_REMOVED
    } else {
        NO_TAG_STANDS_OUT
    }
}
