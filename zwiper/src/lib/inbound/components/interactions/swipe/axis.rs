//! Axis enum for swipe direction locking.

/// The axis along which a swipe gesture is traveling.
#[derive(Debug, Clone, PartialEq)]
#[expect(missing_docs, reason = "the variants are the axis names")]
pub enum Axis {
    X,
    Y,
}
