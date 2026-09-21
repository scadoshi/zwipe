# zerver feature-gate teardown

**Status: IN PROGRESS. Planned 2026-09-21 (external architecture review,
claims verified against the code the same day); phase 1 landed the same day.
Four phases, each independently shippable, each deletes a bucket of gates.
Phase 2 is mechanical and can run any time; 3 and 4 each want a focused
pass.**

Note from phase 1: the check commands need `--lib`. The zerver/zervice
binaries require the feature by design, so a bare `-p zerver
--no-default-features` fails on the bins, not on anything the flag guards.
Second find: four gate lines in handlers/mod.rs ended in CRLF; sweep
patterns need to tolerate trailing `\r` (a few stray CRLF line endings
exist elsewhere in handlers too).

**One sentence:** retire zerver's `zerver` feature flag by removing the four
remaining reasons zwiper links the server crate, phase by phase, until zerver
has one build configuration again.

## Why (verified counts, 2026-09-21)

- 598 `#[cfg(feature = "zerver")]` gates in zerver/src. 335 of them (56%)
  live under `inbound/http/handlers/`, a subtree zwiper imports zero symbols
  from. Those gates compile to nothing in every configuration that matters.
- The flag hides the coupling instead of containing it. `routes.rs:165` is an
  ungated `pub use zwipe_core::http::paths::*;`, so zwiper accumulated 48
  route imports through zerver with no error, no warning. Same mechanism let
  in the 2 `Password` imports. zite, which never had the flag, never grew the
  dependency.
- Discipline is already slipping: `handlers/health.rs` has ungated
  server-only items (`RootResponse`, plus `chrono`/`serde`/`serde_json`
  imports) that survive only because they happen not to touch axum. The rule
  isn't enforced by anything.
- Origin is honorable and documented (decisions.md): the flag predates
  zwipe-core, when zwiper genuinely needed the whole domain layer. The
  core extraction (2026-04-02) removed the reason; the mechanism stayed
  because ApiError couldn't move (orphan rule, correct call). A mechanism
  built to share a domain layer now serves one six-variant enum.

## Phase 1: gate the `handlers` module (kills 335) — DONE 2026-09-21

`inbound/http/mod.rs` already shows the pattern: `cache` and `middleware`
carry one gate on the module declaration and zero inside.

1. Gate the strays in `handlers/health.rs` (`RootResponse` and its imports),
   or just proceed; the module gate makes them moot.
2. Add `#[cfg(feature = "zerver")]` to `pub mod handlers;`.
3. Sweep the now-redundant internal gates out of `handlers/`.

Behavior identical by construction: the subtree already compiled to nothing
under `--no-default-features`.

## Phase 2: repoint the route imports (kills ~20)

1. zwiper: `use zwipe::inbound::http::routes::X` becomes
   `use zwipe_core::http::paths::X`, 48 sites. Same functions, one hop
   shorter, and it's what zite already does.
2. Gate `pub mod routes;` and sweep its internal gates.

Rider while touching paths.rs: three route fns are missing the leading slash
(`api/card/roles`, `api/deck/tags`, `api/card/oracle-tags`). zwiper's
`Url::set_path` normalizes it silently, but any consumer joining with
`format!("{BASE}{path}")` breaks. Normalize to leading `/` and eyeball the
server-side registrations. Optional second rider: static paths become
`const &str` (only the Uuid-taking ones stay functions).

## Phase 3: stop importing the server Password (kills 199)

`register.rs:20` and `change_password.rs:19` import zerver's `Password` only
to validate; `Password::new` delegates to
`zwipe_core::domain::auth::password::validate` (zerver password.rs:112), and
everything else in the type is Argon2 hashing a client must never touch.

1. Call core's `validate` directly at both sites.
2. Gate `pub mod domain;` and sweep.

## Phase 4: flatten ApiError into ClientError (kills the flag itself)

Verified: zwiper never deserializes an ApiError. It constructs one locally
from `(StatusCode, String)` (client/error.rs:68) and matches on variants,
always inside `ClientError::Api(...)`. The enum's own doc comment says so:
clients rebuild the vocabulary in their own error layer.

1. Move the six variants into `ClientError` directly (Unauthorized,
   Forbidden, NotFound, UnprocessableEntity, TooManyRequests, Internal).
2. Update the match sites: ensure_session.rs, deck_warnings.rs,
   email_verification.rs, usage_buffer.rs (kind slugs), error.rs and its
   tests.
3. Drop `zwipe` from zwiper/Cargo.toml, delete the feature from
   zerver/Cargo.toml, remove the last ~44 gates and the `optional = true`
   on zerver's fourteen gated dependencies.

**Not** "move ApiError to core." decisions.md's orphan-rule reasoning stands:
core ApiError makes `IntoResponse` a foreign impl and drags 73 `From` impls
out of the handler files where they belong. The six names written on both
sides of the wire are protocol vocabulary, the same way both sides reference
StatusCode without sharing an enum.

## Verification, every phase

- `cargo check -p zerver --no-default-features --lib` (until phase 4 deletes
  that configuration entirely).
- Full CI gate: nightly fmt, clippy `-D warnings`, workspace tests.
- Client crates warning-free (zwiper, zite, zwipe-components).
- No runtime testing needed; every phase is behavior-identical by
  construction.

## Doc follow-through

structure.md's Family table and CLAUDE.md's dependency graph update as each
phase lands; after phase 4 the `zwiper → zerver` edge is deleted from both.
(The docs currently say "ApiError only," which is why the 48-import drift
went unaudited. See the structure.md refresh, a separate quick task.)

## Explicitly out

- The `Endpoint` trait contract (typed method/path/req/resp in core). Real
  idea, wrong time; it pays off with a second full client. Recorded in
  `zwipe_client_extraction.md` as the later horizon.
