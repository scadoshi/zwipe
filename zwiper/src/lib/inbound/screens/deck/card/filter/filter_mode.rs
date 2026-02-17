//! Filter mode toggle between exact and range matching.

/// The mode for numeric filters (exact value or range).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum FilterMode {
    #[default]
    Exact,
    Range,
}

impl FilterMode {
    /// Toggles between Exact and Range modes.
    pub fn toggle(self) -> Self {
        match self {
            Self::Exact => Self::Range,
            Self::Range => Self::Exact,
        }
    }
}

impl std::fmt::Display for FilterMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact => write!(f, "Exact"),
            Self::Range => write!(f, "Range"),
        }
    }
}
