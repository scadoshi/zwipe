//! Mana colors: the theme's style for each of white, blue, black, red and green, as the deck list, deck charts, home color map and card text draw them.

use ratatui::style::Style;
use ratatui::text::Span;

use crate::domain::theme::Palette;

/// The five colors' letters in WUBRG order.
pub(crate) const LETTERS: [&str; 5] = ["W", "U", "B", "R", "G"];

/// The style for the color at `index` in WUBRG order, muted for anything past green.
pub(crate) fn style(palette: &Palette, index: usize) -> Style {
    palette.mana.get(index).copied().unwrap_or(palette.muted)
}

/// The style for a WUBRG letter, muted for anything else.
pub(crate) fn letter_style(palette: &Palette, letter: &str) -> Style {
    LETTERS
        .iter()
        .position(|l| *l == letter)
        .map_or(palette.muted, |i| style(palette, i))
}

/// A pip: the color's letter in bold in its style, or `?` for an index past green.
pub(crate) fn pip(palette: &Palette, index: usize) -> Span<'static> {
    Span::styled(
        LETTERS.get(index).copied().unwrap_or("?"),
        style(palette, index).bold(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_color_takes_its_theme_style_and_anything_else_is_muted() {
        let palette = Palette::for_account("gruvbox", true);
        assert_eq!(style(&palette, 4), palette.mana[4]);
        assert_eq!(style(&palette, 5), palette.muted);
        assert_eq!(letter_style(&palette, "U"), palette.mana[1]);
        assert_eq!(letter_style(&palette, "C"), palette.muted);
        assert_eq!(pip(&palette, 0).content, "W");
        assert_eq!(pip(&palette, 9).content, "?");
    }
}
