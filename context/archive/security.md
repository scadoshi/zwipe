# Security Review

Audit performed 2026-03-26 ahead of App Store public launch.

---

## Must Fix Before Public Launch — ALL COMPLETE ✓

### ✓ Rate Limiting on Auth Endpoints (Critical)
- **Done**: `tower_governor` per-route on `/login`, `/register`, `/refresh` + blanket on all private routes. Commit: `588f4fa`.
- **⚠️ Latent regression found + fixed 2026-06-19**: the original IP-keying used `PeerIpKeyExtractor`, which reads the TCP peer address. Behind the Cloudflare Tunnel that is **always `127.0.0.1`** (cloudflared proxies from localhost), so every client on earth shared **one global bucket** — a single client could trip the limit and lock out the entire user base, and per-attacker brute-force throttling never worked at all. Confirmed live: a bucket drained from one IP blocked a second real IP (a phone on cellular), which recovered the instant the drain stopped. Fixed with `CfConnectingIpKeyExtractor`, keying on the `CF-Connecting-IP` header (unspoofable — the origin is unreachable from the public internet: ufw default-deny inbound, only loopback + `tailscale0`), with a peer-IP fallback for non-CF paths. Commit: `c3fc98f2`. Private routes were unaffected (keyed per-user via `UserIdKeyExtractor`).

### ✓ CORS Missing Authorization Header (High)
- **Done**: `header::AUTHORIZATION` added to `.allow_headers([...])`. Commit: `da67302`.

### ✓ Account Lockout (High)
- **Done**: 3 columns on `users` (`failed_login_attempts`, `last_failed_at`, `lockout_until`). Atomic sliding-window UPDATE. Locks after 5 failures in 30 min, resets on success. Returns 429. Commit: `dfd7ee6`.

### ✓ HTTP Security Headers (Critical)
- **Done**: `X-Content-Type-Options`, `X-Frame-Options`, `Strict-Transport-Security`, `Referrer-Policy` added via `SetResponseHeaderLayer`. Commit: `da67302`.

### ✓ JWT Secret Length Not Validated (Medium)
- **Done**: Startup panics if `JWT_SECRET < 32 chars`. Commit: `da67302`.

### ✓ `ApiError::Network` Leaks Internal Messages (Medium)
- **Done**: `Network` variant now logs internally and returns generic `"internal server error"`. Commit: `da67302`.

---

## Fix Soon After Launch

### ✓ Security Event Audit Logging (Medium)
- **Done**: Structured `event =` audit logs at all key auth points — `login_success`, `login_failure` (reason: `invalid_password`, `account_locked`, or `user_not_found`), `register`, `token_refresh_failure` (reason: `not_found`/`expired`/`revoked`/`forbidden`), `token_cleanup`. Written to rolling daily file at `/var/log/zwipe/` alongside stdout. Commit: `440f972`.

### ✓ Expired Token Cleanup (Low)
- **Done**: `zervice` has `delete_expired_sessions`, nightly cron confirmed on Pi (`0 4 * * *`). `token_cleanup` event now logs `rows_deleted` on each run. Commit: `440f972`.

### ✓ Refresh Token Format Validation (Low)
- **Done**: Length corrected to 64 chars (was 32), `is_ascii_hexdigit()` check added, whitespace trimmed before storage. Commit: `e02ab5e`.

---

## Launch Audit (2026-03-26) Items — Complete ✓

> A later re-audit (2026-06-19) found additional issues, incl. a latent
> regression in the rate limiting above. See **Post-launch hardening pass** at
> the bottom of this file.

---

## Already Correct

- Passwords are hashed (not stored plaintext)
- Refresh tokens are SHA-256 hashed before storage
- Max 5 sessions per user enforced
- Generic "invalid credentials" on login failure — no username enumeration
- Sessions hard-deleted on revocation (not just flagged)
- HTTPS handled by Cloudflare Tunnel — zerver never terminates TLS directly
- Refresh tokens rotated on use (old token deleted, new issued)

---

## ✓ Email Verification + Password Reset (2026-03-27)

Implemented via Resend (resend.com — 3k emails/month free tier). Domain: `noreply@zwipe.net`.

