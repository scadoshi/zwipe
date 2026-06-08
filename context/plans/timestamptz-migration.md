# TIMESTAMP → TIMESTAMPTZ migration

## Context

Every timestamp column in the schema today is plain `TIMESTAMP` (no time
zone). PostgreSQL treats that type as "wall-clock value with no zone
info attached" — the value stored is literally whatever the writing
session's `TIMEZONE` resolves to at write time.

Today the system is *correct by accident*: the home-server Postgres
cluster defaults to UTC, and as of `feat/user-metrics` the SQLx pool
runs `SET TIME ZONE 'UTC'` on every connection
(`zerver/src/lib/outbound/sqlx/postgres.rs::after_connect`). That
backstop guarantees writes land in UTC regardless of where the process
runs. But the schema doesn't *encode* that intent — a future
maintainer who refactors the pool, a one-off psql session run in local
TZ, or a CI runner with a different cluster TZ could all silently store
values in the wrong zone, and the type system would not catch it.

`TIMESTAMPTZ` (`timestamp with time zone`) fixes this by storing **an
instant in time, canonicalized to UTC internally, presented in the
reader's session TZ on read**. The semantic — "this is the moment X
happened" — is what we actually want for every `created_at`,
`updated_at`, `expires_at`, `occurred_at`, etc.

The benefit over "TIMESTAMP + pool pin" is **defense in depth and
explicitness**, not correctness:

- The type itself says "this is a UTC instant" — no convention to
  remember.
- Even if the pool pin is dropped, `TIMESTAMPTZ` keeps reading correctly
  because Postgres converts on the way out using the session TZ.
- JSON wire format gains a `Z` suffix (`"2026-06-08T04:40:20Z"`), so
  non-Rust consumers (curl, BI tools, future web client) can parse it
  unambiguously.

This work is scheduled as deliberate maintenance, not a feature. It
ships on its own branch, behind its own review, with a fresh DB backup
in hand.

## Design decisions (defaults with override notes)

| Decision | Default | Notes |
|---|---|---|
| **Schema type** | `TIMESTAMPTZ` everywhere a timestamp lives | Drop `TIMESTAMP WITHOUT TIME ZONE` entirely from the schema. One exception below for Scryfall data. |
| **Rust mapping** | `chrono::DateTime<chrono::Utc>` | Replaces `chrono::NaiveDateTime` in all domain + adapter types. Makes "UTC instant" a type-level invariant. |
| **JSON serialization** | RFC3339 with `Z` suffix | `serde_json` does this for `DateTime<Utc>` out of the box. Bumps wire-format compatibility for the iOS client (parses both, but the new format is more explicit). |
| **Migration approach** | Per-table `ALTER COLUMN ... TYPE TIMESTAMPTZ USING ... AT TIME ZONE 'UTC'` | One migration per logical table group, not one giant migration. Failure isolation matters. |
| **Existing-row conversion** | Assume existing values are already UTC and re-tag | The pool pin has been in place since `feat/user-metrics` shipped. Older rows predate the pin but the home cluster has always defaulted to UTC, so this should be safe. Spot-check oldest rows on prod before running each step. |
| **Scryfall `released_at`** | **Out of scope** | `scryfall_data` and related Scryfall-sync tables use `released_at` as a property of the printing, not "an instant in our system." That's a date, not an instant, and conceptually it's `DATE` (or `TIMESTAMP` if Scryfall ever gives us hour-level data). Leave it. |
| **`user_daily_activity.day`** | Already `DATE`, not changed | The `day` column is a `DATE`. Insert already uses `(NOW() AT TIME ZONE 'UTC')::date` so it's UTC-pinned. No change needed. |
| **Pool pin retention** | Keep it | After the migration the pool pin is no longer load-bearing for correctness, but keeping it makes `NOW()`/`CURRENT_DATE` deterministic in raw queries and ad-hoc psql sessions through the pool. Defense in depth. |

## Scope inventory

### Tables to migrate

