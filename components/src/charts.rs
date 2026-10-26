//! Deck distribution charts, shared by the app's Distributions section and
//! the zwipe.net shared-deck page.
//!
//! Pure div-bar charts (no canvas, no JS) styled against the theme variables,
//! fed the chart-ready tuples `DeckMetrics` produces (`mana_curve_bars`,
//! `type_bars`, `color_bars`, `card_role_bars`). Shared so both surfaces render
//! the same picture; the interactive pieces (draw odds, mana fulfillment) are
//! used by the app only.

use dioxus::prelude::*;
use zwipe_core::domain::deck::deck_metrics::ManaBalanceRow;
use zwipe_core::domain::deck::draw_odds::p_at_least_one;

/// Sentence-case accent sub-heading shown above each flattened chart block.
///
/// `hint` is whatever the host wants right of the text, in practice the app's
/// "?" button. It is an `Element` rather than a topic because the hint system
/// lives in the app and this crate is shared with the site, which has none.
#[component]
pub fn ChartLabel(text: &'static str, hint: Option<Element>) -> Element {
    rsx! {
        span { style: "display:inline-flex;align-items:center;font-size:0.75rem;font-weight:600;color:var(--accent-tertiary);",
            "{text}"
            if let Some(h) = hint { {h} }
        }
    }
}

/// One bar of a [`BarChart`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bar {
    /// What `on_bar` reports when the label is tapped; the label unless the
    /// host names something else (a slug behind an abbreviated label).
    pub key: String,
    /// The text under or beside the bar.
    pub label: String,
    /// Shown above (or beside) the bar when above 0.
    pub count: usize,
    /// The bar's size as a percentage of the chart's tallest bar, 0 to 100.
    pub pct: u32,
}

impl Bar {
    /// A bar whose key is its label.
    #[must_use]
    pub fn new(label: impl Into<String>, count: usize, pct: u32) -> Self {
        let label = label.into();
        Self {
            key: label.clone(),
            label,
            count,
            pct,
        }
    }
}

