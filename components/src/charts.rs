//! Deck distribution charts, shared by the app's Distributions section and
//! the zwipe.net shared-deck page.
//!
//! Pure div-bar charts (no canvas, no JS) styled against the theme variables,
//! fed the chart-ready tuples `DeckMetrics` produces (`mana_curve_bars`,
//! `type_bars`, `card_role_bars`, `oracle_tag_bars`) and the color bars
//! zwipe-ui builds from it. Shared so both surfaces render the same picture. A horizontal chart with more than
//! [`FOLD_AT`] rows shows that many and folds the rest behind a [`SeeMore`]
//! chip.

use std::collections::BTreeMap;

use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::deck::deck_metrics::ManaBalanceRow;
use zwipe_core::domain::deck::draw_odds;
/// One bar of a [`BarChart`], zwipe-ui's chart bar, so a client hands the
/// bars zwipe-ui computes straight to the chart.
pub use zwipe_ui::activity::Bar;
use zwipe_ui::colors::Bar as ColorBar;

use crate::color_bars::{ColorBars, mana_var};
use crate::fold::{FOLD_AT, Fold, SeeMore, fold_split};
use crate::role_tags::{OtagDef, RoleTags};

/// A chart's title, a sentence-case chip in the theme's heading accent
/// centered over each flattened chart block.
///
/// `hint` is whatever the host wants right of the chip, in practice the app's
/// "?" button, beside the chip rather than inside it. It is an `Element`
/// rather than a topic because the hint system lives in the app and this
/// crate is shared with the site, which has none.
#[component]
pub fn ChartLabel(text: &'static str, hint: Option<Element>) -> Element {
    rsx! {
        span { class: "chart-label",
            span { class: "stat-chip chart-label-chip", "{text}" }
            if let Some(h) = hint { {h} }
        }
    }
}

/// Which way a [`BarChart`] lays its bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarOrientation {
    /// Columns in a row, counts above, labels in a row beneath.
    #[default]
    Vertical,
    /// One row per bar: label, bar, count.
    Horizontal,
}

/// How a [`BarChart`] cases its labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelCase {
    /// `text-transform:uppercase`, the deck charts' look.
    #[default]
    Upper,
    /// As given, for slugs.
    AsIs,
}

/// Width of a horizontal chart's label column unless the host sets one.
const DEFAULT_LABEL_WIDTH: &str = "5ch";

/// The widest a chip label column grows before its chips wrap, so the bar
/// keeps room on a phone.
const CHIP_LABEL_MAX_WIDTH: &str = "50%";

/// The fewest columns a vertical chart lays out: a chart with fewer bars
/// draws them at this many columns' width, centered, so one bar reads as a
/// bar and not a slab.
const VERTICAL_MIN_SLOTS: usize = 4;

/// A horizontal row's count cell.
const COUNT_CELL_STYLE: &str = "font-size:0.7rem;color:var(--text-primary);opacity:0.85;width:3ch;text-align:right;flex-shrink:0;";

/// [`COUNT_CELL_STYLE`] a character wider, for a count with a suffix.
const SUFFIXED_COUNT_CELL_STYLE: &str = "font-size:0.7rem;color:var(--text-primary);opacity:0.85;width:4ch;text-align:right;flex-shrink:0;";

/// How a [`BarChart`] lays its bars and labels out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layout {
    Vertical,
    /// Horizontal rows with plain text labels in a fixed-width column.
    Rows,
    /// Horizontal rows with tappable chip labels in a column as wide as the
    /// widest chip, and a reveal under the open row.
    ChipRows,
}

impl Layout {
    fn new(orientation: BarOrientation, tappable: bool) -> Self {
        match (orientation, tappable) {
            (BarOrientation::Vertical, _) => Self::Vertical,
            (BarOrientation::Horizontal, false) => Self::Rows,
            (BarOrientation::Horizontal, true) => Self::ChipRows,
        }
    }
}

/// Where a chip chart's reveal sits: under the row whose key is `key`, eased
/// open or shut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RevealAt<'k> {
    key: &'k str,
    open: bool,
}

