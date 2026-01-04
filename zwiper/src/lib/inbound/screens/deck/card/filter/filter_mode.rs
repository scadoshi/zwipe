#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum FilterMode {
    #[default]
    Exact,
    Range,
}

impl FilterMode {
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
