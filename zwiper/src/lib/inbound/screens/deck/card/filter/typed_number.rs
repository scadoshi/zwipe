//! What a number box's text says while it is typed, so a filter can take a
//! value as it is typed rather than only when the box loses focus. A tap on
//! Apply does not blur the box on Android, so a value written only on blur
//! never reaches the filter.

/// A number box's text, read as it is typed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum TypedNumber {
    /// A number: the filter takes it.
    Number(f64),
    /// An empty box: the filter drops the value.
    Empty,
    /// Not a number yet, such as `1.`: the filter keeps what it had, and the
    /// blur says it is invalid if it stays so.
    Partial,
}

impl TypedNumber {
    /// Reads `text`, ignoring the spaces around it.
    pub(crate) fn read(text: &str) -> Self {
        let text = text.trim();
        if text.is_empty() {
            return Self::Empty;
        }
        text.parse().map_or(Self::Partial, Self::Number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_number_an_empty_box_and_half_typed_text_read_apart() {
        assert_eq!(TypedNumber::read("3"), TypedNumber::Number(3.0));
        assert_eq!(TypedNumber::read(" 2.5 "), TypedNumber::Number(2.5));
        assert_eq!(TypedNumber::read(""), TypedNumber::Empty);
        assert_eq!(TypedNumber::read("  "), TypedNumber::Empty);
        assert_eq!(TypedNumber::read("-"), TypedNumber::Partial);
        assert_eq!(TypedNumber::read("x"), TypedNumber::Partial);
    }
}
