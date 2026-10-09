# Access token window

**Status: IN PROGRESS.** This block is the running log; update it as each item lands.

- [x] 2026-10-09 Steps 1, 2 and 3 merged to main in one push. Client buffer, `ACCESS_TOKEN_MINUTES` (default 1440) and refresh families with the 30-day cap.
- [x] 2026-10-09 zerver deploy with the families migration green (run 37987257595) (`20261009130000_refresh_token_families.sql`; additive, keeps `revoked`).
- [ ] Re-run `zcripts/server/sql/zervice_role.sql` on the server so the nightly cleanup can read `replaced_at` (the migration grants it when the role exists, so this is belt and braces).
- [ ] Client 1.11.2 cut and submitted, carrying the refresh-ahead buffer. Cut 2026-10-09 without waiting on 1.11.1's review; iOS build 92 and Android vc55 built and signed, both awaiting upload.
- [ ] `MIN_CLIENT_VERSION` raised to 1.11.2 once the floor query shows old builds gone (or earlier to force them).
- [ ] `ACCESS_TOKEN_MINUTES=15` set in zerver's env and the service restarted. Owner has to do this by hand; until then tokens still live 24 hours.
- [ ] Follow-up migration dropping `revoked` and `idx_refresh_tokens_revoked`, one deploy after the families migration is live.
- [ ] Check that password and email changes require the current password (out of scope here, but noted as the reason there is no 5-minute tier).
- [ ] Android: `session.json` is plaintext (no keyring backend). Own task.

**One sentence:** shrink how long a stolen or logged-out bearer token keeps working, from 24 hours to 15 minutes, without adding a per-request blacklist.

## Where it stands today

- The access JWT lives 24 hours, hardcoded in `zerver/src/lib/domain/auth/models/access_token.rs` (`AccessTokenExt::generate`). The `AuthenticatedUser` extractor checks signature and `exp` only, no database lookup, so logout (`DELETE FROM refresh_tokens WHERE user_id`) stops new tokens but an issued one keeps working until it expires.
- The refresh token lives 14 days, sliding: every rotation mints a fresh 14 days and nothing records the original login, so an active device's session never ends.
- Rotation is strictly single use (`use_refresh_token`, `SELECT ... FOR UPDATE` then delete then insert). A replayed token hits `RowNotFound` and gets a 401, and that is all: there is no family tracking, so if an attacker rotates first the real device is logged out and the attacker's chain stays valid.
- The `revoked` column is checked but nothing sets it. Revocation is hard delete.
- zwiper refreshes only once `expires_at` has passed (`ensure_session.rs`, `access_token.is_expired()` with no buffer) and never retries an authed call on 401. At 24 hours a token expiring mid-request is rare; at 15 minutes it would be about 96 times as common.
- zite holds no session. zwipe-client has no refresh logic of its own.
- The server never enforces `MIN_CLIENT_VERSION` on requests; the floor is the client's own "Update required" screen.

Three things stay out of scope here: single-device logout (logout is all devices and that is fine for now), a 5-minute tier for credential changes (check instead that password and email changes require the current password), and Android session storage (`session.json` is plaintext there; own task).

## Step 1, client: refresh ahead of expiry (branch `access-window-client`)

In `ensure_fresh`, treat the access token as stale when it is within `REFRESH_AHEAD` (2 minutes) of `expires_at`, not only once it has passed. The server's `jsonwebtoken` leeway is 60 seconds, so a token refreshed 2 minutes early never expires in flight unless a request takes longer than that, and a phone clock up to about a minute slow is covered too. The 60-second upkeep tick (`session_upkeep.rs`) already calls `ensure_fresh`, so a foregrounded app refreshes in the background before any screen call needs to.

Keep the single-flight lock and the cancellation-proof commit exactly as they are. The only change is the staleness test, in one place, with a unit test on the boundary.

Not doing: a transparent retry on 401 inside `Authed::execute`. The facade takes `FnOnce` closures from every screen, so a retry means every call site changes. If the buffer turns out not to be enough, that is the follow-up.

Ships in the next zwiper build. The floor is not strictly required before step 2 flips, old clients just see an occasional error toast at the expiry edge, but raise it once the count allows.

## Step 2, server: lifetime from config (branch `access-window`)

`ACCESS_TOKEN_MINUTES` in `zerver/src/lib/config.rs`, default 1440 so the deploy changes nothing. `AccessTokenExt::generate` takes the lifetime instead of hardcoding 24 hours. Malformed or zero refuses startup, like `MIN_CLIENT_VERSION`.

Flip to 15 in prod once step 1 is on most devices. No code change at flip time.

## Step 3, server: refresh families and an absolute cap (branch `access-window`)

Migration on `refresh_tokens`:

- `family_id UUID NOT NULL`, the same value carried through every rotation; a login starts a new family.
- `login_at TIMESTAMPTZ NOT NULL`, carried through rotation.
- `replaced_at TIMESTAMPTZ NULL`: a rotated row is kept with `replaced_at` set instead of deleted, so a replay is recognisable. The nightly cleanup deletes replaced rows older than the grace window and expired rows as now.
- `revoked` and its index stay for one more deploy: the deploy runs migrations minutes before the new binary starts, and the old binary still selects the column. Drop both in a follow-up migration once this is live; nothing sets it.

Backfill: existing rows get `family_id = gen_random_uuid()` each and `login_at = created_at`. Both new columns carry defaults (`gen_random_uuid()`, `NOW()`) so the old binary's inserts keep working during that same window.

In `use_refresh_token`:

- a row with `replaced_at` set is a replay: delete the whole family (`DELETE ... WHERE family_id`) and return 401. The attacker and the real device both lose, and the real device logs in again.
- `login_at` older than `REFRESH_ABSOLUTE_LIFESPAN` (30 days) is `Expired`: delete the family, 401.
- otherwise mark the row replaced and insert the new one in the same transaction, carrying `family_id` and `login_at`.

`prune_users_refresh_tokens` keeps counting live rows only (`replaced_at IS NULL`) against `MAXIMUM_SESSION_COUNT`.

Tests: replay after rotation kills the family and the fresh token from that rotation stops working; a 31-day-old family cannot rotate; four concurrent refreshes still give one 200 and three 401s and the one survivor is still valid. Update `cargo sqlx prepare --workspace` and commit `.sqlx/`.

## Docs to touch when it lands

`context/CLAUDE.md` (session auth line), `architecture/structure.md`, `operations/infrastructure/services.md` for the new env var, and the changelog entry: "Logging out stops a signed-in device within fifteen minutes".
