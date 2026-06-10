# Refresh-token hardening — strict single-use rotation (phase 2)

**Status: branch ready — `feat/refresh-token-hardening` on origin (commit `8f6d2ea0`),
implemented + validated locally 2026-06-09 (4 concurrent refreshes → one 200 / three
401s; replay → 401; exactly 1 live token). Gated on iOS Build 29 (1.0.4) propagation.
Do NOT merge before the gate clears (see below). Expected unlock ~2026-06-23.**

## Context

The 2026-06-09 activity tracking surfaced a client bug: iOS builds ≤25
fire up to 4 concurrent `POST /api/auth/refresh` requests with the same
refresh token on cold start (no single-flight guard). All four *succeed*
because the server's rotation is loose: `use_refresh_token` SELECTs the
token without a row lock and never checks that its DELETE deleted anything,
so concurrent transactions all read the pre-delete state and each mints a
fresh token. One token in, four tokens out.

The client side was fixed on `feat/single-flight-refresh` (ships as 1.0.4 build 29 — builds 26-28 were superseded pre-release):
an awaited, single-flight `ensure_fresh` guard replaced the sprinkled
`upkeep` calls, refresh results are persisted to the keyring, and transient
errors no longer log the user out. See `context/status/progress.md` and the
session notes for that branch.

This plan is the server half: make refresh tokens strictly single-use, so a
replayed (stolen, leaked, or stale) token gets a 401 instead of a fresh
session. It is intentionally a separate, tiny PR.

## The gate (why this must wait)

Builds ≤25 still quad-fire refreshes. Against a strict server, one of the
four wins and three get 401 — and the old client's refresh-error arm does
`session.set(None)` *on any error*, so whichever response lands last decides
whether the user stays logged in. Old-build users would get randomly logged
out on app open.

**Clear the gate before merging:**
1. 1.0.4 (build 29) approved and live on the App Store.
2. ASC Analytics → App Versions shows active installs are ~100% on
   Build ≥26 (realistically 1–2 weeks after release at current user count;
   ping known users to accelerate).
3. Optional confirmation from prod data — clustered refreshes disappear:
   ```sql
   SELECT user_id, date_trunc('second', occurred_at) AS s, count(*)
   FROM user_events WHERE kind = 'refresh'
     AND occurred_at > now() - interval '7 days'
   GROUP BY 1, 2 HAVING count(*) > 1;
   -- expect: zero rows for a week straight
   ```

## Change

One file: `zerver/src/lib/outbound/sqlx/auth/mod.rs`, fn `use_refresh_token`
(~line 214). Two edits:

1. **Row-lock the SELECT** so concurrent refresh transactions serialize:
   ```sql
   SELECT id, user_id, expires_at, revoked
   FROM refresh_tokens WHERE value_hash = $1
   FOR UPDATE
   ```
   Loser transactions block until the winner commits its DELETE, then their
   re-read hits `RowNotFound` → existing mapping to
   `RefreshSessionError::NotFound` → 401. No new error variants needed.

2. **Check the DELETE did work** (belt-and-suspenders):
   ```rust
   let deleted = query!("DELETE FROM refresh_tokens WHERE id = $1", existing.id)
       .execute(&mut *tx)
       .await?;
   if deleted.rows_affected() != 1 {
       return Err(RefreshSessionError::Revoked(request.user_id));
   }
   ```

Then `cargo sqlx prepare --workspace` (the SELECT text changed — new offline
cache entry) and commit `.sqlx/`.

No schema migration. No client change — 1.0.4's `ensure_fresh` is already
correct under a strict server (single-flight, cancellation-proof commit,
persist-before-set).

## Verification

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --lib

# Concurrency test against local zerver — fire N parallel refreshes with the
# same token; exactly one should succeed:
TOKEN_JSON=$(curl -s -X POST localhost:3000/api/auth/login \
  -H 'content-type: application/json' \
  -d '{"identifier":"<user>","password":"<pw>"}')
UID=$(echo "$TOKEN_JSON" | jq -r '.user.id')
RT=$(echo "$TOKEN_JSON" | jq -r '.refresh_token.value')
for i in 1 2 3 4; do
  curl -s -o /dev/null -w "%{http_code}\n" -X POST localhost:3000/api/auth/refresh \
    -H 'content-type: application/json' \
    -d "{\"user_id\":\"$UID\",\"refresh_token\":\"$RT\"}" &
done; wait
# expect: one 200, three 401

# Replay test — refresh once, then replay the consumed token: expect 401.
```

Post-deploy: exercise the live app (1.0.4) — login, use, force-quit,
relaunch. No logouts expected.

## Rollback

`git revert` the merge; CI redeploys the loose-rotation server. No data
cleanup needed — the change is behavioral only.

## Out of scope

- Reuse-detection ("token family" revocation — revoke all of a user's
  sessions when a consumed token is replayed). Standard hardening, but
  overkill at current scale; reconsider with real traffic.
- Client changes of any kind.
