//! The Activity screen's parts: a heatmap of the user's swipes and searches
//! over the last year, what their swipes came to, their rates per card role
//! and oracle tag, and the cards they keep passing on. The first three are
//! sections in one accordion; the hard passes are a card group in Staples'
//! grammar.

use std::cmp::Reverse;
use std::collections::HashMap;

use chrono::{Datelike, Duration, NaiveDate, Utc};
use components::{
    Bar, BarChart, BarOrientation, ChartTip, Chip, HEAT_CELL, HEAT_ROWS, HEAT_STEP, HeatCell,
    HeatGrid, HeatHit, HeatReadout, LabelCase, OtagDef, RoleTags, heat_span, peak_indices,
    use_heat_pick, with_separators,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::card_role::role_short_label;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::http::contracts::suggestions::{
    ACTIVITY_DAYS, HttpActivity, HttpActivityDay, HttpHardPass, HttpRoleRates, HttpSwipeOutcomes,
    HttpTagRates,
};

use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::InfoButton;
use crate::inbound::screens::deck::card::components::card_row::{
    CardRow, OtagDescribe, OtagExamplesOpen,
};
use crate::inbound::screens::deck::components::collapsible_section::CollapsibleSection;

/// The accordion title of the heatmap section.
pub(crate) const ACTIVITY_TITLE: &str = "Activity";
/// The accordion title of the role rates section.
const ROLES_TITLE: &str = "Card roles";
/// The accordion title of the tag rates section.
const TAGS_TITLE: &str = "Oracle tags";
/// The hard passes group's header.
const PASSES_TITLE: &str = "Most skipped";

/// Columns of tiles left of the grid, room for the weekday labels.
const FIELD_LEFT_COLUMNS: usize = 2;
/// Rows of tiles in the field: one over the grid for the month labels, the
/// seven weekdays, and one under it.
const FIELD_ROWS: usize = HEAT_ROWS + 2;
/// Room on the left of the grid for the weekday labels, whole tiles so the
/// days land on the field's tiles.
const LEFT: f64 = 2.0 * HEAT_STEP;
/// Room above the grid for the month labels, one tile.
const TOP: f64 = HEAT_STEP;
/// How far a weekday label sits in from the field's left edge.
const LABEL_INSET: f64 = 4.0;
/// The backdrop's tile shades, as in orangui's `grid-cells.js`. The
/// stylesheet holds the alphas; this is how many there are.
const TILE_SHADES: usize = 5;
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

/// The field's width and height in SVG units for a year `columns` weeks
/// wide: the label room, the grid, and a column of tiles past its right edge
/// and a row under it.
fn field_size(columns: usize) -> (f64, f64) {
    (
        LEFT + heat_span(columns.saturating_add(1)),
        TOP + heat_span(HEAT_ROWS.saturating_add(1)),
    )
}

/// A tile's shade, 0 to 4, picked by a hash of its place, so the field is
/// the same on every load and the skeleton's tiles are the loaded ones.
fn tile_shade(column: usize, row: usize) -> usize {
    let mut h = column.wrapping_mul(2_654_435_761) ^ row.wrapping_mul(2_246_822_519);
    h ^= h >> 13;
    h = h.wrapping_mul(1_540_483_477);
    h ^= h >> 15;
    h % TILE_SHADES
}

/// The field's tiles, `across` columns of [`FIELD_ROWS`], each in one of
/// the backdrop's shades. They hold still; the days sweep in over them.
#[component]
fn HeatTiles(across: usize) -> Element {
    rsx! {
        for column in 0..across {
            for row in 0..FIELD_ROWS {
                rect {
                    key: "t{column}-{row}",
                    class: "heat-tile heat-tile-{tile_shade(column, row)}",
                    x: "{heat_span(column)}",
                    y: "{heat_span(row)}",
                    width: "{HEAT_CELL}",
                    height: "{HEAT_CELL}",
                    rx: "2",
                }
            }
        }
    }
}

/// How many tile columns a year `columns` weeks wide needs.
fn tiles_across(columns: usize) -> usize {
    FIELD_LEFT_COLUMNS.saturating_add(columns).saturating_add(1)
}

/// The heatmap's field with no days on it, for the screen's skeleton: the
/// same tiles, still, scrolled to the latest week, so the loaded grid's
/// colors land on what is already there.
#[component]
pub(crate) fn HeatFieldGhost() -> Element {
    let columns = heat_layout(Utc::now().date_naive(), &[]).columns;
    let (width, height) = field_size(columns);
    rsx! {
        div { class: "activity-field",
            div {
                class: "chart-scroll scroll-fade-x activity-scroll",
                onmounted: move |event| async move {
                    let _ = event.data().scroll(
                        dioxus::html::geometry::PixelsVector2D::new(width, 0.0),
                        ScrollBehavior::Instant,
                    ).await;
                },
                svg {
                    class: "heat-grid activity-field-grid",
                    style: "width:{width}px;",
                    view_box: "0 0 {width} {height}",
                    "aria-hidden": "true",
                    HeatTiles { across: tiles_across(columns) }
                }
            }
        }
    }
}

/// `n` and its noun, singular for one.
fn counted(n: u32, one: &str, many: &str) -> String {
    let noun = if n == 1 { one } else { many };
    format!("{} {noun}", with_separators(u64::from(n)))
}

/// A tapped day's tags: its swipes, its searches and its date.
fn day_tags(cell: &DayCell) -> [String; 3] {
    [
        counted(cell.swipes, "swipe", "swipes"),
        counted(cell.searches, "search", "searches"),
        cell.date.format("%b %-d").to_string(),
    ]
}

/// The heatmap's year in words, its description for screen readers.
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

/// The heatmap's resting tags: the year's swipes, searches and span.
fn year_tags(days: &[HttpActivityDay]) -> [String; 3] {
    let swipes = days
        .iter()
        .map(|d| d.swipes)
        .fold(0u32, u32::saturating_add);
    let searches = days
        .iter()
        .map(|d| d.searches)
        .fold(0u32, u32::saturating_add);
    [
        counted(swipes, "swipe", "swipes"),
        counted(searches, "search", "searches"),
        "Last year".to_string(),
    ]
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

/// Which share of dealt cards the Rates section shows, picked from its
/// chip row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RateKind {
    /// Added to a deck.
    Added,
    /// Skipped.
    Skipped,
    /// Saved to a maybeboard.
    Maybed,
    /// Added and later taken out.
    Removed,
}

impl RateKind {
    /// The chip row's order.
    const ALL: [Self; 4] = [Self::Added, Self::Skipped, Self::Maybed, Self::Removed];

    /// The chip's label, the Swipes section's column name.
    fn label(self) -> &'static str {
        match self {
            Self::Added => "Added",
            Self::Skipped => "Skipped",
            Self::Maybed => "Maybeboard",
            Self::Removed => "Removed",
        }
    }

    /// The outcome counted and what it is a share of: the cards dealt, or
    /// for Removed the cards added.
    fn parts(self, outcomes: &HttpSwipeOutcomes) -> (u64, u64) {
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

    /// The share as a whole percent, rounded down; `None` with nothing to
    /// take a share of.
    fn percent(self, outcomes: &HttpSwipeOutcomes) -> Option<u32> {
        let (count, of) = self.parts(outcomes);
        (of > 0).then(|| u32::try_from(count.saturating_mul(100) / of).unwrap_or(100))
    }
}

/// How many cards' worth of the overall rate a tag's rate among other
/// players starts from, so a tag few others have been dealt sits near the
/// overall rate. The server's hard passes weigh the same way.
const TAG_BASELINE_WEIGHT: f64 = 10.0;

/// The lift a tag needs to be shown: the user's count times the log of how
/// much their rate exceeds other players'. A tag others pass on as often
/// takes many more cards to get there.
const TAG_MIN_SCORE: f64 = 1.0;

/// Rows on the Oracle tags chart.
const TAG_ROWS: usize = 10;

/// A count as a float, capped at `u32::MAX`, far past any real count.
fn float(n: u64) -> f64 {
    f64::from(u32::try_from(n).unwrap_or(u32::MAX))
}

/// One row per role with a share to take, labeled like the Card roles
/// chart, its bar and count the share as a percent, highest first and then
/// in the server's order. A role with any of `tags` under it reveals them.
fn role_rate_bars(kind: RateKind, roles: &[HttpRoleRates], tags: &[HttpTagRates]) -> Vec<Bar> {
    let mut rated: Vec<(&HttpRoleRates, u32)> = roles
        .iter()
        .filter_map(|r| kind.percent(&r.outcomes).map(|rate| (r, rate)))
        .collect();
    rated.sort_by_key(|(_, rate)| Reverse(*rate));
    rated
        .into_iter()
        .map(|(role, rate)| Bar {
            key: role.role.clone(),
            label: role_short_label(&role.role),
            count: usize::try_from(rate).unwrap_or(100),
            pct: rate,
            revealable: tags.iter().any(|t| t.role == role.role),
        })
        .collect()
}

/// Other players' share across every tag sent, the rate a tag few of them
/// have been dealt starts from. Even odds with nothing to go on.
fn baseline(kind: RateKind, tags: &[HttpTagRates]) -> f64 {
    let (count, of) = tags.iter().fold((0u64, 0u64), |(count, of), t| {
        let (c, o) = kind.parts(&t.everyone);
        (count.saturating_add(c), of.saturating_add(o))
    });
    if of == 0 {
        0.5
    } else {
        float(count) / float(of)
    }
}

/// How much a tag's share stands out for the user: their count times the
/// log of their rate over other players', the others' rate smoothed toward
/// `baseline` by [`TAG_BASELINE_WEIGHT`] cards. `None` when the user's rate
/// is no higher.
fn lift_score(kind: RateKind, tag: &HttpTagRates, baseline: f64) -> Option<f64> {
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

/// The [`TAG_ROWS`] tags whose share stands out most from other players',
/// each at [`TAG_MIN_SCORE`] or more, most first, labeled by slug, the bar
/// and count the user's own share as a percent.
fn tag_rate_bars(kind: RateKind, tags: &[HttpTagRates]) -> Vec<Bar> {
    let base = baseline(kind, tags);
    let mut scored: Vec<(&HttpTagRates, f64)> = tags
        .iter()
        .filter_map(|t| lift_score(kind, t, base).map(|score| (t, score)))
        .filter(|(_, score)| *score >= TAG_MIN_SCORE)
        .collect();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.tag.cmp(&b.0.tag)));
    scored
        .into_iter()
        .take(TAG_ROWS)
        .filter_map(|(tag, _)| {
            let rate = kind.percent(&tag.outcomes)?;
            Some(Bar {
                key: tag.tag.clone(),
                label: tag.tag.clone(),
                count: usize::try_from(rate).unwrap_or(100),
                pct: rate,
                revealable: true,
            })
        })
        .collect()
}

