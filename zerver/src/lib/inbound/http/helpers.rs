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
/// - `Some(None)` → JSON `null` → deserializes back to `None` ❌
///
/// # Solution
///
/// `Optdate<T>` explicitly distinguishes "unchanged" from "set to None":
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
/// # Usage
///
/// ```rust,ignore
/// #[derive(Deserialize)]
/// struct UpdateRequest {
///     name: Optdate<String>,
///     commander_id: Optdate<Uuid>,
/// }
///
/// let request: UpdateRequest = serde_json::from_str(json)?;
///
/// // Convert to domain Option<Option<T>>
/// let name: Option<Option<String>> = request.name.into_option();
/// ```
///
/// # Etymology
///
/// "Optdate" = **Opt**ional Up**date** (portmanteau)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Optdate<T> {
    /// Set field to this value (may be None to clear).
    Set(Option<T>),
    /// Leave field unchanged (don't update).
    Unchanged,
}

impl<T> Optdate<T>
where
    T: PartialEq,
{
    /// Returns `true` if this is `Unchanged` (field should not be updated).
    pub fn is_unchanged(&self) -> bool {
        matches!(self, Optdate::Unchanged)
    }

    /// Returns `true` if this is `Set(...)` (field should be updated).
    pub fn is_changed(&self) -> bool {
        matches!(self, Optdate::Set(_))
    }

    /// Converts to domain-layer `Option<Option<T>>` representation.
    ///
    /// - `Unchanged` → `None` (don't update)
    /// - `Set(inner)` → `Some(inner)` (update to this value, possibly None)
    pub fn into_option(self) -> Option<Option<T>> {
        self.into()
    }
}

/// Iterator impl allows consuming `Optdate` in a for loop.
///
/// Yields the inner `Option<T>` once if `Set`, nothing if `Unchanged`.
/// After iteration, the `Optdate` becomes `Unchanged`.
impl<T> Iterator for Optdate<T> {
    type Item = Option<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Optdate::Set(value) => {
                let result = Some(value.take());
                *self = Optdate::Unchanged;
                result
            }
            Optdate::Unchanged => None,
        }
    }
}

/// Converts `Optdate<T>` to domain-layer `Option<Option<T>>` representation.
///
/// - `Unchanged` → `None` (don't update this field)
/// - `Set(inner)` → `Some(inner)` (update field, possibly to None)
impl<T> From<Optdate<T>> for Option<Option<T>>
where
    T: PartialEq,
{
    fn from(value: Optdate<T>) -> Self {
        match value {
            Optdate::Set(inner) => Some(inner),
            Optdate::Unchanged => None,
        }
    }
}
