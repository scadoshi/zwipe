# TIMESTAMPTZ migration + wire-format cutover

**Status: phase 1 shipped 2026-06-08. Phase 2 awaiting iOS Build 25 propagation.**

This is a two-phase rollout. Phase 1 (today) moved the schema to `TIMESTAMPTZ`,
moved Rust types to `DateTime<Utc>`, and shipped a mobile build that accepts
both legacy no-Z and new Z-suffixed wire formats. Phase 2 (whenever Build 25
has reached most/all users) flips the server to emit the canonical Z form and
drops the compatibility adapter entirely.

---

## Why we did this

The pre-2026-06-08 schema used plain `TIMESTAMP` (no time zone) everywhere. PG
treats that as a "wall-clock value with no zone info attached" — the value
stored is literally whatever the writing session's `TIMEZONE` resolves to. The
system was *correct by accident*: the home-server cluster defaults to UTC and
the SQLx pool runs `SET TIME ZONE 'UTC'` on every connect. The type system
didn't enforce it.

`TIMESTAMPTZ` ("timestamp with time zone") stores **an instant in time,
canonicalized to UTC internally**. The semantic — "this is the moment X
happened" — is what we actually want for every `created_at`, `updated_at`,
`expires_at`, `occurred_at`.

The benefits over "TIMESTAMP + pool pin" are explicitness + defense-in-depth:
the type itself says "this is a UTC instant" and PG converts on read using
the session TZ regardless of any future pool refactor. The wire format also
gains the RFC3339 `Z` suffix that's unambiguous for any non-Rust consumer.

---

## What shipped 2026-06-08 (phase 1)

All landed on `main` via PR #5 (`feat/timestamptz-migration` → `main`,
merge commit `97b870cf`).

### Schema migrations

Four sequential migrations under `zerver/migrations/`:

- `20260608120000_timestamptz_tokens.sql` — `refresh_tokens`,
  `email_verification_tokens`, `password_reset_tokens` (warmup, small tables)
- `20260608120100_timestamptz_user.sql` — `users`, `user_preferences`,
  `user_lifetime_counters`, `user_events`, `user_audit_log`
- `20260608120200_timestamptz_decks.sql` — `decks`, `deck_cards`
- `20260608120300_timestamptz_cards.sql` — `card_profiles`, `zervice_metrics`

Each `ALTER COLUMN ... TYPE TIMESTAMPTZ USING <col> AT TIME ZONE 'UTC'`.
The `AT TIME ZONE 'UTC'` clause re-tags the existing wall-clock value as
already-UTC. Safe because the cluster and pool have always been UTC.

