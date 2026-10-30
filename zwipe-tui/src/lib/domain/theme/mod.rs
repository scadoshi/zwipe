//! Color themes: the user's ZWIPE theme from their account, drawn with orangui's palettes, or the terminal's own colors before sign-in.

#[rustfmt::skip]
mod palettes;

use ratatui::style::{Color, Style};
use zwipe_core::domain::user::preferences::{PipStyle, RENAMED_THEMES, UserPreferences};

use self::palettes::{FAMILIES, GROUPS};

/// Mana's classic pip fills in WUBRG order, the same on every theme, from the app's `.pips-natural`.
const NATURAL_PIPS: [Rgb; 5] = [
    [0xf0, 0xf2, 0xc0],
    [0xb5, 0xcd, 0xe3],
    [0xac, 0xa2, 0x9a],
    [0xdb, 0x86, 0x64],
    [0x93, 0xb4, 0x83],
];

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

/// A section of the theme picker.
#[derive(Debug)]
pub(crate) struct Group {
    pub(crate) label: &'static str,
    pub(crate) slugs: &'static [&'static str],
}

/// A row of the theme picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PickerRow {
    /// A section label, which the selection skips.
    Header(&'static str),
    Theme(&'static str),
}

/// The theme picker's rows: each of orangui's groups under its label, in its order.
pub(crate) fn picker_rows() -> Vec<PickerRow> {
    GROUPS
        .iter()
        .flat_map(|group| {
            std::iter::once(PickerRow::Header(group.label))
                .chain(group.slugs.iter().map(|&slug| PickerRow::Theme(slug)))
        })
        .collect()
}

/// The slug a stored theme stands for now: itself, or its new name when it was renamed.
pub(crate) fn current_slug(theme: &str) -> &str {
    RENAMED_THEMES
        .iter()
        .find(|(old, _)| *old == theme)
        .map_or(theme, |(_, new)| *new)
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
        let slug = current_slug(theme);
        let Some(family) = FAMILIES.iter().find(|f| f.slug == slug) else {
            return Self::terminal();
        };
        Self::from_colors(if dark_mode {
            &family.dark
        } else {
            &family.light
        })
    }

    /// The palette the account's preferences ask for: its theme and mode, with pips in the theme's colors or Magic's classic ones.
    pub(crate) fn for_preferences(preferences: &UserPreferences) -> Self {
        let palette = Self::for_account(&preferences.theme, preferences.dark_mode);
        match preferences.pip_style {
            PipStyle::Theme => palette,
            PipStyle::Natural => Self {
                mana: NATURAL_PIPS.map(|[r, g, b]| Style::new().fg(Color::Rgb(r, g, b))),
                ..palette
            },
        }
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
    fn the_picker_lists_every_allowed_theme_once_under_seven_sections() {
        let rows = picker_rows();
        let mut themes: Vec<&str> = rows
            .iter()
            .filter_map(|r| match r {
                PickerRow::Theme(slug) => Some(*slug),
                PickerRow::Header(_) => None,
            })
            .collect();
        let headers = rows.len().saturating_sub(themes.len());
        themes.sort_unstable();
        let mut allowed = ALLOWED_THEMES.to_vec();
        allowed.sort_unstable();
        assert_eq!(themes, allowed);
        assert_eq!(headers, 7);
        assert_eq!(rows.first(), Some(&PickerRow::Header("Warm & earthy")));
    }

    #[test]
    fn natural_pips_keep_the_theme_but_swap_the_mana_colors() {
        let mut preferences = UserPreferences::default();
        let themed = Palette::for_preferences(&preferences);
        preferences.pip_style = PipStyle::Natural;
        let natural = Palette::for_preferences(&preferences);
        assert_eq!(themed.base, natural.base);
        assert_ne!(themed.mana, natural.mana);
        assert_eq!(natural.mana[4].fg, Some(Color::Rgb(0x93, 0xb4, 0x83)));
    }

    #[test]
    fn an_unknown_theme_uses_the_terminals_colors() {
        assert_eq!(Palette::for_account("nope", true), Palette::terminal());
        assert_eq!(Palette::terminal().base.bg, None);
    }
}
