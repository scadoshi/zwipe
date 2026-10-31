//! Sizes of text in terminal cells.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_count_characters_and_heights_count_lines() {
        assert_eq!(width_of("héllo"), 5);
        assert_eq!(height_of(3), 3);
        assert_eq!(height_of(usize::MAX), u16::MAX);
    }
}
