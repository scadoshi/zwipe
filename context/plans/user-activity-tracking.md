# User activity tracking — sign-ins, sessions, last-active

## Context

Today's metrics surface tracks counters (`user_lifetime_counters`,
`user_daily_activity`), rare engagement events (`user_events` — signup,
deck_created, deck_completed), and credential changes (`user_audit_log` —
username/email/password). The "last active" signal piggybacks on
`user_lifetime_counters.updated_at`, which only moves when a user does
something countable (swipes, searches, deck operations).

**Gaps surfaced 2026-06-09:**

- **Sign-ins aren't tracked** — `authenticate_user` emits no event or audit row.
  Only the first registration (`Signup`) is logged. We have no way to ask
  "when did this user last log in" or "how many sessions did they start
  this week".
- **Session refresh isn't tracked** — `refresh_session` is silent. Lose
  visibility into "is this user still using the app".
- **Logout isn't tracked** — `revoke_sessions` is silent. No record of
  explicit sign-out vs token expiry.
- **`EventKind::FirstSwipe`** — defined in the enum, never emitted.
  Dead code.
- **`updated_at` as last-active** — works but conceptually wrong. "Last
  active" is a property of the user, not the counter row. A passive user
  who opens the app and reads but doesn't interact is invisible (their
  counter row never gets touched).

The fix is two things:

1. Emit the missing auth events (sign-in / refresh / logout) into both
   `user_events` (engagement signal) and `user_audit_log` (security
   signal). Wire up `FirstSwipe` from its already-defined enum.
2. Promote "last active" to a dedicated column on `users` and bump it
   from authenticated-request middleware with an in-memory debounce so
   it stays fresh without hammering the DB.

This is the foundation for a richer activity-logging framework later —
once these signals exist, building dashboards, retention cohorts, or
security alerts is a query-side concern.

---

## Design decisions

### 1. Last-active lives on `users.last_active_at`, not on the counter row

**Choice:** add `users.last_active_at TIMESTAMPTZ NULL` and bump it via
middleware on every authenticated request, with a per-user in-memory
debounce.

| Option | Catches | Cost | Verdict |
|---|---|---|---|
| A. Hook auth handlers only | Sessions starts + refresh | trivial | partial — misses in-session activity |
| B. Client heartbeat (zero-counter flush) | Foregrounded app | 1 client line, ~120 writes/user/hr | wastes writes when user is idle in-app |
| **C. Middleware on every auth'd request** | Everything | middleware + debounce | **chosen — most accurate, debounce keeps cost flat** |

Why a dedicated column:
- "Last active" is a property of the user, not of the metrics counter.
- Survives future refactors of how counters are organized (e.g. a sweep
  to rename `user_lifetime_counters` → `user_lifetime_activity` doesn't
  touch this signal).
- Schema reads obviously — anyone scanning `users` sees what it means.

Why middleware (not handler hooks):
- Catches **every** authenticated request: card search, deck load,
  filter change, etc. Not just session-level events.
- Single place to maintain, can't be forgotten on new endpoints.

Why debounce:
- Naive "UPDATE on every request" = hundreds of writes per active
  session. With a 60-second debounce, at most 1 write per user per
  minute regardless of request volume.
- Implementation: `Arc<DashMap<Uuid, Instant>>` in `AppState`. Read
  the cached `Instant` for this user; if absent or older than 60s,
  fire UPDATE in a `tokio::spawn` and update the cache.
- Lost on restart — acceptable. First request after restart writes,
  fine.

Keep `user_lifetime_counters.updated_at` for now (no behavioral change
needed; we just stop *reading* it as a last-active signal in any future
code). Removing it would touch the SQL and is unnecessary churn.

### 2. Sign-in / refresh / logout emit to both `user_events` and `user_audit_log`

The two tables serve different consumers:

- `user_events`: engagement view. Useful for "how many logins did this
  user do this week", funnels, etc.
- `user_audit_log`: security view. Pairs naturally with the existing
  credential-change rows. Useful for "show me the auth timeline for
  this user".

For each of {sign-in, session refresh, sign-out}, emit one row in each
table on success.