From `grep -hn "TIMESTAMP" zerver/migrations/*.sql` as of 2026-06-07:

| Migration file | Table | Columns |
|---|---|---|
| `20250810194439_create_users.sql` | `users` | `created_at`, `updated_at`, `last_failed_at`, `lockout_until`, `email_verified_at` |
| `20250810194451_create_card_profiles.sql` | `card_profiles` | `created_at`, `updated_at` |
| `20250810194454_create_decks.sql` | `decks` | `created_at`, `updated_at` (plus `first_completed_at` from `20260607211058_user_metrics.sql`) |
| `20250810194459_create_deck_cards.sql` | `deck_cards` | `created_at`, `updated_at` |
| `20250824191651_create_scryfall_card_sync_metrics.sql` | `zervice_metrics` | `started_at`, `ended_at` |
| `20251007160206_create_refresh_tokens.sql` | refresh tokens | `created_at`, `expires_at` |
| `20260327000001_create_email_verification_tokens.sql` | email verification tokens | `created_at`, `expires_at` |
| `20260327000002_create_password_reset_tokens.sql` | password reset tokens | `created_at`, `expires_at` |
| `20260329000000_create_user_preferences.sql` | `user_preferences` | `created_at`, `updated_at` |
| `20260607211058_user_metrics.sql` | `user_lifetime_counters` | `updated_at` |
| `20260607211058_user_metrics.sql` | `user_events` | `occurred_at` |
| `20260607211058_user_metrics.sql` | `user_audit_log` | `occurred_at` |
| (skipped) | `user_daily_activity` | `day` stays `DATE` |
| (skipped) | `scryfall_data` | `released_at` is a property of the printing |

24 columns across 12 tables.

### Rust call sites to update

From `grep -rn "NaiveDateTime" zwipe-core/src zerver/src/lib` — 81 hits
across roughly:

| File | Usage |
|---|---|
| `zwipe-core/src/domain/user/models/mod.rs` | `User::email_verified_at: Option<NaiveDateTime>` |
| `zwipe-core/src/domain/auth/models/access_token.rs` | `AccessToken::expires_at: NaiveDateTime` |
| `zwipe-core/src/domain/auth/models/refresh_token.rs` | `RefreshToken::expires_at: NaiveDateTime` |
| `zwipe-core/src/domain/auth/models/session.rs` | Test helper using `chrono::NaiveDateTime` |
| `zwipe-core/src/domain/card/models/card_profile.rs` | `CardProfile::{created_at, updated_at}: NaiveDateTime` |
| `zwipe-core/src/http/contracts/metrics.rs` | `HttpLifetimeCounters::updated_at: NaiveDateTime` |
| `zerver/src/lib/domain/metrics/models/lifetime_counters.rs` | `LifetimeCounters::updated_at: NaiveDateTime` |
| `zerver/src/lib/outbound/sqlx/user/models.rs` | `DatabaseUser::email_verified_at: Option<NaiveDateTime>` |
| `zerver/src/lib/outbound/sqlx/auth/models.rs` | `DatabaseUser::{lockout_until, email_verified_at}`, refresh token `expires_at` |
| `zerver/src/lib/outbound/sqlx/auth/mod.rs` | Token query bindings (`expires_at` params) |
| `zerver/src/lib/outbound/sqlx/card/card_profile.rs` | `DatabaseCardProfile::{created_at, updated_at}` |
| `zerver/src/lib/outbound/sqlx/card/zervice_metrics.rs` | sync run timestamps |
| `zerver/src/lib/outbound/sqlx/card/mod.rs` | `get_last_sync_date: Option<NaiveDateTime>` |

Test fixtures and helper functions that mint timestamps with
`Utc::now().naive_utc()` also need updates (drop the `.naive_utc()`).

### Things that *don't* need touching

- `chrono::Utc` import is already pervasive — `DateTime<Utc>` is one
  fewer keystroke than `NaiveDateTime`.
- `email_verified_at`, `lockout_until`, and other nullable timestamps
  stay nullable — `Option<DateTime<Utc>>` works the same as
  `Option<NaiveDateTime>`.