**Intentionally skipped:**
- `scryfall_data.released_at` (a date property of the printing, not "an
  instant our system recorded")
- `user_daily_activity.day` (already a `DATE` — calendar day, not instant; the
  upsert writes `(NOW() AT TIME ZONE 'UTC')::date`)

### Rust type changes

Every `NaiveDateTime` field that maps to a migrated column flipped to
`DateTime<Utc>` across:

- `zwipe-core/src/domain/auth/models/{access_token,refresh_token,session}.rs`
- `zwipe-core/src/domain/card/models/card_profile.rs`
- `zwipe-core/src/domain/user/models/mod.rs`
- `zwipe-core/src/http/contracts/metrics.rs`
- `zerver/src/lib/domain/auth/{ports,models,services,requests/refresh_session}.rs`
- `zerver/src/lib/domain/card/{ports,services,models/zervice_metrics}.rs`
- `zerver/src/lib/domain/metrics/models/lifetime_counters.rs`
- `zerver/src/lib/outbound/sqlx/{auth/{mod,models},user/models,card/{mod,card_profile,zervice_metrics},deck/mod,metrics/mod}.rs`
- Test fixtures in `zwipe-core/src/test_utils.rs` and
  `zwipe-core/src/domain/card/models/search_card/{filter_cards,group_cards}.rs`

`.sqlx/` offline cache regenerated. All 416 workspace tests pass.

### Wire-format compatibility shim

New file: `zwipe-core/src/wire_time.rs`. Two `#[serde(with = ...)]` adapters:
- `wire_time::utc` for `DateTime<Utc>`
- `wire_time::utc_opt` for `Option<DateTime<Utc>>`

Behavior in phase 1:
- **Serialize**: emit the legacy no-Z form via `dt.naive_utc().serialize(s)`
  — byte-identical to pre-migration servers. iOS Build 24 and earlier
  (parsing as `NaiveDateTime`) keep working.
- **Deserialize**: lenient — accept both no-Z and `Z`/RFC3339 forms. Means
  any client compiled against this `zwipe-core` (i.e. Build 25+) will read
  whatever format the server eventually emits.

Applied to the 6 wire-facing fields:
- `RefreshToken.expires_at`
- `AccessToken.expires_at`
- `User.email_verified_at` (also `#[serde(default)]`)
- `CardProfile.created_at`
- `CardProfile.updated_at`
- `HttpLifetimeCounters.updated_at`

Validated against the live iOS simulator on 2026-06-08 — profile create,
deck create, card add round-tripped with zero parse failures.

### iOS Build 25 (1.0.3) packaged

`CFBundleShortVersionString=1.0.3`, `CFBundleVersion=25`. Submitted to ASC.
This build has the lenient `wire_time` adapter baked in — accepts both
formats forever, regardless of what the server eventually emits.

### Other ride-alongs

- **EnvFilter for zwiper logging** — `zwiper/src/lib/config.rs` flipped from
  `Level::from_str` (single word) to `EnvFilter` directive syntax, matching
  zerver. Was a paper cut that hid in CI builds since they used simple level
  values.
- **Workspace version bump** — `0.1.0` → `1.0.3` in `Cargo.toml` so
  `env!("CARGO_PKG_VERSION")` in zerver's startup log shows the actual
  shipped version.
- **Deploy workflow path filter** — `.github/workflows/deploy-zerver.yml`
  now also watches `Cargo.toml`, `Cargo.lock`, and `.sqlx/**`. Without
  this, the version bump above wouldn't have triggered a redeploy.
- **dev-env setup scripts** — `zcripts/dev-env/{macos,fedora,omarchy}/setup.sh`
  now write the complete `zerver/.env` (added `LOG_DIR`, `RESEND_API_KEY`,
  `RESEND_EMAIL_FROM` — three vars that were missing and would panic
  startup on a fresh box).

---

## Where things stand right now (post-2026-06-08)

| Layer | State |
|---|---|
| Production DB | Schema fully on `TIMESTAMPTZ` |
| Production server (zerver v1.0.3) | Emits no-Z (via `wire_time::utc::serialize`); accepts both on read |
| iOS Build 24 and earlier (still in users' phones) | Parses no-Z only; would break if server emits `Z` |
| iOS Build 25 (in ASC review) | Parses both forms; future-proof against the upcoming server flip |
| `feat/wire-format-rfc3339` branch on origin (`419c4212`) | Phase 2 cleanup ready — deletes `wire_time` module + uses chrono defaults |

---

## Phase 2: wire-format cutover (next step)

**Goal**: server emits RFC3339 with `Z` (the standards-conformant form),
delete the now-redundant `wire_time` adapter from the shared crate.

**The branch is already prepared:** `feat/wire-format-rfc3339` on origin
contains a single commit (`419c4212`) that does the full cleanup:

- Deletes `zwipe-core/src/wire_time.rs` entirely
- Removes `pub mod wire_time;` from `lib.rs`
- Strips the 6 `#[serde(with = "crate::wire_time::*")]` annotations
- Lets chrono's default serde behavior handle everything
  (`DateTime<Utc>` serialize → `"...Z"`, deserialize → strict RFC3339)
- Carries the same `Cargo.toml` workspace version 1.0.3 so no merge
  conflict with main

Net change: −131 lines, +5 lines.

### Why phase 2 is safe (the core argument)

The wire_time-annotated fields are all **server → client only**:
- Token expiries — server stamps and ships in `Session`
- `email_verified_at` — server-controlled, returned in `User` responses
- `CardProfile.created_at`/`updated_at` — server-controlled
- `HttpLifetimeCounters.updated_at` — server-controlled

**No client → server payload contains any of these fields**. So making the
server's *deserialize* strict (Z-only via chrono default) never receives a
no-Z input from any client, because clients never send them at all.

The risk is purely on the client *read* path:
- iOS Build 25+ has lenient parsing baked in → handles Z fine
- iOS Build 24 and older has NaiveDateTime strict parsing → **breaks on Z**

So the gating question for merging phase 2 is: **has Build 25 propagated to
most/all users?**

With ~12 users known, the realistic timeline:
1. ASC approves Build 25 (24–48h from submit)
2. App Store auto-update pushes Build 25 (~24h after approval for most installs)
3. Ping known users to update if you want to accelerate

After ~3–5 days from submit, you should be at 100% on Build 25+.

### How to execute phase 2

```bash
# 1. Confirm Build 25 is approved and live in App Store Connect
#    (TestFlight → Builds or App Store → Distribution)

# 2. Confirm install base via ASC Analytics → App Versions
#    Want to see 100% (or close) of active installs on 1.0.3 build 25+

# 3. Open the PR if it doesn't already exist
gh pr create --base main --head feat/wire-format-rfc3339 \
  --title "wire-format: drop adapter, emit RFC3339 with Z" \
  --body "Phase 2 cutover. Pre-Build-25 installs break if any remain."

# 4. Merge via GitHub UI (squash or rebase your call)
#    CI auto-deploys zerver — workflow watches Cargo.toml + Cargo.lock now,
#    so the version-aligned merge commit triggers a rebuild and redeploy.

# 5. Verify server emits Z after deploy
curl -s https://api.zwipe.net/api/auth/login \
  -X POST -H 'content-type: application/json' \
  -d '{"identifier":"test","password":"..."}' | jq '.access_token.expires_at'
# Expect: "2026-XX-XXTHH:MM:SS.NNNNNNNNNZ"  (trailing Z)
```

### Recovery if something goes sideways

- **Pre-Build-25 user reports broken app** → tell them to update from App
  Store. Recovery is one tap. No data loss.
- **More users than expected still on Build 24** → revert the merge.
  `git revert <merge-sha>` on main, push, CI redeploys the pre-flip server.
  Server is back to emitting no-Z; all old + new clients work again. Sit
  on it for another week.
- **Catastrophe** (highly unlikely) → restore from the pre-2026-06-08
  database backup. Schema-wise this is reversible only by another set of
  `ALTER COLUMN TYPE TIMESTAMP USING ...` migrations; restoring from
  backup is faster.

---

## Phase 3 (not planned, listed for completeness)

If you ever want to tighten the iOS client's parsing too (drop the lenient
both-formats acceptance), that would require a Build 26+ that compiles
against the post-phase-2 `zwipe-core`. The shared-crate model gives that
to you automatically the moment phase 2 merges — no separate phase 3
branch is needed. The cleanup is already part of phase 2's diff.

Practically: as soon as you ship any iOS build after merging phase 2, that
build will inherit chrono's strict default deserialize. By then the
server already emits Z, so strict is fine.

So phase 3 isn't really a separate step — it just happens whenever the
next mobile build ships after phase 2 lands.
