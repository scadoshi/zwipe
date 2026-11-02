//! The Activity screen's parts: a heatmap of the user's swipes and searches
//! over the last year, what their swipes came to, their rates per card role
//! and oracle tag, and the cards they keep passing on. The first three are
//! sections in one accordion; the hard passes are a card group in Staples'
//! grammar. Every number and line comes from the screen's model and
//! `zwipe_ui::activity`; this module draws them.

use components::{
    BarChart, BarOrientation, ChartTip, Chip, HEAT_CELL, HEAT_ROWS, HeatCell, HeatGrid, HeatHit,
    HeatReadout, LabelCase, OtagDef, RoleTags, use_heat_pick,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::http::contracts::suggestions::HttpHardPass;
use zwipe_ui::activity::{self, HeatLayout, RateKind, WEEKDAY_LABELS};
use zwipe_ui::screens::activity::{self as model, Action, Rates, Section, State};

use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::InfoButton;
use crate::inbound::screens::deck::card::components::card_row::{
    CardRow, OtagDescribe, OtagExamplesOpen,
};
use crate::inbound::screens::deck::components::collapsible_section::CollapsibleSection;

/// Room on the left of the grid for the weekday labels.
const LEFT: f64 = 24.0;
/// Room above the grid for the month labels.
const TOP: f64 = 14.0;

/// zwipe-ui's chart bars as the kit's chart draws them.
fn chart_bars(bars: Vec<activity::Bar>) -> Vec<components::Bar> {
    bars.into_iter()
        .map(|bar| components::Bar {
            key: bar.key,
            label: bar.label,
            count: usize::try_from(bar.count).unwrap_or(usize::MAX),
            pct: bar.pct,
            revealable: bar.revealable,
        })
        .collect()
}

/// The year as a heatmap: weekday labels down the left, month labels over
/// the top, a cell per day shaded by its swipes and searches in orangui's
/// [`HeatGrid`]. Wider than a phone, it scrolls sideways and opens on the
/// latest week. Under it, orangui's [`HeatReadout`]: the year's swipes,
/// searches and span as tags, which a tapped day's take over for a few
/// seconds on a touch or narrow screen. A hover device with room names the
/// day under the pointer in a floating chip instead.
#[component]
fn ActivityHeatmap(layout: HeatLayout, rest: Vec<String>, label: String) -> Element {
    let pick = use_heat_pick();
    let mut hover: Signal<Option<(String, f64, f64)>> = use_signal(|| None);
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
                    class: "heat-grid",
                    style: "width:{width}px;",
                    view_box: "0 0 {width} {height}",
                    role: "img",
                    "aria-label": "{label}",
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
        HeatReadout { rest, pick }
    }
}

/// The heatmap while the screen loads: empty space at the loaded grid's
/// height, so nothing under it moves when the grid lands. No squares and no
/// labels, which the loaded grid brings.
#[component]
pub(crate) fn HeatGhost() -> Element {
    let height = TOP + components::heat_span(HEAT_ROWS);
    rsx! {
        div { class: "chart-scroll activity-scroll",
            div { class: "activity-plot", style: "height:{height}px;" }
        }
    }
}