- The SQLx `.sqlx/` offline cache regenerates on `cargo sqlx prepare`
  — no manual edits.

## File-by-file plan

All paths are relative to the repo root.

### 0. Branch + DB backup

```bash
git checkout -b feat/timestamptz-migration
# On the prod server, take a fresh dump before running anything live:
ssh zerver-host 'pg_dump zerver > /var/backups/zwipe/pre-timestamptz-$(date +%Y%m%d).sql'
```

The backup is non-negotiable. `ALTER COLUMN TYPE` rewrites the table on
disk; if something goes sideways mid-migration, the only clean recovery
is a restore.

### 1. Migrations — one per logical table group

Generate timestamps with `date +%Y%m%d%H%M%S` ahead of time so the
sequence is predictable. Suggested order — touches small tables first
so a failure on a big one doesn't leave the schema half-converted:

#### 1a. Tokens (3 tables — small, safe warmup)

**New file:** `zerver/migrations/<ts>_timestamptz_tokens.sql`

```sql
ALTER TABLE refresh_tokens
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN expires_at TYPE TIMESTAMPTZ USING expires_at AT TIME ZONE 'UTC';

ALTER TABLE email_verification_tokens
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN expires_at TYPE TIMESTAMPTZ USING expires_at AT TIME ZONE 'UTC';

ALTER TABLE password_reset_tokens
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN expires_at TYPE TIMESTAMPTZ USING expires_at AT TIME ZONE 'UTC';
```

The `USING ... AT TIME ZONE 'UTC'` clause says "treat the existing
wall-clock value as already being UTC, then convert to a TIMESTAMPTZ
storing that UTC instant." Safe iff older rows really were UTC. Spot
check:

```sql
SELECT min(created_at), max(created_at) FROM refresh_tokens;
```

If `min` looks plausible (no 2025-06-08T04:40:20 that should actually
be 04:40 local), the cluster has been UTC the whole time.

#### 1b. User-adjacent (users, preferences, lifetime, events, audit, daily activity)

**New file:** `zerver/migrations/<ts>_timestamptz_user.sql`

```sql
ALTER TABLE users
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC',
    ALTER COLUMN last_failed_at TYPE TIMESTAMPTZ USING last_failed_at AT TIME ZONE 'UTC',
    ALTER COLUMN lockout_until TYPE TIMESTAMPTZ USING lockout_until AT TIME ZONE 'UTC',
    ALTER COLUMN email_verified_at TYPE TIMESTAMPTZ USING email_verified_at AT TIME ZONE 'UTC';

ALTER TABLE user_preferences
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';

ALTER TABLE user_lifetime_counters
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';

ALTER TABLE user_events
    ALTER COLUMN occurred_at TYPE TIMESTAMPTZ USING occurred_at AT TIME ZONE 'UTC';

ALTER TABLE user_audit_log
    ALTER COLUMN occurred_at TYPE TIMESTAMPTZ USING occurred_at AT TIME ZONE 'UTC';
```

#### 1c. Decks (decks + deck_cards)

**New file:** `zerver/migrations/<ts>_timestamptz_decks.sql`

```sql
ALTER TABLE decks
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC',
    ALTER COLUMN first_completed_at TYPE TIMESTAMPTZ USING first_completed_at AT TIME ZONE 'UTC';

ALTER TABLE deck_cards
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';
```

#### 1d. Cards + sync metrics

**New file:** `zerver/migrations/<ts>_timestamptz_cards.sql`

```sql
ALTER TABLE card_profiles
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';

ALTER TABLE zervice_metrics
    ALTER COLUMN started_at TYPE TIMESTAMPTZ USING started_at AT TIME ZONE 'UTC',
    ALTER COLUMN ended_at   TYPE TIMESTAMPTZ USING ended_at   AT TIME ZONE 'UTC';
```

**Skipped** in this migration: `scryfall_data.released_at` (and any
related columns) — different semantic, separate decision.

### 2. Rust type changes

