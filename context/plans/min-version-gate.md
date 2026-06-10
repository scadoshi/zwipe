# Client min-version gate — force-update kill-switch

**Status: planned, unstarted. Target: ride the next client build (1.0.5).**

## Context

Apple provides no force-update mechanism. The 2026-06 timestamptz wire-format
cutover and the refresh-token hardening both sat parked for weeks behind
"wait for the old clients to die out" gates (`context/status/todo.md`,
Pending Gated Merges). That wait exists because old builds break silently
when the server changes — there is no way to tell a stale client "update
before continuing".

The industry-standard fix is a server-driven minimum-version check baked into
the client: on launch (and periodically), the app asks the server for the
minimum supported version; if the running version is below it, the app
renders a full-screen blocking "Update required" view that deep-links to the
App Store. The graying-out you see in other apps is exactly this pattern.

**The chicken-and-egg rule:** the gate only works in builds that carry it.
Shipping it in 1.0.5 does nothing for 1.0.4-and-older users — but every
build after 1.0.5 becomes force-updatable. Ship the kill-switch before you
need it; future server flips then need days of propagation, not weeks.

## Design

Server-driven, env-configurable, zero-deploy to flip.

### zwipe-core (shared)

- `http/paths.rs` — `min_client_version_route()` → `"/api/client/min-version"`.
- `http/contracts/client.rs` (new) — contract type:
  ```rust
  pub struct HttpMinClientVersion {
      /// Lowest app version allowed to talk to this server, e.g. "1.0.5".
      pub min_version: String,
  }
  ```
- Pure semver-ish compare helper (x.y.z numeric tuple compare) with unit
  tests. Lives in core because both sides reason about it. No `semver` crate
  — three-segment split/parse is ~15 lines.

### zerver

- `.env` — `MIN_CLIENT_VERSION=0.0.0` (0.0.0 = gate open / allow everyone).
  Flipping the gate = edit `.env` on the server + restart zerver. No code
  deploy. Add to `zcripts/dev-env/*/setup.sh` env templates.
- Public handler `get_min_client_version` returning the env value — same
  shape as `get_public_metrics` (no auth). Nest under the existing
  `/api/marketing`-style public group as `/api/client/min-version`, with a
  modest IP rate limit (mirror `public_marketing_config`).
- Optional later: CF cache rule for `/api/client/*` (2h free-plan minimum is
  too coarse for a kill-switch — skip caching, the payload is ~30 bytes and
  the launch-time call volume is tiny at current scale).

### zwiper

- `Config` already exposes the running version via
  `env!("CARGO_PKG_VERSION")` (workspace version, matches
  CFBundleShortVersionString since 1.0.3).
- New client call `get_min_client_version` in `outbound/client/` (public, no
  session).
- New context signal `Signal<bool>` (`upgrade_required`), provided in
  `spawn_upkeeper` alongside the existing providers.
- Check cadence: inside the existing 60s upkeep loop in
  `session_upkeep.rs` — first tick fires immediately (launch check), then
  every minute (catches a mid-session flip within ~60s). On fetch error:
  fail open (never lock users out because the endpoint hiccuped).
- Blocking UI: in the root app component, when `upgrade_required` is true,
  render a full-screen view instead of the router — Zwipe logo, "Update
  required" headline, one button "Open App Store" linking to
  `https://apps.apple.com/us/app/zwipe-tcg/id6761341603`. Sentence case per
  app convention. No dismiss affordance — that's the point.

## What this does NOT solve

- Users who never open the app don't see the gate (nothing can reach them).
- Builds ≤ 1.0.4 ignore it forever. The propagation-wait pattern in
  `context/status/todo.md` still applies one final time, to 1.0.5 itself.

## Files touched

- `zwipe-core/src/http/paths.rs`, `zwipe-core/src/http/contracts/client.rs`
  (new), version-compare helper + tests
- `zerver`: handler (new file under `handlers/`), `routes.rs`, config
  plumbing for `MIN_CLIENT_VERSION`, `.env` + dev-env setup scripts
- `zwiper`: `outbound/client/` call (new), `session_upkeep.rs` (signal +
  loop check), root component blocking view, CSS

## Verification

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --lib   # incl. new version-compare tests

# server: gate open by default
curl -s localhost:3000/api/client/min-version | jq
# expect: {"min_version":"0.0.0"}

# flip locally: MIN_CLIENT_VERSION=99.0.0 in zerver/.env, restart
# sim: app shows the blocking screen within one upkeep tick; tapping the
# button opens the App Store listing; setting back to 0.0.0 + relaunch
# restores normal operation
```

## Operational notes

- Flip the gate **only** to versions whose predecessor breakage is real
  (e.g. after a wire-format change), and only after the new build is live on
  the App Store — gating to a version users can't download yet bricks them.
- The gate and the old propagation-wait are complementary: wait for the
  bulk via auto-update, gate the stragglers.
