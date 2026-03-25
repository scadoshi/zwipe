//! Match mode toggle between any and all matching.

/// Whether a multi-select filter uses OR ("any") or AND ("all") matching.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum MatchMode {
    #[default]
    Any,
    All,
}

impl MatchMode {
    /// Toggles between Any and All modes.
    pub fn toggle(self) -> Self {
        match self {
            Self::Any => Self::All,
            Self::All => Self::Any,
        }
    }

    /// Returns a display label for the current mode.
    pub fn label(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::All => "all",
        }
    }
}

impl std::fmt::Display for MatchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => write!(f, "Any"),
            Self::All => write!(f, "All"),
        }
    }
}