/// The year as a heatmap on a field of the backdrop's tiles, edge to edge
/// in its section: weekday labels down the left, month labels over the
/// top, a day on each tile of the grid shaded by its swipes and searches in
/// orangui's [`HeatGrid`]. Wider than a phone, it scrolls sideways and opens
/// on the latest week. Under a rule, orangui's [`HeatReadout`] like an
/// action bar: the year's swipes, searches and span as tags, which a tapped
/// day's take over for a few seconds on a touch or narrow screen. A hover
/// device with room names the day under the pointer in a floating chip
/// instead.
#[component]
fn ActivityHeatmap(through: NaiveDate, days: Vec<HttpActivityDay>) -> Element {
    let pick = use_heat_pick();
    let mut hover: Signal<Option<(String, f64, f64)>> = use_signal(|| None);
    let layout = heat_layout(through, &days);
    let (width, height) = field_size(layout.columns);
    let tags: Vec<[String; 3]> = layout.cells.iter().map(day_tags).collect();
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
    let tap_tags = tags.clone();
    let on_tap = move |hit: HeatHit| {
        if let Some(day) = tap_tags.get(hit.index) {
            pick.show(day.to_vec());
        }
    };
    let on_enter = move |hit: HeatHit| {
        if let Some(day) = tags.get(hit.index) {
            let x = (hit.x + HEAT_CELL / 2.0) / width * 100.0;
            let y = hit.y / height * 100.0;
            hover.set(Some((day.join(", "), x, y)));
        }
    };
    rsx! {
        div { class: "activity-heat",
            div { class: "activity-field",
                div {
                    class: "chart-scroll scroll-fade-x activity-scroll",
                    onmounted: move |event| async move {
                        let _ = event.data().scroll(
                            dioxus::html::geometry::PixelsVector2D::new(width, 0.0),
                            ScrollBehavior::Instant,
                        ).await;
                    },
                    div {
                        class: "activity-plot",
                        style: "width:{width}px;",
                        onmouseleave: move |_| hover.set(None),
                        svg {
                            class: "heat-grid activity-field-grid",
                            style: "width:{width}px;",
                            view_box: "0 0 {width} {height}",
                            role: "img",
                            "aria-label": "{year_line(&days)}",
                            HeatTiles { across: tiles_across(layout.columns) }
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
                                    x: "{LABEL_INSET}",
                                    y: "{TOP + heat_span(row) + HEAT_CELL - 2.0}",
                                    "{name}"
                                }
                            }
                            HeatGrid {
                                cells: grid,
                                left: LEFT,
                                top: TOP,
                                on_enter,
                                on_tap,
                            }
                        }
                        if let Some((text, left, top)) = hover() {
                            ChartTip { text, left, top, class: "tag heat-float" }
                        }
                    }
                }
            }
            hr { class: "card-row-rule card-row-rule-muted activity-heat-rule" }
            HeatReadout { rest: year_tags(&days).to_vec(), pick }
        }
    }
}

