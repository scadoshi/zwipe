#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum FilterMode {
    #[default]
    Exact,
    Within,
}

impl FilterMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Exact => Self::Within,
            Self::Within => Self::Exact,
        }
    }
}

impl std::fmt::Display for FilterMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact => write!(f, "exact"),
            Self::Within => write!(f, "within"),
        }
    }
}
