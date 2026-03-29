# Session 3 — JWT Claims & Session Integration

Depends on: Session 2 (endpoints must work before embedding in tokens)

---

## Goal

Embed `theme` and `dark_mode` in JWT claims so the frontend has preferences immediately
on login without an extra API call.

---

## UserClaims

**Modify:** `zerver/src/lib/domain/auth/models/access_token.rs`

Add two fields to `UserClaims`:

```rust
pub struct UserClaims {
    pub user_id: Uuid,
    pub username: Username,
    pub email: EmailAddress,
    pub email_verified: bool,
    pub theme: String,      // NEW
    pub dark_mode: bool,    // NEW
    pub exp: i64,
    pub iat: i64,
}
```

Update all places that construct `UserClaims` — search for `UserClaims {` or
`UserClaims::new` (if a constructor exists). These are typically in:

- `authenticate_user` flow (login)
- `refresh_session` flow (token refresh)

Both need to fetch preferences from the `UserRepository` before building claims.

---

## Session Struct

**Modify:** `zerver/src/lib/domain/auth/models/session/mod.rs`

Add preferences to `Session`:

```rust
use crate::domain::user::models::preferences::UserPreferences;

pub struct Session {
    pub user: User,
    pub access_token: AccessToken,
    pub refresh_token: RefreshToken,
    pub preferences: UserPreferences,  // NEW
}
```

Update all places that construct `Session` — these are in the auth service where login
and refresh responses are built.

---

## Auth Service Changes

**Modify:** `zerver/src/lib/domain/auth/services.rs`

The auth service already holds a `UserRepository` (or accesses user data through the
auth repo). The key change: **fetch preferences during token generation**.

### In `authenticate_user`:

After successful password verification, before building the session:

```rust
let preferences = self.user_repo.get_preferences(user.id).await
    .unwrap_or_default();  // Default to zwipe/dark on any error
```

Then pass `preferences.theme` and `preferences.dark_mode` into `UserClaims` construction,
and `preferences` into `Session` construction.

### In `refresh_session`:

Same pattern — after validating the refresh token and loading the user, fetch preferences
before building the new access token:

```rust
let preferences = self.user_repo.get_preferences(user.id).await
    .unwrap_or_default();
```

### Important: service constructor

The auth service needs access to the `UserRepository` to fetch preferences. Check if it
already has one — it likely accesses `AuthRepository` which may not have the preferences
methods. Options:

**Option A:** Add `get_preferences` to `AuthRepository` trait (just the read method).

**Option B:** Pass the `UserRepository` to the auth service constructor alongside the
`AuthRepository`.

Check the existing `Service::new()` in `domain/auth/services.rs` to see what repositories
it already holds. Pick the approach that's least disruptive.

---

## AuthenticatedUser Middleware

**Modify:** `zerver/src/lib/inbound/http/middleware.rs`

The `AuthenticatedUser` extractor reads from JWT claims. Optionally add `theme` and
`dark_mode` to the struct — but these are only needed for the frontend, not for handler
authorization logic. **Skip this** unless a handler needs to know the user's theme.

---

## Frontend Session Reading

**Modify:** `zwiper/` — wherever `Session` is deserialized from the auth response.

The frontend receives the `Session` JSON from the login/refresh response. Since
`UserPreferences` is now a field on `Session`, it will deserialize automatically if
`UserPreferences` derives `Deserialize` (it should from Session 1).

After login:
```rust
// session.preferences.theme → "gruvbox"
// session.preferences.dark_mode → false
```

The frontend can read these from the session signal anywhere in the app.

---

## Token Size Consideration

Adding two small fields (`theme`: ~15 bytes, `dark_mode`: 5 bytes) to the JWT increases
token size by ~20 bytes. Negligible — JWTs are typically 500-800 bytes. No concern.

---

## Edge Case: Preferences Change Mid-Session

When a user updates preferences via the preferences screen:
1. The PUT endpoint updates the database
2. The frontend applies the theme locally (via signal)
3. The JWT still has the old values until the next token refresh

This is fine — the local signal takes precedence for display. On next refresh (within
60 seconds due to the upkeep interval), the new token will have the updated claims.

---

## After this session

- Login and inspect the JWT (decode at jwt.io or log the claims)
- Verify `theme` and `dark_mode` appear in claims
- Verify `preferences` field appears in session response JSON
- Change preferences via PUT, then refresh the token — verify new claims
- `cargo sqlx prepare --workspace` and commit `.sqlx/`
- `cargo test --workspace`