Pattern: every `NaiveDateTime` becomes `DateTime<Utc>`. Every
`Utc::now().naive_utc()` becomes `Utc::now()`.

#### Domain types

**Edit each of:**

- `zwipe-core/src/domain/user/models/mod.rs`
- `zwipe-core/src/domain/auth/models/access_token.rs`
- `zwipe-core/src/domain/auth/models/refresh_token.rs`
- `zwipe-core/src/domain/auth/models/session.rs` (test helper)
- `zwipe-core/src/domain/card/models/card_profile.rs`
- `zwipe-core/src/http/contracts/metrics.rs`
- `zerver/src/lib/domain/metrics/models/lifetime_counters.rs`

For each file:

```rust
// before
use chrono::NaiveDateTime;
pub expires_at: NaiveDateTime,

// after
use chrono::{DateTime, Utc};
pub expires_at: DateTime<Utc>,
```

For nullable columns: `Option<NaiveDateTime>` → `Option<DateTime<Utc>>`.

#### Database adapter wrappers

**Edit each of:**

- `zerver/src/lib/outbound/sqlx/user/models.rs`
- `zerver/src/lib/outbound/sqlx/auth/models.rs`
- `zerver/src/lib/outbound/sqlx/auth/mod.rs` (function params + query bindings)
- `zerver/src/lib/outbound/sqlx/card/card_profile.rs`
- `zerver/src/lib/outbound/sqlx/card/zervice_metrics.rs`
- `zerver/src/lib/outbound/sqlx/card/mod.rs` (`get_last_sync_date` return type)
- `zerver/src/lib/outbound/sqlx/metrics/mod.rs` (the `query!` that hydrates `LifetimeCounters`)

The SQLx `query!` macros pick the type from the column — they'll
complain at compile time if the wrapper expects the old type. The
clippy session-time check catches the rest.

#### Test fixtures

**Grep for `.naive_utc()`** and drop it. Anywhere a test mints a
`NaiveDateTime` from `Utc::now()` becomes a plain `Utc::now()` returning
`DateTime<Utc>`.

```bash
grep -rn "naive_utc()" zwipe-core zerver
```

### 3. Wire-format check

`HttpLifetimeCounters` and any other contract type that surfaces a
timestamp on the wire now serializes as `"2026-06-08T04:40:20Z"`
instead of `"2026-06-08T04:40:20"`. The iOS client's
`serde_json` parse of `DateTime<Utc>` accepts both, but **confirm**
by:

1. Read the relevant client deserialization sites:

   ```bash
   grep -rn "HttpLifetimeCounters\|expires_at" zwiper/src/lib
   ```

2. If anything parses these as plain strings or naive dates, fix it.

3. Sanity smoke after backend deploys:

   ```bash
   TOKEN=$(curl -s -X POST localhost:3000/api/auth/login \
     -H 'content-type: application/json' \
     -d '{"identifier":"test","password":"..."}' | jq -r '.access_token.value')
   curl -s localhost:3000/api/user/metrics -H "Authorization: Bearer $TOKEN" | jq .updated_at
   # expect: "2026-06-08T04:40:20Z"
   ```

### 4. SQLx offline cache regen + lint

```bash
cd zerver && DATABASE_URL=... cargo sqlx prepare
cargo build --workspace --release
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --lib
```

The `.sqlx/` cache fully regenerates because every query touching a
migrated column gets a new column type signature.

### 5. Deploy order

1. **Stage**: run the migration set on the staging DB (or a fresh
   restore of the prod backup) and run the test suite against it.
2. **Prod**: take the fresh backup (step 0), then `git push` — the CI
   pipeline runs the migrations as part of deploy
   (`.github/workflows/deploy-zerver.yml` — sqlx migrate run).
3. **Post-deploy verification**:

   ```bash
   psql $DATABASE_URL -c "\\d+ users" | grep -E "created_at|updated_at"
   # expect: "timestamp with time zone"
   ```

   Pick one column on each table to confirm.

### 6. Rollback plan

