# Session 1 — Database & Domain Layer

## Migration

**Create:** `zerver/migrations/YYYYMMDDHHMMSS_create_user_preferences.sql`

```sql
CREATE TABLE user_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    theme VARCHAR NOT NULL DEFAULT 'zwipe',
    dark_mode BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
```

- `user_id` is the PK (1:1 with users, no separate id)
- `ON DELETE CASCADE` — account deletion already works, preferences go with it
- Defaults match the current hardcoded Zwipe look

Run: `sqlx migrate run` then `cargo sqlx prepare --workspace`

---

## Domain Models

**Create:** `zerver/src/lib/domain/user/models/preferences.rs`

```rust
//! User display preferences (theme, dark mode).

use thiserror::Error;
use uuid::Uuid;

/// Allowed theme identifiers. Validated on update.
pub const ALLOWED_THEMES: &[&str] = &[
    "rustbox", "gruvbox", "dracula", "everforest", "catppuccin",
    "tokyo-night", "nord", "zwipe", "vantablack",
];

/// Themes that do not support light mode.
pub const DARK_ONLY_THEMES: &[&str] = &["zwipe", "vantablack"];

/// User display preferences.
#[derive(Debug, Clone, PartialEq)]
pub struct UserPreferences {
    /// Theme identifier (e.g. "gruvbox", "zwipe").
    pub theme: String,
    /// Whether dark mode is active.
    pub dark_mode: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "zwipe".to_string(),
            dark_mode: true,
        }
    }
}

/// Request to update a user's preferences.
#[derive(Debug)]
pub struct UpdatePreferences {
    /// User to update.
    pub user_id: Uuid,
    /// New theme identifier.
    pub theme: String,
    /// New dark mode setting.
    pub dark_mode: bool,
}

impl UpdatePreferences {
    /// Validates and constructs the request.
    pub fn new(user_id: Uuid, theme: &str, dark_mode: bool) -> Result<Self, InvalidUpdatePreferences> {
        if !ALLOWED_THEMES.contains(&theme) {
            return Err(InvalidUpdatePreferences::InvalidTheme);
        }
        // Force dark mode for dark-only themes
        let dark_mode = if DARK_ONLY_THEMES.contains(&theme) { true } else { dark_mode };
        Ok(Self {
            user_id,
            theme: theme.to_string(),
            dark_mode,
        })
    }
}

/// Validation error for preference updates.
#[derive(Debug, Error)]
pub enum InvalidUpdatePreferences {
    /// Theme is not in the allowed list.
    #[error("invalid theme")]
    InvalidTheme,
}

/// Error from the update preferences operation.
#[derive(Debug, Error)]
pub enum UpdatePreferencesError {
    /// Validation failed.
    #[error(transparent)]
    Invalid(#[from] InvalidUpdatePreferences),
    /// Database error.
    #[error("database error")]
    Database(#[from] anyhow::Error),
}

/// Error from the get preferences operation.
#[derive(Debug, Error)]
pub enum GetPreferencesError {
    /// Database error.
    #[error("database error")]
    Database(#[from] anyhow::Error),
}
```

**Modify:** `zerver/src/lib/domain/user/models/mod.rs` — add:
```rust
/// User display preferences.
pub mod preferences;
```

---

## Ports

**Modify:** `zerver/src/lib/domain/user/ports.rs`

Add to `UserRepository` trait:
```rust
/// Fetches preferences for a user. Returns defaults if no row exists.
fn get_preferences(
    &self,
    user_id: Uuid,
) -> impl Future<Output = Result<UserPreferences, GetPreferencesError>> + Send;

/// Upserts preferences for a user. Creates the row if it doesn't exist.
fn update_preferences(
    &self,
    request: &UpdatePreferences,
) -> impl Future<Output = Result<UserPreferences, UpdatePreferencesError>> + Send;
```

Add to `UserService` trait:
```rust
/// Fetches preferences for a user.
fn get_preferences(
    &self,
    user_id: Uuid,
) -> impl Future<Output = Result<UserPreferences, GetPreferencesError>> + Send;

/// Validates and updates preferences for a user.
fn update_preferences(
    &self,
    request: &UpdatePreferences,
) -> impl Future<Output = Result<UserPreferences, UpdatePreferencesError>> + Send;
```

Add necessary imports for `UserPreferences`, `UpdatePreferences`, `GetPreferencesError`,
`UpdatePreferencesError`.

---

## Service Implementation

**Modify:** `zerver/src/lib/domain/user/services.rs`

```rust
async fn get_preferences(&self, user_id: Uuid) -> Result<UserPreferences, GetPreferencesError> {
    self.user_repo.get_preferences(user_id).await
}

async fn update_preferences(&self, request: &UpdatePreferences) -> Result<UserPreferences, UpdatePreferencesError> {
    self.user_repo.update_preferences(request).await
}
```

Validation already happens in `UpdatePreferences::new()` — the service just delegates.

---

## SQL Adapter (Repository)

**Create or modify:** `zerver/src/lib/outbound/sqlx/user/` — add preferences queries.

### DatabaseUserPreferences model

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DatabaseUserPreferences {
    pub user_id: Uuid,
    pub theme: String,
    pub dark_mode: bool,
}

impl From<DatabaseUserPreferences> for UserPreferences {
    fn from(db: DatabaseUserPreferences) -> Self {
        Self {
            theme: db.theme,
            dark_mode: db.dark_mode,
        }
    }
}
```

### get_preferences

```rust
async fn get_preferences(&self, user_id: Uuid) -> Result<UserPreferences, GetPreferencesError> {
    let result = sqlx::query_as!(
        DatabaseUserPreferences,
        "SELECT user_id, theme, dark_mode FROM user_preferences WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| GetPreferencesError::Database(e.into()))?;

    Ok(result.map(UserPreferences::from).unwrap_or_default())
}
```

### update_preferences

```rust
async fn update_preferences(&self, request: &UpdatePreferences) -> Result<UserPreferences, UpdatePreferencesError> {
    let result = sqlx::query_as!(
        DatabaseUserPreferences,
        r#"INSERT INTO user_preferences (user_id, theme, dark_mode, updated_at)
           VALUES ($1, $2, $3, NOW())
           ON CONFLICT (user_id)
           DO UPDATE SET theme = $2, dark_mode = $3, updated_at = NOW()
           RETURNING user_id, theme, dark_mode"#,
        request.user_id,
        request.theme,
        request.dark_mode
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|e| UpdatePreferencesError::Database(e.into()))?;

    Ok(result.into())
}
```

---

## After this session

- Run `sqlx migrate run`
- Run `cargo sqlx prepare --workspace`
- Commit `.sqlx/` changes
- Run `cargo test --workspace` — ensure nothing breaks
- Run `cargo clippy --workspace --all-targets -- -D warnings`
