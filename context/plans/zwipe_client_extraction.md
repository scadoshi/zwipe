# zwipe-client extraction

**Status: PLANNED 2026-09-21 (external architecture review, claims verified
against the code the same day). Sequenced after
`zerver_feature_gate_teardown.md` phases 1-3; this move naturally absorbs its
phase 4. Gets more expensive the longer it waits: the second copy of the
client layer is the priciest artifact on the review's list, and it hasn't
been written yet.**

**One sentence:** pull zwiper's 61-file typed API client into a new
`zwipe-client` crate (depends on zwipe-core + reqwest only) so both clients
share one implementation before zite grows the authed surface decisions.md
already commits it to.

## Why

- decisions.md 2026-04-06 ("Web App: Unified Domain via Zite") commits zite
  to becoming the full authenticated deck builder. That needs login,
  register, refresh, deck CRUD, card search, filters. zwiper has all of it
  in a crate zite can't import.
- The drift has started. zite bypasses `paths.rs` with four raw literals:
  `reset.rs:58`, `verify.rs:21`, `shared_deck.rs:326`, `shared_deck.rs:430`.
  All four paths already exist in core. The oracle-tags one duplicates an
  endpoint zwiper has a proper typed client function for: two clients, two
  implementations, one of them a string literal.
- A crate that depends only on core **structurally cannot** import zerver,
  which locks in the feature-gate teardown instead of relying on discipline
  (which `handlers/health.rs` proves we don't have).

## Shape

- New workspace member `zwipe-client`: owns `ClientError` (post-flatten, see
  teardown phase 4) and the reqwest call modules. No Dioxus, no zerver, no
  platform code; crash reporting, session storage and URL config all stay in
  the apps.
- zwiper consumes it; zite replaces its literals with typed calls.

## Measured, not guessed (2026-09-21)

The layer turned out to be near-perfectly portable already:

- 61 files, 2,863 lines, and **zero** Dioxus references. No `std::fs`, no
  keyring, no `target_os`, no Android anything.
- External deps are reqwest, std, zwipe-core, tracing, uuid, thiserror.
  That's the whole list.
- Sessions arrive **as parameters** (`session.access_token.value`, `Session`
  being a core type), never read from storage. Token persistence and refresh
  scheduling never cross the boundary. This is the decision that normally
  sinks an extraction, and it was already made correctly.
- Coupling to zwiper is two imports in two files: `crate::config::Config` in
  `mod.rs`, `zwipe::inbound::http::ApiError` in `error.rs`. The other 51
  files import only `ClientError` and `ZwipeClient`, which travel with them.

Difficulty: a focused day, two with the trait collapse.

## The `Send` bounds (tested, not theorized)

57 trait methods carry `impl Future<Output = ...> + Send`. reqwest's wasm
futures are `!Send`, so those bounds would break zite's web build. They are
vestigial: zwiper drives every call through Dioxus's `spawn` (a local
spawner) and has zero `tokio::spawn` / `thread::spawn`. Stripping all 57 and
compiling zwiper was clean, verified 2026-09-21. One sed during the move.

## Base URL: taken at construction (owner decision, 2026-09-21)

`ZwipeClient::new(base_url: Url)` takes the URL as a parameter. The crate
reads no env var and owns no default, so it needs no build.rs and no `.env`
of its own. Each app keeps sourcing the value the way it already does:

- **zwiper keeps `.env` + `build.rs`** (`env!("BACKEND_URL")`) and passes
  `config.backend_url` in. Do NOT retire this in favor of core's `API_BASE`:
  the `.env` is a flip switch for pointing a *debug* build at *prod* (its own
  comment records exactly that, for PATCH-migration testing), and a
  `debug_assertions` const cannot express it.
- **zite keeps `zwipe_core::domain::site::API_BASE`** and passes that in.

Note this is runtime at the crate's API boundary, not on the device: an iOS
bundle ships no `.env` and a browser has no env vars, so the value is still
baked per build. What changes is which crate bakes it.

Do not move zwiper's `Config` type; it also carries `rust_log` and
`rust_backtrace`, which are nothing to do with the client. Pass the `Url`.

## Cargo gotcha

zite's Cargo.toml already records it: the workspace `reqwest` cannot be used
on wasm32 (rustls-tls fails to compile). The new crate needs the same
`[target.'cfg(target_arch = "wasm32")']` / `cfg(not(...))` dependency split
zite has, and must not take `reqwest = { workspace = true }`.

The 48 `tracing` calls compile on wasm but go nowhere without a subscriber in
zite. Not a blocker; just don't expect logs there.

## Scope: move all of it

zite needs about four endpoints today; the client has 61 files, mostly deck
and card operations zite won't touch until it becomes the deck builder. Move
the whole layer anyway. Moving half recreates the two-homes problem the crate
exists to prevent.

## The 52-trait question (owner decision, settle before the move)

`zwiper/src/lib/outbound/client/` has 52 `pub trait ClientX` with exactly 52
impls, all on `ZwipeClient`. No mocks, no second implementor; screens grab
the concrete type via `use_context` and import the trait only to bring the
method into scope. Two honest options:

1. **Collapse to inherent `impl ZwipeClient` blocks during the move**
   (recommended). Deletes a declaration + impl-block per endpoint and every
   trait import at the call sites. A mock, if ever wanted, can be introduced
   then, with the shape that test actually needs.
2. Keep the traits and write the first mock so they pay rent.

Doing the move without deciding copies the ceremony into the new crate.

## Quick win that shouldn't wait for the crate — DONE 2026-09-21

zite's four literals now go through `zwipe_core::http::paths`, joined
`format!("{}{}", API_BASE, route())` like the rest of zite. The
leading-slash trap is gone too: every path fn returns an absolute path
(teardown phase 2). The shared-deck page parses its URL token to a `Uuid`
first and treats a non-uuid as NotShared, which is what the server would
have said anyway.

## Verification

- Workspace suite + full CI gate; client crates warning-free.
- zite manual pass: shared deck page, verify-email flow, reset-password
  flow (the pages that owned the literals).
- zwiper smoke on device/sim: login, deck list, search (proves the moved
  client wires up identically).
- Build zite for **both** targets, wasm and the `server` feature. The wasm
  build is what proves the `Send` bounds are really gone.

## Later horizon, explicitly not now

An `Endpoint` trait in core (`const METHOD`, `fn path()`, `type Req`,
`type Resp`): collapses the per-endpoint client files into one generic
`call<E>()` and lets the router assert coverage at compile time. It's the
endgame of the contract-crate architecture, and it's a big refactor whose
payoff scales with client count. Revisit if/when zite actually grows the
full authed surface, not before.