- Email verification sent on register and on email change
- `email_verified_at` on `User` model — null until confirmed
- `POST /api/auth/verify-email` — validates token, marks verified
- `POST /api/auth/resend-verification` — authenticated, resends if unverified
- `POST /api/auth/forgot-password` — anti-enumeration: always 200 OK; 5-min per-email cooldown
- `POST /api/auth/reset-password` — validates token, atomically updates password + revokes all sessions + clears lockout
- `change_password_and_revoke_sessions` — also revokes all sessions atomically
- `change_email` — clears `email_verified_at`, fires new verification email
- Tokens: 32 random bytes → hex (sent to client) → SHA-256 hash (stored in DB). DB breach doesn't expose usable tokens.

---

## 2026-06-19 — Post-launch hardening pass

A full server re-audit surfaced several issues; fixes shipped the same day. All
pass `cargo test` + `clippy -D warnings` (zwipe-core/zerver); the rate-limit fix
was additionally verified live against prod.

### ✓ Rate-limit shared-bucket regression (Critical)
The original `PeerIpKeyExtractor` collapsed all clients into one global bucket
behind the CF tunnel — see the corrected **Rate Limiting on Auth Endpoints**
entry above. Fixed via `CfConnectingIpKeyExtractor`. Commit `c3fc98f2`.

### ✓ Login timing / username enumeration (Low–Medium)
A non-existent identifier returned before any Argon2 work, while a real account
with a wrong password paid the full verify — so response timing leaked whether an
account existed, despite the generic 401. Fixed: the not-found path now runs a
verify against a lazily-generated dummy hash (same Argon2 default params) before
returning the same `UserNotFound`. Adds the `user_not_found` audit reason. Residual
sub-ms DB-write delta left as-is (network-noisy). Commit `e7fe4999`.

### ✓ Unbounded card-search `limit` + OFFSET cast (High)
`CardFilter.limit`/`offset` came from untrusted JSON uncapped: `limit:1000000`
forced the DB to materialize a huge result set, and an offset above `i32::MAX`
wrapped negative (Postgres rejects negative OFFSET → 500). Clamped at the SQL bind
(`MAX_SEARCH_LIMIT=250` + guarded offset cast; covers card search and deck-aware
search). No newtype — `CardFilter` is dual-use (also client-side in-memory
filtering with large limits). Commit `fe5324ac`. Proper de-dup planned:
`context/plans/card_filter_split.md`.

### ✓ Metrics counter inflation + overflow (Medium)
`HttpUsageBatch` u32 fields were unbounded: a huge value inflated the lifetime +
public-marketing totals, and the daily path's `as i32` cast could wrap negative /
overflow the `INTEGER` accumulation (Postgres 22003 aborts the write). Clamped to
`MAX_PER_FLUSH=10_000` before any counter write. Commit `9d59f9df`.

### ✓ Health endpoints ungated (Low)
`/health*` had no rate limiter and `/health/database` pings Postgres. Added a
per-IP limiter (30/2s). Commit `8767807e`.

### ✓ Sensitive-op re-auth coupled to login lockout (Medium)
`change_password` / `change_email` / `change_username` / `delete_user`
re-verified the current password via `authenticate_user`, which (a) fed the
login lockout — a stolen *access token* could lock the owner out of `/login`,
and a user could self-lock by mistyping — and (b) minted a throwaway `Session`
each call, churning refresh-token rows that count toward the max-5 cap and could
silently evict a real device session. New `verify_password` helper verifies the
password only: no lockout mutation, no token minting. Brute-force protection
unchanged (these routes keep their tight per-user `sensitive_config` limit:
burst 2, then 1/30min). Lockout stays a login control. Commit `7ed67735`.

### Deferred — low risk now, revisit at scale (logged in `context/progress/backlog.md`)
- `AccountLocked` returns 429 vs 401 — an account-state oracle. Kept for UX.
- Registration enumerates ("username/email already exists") — hard to fully close; common in big apps.
- `user_daily_activity` columns are `INTEGER` while lifetime is `BIGINT`; safe via the per-flush clamp, BIGINT migration would decouple.
- `CardFilter` query/predicate split (`context/plans/card_filter_split.md`).

### Identified, pending decision (not yet actioned)
- **Replace-mode import non-atomic (Medium)** — **planned**: insert commits, then delete-not-in runs separately; a crash between leaves a hybrid board. Chosen fix is Option A (full atomicity: lock + limit-check + insert + reconcile in one tx). Deferred (frontend untestable). Plan: `context/plans/import_atomicity.md`.
- Low-severity: refresh `Forbidden` 403-vs-401; Argon2 default params unpinned; `chunks(0)` panic guard; card-limit TOCTOU; Archidekt bad-quantity 500s the import; `last_active_cache` unbounded growth.
