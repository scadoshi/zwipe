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
  teardown phase 4), the reqwest call modules, and base-URL config. No
  Dioxus, no zerver, no platform code; crash reporting and session storage
  stay in zwiper.
- zwiper consumes it; zite replaces its literals with typed calls.

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

## Quick win that shouldn't wait for the crate

zite's four literals can switch to `zwipe_core::http::paths` today. Note the
leading-slash trap: three path fns lack the `/` prefix (teardown phase 2
rider fixes them); until then join via `Url::set_path`, not `format!`.

## Verification

- Workspace suite + full CI gate; client crates warning-free.
- zite manual pass: shared deck page, verify-email flow, reset-password
  flow (the pages that owned the literals).
- zwiper smoke on device/sim: login, deck list, search (proves the moved
  client wires up identically).

## Later horizon, explicitly not now

An `Endpoint` trait in core (`const METHOD`, `fn path()`, `type Req`,
`type Resp`): collapses the per-endpoint client files into one generic
`call<E>()` and lets the router assert coverage at compile time. It's the
endgame of the contract-crate architecture, and it's a big refactor whose
payoff scales with client count. Revisit if/when zite actually grows the
full authed surface, not before.
