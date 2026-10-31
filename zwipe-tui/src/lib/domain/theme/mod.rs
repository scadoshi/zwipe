//! Color themes: the user's ZWIPE theme from their account, drawn with orangui's palettes, or the terminal's own colors before sign-in.

#[rustfmt::skip]
mod extras;
#[rustfmt::skip]
mod palettes;

use ratatui::style::{Color, Style};
use zwipe_core::domain::user::preferences::{PipStyle, RENAMED_THEMES, UserPreferences};

use self::extras::EXTRAS;
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

/// One mode of an orangui theme's colors beyond [`Colors`]: the ones the chrome draws with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Extras {
    pub(crate) accent_secondary: Rgb,
    pub(crate) accent_tertiary: Rgb,
    /// Skeleton bars.
    pub(crate) border_muted: Rgb,
    pub(crate) border_success: Rgb,
    pub(crate) border_error: Rgb,
    pub(crate) border_warning: Rgb,
    /// `--palette-1` to `--palette-6`, the theme's six swatches.
    pub(crate) swatches: [Rgb; 6],
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
    /// The logo, orangui's `--logo`: the text color on dark themes, the accent on light ones, where block letters in the text color look smudged.
    pub(crate) logo: Style,
    /// The app's `--accent-secondary`.
    pub(crate) secondary: Style,
    /// The app's `--accent-tertiary`: sheet titles and some hint words.
    pub(crate) tertiary: Style,
    /// The app's `--border-muted`, which skeleton bars fill with.
    pub(crate) skeleton: Style,
    /// The borders of success, error and warning toasts.
    pub(crate) border_success: Style,
    pub(crate) border_error: Style,
    pub(crate) border_warning: Style,
    /// The app's `--palette-1` to `--palette-6`.
    pub(crate) swatches: [Style; 6],
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
            logo: Style::new(),
            secondary: Style::new().yellow(),
            tertiary: Style::new().magenta(),
            skeleton: Style::new().dark_gray(),
            border_success: Style::new().green(),
            border_error: Style::new().red(),
            border_warning: Style::new().yellow(),
            swatches: [
                Style::new().cyan(),
                Style::new().magenta(),
                Style::new().red(),
                Style::new().green(),
                Style::new().yellow(),
                Style::new().blue(),
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
        let colors = if dark_mode {
            &family.dark
        } else {
            &family.light
        };
        let palette = Self {
            logo: fg(if dark_mode {
                colors.text
            } else {
                colors.accent
            }),
            ..Self::from_colors(colors)
        };
        extras(slug, dark_mode).map_or(palette, |extras| palette.with_extras(&extras))
    }

    /// `--palette-1` to `--palette-6` of `theme` in dark or light mode, the swatches beside it in the theme list. Empty for an unknown theme.
    pub(crate) fn swatches_of(theme: &str, dark_mode: bool) -> Vec<Style> {
        extras(current_slug(theme), dark_mode)
            .map(|extras| extras.swatches.iter().map(|&c| fg(c)).collect())
            .unwrap_or_default()
    }

    fn with_extras(self, extras: &Extras) -> Self {
        Self {
            secondary: fg(extras.accent_secondary),
            tertiary: fg(extras.accent_tertiary),
            skeleton: fg(extras.border_muted),
            border_success: fg(extras.border_success),
            border_error: fg(extras.border_error),
            border_warning: fg(extras.border_warning),
            swatches: extras.swatches.map(fg),
            ..self
        }
    }

    /// Every style the palette holds, in a fixed order, so two palettes can be matched role for role.
    pub(crate) fn styles(&self) -> [Style; 24] {
        let [w, u, b, r, g] = self.mana;
        let [p1, p2, p3, p4, p5, p6] = self.swatches;
        [
            self.base,
            self.muted,
            self.border,
            self.accent,
            self.add,
            self.error,
            self.warning,
            w,
            u,
            b,
            r,
            g,
            self.secondary,
            self.tertiary,
            self.skeleton,
            self.border_success,
            self.border_error,
            self.border_warning,
            p1,
            p2,
            p3,
            p4,
            p5,
            p6,
        ]
    }

    /// Whether the background is dark, for the direction of the theme wipe. The terminal's colors count as dark.
    pub(crate) fn is_dark(&self) -> bool {
        match self.base.bg {
            Some(Color::Rgb(r, g, b)) => {
                u32::from(r) * 299 + u32::from(g) * 587 + u32::from(b) * 114 < 128_000
            }
            _ => true,
        }
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
        let terminal = Self::terminal();
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
            ..terminal
        }
    }
}

/// A style with `rgb` as its text color.
fn fg([r, g, b]: Rgb) -> Style {
    Style::new().fg(Color::Rgb(r, g, b))
}

/// The extra colors of the theme `slug` in one mode.
fn extras(slug: &str, dark_mode: bool) -> Option<Extras> {
    EXTRAS
        .iter()
        .find(|(s, _, _)| *s == slug)
        .map(|(_, dark, light)| if dark_mode { *dark } else { *light })
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
    fn extras_cover_every_theme_and_fill_the_palette() {
        let mut slugs: Vec<&str> = EXTRAS.iter().map(|(s, _, _)| *s).collect();
        let mut families: Vec<&str> = FAMILIES.iter().map(|f| f.slug).collect();
        slugs.sort_unstable();
        families.sort_unstable();
        assert_eq!(
            slugs, families,
            "regenerate with zwipe-tui/scripts/gen_theme_extras.py"
        );
        let gruvbox = Palette::for_account("gruvbox", true);
        assert_eq!(gruvbox.secondary.fg, Some(Color::Rgb(0xfa, 0xbd, 0x2f)));
        assert_eq!(Palette::swatches_of("gruvbox", true).len(), 6);
        assert_eq!(Palette::swatches_of("nope", true), Vec::<Style>::new());
        assert!(gruvbox.is_dark());
        assert!(!Palette::for_account("gruvbox", false).is_dark());
    }

    #[test]
    fn the_logo_is_the_text_color_on_dark_and_the_accent_on_light() {
        let dark = Palette::for_account("gruvbox", true);
        let light = Palette::for_account("gruvbox", false);
        assert_eq!(dark.logo.fg, dark.base.fg);
        assert_eq!(light.logo.fg, light.accent.fg);
        assert_ne!(light.logo.fg, light.base.fg);
        assert_eq!(Palette::terminal().logo, Style::new());
    }

    #[test]
    fn an_unknown_theme_uses_the_terminals_colors() {
        assert_eq!(Palette::for_account("nope", true), Palette::terminal());
        assert_eq!(Palette::terminal().base.bg, None);
    }
}
