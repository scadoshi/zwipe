# zwipe-client extraction

**Status: DONE 2026-09-22. Planned 2026-09-21 (external architecture review, claims verified against the code the same day), executed the next morning after `zerver_feature_gate_teardown.md` cleared its phases 1-3.**

The layer moved whole: 61 files into `zwipe-client`, depending on zwipe-core and reqwest only. The 52 traits collapsed to inherent methods on the way, which took the 57 `Send` bounds with them for free (they lived only on the trait method signatures). The crate checks clean for wasm32, so zite can import it whenever it grows the authenticated surface.

Wire safety held. Every file that defines the protocol hashes identically to before: the `Endpoint` impls, `paths.rs`, `contracts/`, `endpoint.rs`. Two lines of `call.rs` changed, the import path and where the base URL is read; request building, auth, body and decoding are byte-identical. Nothing a shipped 1.10.1 client sends or receives moved.

Three things the plan did not anticipate:

- Grouped imports hid the call sites. `outbound::{client::ZwipeClient, ...}` does not contain the literal `outbound::client`, so both a grep and the first pass of the rewrite missed nine files. Use-tree rewrites need to resolve prefixes, not match substrings.
- `getrandom` needs `wasm_js` on wasm32, one layer under the reqwest split the plan already flagged. uuid pulls it in.
- Multi-method traits (share_deck, skip_deck_card, commander_maybeboard) had one doc comment covering several methods. Copying it onto each one reads wrong; those four needed writing by hand.

CI change: `test.yml` now names `-p zwipe-client` alongside core and zerver. The deploy workflows deliberately do not, so a client-only failure cannot block a zerver or zite deploy. Clippy was already workspace-wide.

**One sentence:** pull zwiper's 61-file typed API client into a new `zwipe-client` crate (depends on zwipe-core + reqwest only) so both clients share one implementation before zite grows the authed surface decisions.md already commits it to.

## Why