If you ever want IP / user-agent on the audit side later, that's a
separate column add — out of scope here.

### 3. `FirstSwipe` wired via a `users.first_swiped_at` column

Mirror the deck-completion pattern (`decks.first_completed_at`):

- New column `users.first_swiped_at TIMESTAMPTZ NULL`.
- Inside `apply_usage`, when any swipe counter in the batch is > 0,
  do an atomic `UPDATE users SET first_swiped_at = NOW() WHERE id = $1
  AND first_swiped_at IS NULL RETURNING id`. If a row comes back, this
  call was the first-ever swipe — emit `EventKind::FirstSwipe`.
- Idempotent. No SELECT needed before INSERT. No race condition.

Don't try to query `user_events` to check "did this user already have a
FirstSwipe?" — that's slower and racy.

### 4. Enum variants — rename existing, add new

Naming convention: **single verb, present-tense, one word** for all
session-related events. Final shape:

**`EventKind`** in `zerver/src/lib/domain/metrics/models/kinds.rs`:

```rust
pub enum EventKind {
    Register,       // RENAMED from Signup
    Login,          // NEW
    Refresh,        // NEW
    Logout,         // NEW
    DeckCreated,
    DeckCompleted,
    FirstSwipe,     // existing, now actually emitted
}
```

DB string values from `as_str()`: `"register"` (was `"signup"`),
`"login"`, `"refresh"`, `"logout"`, plus the unchanged
`"deck_created"`, `"deck_completed"`, `"first_swipe"`.

**`AuditAction`** in the same file:

```rust
pub enum AuditAction {
    UsernameChanged,
    EmailChanged,
    PasswordChanged,
    Login,          // NEW
    Refresh,        // NEW
    Logout,         // NEW
}
```

DB string values: `"login"`, `"refresh"`, `"logout"` (reuses the same
strings as `EventKind` — different tables, naming consistency reads
better).

#### Renaming `Signup` → `Register` impacts code, docs, and existing data

Audited the codebase — no semantic code uses `signup` anywhere except
the enum variant. Touches:

- `kinds.rs` — enum variant rename + `as_str()` mapping
  `"signup"` → `"register"`
- `zerver/src/lib/inbound/http/handlers/auth/register_user.rs:133` —
  `EventKind::Signup` → `EventKind::Register` (plus the
  `tracing::warn!` message updated for consistency)
- `zerver/src/lib/domain/metrics/mod.rs` module doc comment mentions
  `signup` in the list of rare events — reword to `register`
- Any tests / fixtures that reference `EventKind::Signup`
- `.sqlx/` offline cache regenerates with the new string
- `context/status/progress.md` — reword event-kind list
- `zcripts/metrics/README.md` — reword event-kind list
- `zcripts/metrics/overview.sql` — section header `── 2. SIGNUPS ──`
  and column alias `AS signups` are display labels (the query reads
  from `users.created_at`, not `user_events.kind`), but reword to
  `── 2. REGISTRATIONS ──` / `AS registrations` for consistency

Data migration: existing rows in `user_events` have `kind = 'signup'`.
Add a one-liner inside the same migration that creates
`last_active_at` / `first_swiped_at`:

```sql
UPDATE user_events SET kind = 'register' WHERE kind = 'signup';
```

So every row in the table — historical and future — uses the canonical
`'register'` string. Cleaner than mixed data.

---

## Files touched

### New

- `zerver/migrations/<ts>_user_last_active_first_swiped_and_signup_rename.sql`
  — adds two columns to `users` and renames historical `signup` event
  rows to `register`:
  ```sql
  ALTER TABLE users
      ADD COLUMN last_active_at TIMESTAMPTZ,
      ADD COLUMN first_swiped_at TIMESTAMPTZ;

  UPDATE user_events SET kind = 'register' WHERE kind = 'signup';
  ```
- `zerver/src/lib/inbound/http/middleware/last_active.rs` (or however
  the existing middleware layout prefers it) — the debounce + bump
  layer. Should be applied to whatever `Router` carries `private_routes`.

### Modified

- `zerver/src/lib/domain/metrics/models/kinds.rs` — add 3 enum
  variants to each of `EventKind` and `AuditAction` per Decision 4.
