use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvalidCardFilter {
    #[error("must have at least one filter")]
    Empty,
}
