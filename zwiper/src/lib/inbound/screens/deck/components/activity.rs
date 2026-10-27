//! The Playstyle screen's activity: a heatmap of the user's swipes and
//! searches over the last year, what their swipes came to, the keep rate per
//! card role, and the cards they keep passing on. The first three are
//! sections in the profile's accordion; the hard passes are a card group
//! under Staples in its grammar.

use std::cmp::Reverse;
use std::collections::HashMap;

use chrono::{Datelike, Duration, NaiveDate};
use components::{
    Bar, BarChart, BarOrientation, ChartTip, HEAT_CELL, HEAT_ROWS, HeatCell, HeatGrid, HeatHit,
    heat_span, peak_indices, with_separators,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::card_role::role_short_label;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::http::contracts::suggestions::{
    ACTIVITY_DAYS, HttpActivity, HttpActivityDay, HttpHardPass, HttpRoleKeep, HttpSwipeOutcomes,
};

use super::collapsible_section::CollapsibleSection;
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::InfoButton;
use crate::inbound::screens::deck::card::components::card_row::CardRow;

/// The accordion title of the heatmap section.
pub(crate) const ACTIVITY_TITLE: &str = "Activity";
/// The accordion title of the swipe outcomes section.
const SWIPES_TITLE: &str = "Swipes";
/// The accordion title of the keep rate section.
const KEEP_RATE_TITLE: &str = "Keep rate";
/// The hard passes group's header.
const PASSES_TITLE: &str = "Hard passes";

/// Room on the left of the grid for the weekday labels.
const LEFT: f64 = 24.0;
/// Room above the grid for the month labels.
const TOP: f64 = 14.0;
/// A month label needs this many columns before the next one.
const LABEL_SPAN: usize = 3;
/// A day at or past this many times the median busy day is a peak, at most
/// `MAX_PEAKS` of them, the biggest. The portfolio's heatmap reads the same.
const PEAK_RATIO: u32 = 4;
const MAX_PEAKS: usize = 12;

/// One day on the heatmap.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DayCell {
    column: usize,
    row: usize,
    level: u8,
    peak: bool,
    date: NaiveDate,
    swipes: u32,
    searches: u32,
}

/// The heatmap's days in place, its month labels as (column, name), and
/// its width in columns.
#[derive(Debug, Clone, PartialEq, Eq)]
struct HeatLayout {
    cells: Vec<DayCell>,
    months: Vec<(usize, String)>,
    columns: usize,
}

/// A day's activity, the number its cell is shaded by.
fn activity_of(day: &HttpActivityDay) -> u32 {
    day.swipes.saturating_add(day.searches)
}