- `zerver/src/lib/domain/metrics/ports.rs` — add port methods:
  - `touch_last_active(user_id) -> Result<(), MetricsError>`
  - `mark_user_first_swiped(user_id) -> Result<bool, MetricsError>`
    (mirrors `mark_deck_first_completed`)
- `zerver/src/lib/domain/metrics/services.rs` — implement the new port
  methods.
- `zerver/src/lib/outbound/sqlx/metrics/mod.rs` — SQL for the new
  port methods.
- `zerver/src/lib/inbound/http/handlers/auth/authenticate_user.rs` —
  on success, fire-and-forget `record_event(Login)` and
  `record_audit(Login)`.
- `zerver/src/lib/inbound/http/handlers/auth/refresh_session.rs` — on
  success, fire-and-forget `record_event(Refresh)` and
  `record_audit(Refresh)`.
- `zerver/src/lib/inbound/http/handlers/auth/revoke_sessions.rs` — on
  success, fire-and-forget `record_event(Logout)` and
  `record_audit(Logout)`.
- `zerver/src/lib/inbound/http/handlers/auth/register_user.rs:133` —
  update existing `EventKind::Signup` to `EventKind::Register` as part
  of the rename.
- `zerver/src/lib/domain/metrics/mod.rs` — module doc mentions `signup`
  in the rare-events list; reword.
- `context/status/progress.md` — event-kind list mentions `signup`;
  reword to `register`.
- `zcripts/metrics/README.md` — event-kind list mentions `signup`;
  reword.
- `zcripts/metrics/overview.sql` — section header / column alias
  rename (cosmetic, no query change since it reads
  `users.created_at`).
- `zerver/src/lib/inbound/http/handlers/metrics/record_usage.rs` (or
  wherever `apply_usage` is called from) — after the batch is applied,
  if any swipe counter was > 0, run `mark_user_first_swiped`; if it
  returned `true`, fire `record_event(FirstSwipe)`.
- `zerver/src/lib/inbound/http/mod.rs` — add the debounce map
  (`DashMap<Uuid, Instant>`) to `AppState`, plumb it through.
- `zerver/src/lib/inbound/http/routes.rs` — apply the
  `LastActiveLayer` to the private routes router (after JWT auth
  layer so `AuthenticatedUser` is available).
- `Cargo.toml` (workspace) — add `dashmap = "6"` if not already in
  deps.

### Stop reading `user_lifetime_counters.updated_at` as last-active (in scope)

The column stays — it still legitimately reflects "when this counter row
was last updated", which is its honest semantic. What changes:

- No code anywhere should query it for the purpose of "when was this
  user last active". `users.last_active_at` is the canonical signal
  going forward.
- Documentation (`context/status/progress.md` and any other place that
  describes the column as doubling as last-active) gets reworded.
- The `From<LifetimeCounters> for HttpLifetimeCounters` impl still
  surfaces `updated_at` as-is. No behavior change.

This is mostly a documentation + intent-clarification pass. No code
behavior change for `updated_at` itself.

### Out of scope (defer)

- Adding IP / user-agent to audit rows. Schema-level change to
  `user_audit_log`. Defer until there's a real product need.
- Dashboard / aggregate queries. Once the data is being captured, the
  query layer is a separate concern.

---

## Implementation order

Suggested for a clean review story:

1. **Schema migration first** (`users.last_active_at`,
   `users.first_swiped_at`). One commit.
2. **Enum additions + port + repo methods.** Compiles, no behavior
   change yet. One commit.
3. **Handler hooks** (sign-in, refresh, logout, first-swipe). Behavior
   change starts flowing into `user_events` / `user_audit_log`. One
   commit.
4. **Middleware** (last-active debounce + bump). Behavior change for
   `users.last_active_at`. One commit.
5. **Verification** (manual + tests). One commit if tests are added.

Each step compiles + tests pass standalone. Avoids a giant PR.

---

## Verification

