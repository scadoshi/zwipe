#[derive(Debug, Default, Clone, Copy)]
pub enum ColorIdentityFilterMode {
    #[default]
    Within,
    Exact,
}

impl ColorIdentityFilterMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Exact => Self::Within,
            Self::Within => Self::Exact,
        }
    }
}

impl std::fmt::Display for ColorIdentityFilterMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Within => write!(f, "Within"),
            Self::Exact => write!(f, "Exact"),
        }
    }
}
