use thiserror::Error;

/// Error returned when building an invalid card filter.
#[derive(Debug, Error)]
pub enum InvalidCardFilter {
    /// Card filter must have at least one search criterion set.
    #[error("must have at least one filter")]
    Empty,
}