/// The reveal follows `selected` and stays under the last opened row while
/// it closes, so its content is still there for the collapse to ease out.
fn reveal_at<'k>(selected: Option<&'k str>, shown: Option<&'k str>) -> Option<RevealAt<'k>> {
    match selected {
        Some(key) => Some(RevealAt { key, open: true }),
        None => shown.map(|key| RevealAt { key, open: false }),
    }
}

/// One count-based bar chart: an optional [`ChartLabel`] sub-heading, then
/// the bars with their counts and labels.
///
/// With `on_bar` set, each label is tappable and reports its bar's key, and
/// the bar whose key is `selected` is the open one. A vertical chart tints
/// that label in the accent color. A horizontal chart draws each label as a
/// `.chip`, selected when open, and eases `reveal`'s content open directly
/// under the open row, one row at a time. A bar that is not
/// [`Bar::revealable`] draws a muted static chip there and reports no tap.
///
/// With `fold` set, a horizontal chart shows its first [`FOLD_AT`] rows and
/// folds the rest behind a [`SeeMore`] chip; the fold's state is this chart's
/// and starts shut. A vertical chart never folds.
#[component]
pub fn BarChart(
    /// The [`ChartLabel`] over the bars; none renders the bars alone.
    #[props(default)]
    label: Option<&'static str>,
    bars: Vec<Bar>,
    /// "?" beside the label.
    #[props(default)]
    hint: Option<Element>,
    #[props(default)] orientation: BarOrientation,
    #[props(default)] label_case: LabelCase,
    /// Horizontal only: the label column's CSS width. A chip column sizes to
    /// its widest chip instead.
    #[props(default = DEFAULT_LABEL_WIDTH)]
    label_width: &'static str,
    #[props(default)] on_bar: Option<Callback<String>>,
    #[props(default)] selected: Option<String>,
    /// Horizontal with `on_bar` only: the content under the open row, given
    /// that row's key. It is also called for the row just closed while the
    /// reveal eases shut.
    #[props(default)]
    reveal: Option<Callback<String, Element>>,
    /// Printed after every count, such as `%` for a chart of rates. Empty
    /// prints the bare count.
    #[props(default)]
    count_suffix: &'static str,
    /// Horizontal only: show [`FOLD_AT`] rows and fold the rest.
    #[props(default)]
    fold: bool,
) -> Element {
    let layout = Layout::new(orientation, on_bar.is_some());
    let mut fold_open = use_signal(|| false);
    let fold_at = if fold { FOLD_AT } else { usize::MAX };
    let (shown_bars, folded_bars) = fold_split(&bars, fold_at);
    let toggle_fold = use_callback(move |()| fold_open.toggle());
    let count_style = if count_suffix.is_empty() {
        COUNT_CELL_STYLE
    } else {
        SUFFIXED_COUNT_CELL_STYLE
    };
    // The last opened key, kept through a close so the reveal can ease out.
    let mut shown: Signal<Option<String>> = use_signal(|| None);
    let shown_key = shown();
    let reveal_row = reveal_at(selected.as_deref(), shown_key.as_deref());
    let upper = match label_case {
        LabelCase::Upper => "text-transform:uppercase;",
        LabelCase::AsIs => "",
    };
    // A vertical chart's column: an equal share of at least
    // `VERTICAL_MIN_SLOTS` columns, its 0.25rem gaps taken out first.
    let slots = bars.len().max(VERTICAL_MIN_SLOTS);
    let column = format!("flex:0 0 calc((100% - {slots} * 0.25rem + 0.25rem) / {slots});");
    // The label's base style is the deck charts' exact string; a tappable
    // label adds the pointer, and the selected one the accent, after it.
    let horizontal_label = format!(
        "width:{label_width};font-size:0.7rem;color:var(--text-primary);opacity:0.85;text-align:right;flex-shrink:0;{upper}"
    );
    let label_style = |bar: &Bar| {
        let base = match orientation {
            BarOrientation::Vertical => format!(
                "{column}text-align:center;font-size:0.65rem;color:var(--text-primary);opacity:0.85;{upper}"
            ),
            BarOrientation::Horizontal => horizontal_label.clone(),
        };
        if on_bar.is_none() {
            return base;
        }
        let is_selected = selected.as_deref() == Some(bar.key.as_str());
        if is_selected {
            format!("{base}cursor:pointer;color:var(--accent-primary);opacity:1;")
        } else {
            format!("{base}cursor:pointer;")
        }
    };
    let tap = move |key: String| {
        move |evt: MouseEvent| {
            if let Some(cb) = on_bar {
                evt.stop_propagation();
                cb.call(key.clone());
            }
        }
    };
    let tap_chip = move |key: String| {
        move |evt: MouseEvent| {
            evt.stop_propagation();
            shown.set(Some(key.clone()));
            if let Some(cb) = on_bar {
                cb.call(key.clone());
            }
        }
    };
    let chip_label = |bar: &Bar| match label_case {
        LabelCase::Upper => bar.label.to_uppercase(),
        LabelCase::AsIs => bar.label.clone(),
    };
    let chip_class = |bar: &Bar| {
        if selected.as_deref() == Some(bar.key.as_str()) {
            "chip selected"
        } else {
            "chip"
        }
    };
    let reveal_at_bar = |bar: &Bar| reveal_row.filter(|at| at.key == bar.key);
    let reveal_class = |bar: &Bar| {
        if reveal_at_bar(bar).is_some_and(|at| at.open) {
            "keyword-reveal open"
        } else {
            "keyword-reveal"
        }
    };
    let reveal_content = |bar: &Bar| {
        reveal_at_bar(bar)?;
        reveal.map(|r| r.call(bar.key.clone()))
    };
    // Rows space by their own padding, not the column's gap, so a shut fold
    // adds no height.
    let plain_row = |bar: &Bar| {
        rsx! {
            div { style: "display:flex;align-items:center;gap:0.5rem;padding:0.175rem 0;",
                span { style: label_style(bar), onclick: tap(bar.key.clone()),
                    "{bar.label}"
                }
                div { class: "chart-track",
                    div { class: "chart-fill", style: "width:{bar.pct}%;" }
                }
                span { style: count_style, "{bar.count}{count_suffix}" }
            }
        }
    };
    // Three grid cells and the row's reveal, spanning the grid.
    let chip_row = |bar: &Bar| {
        rsx! {
            div { style: "padding:0.175rem 0;text-align:right;",
                if bar.revealable {
                    button { class: chip_class(bar), onclick: tap_chip(bar.key.clone()),
                        "{chip_label(bar)}"
                    }
                } else {
                    span { class: "stat-chip stat-chip-muted", "{chip_label(bar)}" }
                }
            }
            div { class: "chart-track",
                div { class: "chart-fill", style: "width:{bar.pct}%;" }
            }
            span { style: count_style, "{bar.count}{count_suffix}" }
            // Mounted shut under every row so opening one is a class
            // change the reveal can ease, not a fresh node.
            div { class: reveal_class(bar), style: "grid-column:1 / -1;",
                div { class: "keyword-reveal-inner",
                    if let Some(content) = reveal_content(bar) {
                        {content}
                    }
                }
            }
        }
    };
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:0.35rem;padding:0 0.75rem;",
            if let Some(text) = label {
                ChartLabel { text, hint }
            }
            match layout {
                Layout::Vertical => rsx! {
                    div { style: "display:flex;justify-content:center;align-items:flex-end;gap:0.25rem;height:6rem;",
                        for bar in bars.iter() {
                            div { style: "{column}display:flex;flex-direction:column;align-items:center;justify-content:flex-end;height:100%;gap:0.15rem;",
                                if bar.count > 0 {
                                    span { style: "font-size:0.6rem;color:var(--text-primary);opacity:0.85;line-height:1;", "{bar.count}{count_suffix}" }
                                }
                                div { style: format!("width:100%;height:{}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem 0.15rem 0 0;", bar.pct) }
                            }
                        }
                    }
                    div { style: "display:flex;justify-content:center;gap:0.25rem;",
                        for bar in bars.iter() {
                            span { style: label_style(bar), onclick: tap(bar.key.clone()), "{bar.label}" }
                        }
                    }
                },
                Layout::Rows => rsx! {
                    for bar in shown_bars.iter() {
                        {plain_row(bar)}
                    }
                    if let Some(rest) = folded_bars {
                        Fold { open: fold_open(),
                            for bar in rest.iter() {
                                {plain_row(bar)}
                            }
                        }
                        SeeMore { open: fold_open(), on_toggle: toggle_fold }
                    }
                },
                // A grid rather than flex rows so the chip column is as wide as
                // the widest chip on every row. Rows space by the label cell's
                // padding, not a row gap, so the shut reveals and a shut fold
                // add no height.
                Layout::ChipRows => rsx! {
                    div { style: format!("display:grid;grid-template-columns:fit-content({CHIP_LABEL_MAX_WIDTH}) minmax(0,1fr) auto;column-gap:0.5rem;align-items:center;"),
                        for bar in shown_bars.iter() {
                            {chip_row(bar)}
                        }
                        if let Some(rest) = folded_bars {
                            Fold { open: fold_open(), subgrid: true,
                                for bar in rest.iter() {
                                    {chip_row(bar)}
                                }
                            }
                        }
                    }
                    if folded_bars.is_some() {
                        SeeMore { open: fold_open(), on_toggle: toggle_fold }
                    }
                },
            }
        }
    }
}

