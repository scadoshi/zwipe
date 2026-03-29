# Session 2 — HTTP Handlers & Routes

Depends on: Session 1 (domain layer must exist)

---

## HTTP Request/Response Types

**Create:** `zerver/src/lib/inbound/http/handlers/user/get_preferences.rs`

```rust
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::{models::preferences::UserPreferences, ports::UserService},
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState},
};
use axum::{Json, extract::State};

/// Returns the authenticated user's display preferences.
pub async fn get_preferences<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
) -> Result<Json<UserPreferences>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    state
        .user_service
        .get_preferences(user.id)
        .await
        .map(Json)
        .map_err(ApiError::from)
}
```

Note: `UserPreferences` needs `#[derive(Serialize)]` added to its definition in the
domain model for JSON serialization.

---

**Create:** `zerver/src/lib/inbound/http/handlers/user/update_preferences.rs`

```rust
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::{
            models::preferences::{UpdatePreferences, UserPreferences},
            ports::UserService,
        },
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState},
};
use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

/// HTTP request body for updating preferences.
#[derive(Debug, Deserialize)]
pub struct HttpUpdatePreferences {
    /// Theme identifier (e.g. "gruvbox", "dracula").
    pub theme: String,
    /// Whether dark mode is active.
    pub dark_mode: bool,
}

/// Updates the authenticated user's display preferences.
pub async fn update_preferences<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpUpdatePreferences>,
) -> Result<(StatusCode, Json<UserPreferences>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = UpdatePreferences::new(user.id, &body.theme, body.dark_mode)
        .map_err(ApiError::from)?;

    state
        .user_service
        .update_preferences(&request)
        .await
        .map(|prefs| (StatusCode::OK, Json(prefs)))
        .map_err(ApiError::from)
}
```

---

## Error Mappings

**Modify:** `zerver/src/lib/inbound/http/mod.rs` (or wherever `ApiError` From impls live)

Add `From` implementations for the new error types:

```rust
impl From<GetPreferencesError> for ApiError {
    fn from(e: GetPreferencesError) -> Self {
        match e {
            GetPreferencesError::Database(e) => ApiError::InternalServerError(e.to_string()),
        }
    }
}

impl From<UpdatePreferencesError> for ApiError {
    fn from(e: UpdatePreferencesError) -> Self {
        match e {
            UpdatePreferencesError::Invalid(e) => ApiError::UnprocessableEntity(e.to_string()),
            UpdatePreferencesError::Database(e) => ApiError::InternalServerError(e.to_string()),
        }
    }
}

impl From<InvalidUpdatePreferences> for ApiError {
    fn from(e: InvalidUpdatePreferences) -> Self {
        ApiError::UnprocessableEntity(e.to_string())
    }
}
```

---

## Handler Module Registration

**Modify:** `zerver/src/lib/inbound/http/handlers/user/mod.rs`

Add:
```rust
/// Get user preferences.
pub mod get_preferences;
/// Update user preferences.
pub mod update_preferences;
```

And re-export the handler functions.

---

## Routes

**Modify:** `zerver/src/lib/inbound/http/routes.rs`

In the private (authenticated) user routes section, add:

```rust
.route("/preferences", get(get_preferences).put(update_preferences))
```

No rate limiting needed — preferences are lightweight with no security risk.

Import the handler functions at the top of the file.

---

## Path Constants

**Modify:** `zerver/src/lib/inbound/http/paths.rs`

```rust
/// Path for user preferences endpoint.
pub fn preferences_route() -> String {
    "/api/user/preferences".to_string()
}
```

---

## Serde Derives

Make sure these derives exist on domain types:

- `UserPreferences` — add `Serialize, Deserialize` (needed for JSON response and JWT claims)
- `UpdatePreferences` — `Deserialize` is on the HTTP type, not this

---

## After this session

- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- Test manually:
  ```bash
  # Get defaults (no row exists yet)
  curl -H "Authorization: Bearer $TOKEN" https://api.zwipe.net/api/user/preferences

  # Update
  curl -X PUT -H "Authorization: Bearer $TOKEN" \
       -H "Content-Type: application/json" \
       -d '{"theme":"gruvbox","dark_mode":false}' \
       https://api.zwipe.net/api/user/preferences

  # Get again (should return gruvbox)
  curl -H "Authorization: Bearer $TOKEN" https://api.zwipe.net/api/user/preferences
  ```
