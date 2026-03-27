# Security Review

Audit performed 2026-03-26 ahead of App Store public launch.

---

## Must Fix Before Public Launch — ALL COMPLETE ✓

### ✓ Rate Limiting on Auth Endpoints (Critical)
- **Done**: `tower_governor` per-route on `/login`, `/register`, `/refresh` + blanket on all private routes. IP-keyed via `PeerIpKeyExtractor`. Commit: `588f4fa`.

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
- **Done**: Structured `event =` audit logs at all key auth points — `login_success`, `login_failure` (reason: `invalid_password` or `account_locked`), `register`, `token_refresh_failure` (reason: `not_found`/`expired`/`revoked`/`forbidden`), `token_cleanup`. Written to rolling daily file at `/var/log/zwipe/` alongside stdout. Commit: `440f972`.

### ✓ Expired Token Cleanup (Low)
- **Done**: `zervice` has `delete_expired_sessions`, nightly cron confirmed on Pi (`0 4 * * *`). `token_cleanup` event now logs `rows_deleted` on each run. Commit: `440f972`.

### ✓ Refresh Token Format Validation (Low)
- **Done**: Length corrected to 64 chars (was 32), `is_ascii_hexdigit()` check added, whitespace trimmed before storage. Commit: `e02ab5e`.

---

## Remaining Open Item

### Email Verification on Registration (Low)
Users can register with any email — no verification step. Enables typos, impersonation, spam accounts.
- **Fix**: Send verification email on register, require confirmation before full account access. Needs email sending infrastructure (same as Password Reset below — implement both together via Resend).
- **Blocked on**: Resend integration (resend.com — 3k emails/month free tier, Rust-friendly API).

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

## Password Reset

**Not yet implemented.** Needs email sending infrastructure. **Decided: Resend** (resend.com) — modern API, Rust-friendly, 3k emails/month free tier.

Password reset flow design:
1. User submits email on "forgot password" screen
2. Server generates a cryptographically random token (32 bytes hex), stores SHA-256 hash + expiry (15 min) in `password_reset_tokens` table
3. Server sends email with link: `https://zwipe.net/reset?token=<raw_token>`
4. User clicks link → app/web presents new password form
5. Client POSTs `{ token, new_password }` to `/api/auth/reset-password`
6. Server: SHA-256 hashes the submitted token, looks up hash in DB, checks expiry, checks not already used
7. If valid: update password hash, mark token used (or delete), revoke all existing refresh tokens (force re-login everywhere)
8. Respond with success — user logs in fresh

Security properties of this design:
- Only the hash is stored — a DB breach doesn't expose usable reset tokens
- 15 min expiry limits attack window
- Token is single-use
- All existing sessions are revoked on reset (prevents session fixation)
- Always return "if that email exists, a reset link was sent" — no email enumeration
