//! The colors a preview image is drawn in, read from the app's own theme
//! palettes.
//!
//! `assets/share_preview/themes.css` is a copy of orangui's theme sheet (a
//! test in `components` keeps it identical), so a deck's preview takes the
//! exact colors its owner sees in the app.

use std::collections::HashMap;
use std::sync::LazyLock;

use zwipe_core::domain::user::theme::ThemeConfig;

/// orangui's theme sheet.
const THEMES_CSS: &str = include_str!("../../../../../assets/share_preview/themes.css");

/// Every theme block in [`THEMES_CSS`], by its class (`theme-gruvbox-dark`).
static PALETTES: LazyLock<HashMap<String, Palette>> = LazyLock::new(|| parse(THEMES_CSS));

/// The theme drawn when the owner's is unknown: the app's default.
const FALLBACK_CLASS: &str = "theme-gruvbox-dark";

/// One theme's colors, as `#rrggbb`, named after the CSS variables they come
/// from.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Palette {
    /// Whether the theme is a light one.
    pub light: bool,
    /// `--bg-primary`: the panel.
    pub bg: String,
    /// `--bg-sink`: the recessed background under the panel, `--bg-primary`
    /// mixed 15% toward black, or 35% toward white in a light theme.
    pub sink: String,
    /// `--text-primary`: the deck's name.
    pub text: String,
    /// `--text-muted`: secondary lines and labels.
    pub muted: String,
    /// `--border-secondary`: the panel's edge, rules and the curve's baseline.
    pub border_soft: String,
    /// `--accent-primary`: the mana curve and the card count's tag.
    pub accent: String,
    /// `--palette-3`: the format's and the command zone's tags.
    pub palette_3: String,
    /// `--palette-5`: the power level's tag.
    pub palette_5: String,
    /// `--palette-6`: the brand and the art credit's chip.
    pub palette_6: String,
    /// `--color-money`: the price's tag.
    pub money: String,
    /// The tags' text colors, each the theme's `--ink-<tone>`, or the tone
    /// itself when the theme sets no ink for it.
    pub ink: Inks,
    /// `--mana-white`, `--mana-blue`, `--mana-black`, `--mana-red`,
    /// `--mana-green`: the pips' fills, in WUBRG order.
    pub mana: [String; 5],
}

/// A theme's chip inks: each tone as text that reads on its own tint.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Inks {
    /// `--ink-accent-primary`.
    pub accent: String,
    /// `--ink-palette-3`.
    pub palette_3: String,
    /// `--ink-palette-5`.
    pub palette_5: String,
    /// `--ink-palette-6`.
    pub palette_6: String,
    /// `--ink-color-money`.
    pub money: String,
    /// `--ink-text-muted`.
    pub muted: String,
}

impl Palette {
    /// The palette for `theme`, or the app's default when the sheet has no
    /// block for it.
    #[must_use]
    pub fn of(theme: &ThemeConfig) -> Self {
        PALETTES
            .get(&theme.css_class())
            .or_else(|| PALETTES.get(FALLBACK_CLASS))
            .cloned()
            .unwrap_or_default()
    }
}

/// Every `.theme-* { ... }` block in `css` that sets all the colors a
/// palette needs.
fn parse(css: &str) -> HashMap<String, Palette> {
    let mut palettes = HashMap::new();
    let mut rest = css;
    while let Some(start) = rest.find("\n.theme-") {
        let after = rest.get(start.saturating_add(2)..).unwrap_or_default();
        let Some(open) = after.find('{') else { break };
        let Some(close) = after.find('}') else { break };
        let class = after.get(..open).unwrap_or_default().trim();
        let body = after.get(open.saturating_add(1)..close).unwrap_or_default();
        if !class.contains([' ', ',', ':'])
            && let Some(palette) = palette(body, class.ends_with("-light"))
        {
            palettes.insert(class.to_owned(), palette);
        }
        rest = after.get(close..).unwrap_or_default();
    }
    palettes
}

/// The palette a theme block's `body` sets, or `None` when one of its colors
/// is missing. `light` lifts the sink toward white instead of black, as
/// orangui does for its light themes.
fn palette(body: &str, light: bool) -> Option<Palette> {
    let body = without_comments(body);
    let vars: HashMap<&str, &str> = body
        .split(';')
        .filter_map(|decl| {
            let (name, value) = decl.split_once(':')?;
            Some((name.trim().strip_prefix("--")?, value.trim()))
        })
        .collect();
    let color = |name: &str| -> Option<String> {
        let value = vars.get(name)?;
        let is_hex = value.len() == 7
            && value.starts_with('#')
            && value.chars().skip(1).all(|c| c.is_ascii_hexdigit());
        is_hex.then(|| value.to_ascii_lowercase())
    };
    let bg = color("bg-primary")?;
    let sink = if light {
        mix(&bg, "#ffffff", 0.35)?
    } else {
        mix(&bg, "#000000", 0.15)?
    };
    let accent = color("accent-primary")?;
    let palette_3 = color("palette-3")?;
    let palette_5 = color("palette-5")?;
    let palette_6 = color("palette-6")?;
    let money = color("color-money")?;
    let muted = color("text-muted")?;
    let ink = |name: &str, tone: &str| color(name).unwrap_or_else(|| tone.to_owned());
    Some(Palette {
        light,
        ink: Inks {
            accent: ink("ink-accent-primary", &accent),
            palette_3: ink("ink-palette-3", &palette_3),
            palette_5: ink("ink-palette-5", &palette_5),
            palette_6: ink("ink-palette-6", &palette_6),
            money: ink("ink-color-money", &money),
            muted: ink("ink-text-muted", &muted),
        },
        bg,
        sink,
        text: color("text-primary")?,
        muted,
        border_soft: color("border-secondary")?,
        accent,
        palette_3,
        palette_5,
        palette_6,
        money,
        mana: [
            color("mana-white")?,
            color("mana-blue")?,
            color("mana-black")?,
            color("mana-red")?,
            color("mana-green")?,
        ],
    })
}