```bash
# After step 1
cargo sqlx migrate run --source zerver/migrations
psql $DATABASE_URL -c "\d users" | grep -E "last_active_at|first_swiped_at"
# expect: both columns present, both TIMESTAMPTZ, both nullable

# After step 4 — full end-to-end
TOKEN=$(curl -s -X POST localhost:3000/api/auth/login \
  -H 'content-type: application/json' \
  -d '{"identifier":"test","password":"..."}' | jq -r '.access_token.value')

# Login should have emitted a Login event + audit row
psql $DATABASE_URL -c "
  SELECT kind, occurred_at FROM user_events
  WHERE user_id = (SELECT id FROM users WHERE username = 'test')
  ORDER BY occurred_at DESC LIMIT 3;"
# expect: login at top

psql $DATABASE_URL -c "
  SELECT action, occurred_at FROM user_audit_log
  WHERE user_id = (SELECT id FROM users WHERE username = 'test')
  ORDER BY occurred_at DESC LIMIT 3;"
# expect: login at top

# Historical signup rows renamed
psql $DATABASE_URL -c "
  SELECT COUNT(*) FROM user_events WHERE kind = 'signup';"
# expect: 0  (all renamed by the migration)
psql $DATABASE_URL -c "
  SELECT COUNT(*) FROM user_events WHERE kind = 'register';"
# expect: matches your historical signup count + any new registrations

# First swipe wiring
curl -i -X POST localhost:3000/api/metrics/usage \
  -H "Authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' \
  -d '{"swipes_right":1,"swipes_left":0,"swipes_up":0,"swipes_down":0,"searches":0}'
# expect: 204 + first_swiped_at populated + FirstSwipe event row (once per user)

psql $DATABASE_URL -c "
  SELECT username, last_active_at, first_swiped_at
  FROM users WHERE username = 'test';"
# expect: last_active_at recent, first_swiped_at set on first POST after deploy

# Debounce works
for i in 1 2 3 4 5; do
  curl -s localhost:3000/api/user/metrics -H "Authorization: Bearer $TOKEN" > /dev/null
done
# Run 5 requests rapidly. Check users.last_active_at — should NOT have moved 5 times.
# Should have moved at most once in the 60s window.

# Refresh + logout flow
curl -s -X POST localhost:3000/api/auth/refresh \
  -H 'content-type: application/json' \
  -d "{\"user_id\":\"<uid>\",\"refresh_token\":\"...\"}"
# expect: Refresh event + audit row

curl -s -X POST localhost:3000/api/auth/logout \
  -H "Authorization: Bearer $TOKEN"
# expect: Logout event + audit row
```

```bash
# Test suite
cargo test --workspace --lib
# expect: green (incl. new tests if step 5 adds any)

cargo clippy --workspace --all-targets -- -D warnings
# expect: clean
```

---

## Risks / things to watch

- **Fire-and-forget event emission** can fail silently. Already the
  pattern used elsewhere (signup, deck_created, etc.) — log on error
  via `tracing::warn!` and move on.
- **Debounce cache memory** scales O(active_users). For 12 users:
  negligible. At 10k users with 1k active per hour, still kilobytes.
  Not worth bounding for foreseeable scale.
- **Middleware ordering**: the last-active layer must run *after* the
  JWT auth layer so `AuthenticatedUser` is in the request extensions.
  Verify with a quick test (unauthenticated request to a public route
  should not trigger the layer; authenticated request should).
- **`AuthenticatedUser` extractor cost**: middleware will need to
  re-extract or peek at it. If `AuthenticatedUser` already inserts
  itself into request extensions, the middleware reads from there for
  free.

---

## Future framework (out of scope, captured here so we don't forget)

Once these signals are live, the natural next steps as the app grows:

- **Cohort analytics** — `WHERE signup_date BETWEEN X AND Y GROUP BY
  retention_window` against `user_events`.
- **Suspicious-login alerts** — rate of `Login` events from new
  IPs (requires IP capture in `user_audit_log`).
- **Per-screen engagement** — finer-grained events like `ScreenViewed`,
  emitted from client telemetry. Probably needs a separate
  high-volume table with retention policy, not a row per event in
  `user_events`.
- **Pruning** — daily / events / audit tables grow forever. At some
  point (10k+ active users) want a retention policy for older rows.

None of this is needed at 12 users. Capture-now-query-later is the
right move.