/// The screen's three sections in one accordion: Activity, the year's
/// heatmap with its totals under it and what the user's swipes came to as
/// columns; Card roles, the picked share per role, its chips revealing
/// the oracle tags swiped under each; and Oracle tags, the tags whose share
/// stands out most from other players', each revealing its description. The
/// last two share one [`RateChips`] pick of Added, Skipped, Maybeboard or
/// Removed. Activity is left out when it has nothing to show; Card roles
/// and Oracle tags stay, with a line saying what fills them. The
/// heatmap mounts afresh each time its section opens, so its sweep plays in
/// view.
#[component]
pub(crate) fn ActivitySections(
    model: Signal<State>,
    on_action: EventHandler<Action>,
    open_section: Signal<Option<String>>,
) -> Element {
    let open = open_section().as_deref() == Some(Section::Activity.title());
    let mut opens = use_signal(|| 0u32);
    use_effect(use_reactive!(|open| {
        if open {
            let next = opens.peek().wrapping_add(1);
            opens.set(next);
        }
    }));
    let state = model.read();
    let shows_year = state.sections().contains(&Section::Activity);
    let heat = state.heat();
    let year_tags = state.year_tags().to_vec();
    let year_line = state.year_line();
    let shows_totals = state.shows_totals();
    let totals = chart_bars(state.totals());
    let rate = state.rate();
    let role_rates = state.role_rates();
    let tag_rates = state.tag_rates();
    let selected_role = state.revealed_role().map(str::to_owned);
    let selected_tag = state.revealed_tag().map(str::to_owned);
    drop(state);
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let role_reveal = use_callback(move |role: String| {
        let tags = model.peek().tags_under(&role);
        rsx! {
            RoleTags { tags, describe_tag, on_examples }
        }
    });
    let tag_reveal = use_callback(move |slug: String| {
        rsx! {
            OtagDef { slug, describe_tag, on_examples }
        }
    });
    let toggle_role = use_callback(move |key: String| on_action.call(Action::ToggleRole(key)));
    let toggle_tag = use_callback(move |key: String| on_action.call(Action::ToggleTag(key)));
    let on_pick = move |picked: RateKind| on_action.call(Action::PickRate(picked));
    let roles_body = match role_rates {
        Rates::Unrated(line) => rsx! {
            div { class: "activity-total", "{line}" }
        },
        rates => rsx! {
            div { class: "activity-rates",
                RateChips { rate, on_pick }
                // Keyed on the rate, so a switch mounts a fresh chart:
                // patched in place, rows that re-sort and flip between a
                // tappable chip and a grey one kept stale bar widths.
                for label in [rate.label()] {
                    div { key: "{label}",
                        // Only Removed can chart no role: every role sent
                        // has a dealt card, but Removed needs an added one.
                        if let Rates::Charted(bars) = &rates {
                            BarChart {
                                bars: chart_bars(bars.clone()),
                                orientation: BarOrientation::Horizontal,
                                on_bar: toggle_role,
                                selected: selected_role.clone(),
                                reveal: role_reveal,
                                count_suffix: "%",
                            }
                        } else if let Rates::Uncharted(line) = &rates {
                            div { class: "activity-total", "{line}" }
                        }
                    }
                }
            }
        },
    };
    let tags_body = match tag_rates {
        Rates::Unrated(line) => rsx! {
            div { class: "activity-total", "{line}" }
        },
        rates => rsx! {
            div { class: "activity-rates",
                RateChips { rate, on_pick }
                for label in [rate.label()] {
                    div { key: "{label}",
                        if let Rates::Charted(bars) = &rates {
                            BarChart {
                                bars: chart_bars(bars.clone()),
                                orientation: BarOrientation::Horizontal,
                                // Tag slugs read as written, as they do everywhere else.
                                label_case: LabelCase::AsIs,
                                on_bar: toggle_tag,
                                selected: selected_tag.clone(),
                                reveal: tag_reveal,
                                count_suffix: "%",
                            }
                        } else if let Rates::Uncharted(line) = &rates {
                            div { class: "activity-total", "{line}" }
                        }
                    }
                }
            }
        },
    };
    rsx! {
        if shows_year {
            CollapsibleSection {
                title: Section::Activity.title(),
                open_section,
                header_accessory: rsx! {
                    InfoButton { topic: HintTopic::Activity }
                },
                if let Some(layout) = heat.clone() {
                    for run in [opens()] {
                        ActivityHeatmap {
                            key: "{run}",
                            layout: layout.clone(),
                            rest: year_tags.clone(),
                            label: year_line.clone(),
                        }
                    }
                }
                if shows_totals {
                    if heat.is_some() {
                        hr { class: "card-row-rule card-row-rule-muted activity-rule" }
                    }
                    div { class: "activity-outcomes",
                        BarChart { label: model::TOTALS, bars: totals }
                    }
                }
            }
        }
        CollapsibleSection {
            title: Section::Roles.title(),
            open_section,
            header_accessory: rsx! {
                InfoButton { topic: HintTopic::RoleRates }
            },
            {roles_body}
        }
        CollapsibleSection {
            title: Section::Tags.title(),
            open_section,
            header_accessory: rsx! {
                InfoButton { topic: HintTopic::TagRates }
            },
            {tags_body}
        }
    }
}

/// The Show chip row over the role and tag charts: Added, Skipped,
/// Maybeboard or Removed, one always picked. Both sections render it over
/// the model's one pick, so a pick in either switches both.
#[component]
fn RateChips(rate: RateKind, on_pick: EventHandler<RateKind>) -> Element {
    rsx! {
        div { class: "chip-row activity-rate-chips",
            span { class: "stat-chip stat-chip-p1 chip-row-label", {model::SHOW} }
            for choice in RateKind::ALL {
                Chip {
                    key: "{choice.label()}",
                    selected: rate == choice,
                    onclick: move |_| on_pick.call(choice),
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
    let skips = model::skips(&pass);
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
                    span { class: "chips-label", {model::SKIPPED_UNDER} }
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
/// players first. Without any, the header over a line saying what fills it.
#[component]
pub(crate) fn HardPassesPanel(
    passes: Vec<HttpHardPass>,
    line: Option<&'static str>,
    expanded_card: Signal<Option<Uuid>>,
    preview_card: Signal<Option<(ScryfallData, usize)>>,
    preview_dismissing: Signal<bool>,
    on_printing: EventHandler<Card>,
) -> Element {
    rsx! {
        div { class: "maybeboard-list card-group row-enter",
            div { class: "card-group-header card-group-header-with-hint",
                span { {model::PASSES_TITLE} }
                InfoButton { topic: HintTopic::HardPasses }
            }
            if let Some(line) = line {
                div { class: "activity-total activity-total-group", "{line}" }
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
