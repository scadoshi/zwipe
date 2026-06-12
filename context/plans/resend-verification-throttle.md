# Resend-verification throttle + refresh

**Status: BUILT 2026-06-11** (server `3692c410`, client `ae916671`). Server
half live-tested locally: 5 rapid calls → 1 through + 4×429, no-auth → 401
(the lone 500 in testing was the dev .env's stale Resend key, not the route).
Server deploys with the next zerver deploy; client rides the next App Store
build after 33. Original design below, implemented as written.

Server-side throttle plus a client
cooldown/refresh on the email-verification UI. Goal is narrow: stop impatient
multi-clicks from fanning out into many verification emails, and let a user
re-check "did it verify yet?" without leaving the screen. Not an
attacker-grade defense — the caller is authenticated and emailing their own
address; we just don't want 5 fast clicks = 5 emails.

## Current state (investigated 2026-06-11)

- `POST /api/auth/resend-verification` (`zerver/.../routes.rs:315`,
  handler `handlers/auth/resend_verification.rs`) requires `AuthenticatedUser`
  and 422s if already verified.
- **No dedicated throttle.** It only sits under the broad `private_config`
  governor (burst **500**, then 1 per 600ms per user) applied to the whole
  `/api` router — a DDoS backstop, not an email-abuse cooldown. A logged-in
  user can fire hundreds of emails to themselves.
- The UI is **not a dedicated screen** — it's the `EmailVerification` row
  component on the profile screen
  (`zwiper/.../screens/profile/components/email_verification.rs`). Today it has
  only a "Resend" button that disables while the request is in flight
  (`is_resending`), then re-enables immediately. The profile screen already
  re-fetches the user via `get_user` on open so `email_verified_at` is current
  (`screens/profile/mod.rs` `use_effect`).

## Server design — dedicated per-route limiter

Add one governor config and wrap only the resend route. Reuse the
`unauthorized_on_missing_key` error handler added 2026-06-11 (so an
unauthenticated call returns 401, not 500, consistent with the other
user-keyed limiters).

```rust
// burst 1, then 1 req/60s — resend verification: a fast multi-click sends one
// email, the rest get 429. Window matches the client cooldown timer.
let resend_verification_config = Arc::new(
    GovernorConfigBuilder::default()
        .period(Duration::from_secs(60))
        .burst_size(1)
        .key_extractor(UserIdKeyExtractor::new(jwt_secret.clone()))
        .finish()
        .expect("rate limit config: burst_size and period must be non-zero"),
);
```

Then the route gains its own layer (it currently has none):

```rust
.route(
    "/resend-verification",
    post(resend_verification).layer(
        GovernorLayer::new(resend_verification_config)
            .error_handler(unauthorized_on_missing_key),
    ),
)
```

Notes / gotchas:
- **Ordering**: `jwt_secret` is moved (no `.clone()`) into the *last* config
  built (`metrics_usage_config`). Add this config **before** that one, and use
  `jwt_secret.clone()`.
- **Per-route + global both apply** — fine; the per-route 1/60s is strictly
  tighter than the global private limit, so it's the binding constraint.
- **Throttled response is 429** (the error handler only remaps
  `UnableToExtractKey`; `TooManyRequests` keeps the library default). The
  client should treat 429 here as "already sent, wait" rather than a hard
  error — see below.
- **60s is the knob.** Matches a friendly visible countdown. Bump if even one
  per minute feels loose; it already kills the 5-rapid-clicks case dead.

Server is patchable without an app release (deploy-first), so this half can
ship immediately and independently of the client.

## Client design — cooldown button + refresh

In `EmailVerification`:

1. **Optimistic cooldown.** On click, immediately disable the button and start
   a 60s countdown (`Resend in 59s` … matching the server window), per the
   "button should immediately grey out with a timer" requirement. Drive it with
   a `use_signal<u32>` decremented by a `tokio::time::sleep(1s)` loop (the
   codebase already uses tokio timers, e.g. `session_upkeep.rs`; swap to a
   wasm-safe timer if/when zite needs it, per the wasm-blockers todo).
2. **Failure recovery.** If the send fails for a non-429 reason (network,
   500), clear the countdown and re-enable so the user can retry — don't strand
   them behind a timer for a send that didn't happen. On **429** (raced the
   server limit), keep the countdown and show a gentle "Please wait a moment"
   toast rather than a scary error.
3. **Refresh button.** Add a second button ("Check again" / refresh icon) that
   re-fetches `get_user` and updates `session.user`, flipping the badge to
   Verified the moment the user has clicked the email — no logout/return needed.
   The component already has `client` + `session` in context; the fetch mirrors
   the profile screen's existing `use_effect`. The refresh button needs no
   throttle (reads are cheap, under the generic private limit) — maybe a brief
   in-flight spinner.
4. Copy stays sentence case per app convention ("Resend", "Resend in {n}s",
   "Check again", "Sending…").

## Testing

Server (curl, local stack — same pattern as the auth-fix test):
1. Register + log in a throwaway user (unverified).
2. `POST /api/auth/resend-verification` with the bearer → 200 (one email).
3. Immediately fire it 4 more times → first is 200/429 depending on timing,
   the rest **429**. Confirm only ~1 send per 60s window.
4. No-auth call → **401** (the error handler), not 500.

Client (simulator): click Resend → button greys with a live countdown,
re-enables at 0; verify the real email arrived once despite extra clicks;
click "Check again" after verifying via the email link → badge flips to
Verified without leaving the screen.

## Scope / sequencing

- **Server**: one config + one route layer. Patchable now, no app release.
  Ship it whenever — it makes the current clients safe even before the button
  changes land.
- **Client**: the cooldown + refresh button — a build-32-class change (rides
  with the deck-aware add-screen adoption already queued). Low risk, isolated
  to one component.