/// `from` mixed `share` (0 to 1) of the way to `to`, in gamma-encoded sRGB
/// as CSS `color-mix(in srgb, ...)` mixes.
#[must_use]
pub fn mix(from: &str, to: &str, share: f64) -> Option<String> {
    let [a, b] = [hex_rgb(from)?, hex_rgb(to)?];
    let channel = |i: usize| -> Option<f64> {
        let (x, y) = (a.get(i)?, b.get(i)?);
        Some(x + (y - x) * share)
    };
    Some(rgb_hex([channel(0)?, channel(1)?, channel(2)?]))
}

/// `#rrggbb` as three 0 to 1 channels.
pub(super) fn hex_rgb(hex: &str) -> Option<[f64; 3]> {
    let digits = hex.strip_prefix('#')?;
    let channel = |at: usize| -> Option<f64> {
        let pair = digits.get(at..at.saturating_add(2))?;
        u8::from_str_radix(pair, 16)
            .ok()
            .map(|v| f64::from(v) / 255.0)
    };
    Some([channel(0)?, channel(2)?, channel(4)?])
}

/// Three 0 to 1 channels as `#rrggbb`, clamped.
pub(super) fn rgb_hex(rgb: [f64; 3]) -> String {
    let byte = |v: f64| -> u8 {
        let scaled = (v.clamp(0.0, 1.0) * 255.0).round();
        u8::try_from(rounded(scaled)).unwrap_or(u8::MAX)
    };
    let [red, green, blue] = rgb;
    format!("#{:02x}{:02x}{:02x}", byte(red), byte(green), byte(blue))
}

/// `v`, already rounded, as an integer; saturates at the ends.
#[expect(
    clippy::cast_possible_truncation,
    reason = "callers round first and the values are small"
)]
#[expect(
    clippy::as_conversions,
    reason = "f64 to i64 has no TryFrom; `as` saturates"
)]
fn rounded(v: f64) -> i64 {
    v as i64
}

/// `css` with its `/* ... */` comments removed, so a comment's own colon
/// can't be read as a declaration's.
fn without_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(rest.get(..start).unwrap_or_default());
        let after = rest.get(start.saturating_add(2)..).unwrap_or_default();
        rest = after
            .find("*/")
            .and_then(|end| after.get(end.saturating_add(2)..))
            .unwrap_or_default();
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme(name: &str, is_dark: bool) -> ThemeConfig {
        ThemeConfig {
            name: name.to_owned(),
            is_dark,
        }
    }

    #[test]
    fn every_theme_the_app_offers_has_a_palette() {
        for name in zwipe_core::domain::user::settings::ALLOWED_THEMES {
            for dark in [true, false] {
                let class = theme(name, dark).css_class();
                assert!(PALETTES.contains_key(&class), "{class} has no palette");
            }
        }
    }

    #[test]
    fn gruvbox_dark_reads_its_own_colors() {
        let p = Palette::of(&theme("gruvbox", true));
        assert_eq!(p.bg, "#282828");
        assert_eq!(p.text, "#ebdbb2");
        assert_eq!(p.palette_3, "#b8bb26");
        assert_eq!(p.ink.accent, "#86a79a");
        assert_eq!(p.mana[1], "#83a598");
    }

    #[test]
    fn the_sink_matches_orangui_mix() {
        // color-mix(in srgb, #282828, #000 15%) is #222222.
        assert_eq!(Palette::of(&theme("gruvbox", true)).sink, "#222222");
        assert_eq!(mix("#000000", "#ffffff", 0.5).as_deref(), Some("#808080"));
        let light = Palette::of(&theme("gruvbox", false));
        assert!(light.light);
        assert_eq!(light.sink, mix(&light.bg, "#ffffff", 0.35).unwrap());
        assert_ne!(light.sink, light.bg, "a cream panel's sink is lifted");
    }

    #[test]
    fn a_comment_before_a_color_does_not_hide_it() {
        let body = "/* mana: notes */ --mana-white: #ffffff; /* a */";
        assert_eq!(without_comments(body).trim(), "--mana-white: #ffffff;");
    }

    #[test]
    fn an_unknown_theme_draws_in_the_default() {
        assert_eq!(
            Palette::of(&theme("no-such-theme", true)),
            Palette::of(&theme("gruvbox", true))
        );
    }
}
