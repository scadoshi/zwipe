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

## All Security Items Complete ✓

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