If a migration fails mid-flight, the transaction it wrapped rolls back
cleanly (`cargo sqlx migrate run` wraps each file in a transaction). If
all migrations succeed but the new code has a runtime bug, the rollback
path is:

1. Restore the pre-migration backup (`pg_restore < pre-timestamptz-YYYYMMDD.sql`).
2. Revert the deploy commit.

That's a full restore, not an incremental rollback — `ALTER COLUMN TYPE`
rewrites the table, so undoing one column requires another rewrite. For
that reason, the backup must be taken minutes before the deploy, not
days.

## Risks + mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| **Table-rewrite lock blocks writes** | low at current scale | `users`, `decks`, `deck_cards` are all small (<100k rows). The lock is `ACCESS EXCLUSIVE` but should complete in well under a second. Run during low-traffic window anyway. |
| **Older rows weren't actually UTC** | low | The home-server cluster has defaulted to UTC since day one. Spot-check `min(created_at)` on each table before running — anything that looks off by hours signals a TZ drift in the past. |
| **JSON wire format breaks the iOS client** | low | `DateTime<Utc>` JSON form is RFC3339 with `Z`; chrono's `serde` impl parses both forms, so the client deserializes either. Verify before deploy via the smoke step. |
| **Refresh tokens silently expire wrong** | low | Token comparisons are `expires_at < now()` — both sides have the same type and same canonical zone after migration. No semantic change. Sanity: refresh a session and confirm. |
| **`scryfall_data.released_at` left out is confusing** | medium | Document the reason in a code comment on the column when next touched, and leave a follow-up todo. The Scryfall sync is unaffected either way. |
| **Pool pin retention forgotten** | low | Leave the `after_connect` hook in place. The migration doc here explicitly says keep it as defense in depth. |

## Verification

```bash
# 1. Migration applies cleanly to a fresh local DB
cd zerver && DATABASE_URL=postgres:///zerver_test cargo sqlx migrate run

# 2. Spot-check column type on a couple of tables
psql postgres:///zerver_test -c "\\d+ users"   | grep "timestamp"
psql postgres:///zerver_test -c "\\d+ decks"   | grep "timestamp"
psql postgres:///zerver_test -c "\\d+ user_events" | grep "timestamp"
# expect: "timestamp with time zone" everywhere

# 3. Workspace builds clean + tests pass
cargo build --workspace --release
cargo test --workspace --lib

# 4. SQLx offline cache regenerated and consistent
cd zerver && cargo sqlx prepare
git diff --stat .sqlx/   # large diff is expected; just confirm it compiles

# 5. End-to-end smoke after deploy
TOKEN=$(curl -s -X POST localhost:3000/api/auth/login \
  -H 'content-type: application/json' \
  -d '{"identifier":"test","password":"..."}' | jq -r '.access_token.value')

curl -s localhost:3000/api/user/metrics -H "Authorization: Bearer $TOKEN" | jq .updated_at
# expect: "2026-06-08T04:40:20Z" (note trailing Z)

# 6. Refresh-token roundtrip — confirms expiry comparisons still work
curl -i -X POST localhost:3000/api/auth/refresh \
  -H 'content-type: application/json' \
  -d "$(jq -n --arg uid "$USER_ID" --arg rt "$REFRESH_TOKEN" \
    '{user_id: $uid, refresh_token: $rt}')"
# expect: 200 with a new session
```

## Out of scope (deferred)

- **`scryfall_data` timestamp columns** — `released_at` is a date
  semantically; revisit when the Scryfall sync gets re-examined.
- **Custom timestamp validation / clamping** — no business rule needs
  this today; if a future feature wants "future-only" or "past-only"
  validation, that's a domain-layer addition, not a schema concern.
- **Display-zone preferences** — the iOS client could let users see
  timestamps in their local TZ rather than UTC. Worthwhile, but it's a
  rendering concern handled in the UI layer and unaffected by this
  migration.
- **Audit log retention / pruning** — separate ops work; the timestamp
  type doesn't change the retention story.
