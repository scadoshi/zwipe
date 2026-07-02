use thiserror::Error;

/// Error returned when building invalid card criteria.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum InvalidCardCriteria {
    /// Criteria must have at least one criterion set.
    #[error("must have at least one filter")]
    Empty,
    /// A value appears in both an include and an exclude list for the same
    /// attribute — a contradiction that matches zero cards (e.g. include and
    /// exclude the Land card type).
    #[error("filter both includes and excludes {field}: {values}")]
    Contradiction {
        /// Human-readable attribute name (e.g. "card types").
        field: &'static str,
        /// The clashing values, comma-separated.
        values: String,
    },
}
