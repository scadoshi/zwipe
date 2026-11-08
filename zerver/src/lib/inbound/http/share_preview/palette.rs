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
    /// `--bg-primary`: the image's background.
    pub bg: String,
    /// `--text-primary`: the deck's name.
    pub text: String,
    /// `--text-muted`: secondary lines and labels.
    pub muted: String,
    /// `--border-primary`: chip outlines and the frame.
    pub border: String,
    /// `--border-secondary`: rules and the curve's baseline.
    pub border_soft: String,
    /// `--accent-primary`: the mana curve and the brand.
    pub accent: String,
    /// `--accent-secondary`: the format line.
    pub accent_2: String,
    /// `--accent-tertiary`: deck tags.
    pub accent_3: String,
    /// `--color-money`: the price.
    pub money: String,
    /// `--color-stat`: the card count.
    pub stat: String,
    /// `--mana-white`, `--mana-blue`, `--mana-black`, `--mana-red`,
    /// `--mana-green`: the pips' fills, in WUBRG order.
    pub mana: [String; 5],
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
            && let Some(palette) = palette(body)
        {
            palettes.insert(class.to_owned(), palette);
        }
        rest = after.get(close..).unwrap_or_default();
    }
    palettes
}

/// The palette a theme block's `body` sets, or `None` when one of its colors
/// is missing.
fn palette(body: &str) -> Option<Palette> {
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
    Some(Palette {
        bg: color("bg-primary")?,
        text: color("text-primary")?,
        muted: color("text-muted")?,
        border: color("border-primary")?,
        border_soft: color("border-secondary")?,
        accent: color("accent-primary")?,
        accent_2: color("accent-secondary")?,
        accent_3: color("accent-tertiary")?,
        money: color("color-money")?,
        stat: color("color-stat")?,
        mana: [
            color("mana-white")?,
            color("mana-blue")?,
            color("mana-black")?,
            color("mana-red")?,
            color("mana-green")?,
        ],
    })
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
        assert_eq!(p.accent_3, "#d3869b");
        assert_eq!(p.mana[1], "#83a598");
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
