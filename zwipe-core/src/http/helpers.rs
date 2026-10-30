//! HTTP request/response helper types.
//!
//! Provides utilities for handling partial updates and JSON serialization edge cases.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
/// `Patch<T>` explicitly distinguishes "unchanged" from "set to None":
/// - `Unchanged`: Don't update this field
/// - `Set(None)`: Clear field (set to NULL)
/// - `Set(Some(value))`: Update to new value
///
/// # JSON Representation
///
/// Custom serde impls below (not derived) make this shape real:
///
/// ```json
/// {
///   "name": "New Name",           // Set(Some("New Name"))
///   "commander_id": null,          // Set(None) - clear commander
///   // description not present     // Unchanged - keep existing
/// }
/// ```
///
/// Every `Patch` field MUST carry
/// `#[serde(default, skip_serializing_if = "Patch::is_unchanged")]`:
/// `default` maps an absent key to `Unchanged`, and the skip keeps
/// `Unchanged` off the wire entirely. Serializing a bare `Unchanged` is a
/// hard error on purpose: without the skip attr it would emit `null`,
/// which decodes as "clear this field" and silently wipes data.
///
/// The externally tagged dialect (`"Unchanged"` string, `{"Set": value}`
/// wrappers) is not decoded: the 1.7.5 version gate guarantees every
/// client speaks the clean shape.
///
/// Named for HTTP PATCH, where every update request that carries one is sent.
#[derive(Debug, Clone, PartialEq)]
pub enum Patch<T> {
    /// Set field to this value (may be None to clear).
    Set(Option<T>),
    /// Leave field unchanged (don't update).
    Unchanged,
}

/// Clean wire form: `Set(Some(v))` → the bare value, `Set(None)` → `null`.
///
/// `Unchanged` errors: it must be kept off the wire by the field's
/// `skip_serializing_if` attr; emitting `null` instead would read as
/// "clear" on decode and destroy data, so a forgotten attr fails loudly.
impl<T: Serialize> Serialize for Patch<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Patch::Set(inner) => inner.serialize(serializer),
            Patch::Unchanged => Err(serde::ser::Error::custom(
                "Patch::Unchanged must be skipped, not serialized: add \
                 #[serde(default, skip_serializing_if = \"Patch::is_unchanged\")] to the field",
            )),
        }
    }
}

/// Clean-shape decode only: `null` → `Set(None)`, bare value → `Set(Some)`,
/// absent → `Unchanged` via `#[serde(default)]`. The legacy externally-tagged
/// dialect from pre-1.7.5 clients fails to decode (or, for `Patch<String>`,
/// reads as an ordinary value); those clients are behind the version gate.
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Patch<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<T>::deserialize(deserializer).map(Patch::Set)
    }
}

/// Defaults to [`Patch::Unchanged`] so a field marked `#[serde(default)]` that
/// is absent from the request body is treated as "leave unchanged". This keeps
/// newly-added update fields backward-compatible: older clients that don't send
/// the field don't break.
impl<T> Default for Patch<T> {
    fn default() -> Self {
        Self::Unchanged
    }
}

impl<T> Patch<T> {
    /// Returns `true` if this is `Unchanged` (field should not be updated).
    /// Unbounded so `skip_serializing_if` works for any `T`.
    pub fn is_unchanged(&self) -> bool {
        matches!(self, Patch::Unchanged)
    }

    /// Returns `true` if this is `Set(...)` (field should be updated).
    pub fn is_changed(&self) -> bool {
        matches!(self, Patch::Set(_))
    }
}

impl<T> Patch<T> {
    /// Converts to domain-layer `Option<Option<T>>` representation.
    ///
    /// - `Unchanged` → `None` (don't update)
    /// - `Set(inner)` → `Some(inner)` (update to this value, possibly None)
    pub fn into_option(self) -> Option<Option<T>> {
        self.into()
    }
}

