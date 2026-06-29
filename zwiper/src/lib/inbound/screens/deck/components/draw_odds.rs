use dioxus::prelude::*;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::draw_odds;

/// Highest turn the stepper allows.
const MAX_TURN: u32 = 12;

/// One rendered odds bar: a bucket label, its display percentage, and the bar
/// fill width (floored to a visible sliver).
#[derive(Clone, PartialEq)]
struct OddsRow {
    label: String,
    display: String,
    width: u32,
}

/// Builds the odds rows for one bucketing from `(label, K)` pairs, dropping any
/// bucket whose probability rounds to nothing so the list stays low-noise.
///
/// `n_deck` is the active mainboard size, `n_drawn` the cards seen, `threshold`
/// the "at least t" count.
fn odds_rows(
    buckets: &[(String, usize)],
    n_deck: u32,
    n_drawn: u32,
    threshold: u32,
) -> Vec<OddsRow> {
    buckets
        .iter()
        .filter_map(|(label, k)| {
            if *k == 0 {
                return None;
            }
            let p = draw_odds::p_at_least(n_deck, *k as u32, n_drawn, threshold);
            if p <= 0.0 {
                return None;
            }
            let pct = (p * 100.0).round() as u32;
            let display = if pct == 0 {
                "<1%".to_string()
            } else {
                format!("{pct}%")
            };
            Some(OddsRow {
                label: label.clone(),
                display,
                width: pct.max(2),
            })
        })
        .collect()
}

#[component]
pub(crate) fn DrawOdds(metrics: DeckMetrics) -> Element {
    // 0 = opening hand; T >= 1 = "by turn T". On the draw you draw for turn,
    // on the play you skip your first draw step.
    let mut turn = use_signal(|| 0u32);
    // Commander pods are usually "on the draw"-ish; default there, let the user flip.
    let mut on_play = use_signal(|| false);
    let mut threshold = use_signal(|| 1u32);

    let n_deck = metrics.total_cards as u32;
    if n_deck == 0 {
        return rsx! {};
    }

    let extra = match (turn(), on_play()) {
        (0, _) => 0,
        (t, true) => t - 1,
        (t, false) => t,
    };
    let n_drawn = (7 + extra).min(n_deck);

    // By category: lands first (not a mechanical category), then the deck's
    // mechanical categories already sorted by count.
    let mut category_buckets: Vec<(String, usize)> = Vec::new();
    if metrics.land_count > 0 {
        category_buckets.push(("lands".to_string(), metrics.land_count));
    }
    category_buckets.extend(
        metrics
            .mechanical_category_counts
            .iter()
            .map(|(label, count)| ((*label).to_string(), *count)),
    );

    let type_buckets: Vec<(String, usize)> = metrics
        .type_counts
        .iter()
        .map(|(label, count)| (DeckMetrics::abbreviate_type(label).to_string(), *count))
        .collect();

    // Mana value buckets (lands excluded — these are the nonland CMC histogram).
    const MV_LABELS: [&str; 7] = ["0", "1", "2", "3", "4", "5", "6+"];
    let mv_buckets: Vec<(String, usize)> = MV_LABELS
        .iter()
        .zip(metrics.cmc_histogram.iter())
        .map(|(label, count)| ((*label).to_string(), *count))
        .collect();

    let t = threshold();
    let category_rows = odds_rows(&category_buckets, n_deck, n_drawn, t);
    let type_rows = odds_rows(&type_buckets, n_deck, n_drawn, t);
    let mv_rows = odds_rows(&mv_buckets, n_deck, n_drawn, t);

    let window_label = if turn() == 0 {
        "Opening hand".to_string()
    } else {
        format!("By turn {}", turn())
    };

    rsx! {
        div { style: "display:flex;flex-direction:column;gap:0.6rem;padding:0 0.75rem;",
            span { style: "font-size:0.65rem;opacity:0.45;line-height:1.3;",
                "Chance of drawing each group by the chosen turn. Pure shuffle math, so it ignores mulligans, scry, tutors, and extra draws."
            }
            // ── controls ───────────────────────────────────
            div { style: "display:flex;flex-direction:column;gap:0.5rem;",
                // turn stepper
                div { class: "stepper", style: "justify-content:space-between;",
                    button {
                        class: "stepper-btn",
                        onclick: move |_| {
                            let v = turn();
                            if v > 0 { turn.set(v - 1); }
                        },
                        "-"
                    }
                    div { style: "display:flex;flex-direction:column;align-items:center;flex:1;",
                        span { style: "font-size:0.8rem;opacity:0.8;", "{window_label}" }
                        span { style: "font-size:0.65rem;opacity:0.45;", "drawing {n_drawn}" }
                    }
                    button {
                        class: "stepper-btn",
                        onclick: move |_| {
                            let v = turn();
                            if v < MAX_TURN { turn.set(v + 1); }
                        },
                        "+"
                    }
                }
                div { style: "display:flex;gap:0.5rem;align-items:center;flex-wrap:wrap;",
                    div { class: "chip-row", style: "margin-bottom:0;",
                        div {
                            class: if on_play() { "chip selected" } else { "chip" },
                            onclick: move |_| on_play.set(true),
                            "On the play"
                        }
                        div {
                            class: if on_play() { "chip" } else { "chip selected" },
                            onclick: move |_| on_play.set(false),
                            "On the draw"
                        }
                    }
                    span { class: "text-muted", "|" }
                    div { class: "chip-row", style: "margin-bottom:0;",
                        for n in [1u32, 2] {
                            div {
                                class: if threshold() == n { "chip selected" } else { "chip" },
                                onclick: move |_| threshold.set(n),
                                "≥{n}"
                            }
                        }
                    }
                }
            }

            OddsGroup { heading: "By category", rows: category_rows }
            OddsGroup { heading: "By card type", rows: type_rows }
            OddsGroup { heading: "By mana value", rows: mv_rows }
        }
    }
}

#[component]
fn OddsGroup(heading: &'static str, rows: Vec<OddsRow>) -> Element {
    if rows.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:0.35rem;",
            span { style: "font-size:0.75rem;font-weight:600;color:var(--accent-primary);",
                "{heading}"
            }
            for OddsRow { label, display, width } in rows.iter() {
                div { key: "{label}", style: "display:flex;align-items:center;gap:0.5rem;",
                    span { style: "width:5ch;font-size:0.7rem;color:var(--text-primary);opacity:0.85;text-align:right;flex-shrink:0;",
                        "{label}"
                    }
                    div { style: "flex:1;height:0.9rem;background:var(--border-secondary);border-radius:0.15rem;overflow:hidden;",
                        div {
                            style: "height:100%;width:{width}%;background:var(--accent-primary);border-radius:0.15rem;transition:width 0.25s ease;",
                        }
                    }
                    span { style: "font-size:0.7rem;color:var(--text-primary);opacity:0.85;width:4ch;text-align:right;flex-shrink:0;",
                        "{display}"
                    }
                }
            }
        }
    }
}
