# Plan: zwiper end-to-end harness (real client against real router)

**Status: PARKED (sketched 2026-07-29), but the reason for parking it was
disproven on 2026-09-21. Still not scheduled; the case is now stronger than
when it was written.**

The original note said the black-box-against-prod habit covered this ground
and that contract types plus zerver's integration suite caught the drift
class that mattered. Then the client refactor pinned every endpoint to a
single success status, which broke the four endpoints answering 204. Three
failed silently and shipped to main. Nothing in the suite noticed, because
each side of the wire was tested separately and both sides were individually
correct.

That is exactly the bug this harness describes catching, named in the goal
below as "status to `ClientError` mapping against real responses". It was
eventually caught by a person using the app, and every one of the four had
to be re-proven by hand afterwards.

Weigh that against the cost before parking it again.

## Goal

Integration tests that exercise the FULL client path: `ZwipeClient`
(reqwest) → real HTTP → axum router → sqlx → Postgres, instead of testing
each side of the wire separately. Catches the thin-translation-layer bugs
nothing else can: status→`ClientError` mapping against real responses,
header/auth attachment, real-payload decoding, refresh flows.

## The key insight: it lives on the ZWIPER side

zwiper already links zerver; zerver must never know its consumer. So the
harness is zwiper integration tests (`zwiper/tests/`) that construct zerver's
router in-process, bind it to an ephemeral port, and point a real
`ZwipeClient` at `127.0.0.1:<port>`. No architectural change, no code moves,
dependency arrow untouched.

## Shape

```text
#[sqlx::test]                       // fresh DB per test, migrations applied
  → build zerver AppState + Router over the test pool
  → tokio::net::TcpListener::bind("127.0.0.1:0"), spawn axum::serve
  → ZwipeClient pointed at the listener's local_addr
  → drive real flows: register → login → create deck → ... → assert rows
```

## What it needs (the "quite the harness" part)

1. **Router construction without duplication.** zerver's `tests/common/`
   already wires AppState (fake email sender, test config) but isn't
   exported. Preferred: zerver grows a `test-support` feature exposing a
   `router_for_tests(pool) -> Router` helper: one source of truth; zwiper's
   harness and zerver's own tests both consume it. (Alternative: copying the
   wiring into zwiper's tests, drifts.)
2. **Feature unification.** zwiper depends on zerver with
   `default-features = false`; the harness needs the `zerver` feature, added
   via `[dev-dependencies]`. Cargo unifies features across normal+dev deps in
   test builds; audit that nothing in zwiper's lib behaves differently with
   the feature on (the 2026-07-28 `From<reqwest::Error>` incident was this
   exact class).
3. **A client constructor that takes a base URL** (test-only): today
   `ZwipeClient::new` reads env config; the harness needs
   `ZwipeClient::with_backend_url(addr)` or equivalent.
4. **Test DB env** for zwiper's test runs (`DATABASE_URL` sourced, same as
   zerver's suite); CI job addition if it ever gates.

## First tests worth writing (if/when built)

- Register → login → refresh: token attach + rotation through the real wire.
- A 422 and a 401 surface as the right `ClientError` variants with the
  server's actual message copy.
- One fat payload (deck with cards) decodes through the real serde path.
- The usage-batch flush posts and lands counters (closes the one seam the
  error-reporting work tests from each side separately).

## Why parked

Each test spawns a server + needs DB env; the payoff is the thin reqwest
translation layer, which changes rarely. Shared contract types make the
compiler the contract test; zerver's suite posts the same JSON the client
serializes; decode-error reporting (2026-07-29) alarms on drift in prod.
Revisit if a wire bug ever slips through those three nets.
