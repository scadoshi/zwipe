# Plan: zwiper end-to-end harness (real client against real router)

**Status: PARKED (sketched 2026-07-29, reshaped 2026-10-06), but the reason for parking it was disproven on 2026-09-21. Still not scheduled; the case is now stronger than when it was written.**

The original note said the black-box-against-prod habit covered this ground and that contract types plus zerver's integration suite caught the drift class that mattered. Then the client refactor pinned every endpoint to a single success status, which broke the four endpoints answering 204. Three failed silently and shipped to main. Nothing in the suite noticed, because each side of the wire was tested separately and both sides were individually correct.

That is exactly the bug this harness describes catching, named in the goal below as "status to `ClientError` mapping against real responses". It was eventually caught by a person using the app, and every one of the four had to be re-proven by hand afterwards.

Weigh that against the cost before parking it again.

## Goal

Tests that exercise the FULL path against a real zerver, never prod: the app's own requests through `zwipe-client` (reqwest) → real HTTP → axum router → sqlx → Postgres, and on top of that the UI itself. Catches the thin-translation-layer bugs nothing else can: status→`ClientError` mapping against real responses, header/auth attachment, real-payload decoding, refresh flows, and screens that only break when a real server answers.

## The premise: a local zerver, no prod

There is no prod in these tests, but there is a real zerver: a local one over a throwaway Postgres (the dev-env scripts already stand up Postgres 18, and `#[sqlx::test]` hands out a fresh migrated DB per test). zwiper no longer depends on zerver (`c84c9841`), and zerver must never know its consumers, so the harness lives in its own place rather than inside either app.

## Shape, two layers

**1. Client wire tests (Rust, a dedicated test crate, e.g. `e2e/`).** It depends on zerver (to build the router) and `zwipe-client` (to call it), so neither app changes. Each test:

```text
#[sqlx::test]                       // fresh DB per test, migrations applied
  → build zerver AppState + Router over the test pool
  → tokio::net::TcpListener::bind("127.0.0.1:0"), spawn axum::serve
  → ZwipeClient::new(base_url) pointed at the listener's local_addr
  → drive real flows: register → login → create deck → ... → assert rows
```

**2. UI tests (the programmatic layer on top).** Run a local zerver the same way (or the `zerver` binary against a seeded local DB), serve zwiper's web build with `BACKEND_URL` pointed at it, and drive the screens with a browser automation tool, asserting on what renders. This covers what the wire tests can't: the screens, their states and their error toasts against real responses. Native-only behavior (iOS/Android bridges, gestures) stays with device testing.

## What it needs

1. **Router construction without duplication.** zerver's `tests/common/` already wires AppState (fake email sender, test config) but isn't exported. Preferred: zerver grows a small `test-support` feature exposing `router_for_tests(pool) -> Router`, consumed by zerver's own suite and the e2e crate alike. Copying the wiring would drift.
2. **A client constructor that takes a base URL:** `ZwipeClient::new(base_url: Url)` already does. Done.
3. **Seed data for UI tests:** a small card catalog fixture (a full zervice sync works locally in about a minute, but a fixed subset keeps tests fast and stable).
4. **Test DB env** (`DATABASE_URL`, as zerver's suite uses), and a CI job if it ever gates. CI's `postgres:18` service already covers the database side.

## First tests worth writing (if/when built)

- Register → login → refresh: token attach + rotation through the real wire.
- A 422 and a 401 surface as the right `ClientError` variants with the server's actual message copy.
- One fat payload (deck with cards) decodes through the real serde path.
- The usage-batch flush posts and lands counters (closes the one seam the error-reporting work tests from each side separately).

## Why parked

Each test spawns a server + needs DB env; the payoff is the thin reqwest translation layer, which changes rarely. Shared contract types make the compiler the contract test; zerver's suite posts the same JSON the client serializes; decode-error reporting (2026-07-29) alarms on drift in prod. Revisit if a wire bug ever slips through those three nets.
