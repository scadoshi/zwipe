# Per-session client version tracking

**Status: DONE 2026-07-15 (shipped, archived). Additive, server-first; fits the standard
`api_evolution.md` new-field pattern. Deploy server before shipping the client.
Login, register, and refresh all covered. On refresh the client **re-sends** the
current version (it overwrites, unlike platform's pure carry-forward) so the row
tracks the version that is actually running, not the one it was born at. Mirrors
the `platform` work in commit `09763a97` / [`session_platform.md`](session_platform.md).**

**One sentence:** stamp each session (refresh-token row) with the app version
(`CARGO_PKG_VERSION`, e.g. `"1.6.1"`) that created or last rotated it, so we can
query which client versions are live in the wild for analytics and bug triage.

## Why

We can tell a session's platform now, but not its app version. Knowing the
version distribution of live sessions answers "how fast is an update rolling
out," "how many users are still on a version with a known bug," and gates
`MIN_CLIENT_VERSION` decisions with real data instead of guesses.

**Grain = session, not user.** One user holds up to 5 concurrent sessions
(refresh tokens) and can be on different builds across devices, so version
belongs on the token row, not the `users` row.

## Version differs from platform in one way: it changes

Platform is fixed for a given install, so the server carries it forward silently
across refresh rotation (client never re-sends it). **Version changes when the
user updates the app.** If we carried it forward the same way, a session would
show the version it was *born at* and go stale after every update (until the user
re-logs, which can be 14 days or never). So the client must **re-send the current
version on refresh**, and the server **overwrites** on rotation.

Net effect: a session's `client_version` self-corrects within ~24h (the
access-token TTL, i.e. the refresh cadence) of the user updating the app. That is
the whole point.

## Design

### 1. No new core type

Version is a free-form semver `String`, not a closed set, so there is **no enum /
newtype** to add (unlike `ClientPlatform`). It mirrors `HttpMinClientVersion.min_version`
in [`../../zwipe-core/src/http/contracts/client.rs`](../../zwipe-core/src/http/contracts/client.rs),
which is already a plain `String`. Stored/serialized as the raw version string.

### 2. Wire contract — optional field on the auth requests

In `zwipe-core/src/http/contracts/auth.rs`, add to the **login**, **register**,
**and refresh** request contracts:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub client_version: Option<String>,
```

`Option` + `#[serde(default)]` = additive and backward-compatible (old clients
omit it -> `None`). Per [`../development/api_evolution.md`](../development/api_evolution.md):
new request fields are additive, server deploys first, client second, no gate
needed.

Note refresh gets the field here where platform did **not** — that is the one
intentional divergence from the platform commit (see the section above).

### 3. Column — `refresh_tokens.client_version`

New migration `zerver/migrations/<ts>_add_refresh_token_client_version.sql`:

```sql
-- Record the client app version (e.g. "1.6.1") on each session so we can query
-- the live version distribution for analytics and bug triage. Nullable: old
-- rows and older clients stay NULL. Additive; deploy server-first.

ALTER TABLE refresh_tokens ADD COLUMN client_version TEXT;

CREATE INDEX idx_refresh_tokens_client_version ON refresh_tokens(client_version);
```

Nullable text, index so the version query is cheap.

### 4. Thread it through session creation

Same files the platform commit touched:

- **Handlers** `inbound/http/handlers/auth/{authenticate_user, register_user}.rs`
  read the new contract field: `request.client_version = body.client_version;`
- **`CreateSession`** (`domain/auth/requests/create_session.rs`),
  **`RegisterUser`** (`.../register_user.rs`), and **`AuthenticateUser`**
  (`.../authenticate_user.rs`) each gain a `client_version: Option<String>`
  field alongside `platform`.
- **Session service** (`domain/auth/services.rs`) carries it to the token insert:
  `create_refresh_token(user_id, platform, client_version)`.
- **Insert** (`outbound/sqlx/auth/helpers.rs`) extends the `INSERT INTO
  refresh_tokens (... , client_version)` and `RETURNING` list;
  `DatabaseRefreshToken` (`outbound/sqlx/auth/models.rs`) gains
  `client_version: Option<String>`.
- **Refresh rotation** (`outbound/sqlx/auth/mod.rs` `use_refresh_token`): add
  `client_version` to the `SELECT ... FOR UPDATE`, then pass **the incoming
  request's version, falling back to the consumed token's stored value** when the
  client omits it:

  ```rust
  // Version can change on app update, so prefer what the client just sent;
  // fall back to the old row's value for clients that don't send it yet.
  let version = request.client_version.clone().or(existing.client_version);
  let new = tx.create_refresh_token(request.user_id, carried_platform, version).await?;
  ```

  (Platform's line stays pure carry-forward right above this — the two differ by
  design.)

- **Client** `outbound/client/auth/{login, register, refresh}.rs` set
  `client_version: Some(env!("CARGO_PKG_VERSION").to_string())` on the request.
  The client already exposes this as `APP_VERSION` in a few components.

### 5. The payoff query

```sql
SELECT client_version, count(*) AS live_sessions
FROM refresh_tokens
WHERE revoked = false
  AND expires_at > now()
GROUP BY client_version
ORDER BY live_sessions DESC;   -- live version distribution
```

## Rollout

1. **Server first** (own deploy): migration + accept/store the field. Old clients
   send nothing -> `NULL`, no behavior change.
2. **Client after**: send `CARGO_PKG_VERSION` on login/register/refresh; ships in
   a store build.
3. Existing sessions self-populate within ~24h of the client rolling out (as they
   refresh). No backfill.

## Scope / out of scope

- **Out:** retroactive backfill (no reliable server-side signal for existing
  sessions' version).
- **Out:** OS version / build number — app version only for now; extend later if a
  real need appears.
- **Out (possible follow-on):** also stamp version on `user_events` for aggregate
  funnel-by-version analytics. Kept separate to keep this change single-purpose.
- **Privacy:** app version is low-sensitivity and already inside the per-account
  analytics disclosure; a minor data-safety label touch-up at most.

## Checklist

- [ ] Migration + index
- [ ] `client_version` on `HttpAuthenticateUser`, `HttpRegisterUser`, `HttpRefreshSession`
- [ ] Domain requests (`CreateSession`, `RegisterUser`, `AuthenticateUser`)
- [ ] Handlers read `body.client_version`
- [ ] `services.rs` + `helpers.rs` insert + `DatabaseRefreshToken`
- [ ] `use_refresh_token` overwrite-else-carry rotation + `SELECT` column
- [ ] Client login / register / refresh send `CARGO_PKG_VERSION`
- [ ] `cargo sqlx prepare --workspace`, auth tests, `cargo +nightly fmt`
