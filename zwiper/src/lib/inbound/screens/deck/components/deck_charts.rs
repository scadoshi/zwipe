use dioxus::prelude::*;
use zwipe_core::domain::deck::draw_odds::p_at_least_one;

#[derive(Clone, PartialEq)]
pub(crate) struct ManaBalanceRow {
    pub label: &'static str,
    pub consumed: usize,
    pub produced: usize,
    pub fill_pct: u32,
    pub is_surplus: bool,
}

/// Sentence-case accent sub-heading shown above each flattened chart block.
#[component]
fn ChartLabel(text: &'static str) -> Element {
    rsx! {
        span { style: "font-size:0.75rem;font-weight:600;color:var(--accent-primary);",
            "{text}"
        }
    }
}

/// Count-based distribution charts (types, categories, colors), rendered as
/// flat sub-blocks for the "Distributions" collapsible section.
#[component]
pub(crate) fn DeckCharts(
    type_bars: Option<Vec<(&'static str, usize, u32)>>,
    category_bars: Option<Vec<(&'static str, usize, u32)>>,
    color_bars: Option<Vec<(&'static str, usize, u32)>>,
) -> Element {
    rsx! {
        // ── types ──────────────────────────────────────
        if let Some(type_bars) = type_bars.as_ref() {
            div { style: "display:flex;flex-direction:column;gap:0.35rem;padding:0 0.75rem;",
                ChartLabel { text: "Type distribution" }
                div { style: "display:flex;align-items:flex-end;gap:0.25rem;height:6rem;",
                    for (_label, count, pct) in type_bars.iter() {
                        div { style: "flex:1;display:flex;flex-direction:column;align-items:center;justify-content:flex-end;height:100%;gap:0.15rem;",
                            if *count > 0 {
                                span { style: "font-size:0.6rem;color:var(--text-primary);opacity:0.85;line-height:1;", "{count}" }
                            }
                            div { style: format!("width:100%;height:{pct}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem 0.15rem 0 0;") }
                        }
                    }
                }
                div { style: "display:flex;gap:0.25rem;",
                    for (label, _count, _pct) in type_bars.iter() {
                        span { style: "flex:1;text-align:center;font-size:0.65rem;color:var(--text-primary);opacity:0.85;text-transform:uppercase;", "{label}" }
                    }
                }
            }
        }

        // ── categories (horizontal bars) ─────────────
        if let Some(cat_bars) = category_bars.as_ref() {
            if !cat_bars.is_empty() {
                div { style: "display:flex;flex-direction:column;gap:0.35rem;padding:0 0.75rem;",
                    ChartLabel { text: "Category distribution" }
                    for (label, count, pct) in cat_bars.iter() {
                        div { style: "display:flex;align-items:center;gap:0.5rem;",
                            span { style: "width:5ch;font-size:0.7rem;color:var(--text-primary);opacity:0.85;text-align:right;flex-shrink:0;text-transform:uppercase;",
                                "{label}"
                            }
                            div { style: "flex:1;height:0.9rem;background:var(--border-secondary);border-radius:0.15rem;overflow:hidden;",
                                div {
                                    style: format!(
                                        "height:100%;width:{pct}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem;"
                                    ),
                                }
                            }
                            span { style: "font-size:0.7rem;color:var(--text-primary);opacity:0.85;width:3ch;text-align:right;flex-shrink:0;",
                                "{count}"
                            }
                        }
                    }
                }
            }
        }

        // ── colors ─────────────────────────────────────
        if let Some(color_bars) = color_bars.as_ref() {
            div { style: "display:flex;flex-direction:column;gap:0.35rem;padding:0 0.75rem;",
                ChartLabel { text: "Color distribution" }
                div { style: "display:flex;align-items:flex-end;gap:0.25rem;height:6rem;",
                    for (_label, count, pct) in color_bars.iter() {
                        div { style: "flex:1;display:flex;flex-direction:column;align-items:center;justify-content:flex-end;height:100%;gap:0.15rem;",
                            if *count > 0 {
                                span { style: "font-size:0.6rem;color:var(--text-primary);opacity:0.85;line-height:1;", "{count}" }
                            }
                            div { style: format!("width:100%;height:{pct}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem 0.15rem 0 0;") }
                        }
                    }
                }
                div { style: "display:flex;gap:0.25rem;",
                    for (label, _count, _pct) in color_bars.iter() {
                        span { style: "flex:1;text-align:center;font-size:0.65rem;color:var(--text-primary);opacity:0.85;text-transform:uppercase;", "{label}" }
                    }
                }
            }
        }

    }
}

/// Draw odds — `P(>=1)` per category as horizontal bars, with ‹ › to step the
/// draw window from the opening hand through later turns. On the draw:
/// `draws = 7 + turn` (opening hand = turn 0). `buckets` is `(label, count K)`;
/// probabilities recompute live for the selected turn from the deck's engine.
#[component]
pub(crate) fn DrawOdds(deck_size: u32, buckets: Vec<(&'static str, u32)>) -> Element {
    const MAX_TURN: u32 = 20;
    let mut turn = use_signal(|| 0u32);
    let draws = 7 + turn();
    let heading = if turn() == 0 {
        "Odds of ≥1 in opening hand".to_string()
    } else {
        format!("Odds of ≥1 by turn {}", turn())
    };
    let nav_btn = "background:none;border:1px solid var(--border-primary);border-radius:0.3rem;color:var(--accent-primary);font-size:0.85rem;line-height:1;padding:0.1rem 0.45rem;cursor:pointer;";
    // Precompute per-turn odds outside the render tree. A `{ let …; rsx! }` block
    // inside the `for` confuses Dioxus node diffing and drops the bar-width style
    // on alternating turns; a flat, keyed loop over precomputed rows fixes it.
    let rows: Vec<(&'static str, u32)> = buckets
        .iter()
        .map(|(label, k)| (*label, (p_at_least_one(deck_size, *k, draws) * 100.0).round() as u32))
        .collect();
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:0.35rem;padding:0 0.75rem;",
            div { style: "display:flex;align-items:center;justify-content:center;gap:0.6rem;",
                button {
                    r#type: "button",
                    style: format!("{nav_btn}opacity:{};", if turn() == 0 { "0.35" } else { "1" }),
                    disabled: turn() == 0,
                    onclick: move |_| { let t = turn(); if t > 0 { turn.set(t - 1); } },
                    "<-"
                }
                span { style: "font-size:0.75rem;font-weight:600;color:var(--accent-primary);", "{heading}" }
                button {
                    r#type: "button",
                    style: format!("{nav_btn}opacity:{};", if turn() >= MAX_TURN { "0.35" } else { "1" }),
                    disabled: turn() >= MAX_TURN,
                    onclick: move |_| { let t = turn(); if t < MAX_TURN { turn.set(t + 1); } },
                    "->"
                }
            }
            for (label, pct) in rows {
                // Key includes pct so a changed row remounts (fresh DOM) instead
                // of an in-place style update — the WebView drops the latter and
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

/// Mana curve (nonland CMC histogram), rendered flat for the "Mana" section.
#[component]
pub(crate) fn ManaCurve(mana_curve_bars: [(usize, u32); 7]) -> Element {
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:0.35rem;padding:0 0.75rem;",
            ChartLabel { text: "Mana curve" }
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
                for label in ["0","1","2","3","4","5","6+"] {
                    span { style: "flex:1;text-align:center;font-size:0.65rem;color:var(--text-primary);opacity:0.85;", "{label}" }
                }
            }
        }
    }
}

/// Per-color mana cost fulfillment, rendered flat for the "Mana" section.
#[component]
pub(crate) fn ManaFulfillment(rows: Vec<ManaBalanceRow>) -> Element {
    if rows.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:0.4rem;padding:0 0.75rem;",
            ChartLabel { text: "Mana cost fulfillment" }
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
