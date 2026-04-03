//! Deck management HTTP request contracts.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::helpers::Optdate;

/// Deck creation request body.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpCreateDeckProfile {
    /// Deck display name.
    pub name: String,
    /// Optional commander card ID.
    pub commander_id: Option<Uuid>,
    /// Optional deck format.
    pub format: Option<String>,
}

impl HttpCreateDeckProfile {
    /// Creates a new deck creation request.
    pub fn new(name: &str, commander_id: Option<Uuid>, format: Option<String>) -> Self {
        Self {
            name: name.to_string(),
            commander_id,
            format,
        }
    }
}

/// Deck metadata update request body with partial update semantics.
///
/// Uses [`Optdate`] for nullable fields: absent means unchanged, `null` means set to `None`.
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpUpdateDeckProfile {
    /// New deck name, or `None` to leave unchanged.
    pub name: Option<String>,
    /// Commander card ID with partial update semantics.
    pub commander_id: Optdate<Uuid>,
    /// Format with partial update semantics.
    pub format: Optdate<String>,
}

impl HttpUpdateDeckProfile {
    /// Creates a new deck update request.
    pub fn new(name: Option<&str>, commander_id: Optdate<Uuid>, format: Optdate<String>) -> Self {
        Self {
            name: name.map(|name| name.to_string()),
            commander_id,
            format,
        }
    }
}
