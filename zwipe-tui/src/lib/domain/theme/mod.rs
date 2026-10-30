//! Color themes: the user's ZWIPE theme from their account, drawn with orangui's palettes, or the terminal's own colors before sign-in.

#[rustfmt::skip]
mod palettes;

use ratatui::style::{Color, Style};
use zwipe_core::domain::user::preferences::RENAMED_THEMES;

use self::palettes::FAMILIES;

/// A 24-bit color: red, green, blue.
pub(crate) type Rgb = [u8; 3];

/// One mode of an orangui theme, by the role each color plays here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Colors {
    pub(crate) background: Rgb,
    pub(crate) text: Rgb,
    /// Hints and secondary text.
    pub(crate) muted: Rgb,
    pub(crate) border: Rgb,
    /// The selection and headings.
    pub(crate) accent: Rgb,
    /// Adding a card and focused boxes.
    pub(crate) success: Rgb,
    pub(crate) error: Rgb,
    pub(crate) warning: Rgb,
    pub(crate) mana_white: Rgb,
    pub(crate) mana_blue: Rgb,
    pub(crate) mana_black: Rgb,
    pub(crate) mana_red: Rgb,
    pub(crate) mana_green: Rgb,
}

/// An orangui theme: its slug and both modes.
#[derive(Debug)]
pub(crate) struct Family {
    pub(crate) slug: &'static str,
    pub(crate) dark: Colors,
    pub(crate) light: Colors,
}

/// The styles every part of the UI draws with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Palette {
    /// Text and background for every cell.
    pub(crate) base: Style,
    pub(crate) muted: Style,
    pub(crate) border: Style,
    pub(crate) accent: Style,
    /// Adding a card, success messages and focused boxes.
    pub(crate) add: Style,
    /// Passing on a card and errors.
    pub(crate) error: Style,
    pub(crate) warning: Style,
    /// The five mana colors, in WUBRG order.
    pub(crate) mana: [Style; 5],
}

impl Palette {
    /// The terminal's own colors, by their ANSI names, for before the account's theme is known.
    pub(crate) fn terminal() -> Self {
        Self {
            base: Style::new(),
            muted: Style::new().dim(),
            border: Style::new(),
            accent: Style::new().cyan(),
            add: Style::new().green(),
            error: Style::new().red(),
            warning: Style::new().yellow(),
            mana: [
                Style::new().white(),
                Style::new().blue(),
                Style::new().magenta(),
                Style::new().red(),
                Style::new().green(),
            ],
        }
    }

    /// The account's ZWIPE theme: an orangui slug, possibly a renamed one, and dark mode.
    ///
    /// An unknown slug falls back to the terminal's colors.
    pub(crate) fn for_account(theme: &str, dark_mode: bool) -> Self {
        let slug = RENAMED_THEMES
            .iter()
            .find(|(old, _)| *old == theme)
            .map_or(theme, |(_, new)| *new);
        let Some(family) = FAMILIES.iter().find(|f| f.slug == slug) else {
            return Self::terminal();
        };
        Self::from_colors(if dark_mode {
            &family.dark
        } else {
            &family.light
        })
    }

    fn from_colors(colors: &Colors) -> Self {
        let color = |[r, g, b]: Rgb| Color::Rgb(r, g, b);
        let fg = |rgb: Rgb| Style::new().fg(color(rgb));
        Self {
            base: fg(colors.text).bg(color(colors.background)),
            muted: fg(colors.muted),
            border: fg(colors.border),
            accent: fg(colors.accent),
            add: fg(colors.success),
            error: fg(colors.error),
            warning: fg(colors.warning),
            mana: [
                fg(colors.mana_white),
                fg(colors.mana_blue),
                fg(colors.mana_black),
                fg(colors.mana_red),
                fg(colors.mana_green),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::user::ALLOWED_THEMES;

    use super::*;

    #[test]
    fn palettes_cover_exactly_the_themes_zwipe_allows() {
        let mut generated: Vec<&str> = FAMILIES.iter().map(|f| f.slug).collect();
        let mut allowed = ALLOWED_THEMES.to_vec();
        generated.sort_unstable();
        allowed.sort_unstable();
        assert_eq!(
            generated, allowed,
            "regenerate with zwipe-tui/scripts/gen_themes.py"
        );
    }

    #[test]
    fn an_account_theme_picks_its_mode_and_follows_renames() {
        let dark = Palette::for_account("gruvbox", true);
        let light = Palette::for_account("gruvbox", false);
        assert_eq!(dark.base.bg, Some(Color::Rgb(0x28, 0x28, 0x28)));
        assert_ne!(dark.base.bg, light.base.bg);
        assert_eq!(
            Palette::for_account("ristretto", true),
            Palette::for_account("monokai-pro-ristretto", true)
        );
    }

    #[test]
    fn an_unknown_theme_uses_the_terminals_colors() {
        assert_eq!(Palette::for_account("nope", true), Palette::terminal());
        assert_eq!(Palette::terminal().base.bg, None);
    }
}