/// The shade for `count` given the busy days' counts sorted ascending: 0
/// for none, then 1 to 4 by which quarter of the busy days it falls in.
fn level(count: u32, busy: &[u32]) -> u8 {
    if count == 0 || busy.is_empty() {
        return 0;
    }
    let quartile = |q: usize| {
        busy.get(busy.len().saturating_sub(1).saturating_mul(q) / 4)
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

/// Every day of the year ending `through`, laid out like GitHub's grid:
/// columns are weeks starting on Sunday, rows are weekdays. A day missing
/// from `days` is empty. A month is labeled at the first column whose
/// Sunday falls in it, unless the next label lands within [`LABEL_SPAN`]
/// columns, which leaves a partial first month unlabeled.
fn heat_layout(through: NaiveDate, days: &[HttpActivityDay]) -> HeatLayout {
    let start = through - Duration::days(ACTIVITY_DAYS.saturating_sub(1));
    let first_sunday = start - Duration::days(i64::from(start.weekday().num_days_from_sunday()));
    let by_day: HashMap<NaiveDate, &HttpActivityDay> = days.iter().map(|d| (d.day, d)).collect();
    let mut busy: Vec<u32> = days.iter().map(activity_of).filter(|&n| n > 0).collect();
    busy.sort_unstable();

    let mut cells = Vec::new();
    let mut months: Vec<(usize, String)> = Vec::new();
    let mut last_month = None;
    let mut date = start;
    while date <= through {
        let offset = usize::try_from((date - first_sunday).num_days()).unwrap_or(0);
        let (column, row) = (offset / 7, offset % 7);
        let day = by_day.get(&date);
        let (swipes, searches) = day.map_or((0, 0), |d| (d.swipes, d.searches));
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
        date = date.succ_opt().unwrap_or(through + Duration::days(1));
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

/// `n` and its noun, singular for one.
fn counted(n: u32, one: &str, many: &str) -> String {
    let noun = if n == 1 { one } else { many };
    format!("{} {noun}", with_separators(u64::from(n)))
}

/// What a tapped day's chip says.
fn day_text(cell: &DayCell) -> String {
    let date = cell.date.format("%b %-d");
    match (cell.swipes, cell.searches) {
        (0, 0) => format!("Nothing on {date}"),
        (swipes, 0) => format!("{} on {date}", counted(swipes, "swipe", "swipes")),
        (0, searches) => format!("{} on {date}", counted(searches, "search", "searches")),
        (swipes, searches) => format!(
            "{} and {} on {date}",
            counted(swipes, "swipe", "swipes"),
            counted(searches, "search", "searches")
        ),
    }
}

/// The line under the heatmap: the year's swipes and searches.
fn year_line(days: &[HttpActivityDay]) -> String {
    let swipes = days
        .iter()
        .map(|d| d.swipes)
        .fold(0u32, u32::saturating_add);
    let searches = days
        .iter()
        .map(|d| d.searches)
        .fold(0u32, u32::saturating_add);
    format!(
        "{} and {} in the last year",
        counted(swipes, "swipe", "swipes"),
        counted(searches, "search", "searches")
    )
}

/// The four outcomes as columns: added, skipped, saved and removed, each
/// against the largest.
fn outcome_bars(outcomes: &HttpSwipeOutcomes) -> Vec<Bar> {
    let values = [
        ("Added", outcomes.added),
        ("Skipped", outcomes.skipped),
        ("Maybeboard", outcomes.maybed),
        ("Removed", outcomes.removed),
    ];
    let top = values.iter().map(|(_, n)| *n).max().unwrap_or(0).max(1);
    values
        .into_iter()
        .map(|(label, n)| {
            let pct = u32::try_from(n.saturating_mul(100) / top).unwrap_or(100);
            Bar::new(label, usize::try_from(n).unwrap_or(usize::MAX), pct)
        })
        .collect()
}

/// Whether any swipe came to anything.
fn has_outcomes(outcomes: &HttpSwipeOutcomes) -> bool {
    outcomes.added > 0 || outcomes.skipped > 0 || outcomes.maybed > 0 || outcomes.removed > 0
}

/// A role's kept cards as a whole percent of those it was dealt.
fn keep_rate(role: &HttpRoleKeep) -> u32 {
    let seen = role.kept.saturating_add(role.passed);
    if seen == 0 {
        return 0;
    }
    u32::try_from(role.kept.saturating_mul(100) / seen).unwrap_or(100)
}

/// One row per role, labeled like the Card roles chart, its bar and count
/// the keep rate, highest first and then in the server's order.
fn keep_rate_bars(roles: &[HttpRoleKeep]) -> Vec<Bar> {
    let mut rated: Vec<(&HttpRoleKeep, u32)> = roles.iter().map(|r| (r, keep_rate(r))).collect();
    rated.sort_by_key(|(_, rate)| Reverse(*rate));
    rated
        .into_iter()
        .map(|(role, rate)| Bar {
            key: role.role.clone(),
            label: role_short_label(&role.role),
            count: usize::try_from(rate).unwrap_or(100),
            pct: rate,
            revealable: false,
        })
        .collect()
}

/// The year as a heatmap: weekday labels down the left, month labels over
/// the top, a cell per day shaded by its swipes and searches in orangui's
/// [`HeatGrid`], and a chip with the day's counts on a tap. Wider than a
/// phone, it scrolls sideways and opens on the latest week.
#[component]
fn ActivityHeatmap(through: NaiveDate, days: Vec<HttpActivityDay>) -> Element {
    let mut tip: Signal<Option<(String, f64, f64)>> = use_signal(|| None);
    let layout = heat_layout(through, &days);
    let width = LEFT + heat_span(layout.columns);
    let height = TOP + heat_span(HEAT_ROWS);
    let texts: Vec<String> = layout.cells.iter().map(day_text).collect();
    let grid: Vec<HeatCell> = layout
        .cells
        .iter()
        .map(|cell| HeatCell {
            column: cell.column,
            row: cell.row,
            level: cell.level,
            peak: cell.peak,
            key: cell.date.to_string(),
        })
        .collect();
    let show = move |hit: HeatHit| {
        if let Some(text) = texts.get(hit.index) {
            let x = (hit.x + HEAT_CELL / 2.0) / width * 100.0;
            let y = hit.y / height * 100.0;
            tip.set(Some((text.clone(), x, y)));
        }
    };
    rsx! {
        div {
            class: "chart-scroll scroll-fade-x activity-scroll",
            onmounted: move |event| async move {
                let _ = event.data().scroll(
                    dioxus::html::geometry::PixelsVector2D::new(width, 0.0),
                    ScrollBehavior::Instant,
                ).await;
            },
            div { class: "activity-plot", style: "width:{width}px;",
                svg {
                    class: "heat-grid",
                    style: "width:{width}px;",
                    view_box: "0 0 {width} {height}",
                    role: "img",
                    "aria-label": "{year_line(&days)}",
                    for (column, name) in layout.months.iter() {
                        text {
                            key: "m{column}",
                            class: "heat-label",
                            x: "{LEFT + heat_span(*column)}",
                            y: "{TOP - 4.0}",
                            "{name}"
                        }
                    }
                    for (row, name) in [(1usize, "Mon"), (3, "Wed"), (5, "Fri")] {
                        text {
                            key: "d{row}",
                            class: "heat-label",
                            x: "0",
                            y: "{TOP + heat_span(row) + HEAT_CELL - 2.0}",
                            "{name}"
                        }
                    }
                    HeatGrid {
                        cells: grid,
                        left: LEFT,
                        top: TOP,
                        on_tap: show,
                    }
                }
                if let Some((text, left, top)) = tip() {
                    ChartTip { text, left, top }
                }
            }
        }
    }
}

/// The activity's three sections in the profile's accordion: Activity, the
/// year's heatmap with its totals under it; Swipes, what the user's swipes
/// came to as columns; Keep rate, the share of each card role's dealt cards
/// they added, as rows. Each is left out when it has nothing to show. The
/// heatmap mounts afresh each time its section opens, so its sweep plays in
/// view.
#[component]
pub(crate) fn ActivitySections(
    activity: HttpActivity,
    open_section: Signal<Option<String>>,
) -> Element {
    let open = open_section().as_deref() == Some(ACTIVITY_TITLE);
    let mut opens = use_signal(|| 0u32);
    use_effect(use_reactive!(|open| {
        if open {
            let next = opens.peek().wrapping_add(1);
            opens.set(next);
        }
    }));
    let has_days = !activity.days.is_empty();
    let outcomes = activity.outcomes;
    let keep = keep_rate_bars(&activity.roles);
    rsx! {
        if has_days {
            CollapsibleSection {
                title: ACTIVITY_TITLE,
                open_section,
                header_accessory: rsx! {
                    InfoButton { topic: HintTopic::Activity }
                },
                for run in [opens()] {
                    ActivityHeatmap {
                        key: "{run}",
                        through: activity.through,
                        days: activity.days.clone(),
                    }
                }
                div { class: "activity-total", "{year_line(&activity.days)}" }
            }
        }
        if has_outcomes(&outcomes) {
            CollapsibleSection {
                title: SWIPES_TITLE,
                open_section,
                header_accessory: rsx! {
                    InfoButton { topic: HintTopic::SwipeOutcomes }
                },
                BarChart { bars: outcome_bars(&outcomes) }
            }
        }
        if !keep.is_empty() {
            CollapsibleSection {
                title: KEEP_RATE_TITLE,
                open_section,
                header_accessory: rsx! {
                    InfoButton { topic: HintTopic::KeepRate }
                },
                BarChart {
                    bars: keep,
                    orientation: BarOrientation::Horizontal,
                    count_suffix: "%",
                }
            }
        }
    }
}

/// One hard pass: the shared expandable card row with its skips as a tag
/// where a deck list puts copies, and how many commanders it was skipped
/// under when expanded. No actions beyond the row's own Printing.
#[component]
fn HardPassRow(
    pass: HttpHardPass,
    expanded_card: Signal<Option<Uuid>>,
    preview_card: Signal<Option<(ScryfallData, usize)>>,
    preview_dismissing: Signal<bool>,
    on_printing: EventHandler<Card>,
) -> Element {
    let skips = counted(pass.skipped, "skip", "skips");
    let under = format!(
        "Under {}",
        counted(pass.commanders, "commander", "commanders")
    );
    rsx! {
        CardRow {
            card: pass.card,
            qty: i32::try_from(pass.skipped).unwrap_or(i32::MAX),
            qty_tags: vec![skips],
            expanded_card,
            preview_card,
            preview_dismissing,
            on_printing: move |c: Card| on_printing.call(c),
            footer: rsx! {
                div { class: "keyword-chips",
                    span { class: "detail-chip detail-chip-pt", "{under}" }
                }
            },
        }
    }
}

/// The cards the user keeps passing on, as [`HardPassRow`]s in a group
/// headed like Staples', in the server's order: most skipped against other
/// players first. Renders nothing without any.
#[component]
pub(crate) fn HardPassesPanel(
    passes: Vec<HttpHardPass>,
    expanded_card: Signal<Option<Uuid>>,
    preview_card: Signal<Option<(ScryfallData, usize)>>,
    preview_dismissing: Signal<bool>,
    on_printing: EventHandler<Card>,
) -> Element {
    rsx! {
        if !passes.is_empty() {
            div { class: "maybeboard-list card-group row-enter",
                div { class: "card-group-header card-group-header-with-hint",
                    span { "{PASSES_TITLE}" }
                    InfoButton { topic: HintTopic::HardPasses }
                }
                for pass in passes {
                    HardPassRow {
                        key: "{pass.card.scryfall_data.id}",
                        pass,
                        expanded_card,
                        preview_card,
                        preview_dismissing,
                        on_printing,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn day(on: NaiveDate, swipes: u32, searches: u32) -> HttpActivityDay {
        HttpActivityDay {
            day: on,
            swipes,
            searches,
        }
    }

    #[test]
    fn the_year_is_every_day_in_week_columns_ending_on_through() {
        let through = date(2026, 10, 27);
        let layout = heat_layout(through, &[]);
        assert_eq!(layout.cells.len(), 371, "every day of the reach");
        let last = layout.cells.last().unwrap();
        assert_eq!(last.date, through);
        assert_eq!(last.row, 2, "Oct 27 2026 is a Tuesday");
        assert!(layout.cells.iter().all(|c| c.level == 0 && !c.peak));
        assert_eq!(layout.columns, last.column + 1);
    }

    #[test]
    fn busy_days_shade_by_quarter_and_empty_days_stay_blank() {
        let through = date(2026, 10, 27);
        let days: Vec<HttpActivityDay> = (1..=8)
            .map(|n| {
                day(
                    through - Duration::days(n),
                    u32::try_from(n).unwrap() * 10,
                    0,
                )
            })
            .collect();
        let layout = heat_layout(through, &days);
        let level_on = |d: NaiveDate| layout.cells.iter().find(|c| c.date == d).unwrap().level;
        assert_eq!(level_on(through), 0, "no activity today");
        assert_eq!(
            level_on(through - Duration::days(1)),
            1,
            "the quietest busy day"
        );
        assert_eq!(level_on(through - Duration::days(8)), 4, "the busiest");
    }

    #[test]
    fn a_spike_is_a_peak() {
        let through = date(2026, 10, 27);
        let mut days: Vec<HttpActivityDay> = (1..=10)
            .map(|n| day(through - Duration::days(n), 5, 0))
            .collect();
        days.push(day(through, 200, 0));
        let layout = heat_layout(through, &days);
        let peaks: Vec<NaiveDate> = layout
            .cells
            .iter()
            .filter(|c| c.peak)
            .map(|c| c.date)
            .collect();
        assert_eq!(peaks, [through]);
    }

    #[test]
    fn months_are_labeled_apart_and_in_order() {
        let layout = heat_layout(date(2026, 10, 27), &[]);
        for pair in layout.months.windows(2) {
            assert!(pair[1].0 >= pair[0].0 + LABEL_SPAN, "{:?}", layout.months);
        }
        assert_eq!(layout.months.last().unwrap().1, "Oct");
    }

    #[test]
    fn a_day_says_what_happened_on_it() {
        let on = date(2026, 10, 3);
        let cell = |swipes, searches| DayCell {
            column: 0,
            row: 0,
            level: 0,
            peak: false,
            date: on,
            swipes,
            searches,
        };
        assert_eq!(day_text(&cell(0, 0)), "Nothing on Oct 3");
        assert_eq!(day_text(&cell(1, 0)), "1 swipe on Oct 3");
        assert_eq!(day_text(&cell(0, 2)), "2 searches on Oct 3");
        assert_eq!(
            day_text(&cell(1200, 1)),
            "1,200 swipes and 1 search on Oct 3"
        );
    }

    #[test]
    fn keep_rate_rounds_down_and_sorts_highest_first() {
        let role = |slug: &str, kept, passed| HttpRoleKeep {
            role: slug.to_string(),
            kept,
            passed,
        };
        let bars = keep_rate_bars(&[role("ramp", 1, 2), role("removal", 9, 1)]);
        let rates: Vec<(String, usize)> = bars.into_iter().map(|b| (b.key, b.count)).collect();
        assert_eq!(
            rates,
            [("removal".to_string(), 90), ("ramp".to_string(), 33)]
        );
    }

    #[test]
    fn outcomes_scale_against_the_largest() {
        let bars = outcome_bars(&HttpSwipeOutcomes {
            added: 5,
            skipped: 20,
            maybed: 0,
            removed: 1,
        });
        let pcts: Vec<u32> = bars.iter().map(|b| b.pct).collect();
        assert_eq!(pcts, [25, 100, 0, 5]);
    }
}
