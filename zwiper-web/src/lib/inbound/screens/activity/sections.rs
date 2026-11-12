//! Activity's parts: the year's heatmap with the totals, the rates per card
//! role and per oracle tag, and the cards the user keeps passing on. Every
//! number comes from zwipe-ui's `activity`; this module draws it.

use std::collections::HashMap;

use chrono::NaiveDate;
use components::{
    BarChart, BarOrientation, CardListFold, ChartTip, Chip, HEAT_CELL, HEAT_ROWS, HeatCell,
    HeatGrid, HeatHit, HeatReadout, LabelCase, OtagDef, RoleTags, use_heat_pick,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::suggestions::{HttpActivity, HttpActivityDay, HttpHardPass};
use zwipe_ui::activity::{
    self, NONE_REMOVED, PASSES_EMPTY, ROLES_EMPTY, RateKind, TAGS_EMPTY, WEEKDAY_LABELS,
};
use zwipe_ui::deck_cards::stacks::CardsView;

use crate::inbound::cards::card_row::CardRow;
use crate::inbound::cards::charts::chart_bars;
use crate::inbound::cards::otag::{OtagDescribe, OtagExamplesOpen};
use crate::inbound::cards::stack_view::{CardsStack, CardsViewSwitch, StackDrawer};
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::chrome::section::CollapsibleSection;

/// The accordion title of the heatmap section, open on entry.
pub const ACTIVITY_TITLE: &str = "Activity";
/// The accordion title of the role rates section.
const ROLES_TITLE: &str = "Card roles";
/// The accordion title of the tag rates section.
const TAGS_TITLE: &str = "Oracle tags";
/// The hard passes group's header.
const PASSES_TITLE: &str = "Most skipped";

/// Room on the left of the grid for the weekday labels, in grid units.
const LEFT: f64 = 24.0;
/// Room above the grid for the month labels, in grid units.
const TOP: f64 = 14.0;
/// A year's week columns, the heatmap's width while it loads.
const GHOST_COLUMNS: usize = 54;

/// The year as a heatmap: weekday labels down the left, month labels over
/// the top, a cell per day in the kit's [`HeatGrid`], opening scrolled to
/// the newest week. Under it the kit's [`HeatReadout`]: the year's tags,
/// which a clicked day's take over for a few seconds. Hovering a day names
/// it in a floating chip.
#[component]
fn ActivityHeatmap(through: NaiveDate, days: Vec<HttpActivityDay>) -> Element {
    let pick = use_heat_pick();
    let mut hover: Signal<Option<(String, f64, f64)>> = use_signal(|| None);
    let layout = activity::heat_layout(through, &days);
    let width = LEFT + components::heat_span(layout.columns);
    let height = TOP + components::heat_span(HEAT_ROWS);
    let tags: Vec<[String; 3]> = layout.cells.iter().map(activity::day_tags).collect();
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
        div {
            class: "chart-scroll scroll-fade-x activity-scroll",
            onmounted: move |event| async move {
                let _ = event
                    .data()
                    .scroll(
                        dioxus::html::geometry::PixelsVector2D::new(width, 0.0),
                        ScrollBehavior::Instant,
                    )
                    .await;
            },
            div {
                class: "activity-plot",
                style: "width:{width}px;",
                onmouseleave: move |_| hover.set(None),
                // The weekday names again, pinned at the scroller's left
                // over the grid's own, so they stay in view as the year
                // scrolls under them.
                div {
                    class: "activity-days",
                    style: "width:{LEFT}px;height:{height}px;margin-bottom:-{height}px;",
                    "aria-hidden": "true",
                    svg {
                        width: "{LEFT}",
                        height: "{height}",
                        view_box: "0 0 {LEFT} {height}",
                        for (row, name) in WEEKDAY_LABELS {
                            text {
                                key: "p{row}",
                                class: "heat-label",
                                x: "0",
                                y: "{TOP + components::heat_span(row) + HEAT_CELL - 2.0}",
                                "{name}"
                            }
                        }
                    }
                }
                svg {
                    class: "heat-grid",
                    style: "width:{width}px;",
                    view_box: "0 0 {width} {height}",
                    role: "img",
                    "aria-label": "{activity::year_line(&days)}",
                    for (column, name) in layout.months.iter() {
                        text {
                            key: "m{column}",
                            class: "heat-label",
                            x: "{LEFT + components::heat_span(*column)}",
                            y: "{TOP - 4.0}",
                            "{name}"
                        }
                    }
                    for (row, name) in WEEKDAY_LABELS {
                        text {
                            key: "d{row}",
                            class: "heat-label",
                            x: "0",
                            y: "{TOP + components::heat_span(row) + HEAT_CELL - 2.0}",
                            "{name}"
                        }
                    }
                    HeatGrid { cells: grid, left: LEFT, top: TOP, on_enter, on_tap }
                }
                if let Some((text, left, top)) = hover() {
                    ChartTip { text, left, top, class: "tag heat-float" }
                }
            }
        }
        HeatReadout { rest: activity::year_tags(&days).to_vec(), pick }
    }
}

