# Per-session client platform tracking

**Status: BUILT 2026-07-09 (server + client compile, auth tests pass; migration
applied to local dev DB, `.sqlx` regenerated). Additive, server-first; fits the
standard `api_evolution.md` new-field pattern. Deploy server before shipping the
client. Login, register, and refresh (carry-forward) all covered; no `Unknown`
variant (absence = NULL/None).**

**One sentence:** stamp each session (refresh-token row) with the client platform
it was created from (iOS / Android / desktop / web) so we can query "which users
have an Android session" for targeted comms and platform analytics.

## Why

We can't currently tell, from the database, what platform a user is on — a real
gap now that the user base is split across iOS + Android and we may need to reach
one platform (e.g. Android testers about a production move). Platform also unlocks
funnel-by-platform analytics and platform-specific bug triage.

**Grain = session, not user.** One user can hold up to 5 concurrent sessions
(refresh tokens) and can legitimately be on iOS *and* Android *and* web at once,
so platform belongs on the token row, not the `users` row.

**Important limitation (set expectations):** this is **forward-only**. It tags
sessions created *after* the client ships; it does **not** retroactively identify
existing users. Because access tokens are 24h and refresh tokens rotate (14d),
the table **self-populates within ~14 days** of the client rolling out as clients
refresh. So it does *not* solve an immediate "email today's Android testers" need
— for that, rely on Play auto-update + the existing tester channels (the `zwipers`
group / Discord). It's an investment in going-forward visibility.

## Design

### 1. Shared type — `ClientPlatform` (zwipe-core, pure)

New `zwipe-core/src/domain/auth/models/platform.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClientPlatform { Ios, Android, Desktop, Web }
```

Pure enum: serde + `Display`/`FromStr` only (allowed core deps). Both client (to
produce the value) and server (to validate/store) use it. Stored/serialized as a
lowercase string (`"ios"`, `"android"`, `"desktop"`, `"web"`).

`ClientPlatform::current()` derives the running platform via `#[cfg(...)]` — this
generalizes the existing ad-hoc const in
`zwiper/src/lib/inbound/components/support.rs` (which today is only a binary
Android-vs-"iOS" cfg and ignores desktop/web). Fold that const into the shared
`current()` so the report-a-problem prefill and the session field agree.

### 2. Wire contract — optional field on the auth requests

In `zwipe-core/src/http/contracts/auth.rs`, add to the **login**, **register**,
and **refresh** request contracts:

```rust
#[serde(default)]
pub platform: Option<ClientPlatform>,
```

`Option` + `#[serde(default)]` = additive and backward-compatible (old clients
omit it → `None`). This is exactly the documented rule in
[`../development/api_evolution.md`](../development/api_evolution.md): new request
fields are additive with `#[serde(default)]`, server deploys first, client second,
no gate needed.

*(Alternative considered: an `X-Client-Platform` header extracted by middleware —
DRYer across endpoints, but bypasses the typed contract layer. The typed field
fits this codebase's contract-first style and the api_evolution rule, so prefer
it. Fall back to the header only if threading the field through three requests is
noisier than expected.)*

### 3. Column — `refresh_tokens.platform`

New migration `zerver/migrations/<ts>_add_refresh_token_platform.sql` (mirrors the
shape of `20251007160206_create_refresh_tokens.sql`):

```sql
ALTER TABLE refresh_tokens ADD COLUMN platform TEXT;   -- nullable; old rows stay NULL
CREATE INDEX idx_refresh_tokens_platform ON refresh_tokens(platform);
```

Nullable text (validated against `ClientPlatform` at the app layer, same as other
domain-typed text columns). Index so the platform query is cheap.

### 4. Thread it through session creation

- **Handlers** `inbound/http/handlers/auth/{authenticate_user, register_user,
  refresh_session}.rs` read the new contract field and pass it down.
- **`CreateSession`** (`domain/auth/requests/create_session.rs`) gains a
  `platform: Option<ClientPlatform>` field alongside `user_id`.
- **Session service** (`domain/auth/services.rs`) carries it to the token insert.
- **Insert** (`outbound/sqlx/auth/helpers.rs` — the `INSERT INTO refresh_tokens`)
  writes the platform string. On **refresh**, the client re-sends platform so the
  rotated row keeps it (simpler than copying from the consumed token).
- **Client** `outbound/client/auth/{login, refresh}.rs` (+ register) set
  `platform: Some(ClientPlatform::current())` on the request.

### 5. The payoff query

```sql
SELECT DISTINCT u.email
FROM users u
JOIN refresh_tokens rt ON rt.user_id = u.id
WHERE rt.platform = 'android'
  AND rt.revoked = false
  AND rt.expires_at > now();   -- users with a live Android session
```

## Rollout

1. **Server first** (own deploy): migration + accept/store the field. Old clients
   send nothing → `NULL`, no behavior change.
2. **Client after**: send `ClientPlatform::current()` on login/register/refresh;
   ships in a store build (rides 1.4.1+).
3. Table self-populates within ~14 days as sessions rotate. No backfill.

## Scope / out of scope

- **Out:** retroactive backfill (no reliable server-side signal for existing
  sessions).
- **Out:** device model / OS version — keep it to platform for now; extend the
  enum/columns later if a real need appears.
- **Out (possible follow-on):** also stamp platform on `user_events` for
  aggregate funnel-by-platform analytics. Kept separate to keep this change
  single-purpose.
- **Privacy:** platform is low-sensitivity and already inside the per-account
  analytics disclosure; a minor Play data-safety / App Store label touch-up at
  most.

## Open questions

- Enum home: `domain/auth/models/platform.rs` (proposed) vs a more neutral
  `domain/session/` — auth/models is fine since sessions live there.
- Send OS version too (for "min Android version" style decisions), or platform
  only? Defaulting to platform only.
- Whether the immediate re-install worry is even real (Play closed→production
  usually auto-updates) — resolve that separately before treating comms as
  urgent.
