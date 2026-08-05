//! HTTP request/response helper types.
//!
//! Provides utilities for handling partial updates and JSON serialization edge cases.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned};

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
/// Every `Opdate` field MUST carry
/// `#[serde(default, skip_serializing_if = "Opdate::is_unchanged")]`:
/// `default` maps an absent key to `Unchanged`, and the skip keeps
/// `Unchanged` off the wire entirely. Serializing a bare `Unchanged` is a
/// hard error on purpose — without the skip attr it would emit `null`,
/// which decodes as "clear this field" and silently wipes data.
///
/// # Legacy dialect (accepted until the PATCH-migration cleanup)
///
/// The pre-1.7.5 derive shipped serde's externally-tagged encoding:
/// `"Unchanged"` as a literal string and `{"Set": value}` wrappers. The
/// custom `Deserialize` still accepts both so shipped clients keep working;
/// the acceptance is dropped once the version gate passes the clean-wire
/// clients (see `context/plans/patch_idempotent_updates.md`, Layer 3).
///
/// # Etymology
///
/// "Opdate" = **Op**tional Up**date** (portmanteau)
#[derive(Debug, Clone, PartialEq)]
pub enum Opdate<T> {
    /// Set field to this value (may be None to clear).
    Set(Option<T>),
    /// Leave field unchanged (don't update).
    Unchanged,
}

/// Clean wire form: `Set(Some(v))` → the bare value, `Set(None)` → `null`.
///
/// `Unchanged` errors: it must be kept off the wire by the field's
/// `skip_serializing_if` attr — emitting `null` instead would read as
/// "clear" on decode and destroy data, so a forgotten attr fails loudly.
impl<T: Serialize> Serialize for Opdate<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Opdate::Set(inner) => inner.serialize(serializer),
            Opdate::Unchanged => Err(serde::ser::Error::custom(
                "Opdate::Unchanged must be skipped, not serialized — add \
                 #[serde(default, skip_serializing_if = \"Opdate::is_unchanged\")] to the field",
            )),
        }
    }
}

/// Dual-accept decode: the clean shape (`null` → `Set(None)`, bare value →
/// `Set(Some)`, absent → `Unchanged` via `#[serde(default)]`) plus the
/// legacy externally-tagged dialect (`"Unchanged"` string, `{"Set": value}`)
/// from pre-1.7.5 clients. Goes clean-only at the migration cleanup.
///
/// Window ambiguity, accepted: an `Opdate<String>` whose legitimate value is
/// the literal string `"Unchanged"` decodes as `Unchanged`. Current string
/// fields are controlled vocab (format keys, power-level slugs), so this is
/// unreachable in practice, and it disappears with the legacy arms.
impl<'de, T: DeserializeOwned> Deserialize<'de> for Opdate<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::Null => Ok(Opdate::Set(None)),
            serde_json::Value::String(s) if s == "Unchanged" => Ok(Opdate::Unchanged),
            serde_json::Value::Object(map) if map.len() == 1 && map.contains_key("Set") => {
                let inner = map.into_values().next().unwrap_or(serde_json::Value::Null);
                match inner {
                    serde_json::Value::Null => Ok(Opdate::Set(None)),
                    other => T::deserialize(other)
                        .map(|v| Opdate::Set(Some(v)))
                        .map_err(D::Error::custom),
                }
            }
            other => T::deserialize(other)
                .map(|v| Opdate::Set(Some(v)))
                .map_err(D::Error::custom),
        }
    }
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

impl<T> Opdate<T> {
    /// Returns `true` if this is `Unchanged` (field should not be updated).
    /// Unbounded so `skip_serializing_if` works for any `T`.
    pub fn is_unchanged(&self) -> bool {
        matches!(self, Opdate::Unchanged)
    }

    /// Returns `true` if this is `Set(...)` (field should be updated).
    pub fn is_changed(&self) -> bool {
        matches!(self, Opdate::Set(_))
    }
}

impl<T> Opdate<T>
where
    T: PartialEq,
{
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Probe {
        #[serde(default, skip_serializing_if = "Opdate::is_unchanged")]
        field: Opdate<i32>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StringProbe {
        #[serde(default, skip_serializing_if = "Opdate::is_unchanged")]
        field: Opdate<String>,
    }

    #[test]
    fn clean_wire_decodes() {
        // absent -> Unchanged (via #[serde(default)])
        let p: Probe = serde_json::from_str("{}").unwrap();
        assert_eq!(p.field, Opdate::Unchanged);
        // null -> Set(None) (clear)
        let p: Probe = serde_json::from_str(r#"{"field":null}"#).unwrap();
        assert_eq!(p.field, Opdate::Set(None));
        // bare value -> Set(Some)
        let p: Probe = serde_json::from_str(r#"{"field":7}"#).unwrap();
        assert_eq!(p.field, Opdate::Set(Some(7)));
    }

    #[test]
    fn clean_wire_encodes() {
        // Unchanged stays off the wire entirely
        let wire = serde_json::to_string(&Probe {
            field: Opdate::Unchanged,
        })
        .unwrap();
        assert_eq!(wire, "{}");
        // Set(None) -> null
        let wire = serde_json::to_string(&Probe {
            field: Opdate::Set(None),
        })
        .unwrap();
        assert_eq!(wire, r#"{"field":null}"#);
        // Set(Some) -> bare value
        let wire = serde_json::to_string(&Probe {
            field: Opdate::Set(Some(7)),
        })
        .unwrap();
        assert_eq!(wire, r#"{"field":7}"#);
    }

    #[test]
    fn legacy_dialect_still_decodes() {
        // Pre-1.7.5 derive shapes must keep working until the gate cleanup.
        let p: Probe = serde_json::from_str(r#"{"field":"Unchanged"}"#).unwrap();
        assert_eq!(p.field, Opdate::Unchanged);
        let p: Probe = serde_json::from_str(r#"{"field":{"Set":7}}"#).unwrap();
        assert_eq!(p.field, Opdate::Set(Some(7)));
        let p: Probe = serde_json::from_str(r#"{"field":{"Set":null}}"#).unwrap();
        assert_eq!(p.field, Opdate::Set(None));
    }

    #[test]
    fn window_ambiguity_documented() {
        // An Opdate<String> literally valued "Unchanged" decodes as Unchanged
        // during the dual-accept window. Controlled-vocab fields make this
        // unreachable in practice; the legacy arm's removal ends it.
        let p: StringProbe = serde_json::from_str(r#"{"field":"Unchanged"}"#).unwrap();
        assert_eq!(p.field, Opdate::Unchanged);
        // Any other string is a normal set.
        let p: StringProbe = serde_json::from_str(r#"{"field":"commander"}"#).unwrap();
        assert_eq!(p.field, Opdate::Set(Some("commander".to_string())));
    }

    #[test]
    fn serializing_bare_unchanged_errors() {
        // The loud guard: a field missing skip_serializing_if fails instead
        // of emitting null (which would decode as "clear").
        assert!(serde_json::to_string(&Opdate::<i32>::Unchanged).is_err());
    }
}
