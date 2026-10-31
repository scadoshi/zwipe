//! Sizes of text in terminal cells.

use ratatui::text::Span;

/// A border on each side.
pub(crate) const BORDERS: u16 = 2;

/// How many cells `text` takes, one per character.
pub(crate) fn width_of(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

/// A line count as a height in cells.
pub(crate) fn height_of(lines: usize) -> u16 {
    u16::try_from(lines).unwrap_or(u16::MAX)
}

/// `text` cut to at most `width` cells, ending in an ellipsis when cut. Cuts fall between characters, so a wide glyph is dropped whole rather than split.
pub(crate) fn ellipsize(text: &str, width: usize) -> String {
    let cells = |c: char| Span::raw(c.to_string()).width();
    if text.chars().map(cells).sum::<usize>() <= width {
        return text.to_owned();
    }
    let room = width.saturating_sub(1);
    let mut out = String::new();
    let mut used = 0_usize;
    for c in text.chars() {
        let w = cells(c);
        if used.saturating_add(w) > room {
            break;
        }
        used = used.saturating_add(w);
        out.push(c);
    }
    if width > 0 {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ellipsize_keeps_what_fits_and_never_splits_a_wide_glyph() {
        assert_eq!(ellipsize("Krenko", 6), "Krenko");
        assert_eq!(ellipsize("Krenko, Mob Boss", 8), "Krenko,…");
        assert_eq!(
            ellipsize("日本語", 4),
            "日…",
            "a wide glyph that does not fit goes whole"
        );
        assert_eq!(ellipsize("anything", 0), "");
    }

    #[test]
    fn widths_count_characters_and_heights_count_lines() {
        assert_eq!(width_of("héllo"), 5);
        assert_eq!(height_of(3), 3);
        assert_eq!(height_of(usize::MAX), u16::MAX);
    }
}
