//! Activity's parts: the year's heatmap with the totals, the rates per card
//! role and per oracle tag, and the cards the user keeps passing on. Every
//! number comes from zwipe-ui's `activity`; this module draws it.

use chrono::NaiveDate;
use components::{
    BarChart, BarOrientation, CardListFold, Chip, HeatCell, Heatmap, LabelCase, OtagDef, RoleTags,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::http::contracts::suggestions::{HttpActivity, HttpActivityDay, HttpHardPass};
use zwipe_ui::activity::{
    self, NONE_REMOVED, PASSES_EMPTY, ROLES_EMPTY, RateKind, TAGS_EMPTY, WEEKDAY_LABELS,
};

use crate::inbound::cards::card_row::CardRow;
use crate::inbound::cards::charts::chart_bars;
use crate::inbound::cards::otag::{OtagDescribe, OtagExamplesOpen};
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

/// A year's week columns, the heatmap's width while it loads.
pub(crate) const GHOST_COLUMNS: usize = 54;

/// The year as the kit's [`Heatmap`]: weekday labels down the left, month labels over the top, a
/// cell per day, opening scrolled to the newest week. Under it the year's tags, which a clicked
/// day's take over for a few seconds. Hovering a day names it in a floating chip.
#[component]
fn ActivityHeatmap(through: NaiveDate, days: Vec<HttpActivityDay>) -> Element {
    let layout = activity::heat_layout(through, &days);
    let weekdays: Vec<(usize, String)> = WEEKDAY_LABELS
        .iter()
        .map(|&(row, name)| (row, name.to_owned()))
        .collect();
    let tags: Vec<Vec<String>> = layout
        .cells
        .iter()
        .map(|cell| activity::day_tags(cell).to_vec())
        .collect();
    let chips: Vec<String> = tags.iter().map(|day| day.join(", ")).collect();
    let cells: Vec<HeatCell> = layout
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
    rsx! {
        Heatmap {
            cells,
            columns: layout.columns,
            months: layout.months,
            weekdays,
            chips,
            tags,
            rest: activity::year_tags(&days).to_vec(),
            label: activity::year_line(&days),
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
/// without any.
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
/// against other players first), in a list that folds. Without any, the
/// header over a line saying what fills it.
#[component]
pub fn HardPassesPanel(passes: Vec<HttpHardPass>, expanded_card: Signal<Option<Uuid>>) -> Element {
    rsx! {
        CardListFold {
            title: PASSES_TITLE,
            hint: rsx! {
                InfoButton { topic: HintTopic::HardPasses }
            },
            if passes.is_empty() {
                div { class: "activity-total activity-total-group",
                    span { class: "chip-note", "{PASSES_EMPTY}" }
                }
            }
            for pass in passes {
                HardPassRow { key: "{pass.card.scryfall_data.id}", pass, expanded_card }
            }
        }
    }
}
