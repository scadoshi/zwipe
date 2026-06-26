//! HTTP request/response helper types.
//!
//! Provides utilities for handling partial updates and JSON serialization edge cases.

use serde::{Deserialize, Serialize};

/// Wrapper for optional field updates that preserves `None` vs "unchanged" semantics.
///
/// # Problem
///
/// Domain operations use `Option<Option<T>>` for partial updates:
/// - `None`: Field unchanged (don't update)
/// - `Some(None)`: Set field to NULL (clear value)
/// - `Some(Some(value))`: Update to new value
///
/// However, JSON serialization is lossy:
/// - `Some(None)` → JSON `null` → deserializes back to `None`
///
/// # Solution
///
/// `Opdate<T>` explicitly distinguishes "unchanged" from "set to None":
/// - `Unchanged`: Don't update this field
/// - `Set(None)`: Clear field (set to NULL)
/// - `Set(Some(value))`: Update to new value
///
/// # JSON Representation
///
/// ```json
/// {
///   "name": "New Name",           // Set(Some("New Name"))
///   "commander_id": null,          // Set(None) - clear commander
///   // description not present     // Unchanged - keep existing
/// }
/// ```
///
/// # Etymology
///
/// "Opdate" = **Op**tional Up**date** (portmanteau)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Opdate<T> {
    /// Set field to this value (may be None to clear).
    Set(Option<T>),
    /// Leave field unchanged (don't update).
    Unchanged,
}

/// Defaults to [`Opdate::Unchanged`] so a field marked `#[serde(default)]` that
/// is absent from the request body is treated as "leave unchanged". This keeps
/// newly-added update fields backward-compatible: older clients that don't send
/// the field don't break.
impl<T> Default for Opdate<T> {
    fn default() -> Self {
        Self::Unchanged
    }
}

impl<T> Opdate<T>
where
    T: PartialEq,
{
    /// Returns `true` if this is `Unchanged` (field should not be updated).
    pub fn is_unchanged(&self) -> bool {
        matches!(self, Opdate::Unchanged)
    }

    /// Returns `true` if this is `Set(...)` (field should be updated).
    pub fn is_changed(&self) -> bool {
        matches!(self, Opdate::Set(_))
    }

    /// Converts to domain-layer `Option<Option<T>>` representation.
    ///
    /// - `Unchanged` → `None` (don't update)
    /// - `Set(inner)` → `Some(inner)` (update to this value, possibly None)
    pub fn into_option(self) -> Option<Option<T>> {
        self.into()
    }
}

/// Iterator impl allows consuming `Opdate` in a for loop.
///
/// Yields the inner `Option<T>` once if `Set`, nothing if `Unchanged`.
/// After iteration, the `Opdate` becomes `Unchanged`.
impl<T> Iterator for Opdate<T> {
    type Item = Option<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Opdate::Set(value) => {
                let result = Some(value.take());
                *self = Opdate::Unchanged;
                result
            }
            Opdate::Unchanged => None,
        }
    }
}

/// Converts `Opdate<T>` to domain-layer `Option<Option<T>>` representation.
///
/// - `Unchanged` → `None` (don't update this field)
/// - `Set(inner)` → `Some(inner)` (update field, possibly to None)
impl<T> From<Opdate<T>> for Option<Option<T>>
where
    T: PartialEq,
{
    fn from(value: Opdate<T>) -> Self {
        match value {
            Opdate::Set(inner) => Some(inner),
            Opdate::Unchanged => None,
        }
    }
}