/// The screen's three sections in one accordion: Activity, the year's
/// heatmap with its totals under it and what the user's swipes came to as
/// columns; Card roles, the picked share per role, its chips revealing
/// the oracle tags swiped under each; and Oracle tags, the tags whose share
/// stands out most from other players', each revealing its description. The
/// last two share one [`RateChips`] pick of Added, Skipped, Maybeboard or
/// Removed. Each section is left out when it has nothing to show. The
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
    let kind = use_signal(|| RateKind::Added);
    let has_days = !activity.days.is_empty();
    let outcomes = activity.outcomes;
    let swipe_tags = activity.tags.clone();
    let role_bars = role_rate_bars(kind(), &activity.roles, &activity.tags);
    let tag_bars = tag_rate_bars(kind(), &activity.tags);
    let mut selected_role: Signal<Option<String>> = use_signal(|| None);
    let mut selected_tag: Signal<Option<String>> = use_signal(|| None);
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let role_reveal = use_callback(move |role: String| {
        let tags: Vec<String> = swipe_tags
            .iter()
            .filter(|t| t.role == role)
            .map(|t| t.tag.clone())
            .collect();
        rsx! {
            RoleTags { tags, describe_tag, on_examples }
        }
    });
    let tag_reveal = use_callback(move |slug: String| {
        rsx! {
            OtagDef { slug, describe_tag, on_examples }
        }
    });
    let toggle_role = use_callback(move |key: String| {
        if selected_role().as_deref() == Some(key.as_str()) {
            selected_role.set(None);
        } else {
            selected_role.set(Some(key));
        }
    });
    let toggle_tag = use_callback(move |key: String| {
        if selected_tag().as_deref() == Some(key.as_str()) {
            selected_tag.set(None);
        } else {
            selected_tag.set(Some(key));
        }
    });
    rsx! {
        if has_days || has_outcomes(&outcomes) {
            CollapsibleSection {
                title: ACTIVITY_TITLE,
                open_section,
                header_accessory: rsx! {
                    InfoButton { topic: HintTopic::Activity }
                },
                if has_days {
                    for run in [opens()] {
                        ActivityHeatmap {
                            key: "{run}",
                            through: activity.through,
                            days: activity.days.clone(),
                        }
                    }
                }
                if has_outcomes(&outcomes) {
                    if has_days {
                        hr { class: "card-row-rule card-row-rule-muted activity-rule" }
                    }
                    div { class: "activity-outcomes",
                        BarChart { label: "Totals", bars: outcome_bars(&outcomes) }
                    }
                }
            }
        }
        if !activity.roles.is_empty() {
            CollapsibleSection {
                title: ROLES_TITLE,
                open_section,
                header_accessory: rsx! {
                    InfoButton { topic: HintTopic::RoleRates }
                },
                div { class: "activity-rates",
                    RateChips { kind }
                    // Keyed on the rate, so a switch mounts a fresh chart:
                    // patched in place, rows that re-sort and flip between a
                    // chip and a plain label kept stale bar widths.
                    for rate in [kind().label()] {
                        div { key: "{rate}",
                            if role_bars.is_empty() {
                                div { class: "activity-total", "Nothing to show for this rate yet" }
                            } else {
                                BarChart {
                                    bars: role_bars.clone(),
                                    orientation: BarOrientation::Horizontal,
                                    on_bar: toggle_role,
                                    selected: selected_role(),
                                    reveal: role_reveal,
                                    count_suffix: "%",
                                }
                            }
                        }
                    }
                }
            }
        }
        if !activity.tags.is_empty() {
            CollapsibleSection {
                title: TAGS_TITLE,
                open_section,
                header_accessory: rsx! {
                    InfoButton { topic: HintTopic::TagRates }
                },
                div { class: "activity-rates",
                    RateChips { kind }
                    for rate in [kind().label()] {
                        div { key: "{rate}",
                            if tag_bars.is_empty() {
                                div { class: "activity-total", "Nothing to show for this rate yet" }
                            } else {
                                BarChart {
                                    bars: tag_bars.clone(),
                                    orientation: BarOrientation::Horizontal,
                                    // Tag slugs read as written, as they do everywhere else.
                                    label_case: LabelCase::AsIs,
                                    on_bar: toggle_tag,
                                    selected: selected_tag(),
                                    reveal: tag_reveal,
                                    count_suffix: "%",
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The Show chip row over the role and tag charts: Added, Skipped,
/// Maybeboard or Removed, one always picked. Both sections render it over
/// the same signal, so a pick in either switches both.
#[component]
fn RateChips(mut kind: Signal<RateKind>) -> Element {
    rsx! {
        div { class: "chip-row activity-rate-chips",
            span { class: "stat-chip stat-chip-p1 chip-row-label", "Show" }
            for choice in RateKind::ALL {
                Chip {
                    key: "{choice.label()}",
                    selected: kind() == choice,
                    onclick: move |_| kind.set(choice),
                    "{choice.label()}"
                }
            }
        }
    }
}

/// One hard pass: the shared expandable card row with its skips as a tag
/// where a deck list puts copies and, when expanded, the commanders it was
/// skipped under as chips under a "Skipped under" label, in the Card roles
/// label's grammar. No actions beyond the row's own Printing.
#[component]
fn HardPassRow(
    pass: HttpHardPass,
    expanded_card: Signal<Option<Uuid>>,
    preview_card: Signal<Option<(ScryfallData, usize)>>,
    preview_dismissing: Signal<bool>,
    on_printing: EventHandler<Card>,
) -> Element {
    let skips = counted(pass.skipped, "skip", "skips");
    let commanders = pass.commanders;
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
                if !commanders.is_empty() {
                    span { class: "chips-label", "Skipped under" }
                    div { class: "keyword-chips",
                        for name in commanders {
                            span { key: "{name}", class: "detail-chip detail-chip-pt", "{name}" }
                        }
                    }
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
    fn days_land_on_whole_tiles_in_every_shade() {
        assert!((LEFT / HEAT_STEP - 2.0).abs() < f64::EPSILON);
        assert!((TOP / HEAT_STEP - 1.0).abs() < f64::EPSILON);
        let layout = heat_layout(date(2026, 10, 27), &[]);
        let (width, height) = field_size(layout.columns);
        let across = tiles_across(layout.columns);
        assert!((heat_span(across) - width).abs() < f64::EPSILON);
        assert!((heat_span(FIELD_ROWS) - height).abs() < f64::EPSILON);
        let mut seen = [false; TILE_SHADES];
        for column in 0..across {
            for row in 0..FIELD_ROWS {
                let shade = tile_shade(column, row);
                assert_eq!(shade, tile_shade(column, row));
                if let Some(slot) = seen.get_mut(shade) {
                    *slot = true;
                }
            }
        }
        assert!(seen.iter().all(|&s| s), "{seen:?}");
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
    fn the_year_rests_as_its_totals_and_span() {
        let on = date(2026, 10, 3);
        let days = [day(on, 1200, 2), day(on, 34, 0)];
        assert_eq!(
            year_tags(&days),
            ["1,234 swipes", "2 searches", "Last year"]
        );
        assert_eq!(year_tags(&[]), ["0 swipes", "0 searches", "Last year"]);
    }

    #[test]
    fn a_day_reads_as_its_swipes_searches_and_date() {
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
        assert_eq!(day_tags(&cell(0, 0)), ["0 swipes", "0 searches", "Oct 3"]);
        assert_eq!(
            day_tags(&cell(1200, 1)),
            ["1,200 swipes", "1 search", "Oct 3"]
        );
    }

    fn outcomes(added: u64, skipped: u64, maybed: u64, removed: u64) -> HttpSwipeOutcomes {
        HttpSwipeOutcomes {
            added,
            skipped,
            maybed,
            removed,
        }
    }

    fn tag(
        slug: &str,
        role: &str,
        mine: HttpSwipeOutcomes,
        everyone: HttpSwipeOutcomes,
    ) -> HttpTagRates {
        HttpTagRates {
            tag: slug.to_string(),
            role: role.to_string(),
            outcomes: mine,
            everyone,
        }
    }

    #[test]
    fn removed_is_a_share_of_the_added_cards_and_the_rest_of_the_dealt() {
        let o = outcomes(4, 5, 1, 1);
        assert_eq!(RateKind::Added.percent(&o), Some(40));
        assert_eq!(RateKind::Skipped.percent(&o), Some(50));
        assert_eq!(RateKind::Maybed.percent(&o), Some(10));
        assert_eq!(RateKind::Removed.percent(&o), Some(25));
        assert_eq!(
            RateKind::Removed.percent(&outcomes(0, 9, 0, 0)),
            None,
            "nothing added"
        );
    }

    #[test]
    fn role_rates_sort_highest_first_and_reveal_only_swiped_tags() {
        let role = |slug: &str, o| HttpRoleRates {
            role: slug.to_string(),
            outcomes: o,
        };
        let swiped = [tag(
            "mana-rock",
            "ramp",
            outcomes(5, 5, 0, 0),
            outcomes(0, 0, 0, 0),
        )];
        let bars = role_rate_bars(
            RateKind::Added,
            &[
                role("ramp", outcomes(1, 2, 0, 0)),
                role("removal", outcomes(9, 1, 0, 0)),
            ],
            &swiped,
        );
        let rows: Vec<(String, usize, bool)> = bars
            .into_iter()
            .map(|b| (b.key, b.count, b.revealable))
            .collect();
        assert_eq!(
            rows,
            [
                ("removal".to_string(), 90, false),
                ("ramp".to_string(), 33, true)
            ]
        );
    }

    #[test]
    fn a_tag_everyone_passes_on_needs_far_more_skips_than_one_others_keep() {
        let common = tag(
            "generic",
            "ramp",
            outcomes(1, 9, 0, 0),
            outcomes(10, 90, 0, 0),
        );
        let standout = tag(
            "mana-rock",
            "ramp",
            outcomes(1, 9, 0, 0),
            outcomes(80, 20, 0, 0),
        );
        let bars = tag_rate_bars(RateKind::Skipped, &[common, standout]);
        let keys: Vec<String> = bars.into_iter().map(|b| b.key).collect();
        assert_eq!(
            keys,
            ["mana-rock"],
            "skipping what everyone skips is no news"
        );
    }

    #[test]
    fn the_tag_chart_shows_at_most_its_rows() {
        let tags: Vec<HttpTagRates> = (0..20)
            .map(|i| {
                tag(
                    &format!("t{i:02}"),
                    "ramp",
                    outcomes(0, 20, 0, 0),
                    outcomes(90, 10, 0, 0),
                )
            })
            .collect();
        assert_eq!(tag_rate_bars(RateKind::Skipped, &tags).len(), TAG_ROWS);
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
