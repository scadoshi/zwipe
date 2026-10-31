//! Which way a focus moves along an ordered list of stops, fields, rows or chips.

/// Which way a focus moves along its order: [`Toward::Next`] is Tab, Down, `j` or Right, and [`Toward::Previous`] their opposites.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Toward {
    Next,
    Previous,
}

impl Toward {
    /// The index one step from `at`, or `None` past either end.
    pub(crate) fn neighbor(self, at: usize) -> Option<usize> {
        match self {
            Self::Next => at.checked_add(1),
            Self::Previous => at.checked_sub(1),
        }
    }

    /// The index one step from `at` among `len` items, staying on the first or last at the ends.
    pub(crate) fn clamped(self, at: usize, len: usize) -> usize {
        match self {
            Self::Next => at.saturating_add(1).min(len.saturating_sub(1)),
            Self::Previous => at.saturating_sub(1),
        }
    }

    /// The index one step from `at` among `len` items, wrapping round from either end to the other. An empty list gives 0.
    pub(crate) fn wrapped(self, at: usize, len: usize) -> usize {
        match self {
            Self::Next => at.saturating_add(1).checked_rem(len).unwrap_or(0),
            Self::Previous => at.checked_sub(1).unwrap_or_else(|| len.saturating_sub(1)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbor_stops_past_either_end() {
        assert_eq!(Toward::Next.neighbor(2), Some(3));
        assert_eq!(Toward::Previous.neighbor(2), Some(1));
        assert_eq!(Toward::Previous.neighbor(0), None);
    }

    #[test]
    fn clamped_holds_at_the_ends() {
        assert_eq!(Toward::Next.clamped(2, 3), 2);
        assert_eq!(Toward::Next.clamped(0, 3), 1);
        assert_eq!(Toward::Previous.clamped(0, 3), 0);
        assert_eq!(Toward::Next.clamped(0, 0), 0);
    }

    #[test]
    fn wrapped_goes_round() {
        assert_eq!(Toward::Next.wrapped(2, 3), 0);
        assert_eq!(Toward::Previous.wrapped(0, 3), 2);
        assert_eq!(Toward::Next.wrapped(0, 0), 0);
        assert_eq!(Toward::Previous.wrapped(0, 0), 0);
    }
}