impl From<(&'static str, usize, u32)> for Bar {
    fn from((label, count, pct): (&'static str, usize, u32)) -> Self {
        Self::new(label, count, pct)
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

/// One count-based bar chart: an optional [`ChartLabel`] sub-heading, then
/// the bars with their counts and labels.
///
/// With `on_bar` set, each label is tappable and reports its bar's key; the
/// label whose key is `selected` reads in the accent color.
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
    /// Horizontal only: the label column's CSS width.
    #[props(default = DEFAULT_LABEL_WIDTH)]
    label_width: &'static str,
    #[props(default)] on_bar: Option<Callback<String>>,
    #[props(default)] selected: Option<String>,
) -> Element {
    let upper = match label_case {
        LabelCase::Upper => "text-transform:uppercase;",
        LabelCase::AsIs => "",
    };
    // The label's base style is the deck charts' exact string; a tappable
    // label adds the pointer, and the selected one the accent, after it.
    let label_style = |bar: &Bar| {
        let base = match orientation {
            BarOrientation::Vertical => format!(
                "flex:1;text-align:center;font-size:0.65rem;color:var(--text-primary);opacity:0.85;{upper}"
            ),
            BarOrientation::Horizontal => format!(
                "width:{label_width};font-size:0.7rem;color:var(--text-primary);opacity:0.85;text-align:right;flex-shrink:0;{upper}"
            ),
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
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:0.35rem;padding:0 0.75rem;",
            if let Some(text) = label {
                ChartLabel { text, hint }
            }
            match orientation {
                BarOrientation::Vertical => rsx! {
                    div { style: "display:flex;align-items:flex-end;gap:0.25rem;height:6rem;",
                        for bar in bars.iter() {
                            div { style: "flex:1;display:flex;flex-direction:column;align-items:center;justify-content:flex-end;height:100%;gap:0.15rem;",
                                if bar.count > 0 {
                                    span { style: "font-size:0.6rem;color:var(--text-primary);opacity:0.85;line-height:1;", "{bar.count}" }
                                }
                                div { style: format!("width:100%;height:{}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem 0.15rem 0 0;", bar.pct) }
                            }
                        }
                    }
                    div { style: "display:flex;gap:0.25rem;",
                        for bar in bars.iter() {
                            span { style: label_style(bar), onclick: tap(bar.key.clone()), "{bar.label}" }
                        }
                    }
                },
                BarOrientation::Horizontal => rsx! {
                    for bar in bars.iter() {
                        div { style: "display:flex;align-items:center;gap:0.5rem;",
                            span { style: label_style(bar), onclick: tap(bar.key.clone()),
                                "{bar.label}"
                            }
                            div { style: "flex:1;height:0.9rem;background:var(--border-secondary);border-radius:0.15rem;overflow:hidden;",
                                div {
                                    style: format!(
                                        "height:100%;width:{}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem;",
                                        bar.pct
                                    ),
                                }
                            }
                            span { style: "font-size:0.7rem;color:var(--text-primary);opacity:0.85;width:3ch;text-align:right;flex-shrink:0;",
                                "{bar.count}"
                            }
                        }
                    }
                },
            }
        }
    }
}

/// `DeckMetrics` tuples as bars.
fn tuple_bars(bars: &[(&'static str, usize, u32)]) -> Vec<Bar> {
    bars.iter().copied().map(Bar::from).collect()
}

/// Count-based distribution charts (types, categories, colors), rendered as
/// flat sub-blocks: three [`BarChart`]s, the role one horizontal.
#[component]
pub fn DeckCharts(
    type_bars: Option<Vec<(&'static str, usize, u32)>>,
    category_bars: Option<Vec<(&'static str, usize, u32)>>,
    color_bars: Option<Vec<(&'static str, usize, u32)>>,
    /// "?" beside each label; the site passes none.
    type_hint: Option<Element>,
    role_hint: Option<Element>,
    color_hint: Option<Element>,
) -> Element {
    rsx! {
        if let Some(type_bars) = type_bars.as_ref() {
            BarChart {
                label: "Type distribution",
                bars: tuple_bars(type_bars),
                hint: type_hint,
            }
        }

        if let Some(cat_bars) = category_bars.as_ref() {
            if !cat_bars.is_empty() {
                BarChart {
                    label: "Role distribution",
                    bars: tuple_bars(cat_bars),
                    hint: role_hint,
                    orientation: BarOrientation::Horizontal,
                }
            }
        }

        if let Some(color_bars) = color_bars.as_ref() {
            BarChart {
                label: "Color distribution",
                bars: tuple_bars(color_bars),
                hint: color_hint,
            }
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

/// Draw odds: `P(>=1)` per category as horizontal bars, with ‹ › to step the
/// draw window from the opening hand through later turns. On the draw:
/// `draws = 7 + turn` (opening hand = turn 0). `buckets` is `(label, count K)`;
/// probabilities recompute live for the selected turn from the deck's engine.
#[component]
pub fn DrawOdds(
    deck_size: u32,
    buckets: Vec<(&'static str, u32)>,
    hint: Option<Element>,
) -> Element {
    const MAX_TURN: u32 = 20;
    let mut turn = use_signal(|| 0u32);
    let draws = 7 + turn();
    let heading = if turn() == 0 {
        "Odds of ≥1 in opening hand".to_string()
    } else {
        format!("Odds of ≥1 by turn {}", turn())
    };
    // Map each bucket to its odds in the loop header rather than a
    // `{ let …; rsx! }` block inside the `for` (which confuses Dioxus node
    // diffing and drops the bar-width style on alternating turns); the keyed
    // loop below remounts changed rows.
    let rows = buckets.iter().map(|(label, k)| {
        (
            *label,
            (p_at_least_one(deck_size, *k, draws) * 100.0).round() as u32,
        )
    });
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:0.35rem;padding:0 0.75rem;",
            div { style: "display:flex;align-items:center;justify-content:center;gap:0.6rem;",
                // Reuse the combat-filter stepper button: solid background gives
                // native press feedback and suppresses WKWebView's focus ring.
                // The onclick guards no-op at the bounds; no gray-out.
                button {
                    class: "stepper-btn",
                    onclick: move |_| { let t = turn(); if t > 0 { turn.set(t - 1); } },
                    "-"
                }
                span { style: "display:inline-flex;align-items:center;font-size:0.75rem;font-weight:600;color:var(--accent-tertiary);",
                    "{heading}"
                    if let Some(h) = hint { {h} }
                }
                button {
                    class: "stepper-btn",
                    onclick: move |_| { let t = turn(); if t < MAX_TURN { turn.set(t + 1); } },
                    "+"
                }
            }
            for (label, pct) in rows {
                // Key includes pct so a changed row remounts (fresh DOM) instead
                // of an in-place style update; the WebView drops the latter and
                // leaves the bar unfilled on alternating turns otherwise.
                div {
                    key: "{label}-{pct}",
                    style: "display:flex;align-items:center;gap:0.5rem;",
                    span { style: "width:5ch;font-size:0.7rem;color:var(--text-primary);opacity:0.85;text-align:right;flex-shrink:0;text-transform:uppercase;",
                        "{label}"
                    }
                    div { style: "flex:1;height:0.9rem;background:var(--border-secondary);border-radius:0.15rem;overflow:hidden;",
                        div { style: "height:100%;width:{pct}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem;" }
                    }
                    span { style: "font-size:0.7rem;color:var(--text-primary);opacity:0.85;width:4ch;text-align:right;flex-shrink:0;",
                        "{pct}%"
                    }
                }
            }
        }
    }
}

/// Per-color mana cost fulfillment, rendered flat.
#[component]
pub fn ManaFulfillment(rows: Vec<ManaBalanceRow>, hint: Option<Element>) -> Element {
    if rows.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:0.4rem;padding:0 0.75rem;",
            ChartLabel { text: "Mana cost fulfillment", hint }
            for ManaBalanceRow { label: color_label, consumed, produced, fill_pct, is_surplus } in rows.iter() {
                div { style: "display:flex;align-items:center;gap:0.5rem;",
                    span { style: "width:1ch;font-size:0.75rem;color:var(--text-primary);opacity:0.85;", "{color_label}" }
                    div { style: "flex:1;height:1rem;background:var(--border-secondary);border-radius:0.15rem;overflow:hidden;",
                        div {
                            style: format!(
                                "height:100%;width:{fill_pct}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem;"
                            ),
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
