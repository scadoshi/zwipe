# Security Review

Audit performed 2026-03-26 ahead of App Store public launch.

---

## Must Fix Before Public Launch

### Rate Limiting on Auth Endpoints (Critical)
No rate limiting on `/api/auth/login`, `/api/auth/register`, `/api/auth/refresh`. Open to brute force.
- **Fix**: `tower-governor` crate on auth routes — 5 attempts/IP/15 min on login, 3 registrations/IP/hour
- **File**: `zerver/src/lib/inbound/http/mod.rs` (router setup)

### CORS Missing Authorization Header (High)
CORS only allows `Content-Type`. JWT tokens are sent via `Authorization` header — any web client would be blocked. iOS native client doesn't preflight so this isn't visible yet.
- **Fix**: Add `header::AUTHORIZATION` to `.allow_headers([...])` in CORS config
- **File**: `zerver/src/lib/inbound/http/mod.rs` ~line 220

### Account Lockout (High)
No failed login tracking. No per-account lockout after repeated failures. Rate limiting handles IP-level, lockout handles account-level (different attacks).
- **Fix**: `failed_login_attempts` table (user_id, count, last_attempt). Lock after 5 failures in 30 min. Reset on success.
- **Files**: new migration + `zerver/src/lib/domain/auth/services.rs`

### HTTP Security Headers (Critical)
Missing: `X-Content-Type-Options`, `X-Frame-Options`, `Strict-Transport-Security`, `Referrer-Policy`.
- **Fix**: `tower-http` `SetResponseHeaderLayer` — add to middleware stack
- **File**: `zerver/src/lib/inbound/http/mod.rs`

### JWT Secret Length Not Validated (Medium)
`JWT_SECRET` from `.env` accepts any string — a short/weak secret silently weakens signing.
- **Fix**: Reject at startup if `< 32 chars`
- **File**: `zerver/src/lib/domain/auth/models/access_token.rs`

### `ApiError::Network` Leaks Internal Messages (Medium)
`ApiError::Network(message)` returns the raw message to the client — could include DB errors or stack details.
- **Fix**: Log internally, return generic `"internal server error"` to client
- **File**: `zerver/src/lib/inbound/http/mod.rs` ~line 102-122

---

## Fix Soon After Launch

### No Security Event Audit Logging (Medium)
No structured logging of login attempts, token refreshes, password changes, or account deletion. Makes detecting abuse impossible.
- **Fix**: `tracing::info!(event = "login_attempt", success = false, ...)` on auth events. Consider audit log table long-term.

### Email Verification on Registration (Low)
Users can register with any email — no verification step. Enables typos, impersonation, spam accounts.
- **Fix**: Send verification email on register, require confirmation before full account access. Needs email sending infrastructure (see password reset below — same infrastructure).

### Expired Token Cleanup (Low)
`zervice` has `delete_expired_sessions` but it's not documented or guaranteed to run on a schedule. DB grows unbounded otherwise.
- **Fix**: Ensure nightly cron for zervice covers this. Add logging when cleanup runs.

### Refresh Token Format Validation (Low)
`UnvalidatedRefreshToken` validates length (32 chars) but not character set — non-hex chars pass and reach the DB layer.
- **Fix**: Add `chars().all(|c| c.is_ascii_hexdigit())` check in `from_str`
- **File**: `zerver/src/lib/domain/auth/models/refresh_token.rs`

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

**Not yet implemented.** See `context/project/security.md` password reset design section. Needs email sending infrastructure (transactional email provider — Resend or Postmark recommended).

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