- decisions.md 2026-04-06 ("Web App: Unified Domain via Zite") commits zite to becoming the full authenticated deck builder. That needs login, register, refresh, deck CRUD, card search, filters. zwiper has all of it in a crate zite can't import.
- The drift has started. zite bypasses `paths.rs` with four raw literals: `reset.rs:58`, `verify.rs:21`, `shared_deck.rs:326`, `shared_deck.rs:430`. All four paths already exist in core. The oracle-tags one duplicates an endpoint zwiper has a proper typed client function for: two clients, two implementations, one of them a string literal.
- A crate that depends only on core **structurally cannot** import zerver, which locks in the feature-gate teardown instead of relying on discipline (which `handlers/health.rs` proves we don't have).

## Shape

- New workspace member `zwipe-client`: owns `ClientError` (post-flatten, see teardown phase 4) and the reqwest call modules. No Dioxus, no zerver, no platform code; crash reporting, session storage and URL config all stay in the apps.
- zwiper consumes it; zite replaces its literals with typed calls.

## Measured, not guessed (2026-09-21)

The layer turned out to be near-perfectly portable already:

- 61 files, 2,863 lines, and **zero** Dioxus references. No `std::fs`, no keyring, no `target_os`, no Android anything.
- External deps are reqwest, std, zwipe-core, tracing, uuid, thiserror. That's the whole list.
- Sessions arrive **as parameters** (`session.access_token.value`, `Session` being a core type), never read from storage. Token persistence and refresh scheduling never cross the boundary. This is the decision that normally sinks an extraction, and it was already made correctly.
- Coupling to zwiper is two imports in two files: `crate::config::Config` in `mod.rs`, `zwipe::inbound::http::ApiError` in `error.rs`. The other 51 files import only `ClientError` and `ZwipeClient`, which travel with them.

Difficulty: a focused day, two with the trait collapse.

## The `Send` bounds (tested, not theorized)

57 trait methods carry `impl Future<Output = ...> + Send`. reqwest's wasm futures are `!Send`, so those bounds would break zite's web build. They are vestigial: zwiper drives every call through Dioxus's `spawn` (a local spawner) and has zero `tokio::spawn` / `thread::spawn`. Stripping all 57 and compiling zwiper was clean, verified 2026-09-21. One sed during the move.

## Base URL: taken at construction (owner decision, 2026-09-21)

`ZwipeClient::new(base_url: Url)` takes the URL as a parameter. The crate reads no env var and owns no default, so it needs no build.rs and no `.env` of its own. Each app keeps sourcing the value the way it already does:

- **zwiper keeps `.env` + `build.rs`** (`env!("BACKEND_URL")`) and passes `config.backend_url` in. Do NOT retire this in favor of core's `API_BASE`: the `.env` is a flip switch for pointing a *debug* build at *prod* (its own comment records exactly that, for PATCH-migration testing), and a `debug_assertions` const cannot express it.
- **zite keeps `zwipe_core::domain::site::API_BASE`** and passes that in.

Note this is runtime at the crate's API boundary, not on the device: an iOS bundle ships no `.env` and a browser has no env vars, so the value is still baked per build. What changes is which crate bakes it.

If zite ever grows the authed deck builder it will want zwiper's flip switch too, for dev testing. It should NOT copy zwiper's panic-if-unset build.rs: zite deploys from GitHub Actions, and requiring a `.env` there means handing the value to CI for no gain. Use an optional override with the const as fallback, resolved in the lib (zite's build.rs can't import zwipe-core, which is why it already mirrors one `WEB_BASE` literal):

```rust
// build.rs: emit only when someone set it
if let Ok(url) = std::env::var("API_BASE") {
    println!("cargo:rustc-env=ZITE_API_BASE={url}");
}

// lib: const fallback, no .env needed for a normal build
const API_BASE: &str = match option_env!("ZITE_API_BASE") {
    Some(url) => url,
    None => zwipe_core::domain::site::API_BASE,
};
```

(`match`, not `unwrap_or`: `Option::unwrap_or` isn't const.) Not needed now; today's const is fine.

Do not move zwiper's `Config` type; it also carries `rust_log` and `rust_backtrace`, which are nothing to do with the client. Pass the `Url`.

## Cargo gotcha

zite's Cargo.toml already records it: the workspace `reqwest` cannot be used on wasm32 (rustls-tls fails to compile). The new crate needs the same `[target.'cfg(target_arch = "wasm32")']` / `cfg(not(...))` dependency split zite has, and must not take `reqwest = { workspace = true }`.

The 48 `tracing` calls compile on wasm but go nowhere without a subscriber in zite. Not a blocker; just don't expect logs there.

## Scope: move all of it

zite needs about four endpoints today; the client has 61 files, mostly deck and card operations zite won't touch until it becomes the deck builder. Move the whole layer anyway. Moving half recreates the two-homes problem the crate exists to prevent.

## Quick win that shouldn't wait for the crate: DONE 2026-09-21

zite's four literals now go through `zwipe_core::http::paths`, joined `format!("{}{}", API_BASE, route())` like the rest of zite. The leading-slash trap is gone too: every path fn returns an absolute path (teardown phase 2). The shared-deck page parses its URL token to a `Uuid` first and treats a non-uuid as NotShared, which is what the server would have said anyway.

## Verification

Done 2026-09-22:

- Workspace suite green, 736 tests, and `cargo clippy --workspace --all-targets` clean. `cargo +nightly fmt --check` clean, which is the form the CI gate runs.
- `cargo check -p zwipe-client --target wasm32-unknown-unknown` passes. That is the proof the `Send` bounds are really gone.
- Wire hashes unchanged on every protocol-defining file.

Still owed, and it needs a device:

- zwiper smoke: login, deck list, search. The moved client should wire up identically, but nothing here has actually talked to the server yet.
- Fold this into the hands-on pass owed before 1.10.2 anyway, which already has to exercise crash reporting and telemetry by hand.

zite is untouched: it still reaches the API through `zwipe_core::http::paths` and does not depend on the new crate yet, so its pages need no re-test.
