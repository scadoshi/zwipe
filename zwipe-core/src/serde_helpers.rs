//! Shared serde helpers for forward-compatible wire deserialization.

use serde::{Deserialize, Deserializer};

/// Deserialize a JSON array into `Vec<T>`, silently **dropping** any element
/// that fails to deserialize into `T` (instead of failing the whole payload).
///
/// Forward-compatibility bridge for enum-vecs on **served** types — card roles
/// (`CardRole`), deck tags (`DeckTag`/`DeckOtherTag`). A newer server can add a
/// role/tag slug that an older client's compiled enum doesn't know; with a strict
/// derived `Deserialize` that unknown variant would error the *entire* card/deck.
/// This helper drops the unknown element and keeps the rest, so an already-shipped
/// client keeps working against every future server (it just doesn't *show* the new
/// value until it adopts the server-driven catalog). See
/// `context/plans/server_driven_catalogs.md` (Part 0).
///
/// Apply with `#[serde(default, deserialize_with = "crate::serde_helpers::lossy_vec")]`.
/// Deserialize-only: serialization is unchanged, so the wire is identical and
/// existing clients are unaffected.
pub fn lossy_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    let raw = Vec::<serde_json::Value>::deserialize(deserializer)?;
    Ok(raw
        .into_iter()
        .filter_map(|value| serde_json::from_value(value).ok())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::lossy_vec;
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Role {
        Ramp,
        Removal,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Wrap {
        #[serde(default, deserialize_with = "lossy_vec")]
        roles: Vec<Role>,
    }

    #[test]
    fn drops_unknown_keeps_known_and_order() {
        // A future server sends a slug this binary's enum doesn't know.
        let w: Wrap =
            serde_json::from_str(r#"{"roles":["ramp","future_role","removal"]}"#).unwrap();
        assert_eq!(w.roles, vec![Role::Ramp, Role::Removal]);
    }

    #[test]
    fn empty_and_missing_are_fine() {
        assert!(
            serde_json::from_str::<Wrap>(r#"{"roles":[]}"#)
                .unwrap()
                .roles
                .is_empty()
        );
        assert!(
            serde_json::from_str::<Wrap>(r#"{}"#)
                .unwrap()
                .roles
                .is_empty()
        );
    }
}