/// A `DeckMetrics` tuple as a bar.
fn tuple_bar((label, count, pct): (&'static str, usize, u32)) -> Bar {
    Bar::new(label, u64::try_from(count).unwrap_or(u64::MAX), pct)
}

/// `DeckMetrics` tuples as bars.
fn tuple_bars(bars: &[(&'static str, usize, u32)]) -> Vec<Bar> {
    bars.iter().copied().map(tuple_bar).collect()
}

/// Role bars whose roles reveal their tags: a role with none in
/// `role_tags` is a grey chip.
fn role_bars(
    bars: &[(&'static str, usize, u32)],
    role_tags: &BTreeMap<&'static str, Vec<String>>,
) -> Vec<Bar> {
    bars.iter()
        .copied()
        .map(|(label, count, pct)| {
            let bar = tuple_bar((label, count, pct));
            if role_tags.get(label).is_some_and(|tags| !tags.is_empty()) {
                bar
            } else {
                bar.with_no_reveal()
            }
        })
        .collect()
}

/// Count-based distribution charts (types, roles, oracle tags), rendered as
/// flat sub-blocks: [`BarChart`]s, the role and oracle tag ones horizontal
/// and folded past their first [`FOLD_AT`] rows.
///
/// With `role_tags` set, the role chart's labels are chips and a tapped role
/// eases its tags open under its row as [`RoleTags`], one role at a time, the
/// way the Playstyle screen's Card roles chart does. The oracle tag chart's
/// labels are chips too, and a tapped tag opens its [`OtagDef`]. `describe_tag`
/// and `on_examples` go to both.
#[component]
pub fn DeckCharts(
    type_bars: Option<Vec<(&'static str, usize, u32)>>,
    category_bars: Option<Vec<(&'static str, usize, u32)>>,
    /// The deck's oracle tags by count (`DeckMetrics::oracle_tag_bars`).
    #[props(default)]
    oracle_tag_bars: Option<Vec<(String, usize, u32)>>,
    /// "?" beside each label; the site passes none.
    type_hint: Option<Element>,
    role_hint: Option<Element>,
    /// "?" beside the Oracle tags label.
    #[props(default)]
    oracle_tag_hint: Option<Element>,
    /// Each role's oracle tags, keyed by the role bar's label
    /// (`DeckMetrics::card_role_tags`).
    #[props(default)]
    role_tags: Option<BTreeMap<&'static str, Vec<String>>>,
    #[props(default)] describe_tag: Option<Callback<String, Option<String>>>,
    #[props(default)] on_examples: Option<Callback<String>>,
) -> Element {
    let mut selected_role: Signal<Option<String>> = use_signal(|| None);
    let reveal_tags = role_tags.clone().unwrap_or_default();
    let role_reveal = use_callback(move |role: String| {
        let tags = reveal_tags.get(role.as_str()).cloned().unwrap_or_default();
        rsx! {
            RoleTags { tags, describe_tag, on_examples }
        }
    });
    let toggle_role = use_callback(move |key: String| {
        if selected_role().as_deref() == Some(key.as_str()) {
            selected_role.set(None);
        } else {
            selected_role.set(Some(key));
        }
    });
    let mut selected_tag: Signal<Option<String>> = use_signal(|| None);
    let tag_reveal = use_callback(move |slug: String| {
        rsx! {
            OtagDef { slug, describe_tag, on_examples }
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
        if let Some(type_bars) = type_bars.as_ref() {
            BarChart {
                label: "Types",
                bars: tuple_bars(type_bars),
                hint: type_hint,
            }
        }

        if let Some(cat_bars) = category_bars.as_ref() {
            if !cat_bars.is_empty() {
                if let Some(role_tags) = role_tags.as_ref() {
                    BarChart {
                        label: "Card roles",
                        bars: role_bars(cat_bars, role_tags),
                        hint: role_hint,
                        orientation: BarOrientation::Horizontal,
                        on_bar: toggle_role,
                        selected: selected_role(),
                        reveal: role_reveal,
                        fold: true,
                    }
                } else {
                    BarChart {
                        label: "Card roles",
                        bars: tuple_bars(cat_bars),
                        hint: role_hint,
                        orientation: BarOrientation::Horizontal,
                        fold: true,
                    }
                }
            }
        }

        if let Some(tag_bars) = oracle_tag_bars.as_ref() {
            if !tag_bars.is_empty() {
                BarChart {
                    label: "Oracle tags",
                    bars: slug_bars(tag_bars),
                    hint: oracle_tag_hint,
                    orientation: BarOrientation::Horizontal,
                    label_case: LabelCase::AsIs,
                    on_bar: toggle_tag,
                    selected: selected_tag(),
                    reveal: tag_reveal,
                    fold: true,
                }
            }
        }
    }
}

/// `DeckMetrics` slug tuples as bars.
fn slug_bars(bars: &[(String, usize, u32)]) -> Vec<Bar> {
    bars.iter()
        .map(|(slug, count, pct)| {
            Bar::new(
                slug.clone(),
                u64::try_from(*count).unwrap_or(u64::MAX),
                *pct,
            )
        })
        .collect()
}

/// The deck's cards per color as [`ColorBars`] under a Colors label, shown
/// in the Mana section in the spacing of the section's other charts. `bars`
/// are [`ColorCounts::bars`] of the deck's metrics.
///
/// [`ColorCounts::bars`]: zwipe_ui::colors::ColorCounts::bars
#[component]
pub fn ColorChart(bars: Vec<ColorBar>, hint: Option<Element>) -> Element {
    rsx! {
        div { class: "color-chart",
            ChartLabel { text: "Colors", hint }
            ColorBars { bars }
        }
    }
}

/// Mana curve (nonland CMC histogram), rendered flat.
#[component]
pub fn ManaCurve(mana_curve_bars: [(usize, u32); 7], hint: Option<Element>) -> Element {
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:0.35rem;padding:0 0.75rem;",
            ChartLabel { text: "Mana curve", hint }
            div { style: "display:flex;align-items:flex-end;gap:0.25rem;height:6rem;",
                for (count, pct) in mana_curve_bars.iter() {
                    div { style: "flex:1;display:flex;flex-direction:column;align-items:center;justify-content:flex-end;height:100%;gap:0.15rem;",
                        if *count > 0 {
                            span { style: "font-size:0.6rem;color:var(--text-primary);opacity:0.85;line-height:1;", "{count}" }
                        }
                        div { style: format!("width:100%;height:{pct}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem 0.15rem 0 0;") }
                    }
                }
            }
            div { style: "display:flex;gap:0.25rem;",
                for label in ["0", "1", "2", "3", "4", "5", "6+"] {
                    span { style: "flex:1;text-align:center;font-size:0.65rem;color:var(--text-primary);opacity:0.85;", "{label}" }
                }
            }
        }
    }
}

/// Draw odds: `P(>=1)` per category as horizontal bars, with - and + to step
/// the draw window from turn 0, the opening hand, through later turns. On the
/// draw: `draws = 7 + turn`. `buckets` is `(label, count K)` in the order to
/// show; probabilities recompute live for the selected turn from the deck's
/// engine. The first [`FOLD_AT`] buckets show under the stepper and the rest
/// fold behind a [`SeeMore`] chip.
#[component]
pub fn DrawOdds(
    deck_size: u32,
    buckets: Vec<(&'static str, u32)>,
    hint: Option<Element>,
) -> Element {
    const MAX_TURN: u32 = 20;
    let mut turn = use_signal(|| 0u32);
    let mut fold_open = use_signal(|| false);
    let toggle_fold = use_callback(move |()| fold_open.toggle());
    // The stepper holds `turn` at MAX_TURN or below, so this never saturates.
    let draws = turn().saturating_add(7);
    // Turn 0 is the opening hand, named as a turn so the heading keeps one
    // width as the turn steps; the figure space pads single digits.
    let heading = format!("Odds of \u{2265}1 by turn {:\u{2007}>2}", turn());
    // Map each bucket to its odds in the loop header rather than a
    // `{ let …; rsx! }` block inside the `for` (which confuses Dioxus node
    // diffing and drops the bar-width style on alternating turns); the keyed
    // loop below remounts changed rows.
    let rows: Vec<(&'static str, u32)> = buckets
        .iter()
        .map(|(label, k)| {
            (
                *label,
                whole_percent(draw_odds::p_at_least_one(deck_size, *k, draws)),
            )
        })
        .collect();
    let (shown_rows, folded_rows) = fold_split(&rows, FOLD_AT);
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:0.35rem;padding:0 0.75rem;",
            div { style: "display:flex;align-items:center;justify-content:center;gap:0.6rem;",
                // Reuse the combat-filter stepper button: solid background gives
                // native press feedback and suppresses WKWebView's focus ring.
                // Each step is off at its bound, turn 0 or the last turn.
                button {
                    class: "stepper-btn",
                    disabled: turn() == 0,
                    onclick: move |_| { let t = turn(); if t > 0 { turn.set(t - 1); } },
                    "-"
                }
                span { class: "chart-label",
                    span { class: "stat-chip chart-label-chip", "{heading}" }
                    if let Some(h) = hint { {h} }
                }
                button {
                    class: "stepper-btn",
                    disabled: turn() >= MAX_TURN,
                    onclick: move |_| { let t = turn(); if t < MAX_TURN { turn.set(t + 1); } },
                    "+"
                }
            }
            for (label, pct) in shown_rows.iter().copied() {
                OddsRow { label, pct }
            }
            if let Some(rest) = folded_rows {
                Fold { open: fold_open(),
                    for (label, pct) in rest.iter().copied() {
                        OddsRow { label, pct }
                    }
                }
                SeeMore { open: fold_open(), on_toggle: toggle_fold }
            }
        }
    }
}

/// One draw odds bar. Its key includes `pct` so a changed row remounts
/// (fresh DOM) instead of an in-place style update; the web view drops the
/// latter and leaves the bar unfilled on alternating turns otherwise.
#[component]
fn OddsRow(label: &'static str, pct: u32) -> Element {
    rsx! {
        div {
            key: "{label}-{pct}",
            style: "display:flex;align-items:center;gap:0.5rem;padding:0.175rem 0;",
            span { style: "width:5ch;font-size:0.7rem;color:var(--text-primary);opacity:0.85;text-align:right;flex-shrink:0;text-transform:uppercase;",
                "{label}"
            }
            div { class: "chart-track",
                div { class: "chart-fill", style: "width:{pct}%;" }
            }
            span { style: "font-size:0.7rem;color:var(--text-primary);opacity:0.85;width:4ch;text-align:right;flex-shrink:0;",
                "{pct}%"
            }
        }
    }
}

/// `probability`, from 0 to 1, as a whole percent rounded half away from
/// zero. NaN and anything below 0 read 0; anything above 1 reads 100.
fn whole_percent(probability: f64) -> u32 {
    let rounded = (probability * 100.0).round();
    (0..=100)
        .rev()
        .find(|&percent| f64::from(percent) <= rounded)
        .unwrap_or(0)
}

/// The mana fill a fulfillment row's bar takes, from its W/U/B/R/G letter;
/// an unknown letter takes the text color.
fn balance_fill(label: &str) -> String {
    Color::try_from(label).map_or_else(
        |_| "var(--text-primary)".to_owned(),
        |color| format!("var({})", mana_var(color)),
    )
}

/// Per-color mana cost fulfillment: a [`ColorBars`]-style row per color,
/// the pip at the left and the track filled in the color's mana fill to the
/// produced share, then a check when production covers the cost and the
/// produced over consumed pips.
#[component]
pub fn ManaFulfillment(rows: Vec<ManaBalanceRow>, hint: Option<Element>) -> Element {
    if rows.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: "color-chart",
            ChartLabel { text: "Mana cost fulfillment", hint }
            for ManaBalanceRow { label: color_label, consumed, produced, fill_pct, is_surplus } in rows.iter() {
                div { class: "color-bar-row",
                    i { class: "ms ms-{color_label.to_lowercase()} ms-cost" }
                    div { class: "color-bar-track",
                        div {
                            class: "color-bar-fill",
                            style: "width: {fill_pct}%; background-color: {balance_fill(color_label)};",
                        }
                    }
                    span { style: "font-size:0.75rem;color:var(--text-primary);opacity:0.85;width:1.25rem;text-align:center;flex-shrink:0;",
                        if *is_surplus { "✔" } else { "" }
                    }
                    span { style: "font-size:0.75rem;color:var(--text-primary);opacity:0.85;white-space:nowrap;width:6ch;text-align:right;flex-shrink:0;",
                        "{produced}/{consumed}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_percent_rounds_probabilities() {
        let cases = [
            (0.0, 0),
            (1.0, 100),
            (0.5, 50),
            (0.125, 13),
            (0.004, 0),
            (0.996, 100),
            (0.334, 33),
            (f64::NAN, 0),
            (-0.2, 0),
        ];
        for (probability, want) in cases {
            assert_eq!(
                whole_percent(probability),
                want,
                "probability {probability}"
            );
        }
    }

    #[test]
    fn reveal_follows_the_selected_row() {
        assert_eq!(
            reveal_at(Some("ramp"), Some("removal")),
            Some(RevealAt {
                key: "ramp",
                open: true
            }),
            "a selection moves the reveal off the last shown row"
        );
    }

    #[test]
    fn reveal_stays_under_the_closed_row_while_shut() {
        assert_eq!(
            reveal_at(None, Some("ramp")),
            Some(RevealAt {
                key: "ramp",
                open: false
            })
        );
        assert_eq!(reveal_at(None, None), None, "nothing opened yet");
    }

    #[test]
    fn a_role_reveals_only_with_tags() {
        let role_tags = BTreeMap::from([
            ("ramp", vec!["mana-rock".to_string()]),
            ("draw", Vec::new()),
        ]);
        let bars = role_bars(
            &[("ramp", 5, 100), ("draw", 3, 60), ("remov", 2, 40)],
            &role_tags,
        );
        let got: Vec<(&str, u64, u32, bool)> = bars
            .iter()
            .map(|b| (b.key.as_str(), b.count, b.pct, b.revealable))
            .collect();
        assert_eq!(
            got,
            vec![
                ("ramp", 5, 100, true),
                ("draw", 3, 60, false),
                ("remov", 2, 40, false)
            ],
            "the bars keep their order and sizes"
        );
    }

    #[test]
    fn only_a_horizontal_tappable_chart_draws_chips() {
        assert_eq!(
            Layout::new(BarOrientation::Horizontal, true),
            Layout::ChipRows
        );
        assert_eq!(Layout::new(BarOrientation::Horizontal, false), Layout::Rows);
        assert_eq!(
            Layout::new(BarOrientation::Vertical, true),
            Layout::Vertical
        );
        assert_eq!(
            Layout::new(BarOrientation::Vertical, false),
            Layout::Vertical
        );
    }
}
