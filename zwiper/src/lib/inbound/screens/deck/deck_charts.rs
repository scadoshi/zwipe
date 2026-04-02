use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub(super) struct ManaBalanceRow {
    pub label: &'static str,
    pub consumed: usize,
    pub produced: usize,
    pub fill_pct: u32,
    pub is_surplus: bool,
}

#[component]
pub(super) fn DeckCharts(
    mana_curve_bars: [(usize, u32); 7],
    type_bars: Option<Vec<(&'static str, usize, u32)>>,
    color_bars: Option<Vec<(&'static str, usize, u32)>>,
    mana_balance_rows: Option<Vec<ManaBalanceRow>>,
) -> Element {
    rsx! {
        // ── mana curve ─────────────────────────────────
        label { class: "label", "mana curve" }
        div { style: "width:100%;border:1px solid var(--border-secondary);border-radius:0.5rem;padding:0.75rem;margin-bottom:0.5rem;",
            div { style: "display:flex;align-items:flex-end;gap:0.25rem;height:6rem;",
                for (count, pct) in mana_curve_bars.iter() {
                    div { style: "flex:1;display:flex;flex-direction:column;align-items:center;justify-content:flex-end;height:100%;gap:0.15rem;",
                        if *count > 0 {
                            span { style: "font-size:0.6rem;font-family:monospace;opacity:0.5;line-height:1;", "{count}" }
                        }
                        div { style: format!("width:100%;height:{pct}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem 0.15rem 0 0;") }
                    }
                }
            }
            div { style: "display:flex;gap:0.25rem;margin-top:0.35rem;",
                for label in ["0","1","2","3","4","5","6+"] {
                    span { style: "flex:1;text-align:center;font-size:0.65rem;font-family:monospace;opacity:0.5;", "{label}" }
                }
            }
        }

        // ── types ──────────────────────────────────────
        if let Some(type_bars) = type_bars.as_ref() {
            label { class: "label", "basic type distribution" }
            div { style: "width:100%;border:1px solid var(--border-secondary);border-radius:0.5rem;padding:0.75rem;margin-bottom:0.5rem;",
                div { style: "display:flex;align-items:flex-end;gap:0.25rem;height:6rem;",
                    for (_label, count, pct) in type_bars.iter() {
                        div { style: "flex:1;display:flex;flex-direction:column;align-items:center;justify-content:flex-end;height:100%;gap:0.15rem;",
                            if *count > 0 {
                                span { style: "font-size:0.6rem;font-family:monospace;opacity:0.5;line-height:1;", "{count}" }
                            }
                            div { style: format!("width:100%;height:{pct}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem 0.15rem 0 0;") }
                        }
                    }
                }
                div { style: "display:flex;gap:0.25rem;margin-top:0.35rem;",
                    for (label, _count, _pct) in type_bars.iter() {
                        span { style: "flex:1;text-align:center;font-size:0.65rem;font-family:monospace;opacity:0.5;", "{label}" }
                    }
                }
            }
        }

        // ── colors ─────────────────────────────────────
        if let Some(color_bars) = color_bars.as_ref() {
            label { class: "label", "color distribution" }
            div { style: "width:100%;border:1px solid var(--border-secondary);border-radius:0.5rem;padding:0.75rem;margin-bottom:0.5rem;",
                div { style: "display:flex;align-items:flex-end;gap:0.25rem;height:6rem;",
                    for (_label, count, pct) in color_bars.iter() {
                        div { style: "flex:1;display:flex;flex-direction:column;align-items:center;justify-content:flex-end;height:100%;gap:0.15rem;",
                            if *count > 0 {
                                span { style: "font-size:0.6rem;font-family:monospace;opacity:0.5;line-height:1;", "{count}" }
                            }
                            div { style: format!("width:100%;height:{pct}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem 0.15rem 0 0;") }
                        }
                    }
                }
                div { style: "display:flex;gap:0.25rem;margin-top:0.35rem;",
                    for (label, _count, _pct) in color_bars.iter() {
                        span { style: "flex:1;text-align:center;font-size:0.65rem;font-family:monospace;opacity:0.5;", "{label}" }
                    }
                }
            }
        }

        // ── mana balance ───────────────────────────────
        if let Some(rows) = mana_balance_rows.as_ref() {
            if !rows.is_empty() {
                label { class: "label", "mana cost fulfillment" }
                div { style: "width:100%;border:1px solid var(--border-secondary);border-radius:0.5rem;padding:0.75rem;margin-bottom:0.5rem;display:flex;flex-direction:column;gap:0.4rem;",
                    for ManaBalanceRow { label: color_label, consumed, produced, fill_pct, is_surplus } in rows {
                        div { style: "display:flex;align-items:center;gap:0.5rem;",
                            span { style: "width:1ch;font-family:monospace;font-size:0.75rem;opacity:0.8;",
                                "{color_label}"
                            }
                            div { style: "flex:1;height:1rem;background:var(--border-secondary);border-radius:0.15rem;overflow:hidden;",
                                div {
                                    style: format!(
                                        "height:100%;width:{fill_pct}%;background:var(--text-primary);opacity:0.65;border-radius:0.15rem;"
                                    ),
                                }
                            }
                            span { style: "font-family:monospace;font-size:0.75rem;opacity:0.5;width:1.25rem;text-align:center;flex-shrink:0;",
                                if *is_surplus { "✔" } else { "" }
                            }
                            span { style: "font-family:monospace;font-size:0.75rem;opacity:0.5;white-space:nowrap;width:6ch;text-align:right;flex-shrink:0;",
                                "{produced}/{consumed}"
                            }
                        }
                    }
                }
            }
        }
    }
}

pub(super) fn abbreviate_type(label: &str) -> &str {
    match label {
        "lands" => "lands",
        "creatures" => "creat",
        "planeswalkers" => "plnsw",
        "artifacts" => "artif",
        "enchantments" => "enchn",
        "instants" => "instn",
        "sorceries" => "sorcr",
        "other" => "other",
        _ => label,
    }
}

pub(super) fn abbreviate_color(label: &str) -> &str {
    match label {
        "white" => "white",
        "blue" => "blue",
        "black" => "black",
        "red" => "red",
        "green" => "green",
        "multicolor" => "multi",
        "colorless" => "clrls",
        _ => label,
    }
}