/// The heatmap while the screen loads: empty space in a year's grid's
/// shape, at its width on a phone and scaled to the column on a wide
/// window as the grid is, so nothing under it moves when the grid lands.
#[component]
pub fn HeatGhost() -> Element {
    let width = LEFT + components::heat_span(GHOST_COLUMNS);
    let height = TOP + components::heat_span(HEAT_ROWS);
    rsx! {
        div { class: "chart-scroll activity-scroll",
            div { class: "activity-plot", style: "width:{width}px;aspect-ratio:{width} / {height};" }
        }
    }
}

/// The Activity section: the year's heatmap over the Totals columns, left
/// out with nothing to show. The heatmap mounts afresh each time the
/// section opens, so its sweep plays in view. On a wide window it is open.
#[component]
pub fn ActivityYear(activity: HttpActivity, open_section: Signal<Option<String>>) -> Element {
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
    let has_outcomes = activity::has_outcomes(&outcomes);
    rsx! {
        if has_days || has_outcomes {
            div { class: "web-activity-year",
                CollapsibleSection {
                    title: ACTIVITY_TITLE,
                    open_when_wide: true,
                    open_section,
                    header_accessory: rsx! { InfoButton { topic: HintTopic::Activity } },
                    div { class: "web-activity-year-body",
                        if has_days {
                            div { class: "web-activity-heat",
                                for run in [opens()] {
                                    ActivityHeatmap {
                                        key: "{run}",
                                        through: activity.through,
                                        days: activity.days.clone(),
                                    }
                                }
                            }
                        }
                        if has_outcomes {
                            if has_days {
                                hr { class: "card-row-rule card-row-rule-muted activity-rule" }
                            }
                            div { class: "activity-outcomes",
                                BarChart { label: "Totals", bars: chart_bars(activity::outcome_bars(&outcomes)) }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Card roles and Oracle tags: a rate chart each over one shared `Show`
/// pick. Both stay with nothing to show, with a line saying what fills
/// them. On a wide window both are open.
#[component]
pub fn ActivityRates(activity: HttpActivity, open_section: Signal<Option<String>>) -> Element {
    let kind = use_signal(RateKind::default);
    let swipe_tags = activity.tags.clone();
    let role_bars = chart_bars(activity::role_rate_bars(
        kind(),
        &activity.roles,
        &activity.tags,
    ));
    let tag_bars = chart_bars(activity::tag_rate_bars(kind(), &activity.tags));
    let tag_empty = activity::tag_empty_line(kind(), &activity.tags);
    let mut selected_role: Signal<Option<String>> = use_signal(|| None);
    let mut selected_tag: Signal<Option<String>> = use_signal(|| None);
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let role_reveal = use_callback(move |role: String| {
        let tags = activity::tags_under(&role, &swipe_tags);
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
        let next = (selected_role().as_deref() != Some(key.as_str())).then_some(key);
        selected_role.set(next);
    });
    let toggle_tag = use_callback(move |key: String| {
        let next = (selected_tag().as_deref() != Some(key.as_str())).then_some(key);
        selected_tag.set(next);
    });
    rsx! {
        div { class: "web-activity-rates",
            CollapsibleSection {
                title: ROLES_TITLE,
                open_when_wide: true,
                open_section,
                header_accessory: rsx! { InfoButton { topic: HintTopic::RoleRates } },
                if activity.roles.is_empty() {
                    div { class: "activity-total",
                        span { class: "chip-note", "{ROLES_EMPTY}" }
                    }
                } else {
                    div { class: "activity-rates",
                        RateChips { kind }
                        for rate in [kind().label()] {
                            div { key: "{rate}",
                                if role_bars.is_empty() {
                                    div { class: "activity-total",
                                        span { class: "chip-note", "{NONE_REMOVED}" }
                                    }
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
            CollapsibleSection {
                title: TAGS_TITLE,
                open_when_wide: true,
                open_section,
                header_accessory: rsx! { InfoButton { topic: HintTopic::TagRates } },
                if activity.tags.is_empty() {
                    div { class: "activity-total",
                        span { class: "chip-note", "{TAGS_EMPTY}" }
                    }
                } else {
                    div { class: "activity-rates",
                        RateChips { kind }
                        for rate in [kind().label()] {
                            div { key: "{rate}",
                                if tag_bars.is_empty() {
                                    div { class: "activity-total",
                                        span { class: "chip-note", "{tag_empty}" }
                                    }
                                } else {
                                    BarChart {
                                        bars: tag_bars.clone(),
                                        orientation: BarOrientation::Horizontal,
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
}

/// The `Show` chips over the role and tag charts: Added, Skipped,
/// Maybeboard or Removed, one always picked. Both sections draw it over the
/// same signal, so a pick in either switches both.
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

/// One hard pass: its skips as a tag where a deck list puts copies and,
/// when open, the commanders it was skipped under.
#[component]
fn HardPassRow(pass: HttpHardPass, expanded_card: Signal<Option<Uuid>>) -> Element {
    let skips = activity::counted(pass.skipped, "skip", "skips");
    rsx! {
        CardRow {
            card: pass.card,
            qty: i32::try_from(pass.skipped).unwrap_or(i32::MAX),
            qty_tags: vec![skips],
            expanded_card,
            footer: skipped_under(pass.commanders),
        }
    }
}

/// The commanders a card was skipped under, as chips after a label; nothing
/// without any. A pass's row and its stack card's drawer both show it.
fn skipped_under(commanders: Vec<String>) -> Element {
    rsx! {
        if !commanders.is_empty() {
            div { class: "keyword-chips",
                span { class: "stat-chip stat-chip-p1 chip-row-label", "Skipped under" }
                for name in commanders {
                    span { key: "{name}", class: "stat-chip", "{name}" }
                }
            }
        }
    }
}

/// The cards the user keeps passing on, in the server's order (most skipped
/// against other players first), in a list that folds: rows, or one stack
/// in `view`'s Stacks, with its card details drawer. Without any, the header
/// over a line saying what fills it.
#[component]
pub fn HardPassesPanel(
    passes: Vec<HttpHardPass>,
    expanded_card: Signal<Option<Uuid>>,
    view: Signal<CardsView>,
) -> Element {
    let cards: Vec<Card> = passes.iter().map(|p| p.card.clone()).collect();
    let under: HashMap<Uuid, Vec<String>> = passes
        .iter()
        .map(|p| (p.card.scryfall_data.id, p.commanders.clone()))
        .collect();
    let extras = Callback::new(move |card: Card| {
        skipped_under(
            under
                .get(&card.scryfall_data.id)
                .cloned()
                .unwrap_or_default(),
        )
    });
    rsx! {
        CardListFold {
            title: PASSES_TITLE,
            actions: rsx! {
                InfoButton { topic: HintTopic::HardPasses }
            },
            if passes.is_empty() {
                div { class: "activity-total activity-total-group",
                    span { class: "chip-note", "{PASSES_EMPTY}" }
                }
            } else {
                div { class: "web-card-group-filters",
                    CardsViewSwitch { view, on_change: move |()| {
                        let mut expanded = expanded_card;
                        expanded.set(None);
                    } }
                }
            }
            if view() == CardsView::Stacks {
                if !cards.is_empty() {
                    CardsStack { cards: cards.clone() }
                }
            } else {
                for pass in passes {
                    HardPassRow { key: "{pass.card.scryfall_data.id}", pass, expanded_card }
                }
            }
        }
        if view() == CardsView::Stacks {
            StackDrawer { cards, extras }
        }
    }
}
