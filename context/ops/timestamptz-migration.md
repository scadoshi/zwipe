# Planned migration: TIMESTAMP → TIMESTAMPTZ

**Status: not started.** Captured 2026-06-07 while wiring user metrics.
The metrics work landed a defensive backstop (UTC pool pin) so the
migration can be scheduled deliberately rather than rushed.

---

## Why

Every timestamp column in `zerver` migrations is `TIMESTAMP` (no time
zone), populated by `NOW()` / `CURRENT_DATE`. That type stores a
**wall-clock value with no zone info attached** — the value is whatever
the writing session's `TIMEZONE` setting resolves to at the moment of
the write.

Today this works because:

- Postgres on the home server defaults to UTC at the cluster level
- The SQLx pool now runs `SET TIME ZONE 'UTC'` on every connection
  (`zerver/src/lib/outbound/sqlx/postgres.rs` — `after_connect` hook)

So *every write* hits the DB while the session is UTC. But that's a
backstop, not a contract. A future maintainer who forgets the pool hook,
a one-off psql session, a migration run by a runner with a different TZ
— any of those silently stores wall-clock values in the wrong zone, and
nothing in the type system catches it. The symptom that surfaced this:
`CURRENT_DATE` on a local-TZ psql session returned a different day from
the server's UTC-TZ connection, so `user_daily_activity` queries
appeared to lose rows.

`TIMESTAMPTZ` (a.k.a. `timestamp with time zone`) fixes this by storing
**an instant in time, canonicalized to UTC at write, presented in the
reader's session TZ at read**. The semantic — "this is the moment X
happened" — is what we actually want for every `created_at`,
`updated_at`, `expires_at`, etc.

## Scope (tables to migrate)

From `grep -hn "TIMESTAMP" zerver/migrations/*.sql` as of 2026-06-07:

- `users` — `created_at`, `updated_at`, `last_failed_at`, `lockout_until`, `email_verified_at`
- `decks` — `created_at`, `updated_at`, `first_completed_at`
- `deck_cards` — `created_at`, `updated_at`
- `user_preferences` — `created_at`, `updated_at`
- `user_lifetime_counters` — `updated_at`
- `user_events` — `occurred_at`
- `user_audit_log` — `occurred_at`
- `sessions` (refresh tokens) — `created_at`, `expires_at`
- `email_verification_tokens` — `created_at`, `expires_at`
- `password_reset_tokens` — `created_at`, `expires_at`
- `zervice_metrics` — `started_at`, `ended_at`
- `user_daily_activity` — `day` is `DATE`, not timestamp — leave as-is
  but switch the upsert from `CURRENT_DATE` to
  `(NOW() AT TIME ZONE 'UTC')::date` (already done in
  `zerver/src/lib/outbound/sqlx/metrics/mod.rs`).

`scryfall_data` and related Scryfall tables likely use TIMESTAMP for
`released_at` — those come from Scryfall's data dump, not server-side
writes, so the semantics there are different (the date is a property of
the printing, not "an instant in our system"). Worth a separate look
before migrating those.

## Code changes that follow the schema change

- Domain types using `chrono::NaiveDateTime` → `chrono::DateTime<chrono::Utc>`
  (or keep `NaiveDateTime` and read TZ-naive on the rust side — sqlx
  supports both mappings for `TIMESTAMPTZ`).
  - Mainly: `zwipe-core/src/domain/user/models/mod.rs`,
    `auth/models/session.rs`, `auth/models/refresh_token.rs`,
    `card/models/scryfall_data/mod.rs`,
    `zerver/src/lib/domain/metrics/models/lifetime_counters.rs`,
    `zwipe-core/src/http/contracts/metrics.rs` (HttpLifetimeCounters).
- HTTP contract serialization: `DateTime<Utc>` serializes with a `Z`
  suffix in JSON; `NaiveDateTime` does not. Pick one and be consistent —
  the client should parse both, but the wire format should be stable.
- `.sqlx/` offline cache regenerates entirely (column type changed on
  every query touching these columns). `cargo sqlx prepare` after.

## Migration shape

One migration per logical group (don't bundle all 13 tables into one
huge `ALTER` — failure isolation matters). For each:

```sql
ALTER TABLE users
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC',
    ALTER COLUMN last_failed_at TYPE TIMESTAMPTZ USING last_failed_at AT TIME ZONE 'UTC',
    ALTER COLUMN lockout_until TYPE TIMESTAMPTZ USING lockout_until AT TIME ZONE 'UTC',
    ALTER COLUMN email_verified_at TYPE TIMESTAMPTZ USING email_verified_at AT TIME ZONE 'UTC';
```

The `USING ... AT TIME ZONE 'UTC'` clause says "treat the existing
wall-clock value as already being UTC, then convert." Safe iff the
backstop has been in place — values written before that point may or
may not actually be UTC depending on cluster TZ at the time. Spot-check
oldest rows on prod before running.

## Risks

- **Table-rewrite locks**: `ALTER COLUMN TYPE` on a large table can
  rewrite the whole table and take a long `ACCESS EXCLUSIVE` lock.
  `users`, `decks`, `deck_cards` are small (<1M rows) so this is
  probably under a second each, but worth running on a backup first.
  `scryfall_data` is large and shouldn't be touched in this round.
- **Library defaults change**: any code path that relied on
  `NaiveDateTime` semantics ("just a wall clock") needs review. Most of
  the codebase treats timestamps as instants already, so this should be
  a wash, but grep for naive comparisons before merging.
- **External integrations**: anything reading the DB directly outside
  the Rust code (manual psql, future BI tools) sees `TIMESTAMPTZ` in
  their session TZ. That's a feature, not a bug — just call it out.

## When to do this

After the metrics PR lands and burns in for a week. Not coupled to any
feature work. Best scheduled as a deliberate maintenance window with a
fresh DB backup just before the migration runs.