/// Converts `Patch<T>` to domain-layer `Option<Option<T>>` representation.
///
/// - `Unchanged` → `None` (don't update this field)
/// - `Set(inner)` → `Some(inner)` (update field, possibly to None)
impl<T> From<Patch<T>> for Option<Option<T>> {
    fn from(value: Patch<T>) -> Self {
        match value {
            Patch::Set(inner) => Some(inner),
            Patch::Unchanged => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Probe {
        #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
        field: Patch<i32>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StringProbe {
        #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
        field: Patch<String>,
    }

    #[test]
    fn clean_wire_decodes() {
        // absent -> Unchanged (via #[serde(default)])
        let p: Probe = serde_json::from_str("{}").unwrap();
        assert_eq!(p.field, Patch::Unchanged);
        // null -> Set(None) (clear)
        let p: Probe = serde_json::from_str(r#"{"field":null}"#).unwrap();
        assert_eq!(p.field, Patch::Set(None));
        // bare value -> Set(Some)
        let p: Probe = serde_json::from_str(r#"{"field":7}"#).unwrap();
        assert_eq!(p.field, Patch::Set(Some(7)));
    }

    #[test]
    fn clean_wire_encodes() {
        // Unchanged stays off the wire entirely
        let wire = serde_json::to_string(&Probe {
            field: Patch::Unchanged,
        })
        .unwrap();
        assert_eq!(wire, "{}");
        // Set(None) -> null
        let wire = serde_json::to_string(&Probe {
            field: Patch::Set(None),
        })
        .unwrap();
        assert_eq!(wire, r#"{"field":null}"#);
        // Set(Some) -> bare value
        let wire = serde_json::to_string(&Probe {
            field: Patch::Set(Some(7)),
        })
        .unwrap();
        assert_eq!(wire, r#"{"field":7}"#);
    }

    #[test]
    fn legacy_dialect_no_longer_decodes() {
        // Pre-1.7.5 derive shapes are behind the version gate: the tagged
        // object form fails outright for non-string targets...
        assert!(serde_json::from_str::<Probe>(r#"{"field":{"Set":7}}"#).is_err());
        assert!(serde_json::from_str::<Probe>(r#"{"field":"Unchanged"}"#).is_err());
        // ...and an "Unchanged" string is just a string value.
        let p: StringProbe = serde_json::from_str(r#"{"field":"Unchanged"}"#).unwrap();
        assert_eq!(p.field, Patch::Set(Some("Unchanged".to_string())));
    }

    #[test]
    fn wrong_type_errors() {
        assert!(serde_json::from_str::<Probe>(r#"{"field":"seven"}"#).is_err());
        assert!(serde_json::from_str::<Probe>(r#"{"field":[7]}"#).is_err());
        assert!(serde_json::from_str::<StringProbe>(r#"{"field":7}"#).is_err());
    }

    #[test]
    fn decodes_nested_values() {
        #[derive(Debug, Deserialize)]
        struct ListProbe {
            #[serde(default)]
            field: Patch<Vec<String>>,
        }
        let p: ListProbe = serde_json::from_str(r#"{"field":["a","b"]}"#).unwrap();
        assert_eq!(
            p.field,
            Patch::Set(Some(vec!["a".to_string(), "b".to_string()]))
        );
        let p: ListProbe = serde_json::from_str(r#"{"field":[]}"#).unwrap();
        assert_eq!(p.field, Patch::Set(Some(Vec::new())));
    }

    #[test]
    fn wrong_type_error_message() {
        let err = serde_json::from_str::<Probe>(r#"{"field":"seven"}"#).unwrap_err();
        let message = err.to_string();
        assert!(
            message.starts_with(r#"invalid type: string "seven", expected i32 at line 1"#),
            "message {message:?}"
        );
    }

    #[test]
    fn serializing_bare_unchanged_errors() {
        // The loud guard: a field missing skip_serializing_if fails instead
        // of emitting null (which would decode as "clear").
        assert!(serde_json::to_string(&Patch::<i32>::Unchanged).is_err());
    }
}
