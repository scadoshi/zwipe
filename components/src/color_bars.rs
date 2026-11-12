//! Colors as horizontal bars: one row per color in WUBRG order, then
//! colorless, each with its mana pip at the left, a track filled in the
//! color's mana fill to the bar's share, and the count at the end.
//!
//! Home's Colors panel, Playstyle's Colors section and a deck's Mana
//! section all draw [`ColorBars`] from [`zwipe_ui::colors::Bar`]s, so every
//! color count in the app and on the site has one look. The look is the
//! `.color-bar*` rules in `magic.css`; each fill is one of `--mana-white`
//! through `--mana-green`, and the colorless bar a muted neutral.

use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_ui::colors::Bar;

/// The fill of the colorless bar, which has no mana variable.
const COLORLESS_FILL: &str = "var(--text-muted)";

/// The rows a [`ColorBarsGhost`] draws: five colors and colorless.
const GHOST_ROWS: u32 = 6;

/// The CSS custom property holding `color`'s mana fill, `--mana-white` through
/// `--mana-green`: what the color bars fill with.
#[must_use]
pub(crate) fn mana_var(color: Color) -> &'static str {
    match color {
        Color::White => "--mana-white",
        Color::Blue => "--mana-blue",
        Color::Black => "--mana-black",
        Color::Red => "--mana-red",
        Color::Green => "--mana-green",
    }
}

/// The bars in the order given: the mana pip at the left, the track filled
/// in the color's mana variable (a muted neutral for colorless) to the bar's
/// share, and the count right-aligned at the end. A zero count draws its pip
/// and count over an empty track.
#[component]
pub fn ColorBars(bars: Vec<Bar>) -> Element {
    let rows = bars.into_iter().map(|bar| {
        let (pip, fill) = match bar.color {
            Some(color) => (
                color.to_short_name().to_lowercase(),
                format!("var({})", mana_var(color)),
            ),
            None => ("c".to_owned(), COLORLESS_FILL.to_owned()),
        };
        (pip, fill, bar.count, bar.pct)
    });
    rsx! {
        div { class: "color-bars",
            for (index, (pip, fill, count, pct)) in rows.enumerate() {
                div { key: "{pip}", class: "color-bar-row",
                    i { class: "ms ms-{pip} ms-cost" }
                    div { class: "color-bar-track",
                        if count > 0 {
                            div {
                                class: "color-bar-fill chart-bar",
                                style: "width: {pct}%; background-color: {fill}; {crate::charts::bar_delay(index)}",
                            }
                        }
                    }
                    span { class: "color-bar-count", "{count}" }
                }
            }
        }
    }
}

/// [`ColorBars`] while its counts load: six rows, each a ghost pip, an
/// empty track and a ghost count, the loaded bars' size so the swap stays
/// in place.
#[component]
pub fn ColorBarsGhost() -> Element {
    rsx! {
        div { class: "color-bars",
            for i in 0..GHOST_ROWS {
                div { key: "{i}", class: "color-bar-row",
                    div { class: "skeleton-bar color-bar-pip-ghost" }
                    div { class: "skeleton-bar color-bar-track" }
                    div { class: "skeleton-bar color-bar-count-ghost" }
                }
            }
        }
    }
}
