# Plan: client error + crash reporting

**Status: BUILT (2026-07-29), NOT yet verified in production.** Server +
client halves implemented and tested locally (668 tests green across the
three crates); the call-site sweep is COMPLETE: all 73 authed
`to_user_message` toast sites report (every deck/card/profile/otag screen +
the shared components — multi-host `PrintingSheet`/`SwipeSelect` carry a
`host_screen` prop so the aggregation axis stays honest). Pre-auth screens
(login/register/forgot) deliberately excluded per non-goals.

**Rollout state + remaining work:**
- Server half deploys first (standing ordering) and sits ~a day before a new
  client build ships. Harmless by design: old clients omit `client_errors`
  (`#[serde(default)]`), and the crash endpoint just idles.
- ✔ Deployed to prod 2026-07-29 evening AND the role grants are APPLIED
  (`zervice_role.sql` re-run clean after the deploy's migration; scoped role
  verified reading both new tables). Do not re-run anything server-side —
  next checkpoint is the 04:03 UTC nightly (expect 5/5 with three prune log
  lines, no alert email).
- **Prod verification still owed** (after the next client build ships): the
  Verification checklist below — force a 422 → `client_errors` row; debug
  panic → exactly one `crash_reports` row across two relaunches; old client
  (1.7.3) still posts usage fine; one green nightly with the 5/5 upkeep
  step's prune log lines.
- Store data-safety label review before the client build submits.

Server currently logs every error it produces
(single exit path in `ApiError`'s `IntoResponse`), but errors that never reach
the server are invisible: transport failures, decode failures (contract drift
between client versions and the API), and outright crashes. We test against
live prod black-box, so this gap is exactly where debugging stalls.

## Goal

Two report kinds, one first-party pipe, no vendor SDK:

1. **Handled errors** — a `ClientError` was shown to the user (toast/dialog).
   Batched through the existing usage-flush pipeline with screen + action
   context.
2. **Crashes** — a Rust panic. Written to disk by a panic hook, found on next
   launch, sent once, cleared.

Explicitly *not* Sentry-grade: no breadcrumb trails, no device fingerprinting,
no session replay. Aggregate debugging signal in our own Postgres, consistent
with the existing "aggregate-only, no user identity" metrics posture (keeps
store data-safety labels unchanged).

## Reuse these (do NOT reimplement)

- `UsageBuffer` + `spawn_usage_flusher` / background flush
  (`zwiper/src/lib/inbound/components/telemetry/`) — the batching, 30s cadence,
  flush-on-background, and drop-on-failure semantics already exist.
- `HttpUsageBatch` (`zwipe-core/src/http/contracts/metrics.rs`) — extend with a
  `#[serde(default)]` field like `signals`/`deck_skips` did (older clients stay
  compatible).
- `record_usage` handler + repo (`zerver/.../handlers/metrics/record_usage.rs`)
  — the authed ingest path.
- `record_anonymous_event` pattern — model for the unauthed crash endpoint.
- `ClientError` (`zwiper/src/lib/outbound/client/error.rs`) — the single client
  error currency; every report derives from one.

## Design

### Contract (zwipe-core)

```rust
/// One handled-error report. Aggregate-only: no user identity beyond the
/// authed usage batch it rides in.
pub struct ClientErrorReport {
    pub screen: String,         // host screen, flattened module path ("deck_edit", "auth_login",
                                // "profile_change_email") — closed vocabulary, client-side consts.
                                // THE aggregation axis; component names never leak into it.
    pub component: String,      // "" for the screen itself; else the component module name
                                // ("card_filter_sheet") — the drill-down breadcrumb. Dialogs/
                                // sheets report host screen + their own component name.
    pub action: String,         // e.g. "save_profile" — what the user was doing
    pub kind: String,           // ClientError variant + ApiError variant, e.g. "api_unauthorized", "decode", "network"
    pub message: String,        // truncated ~300 chars; 4xx messages are user-safe by contract.
                                // HARD RULE: Decode messages reduced to error SHAPE (expected
                                // type / field path / position) — serde errors quote input
                                // fragments and response bodies must never ride a report or log.
    pub count: u32,             // dedupe within a flush window; clamped() caps to MAX_PER_FLUSH
                                // (10_000, the batch's existing ceiling) — hitting it means
                                // malicious or a broken loop, either way it stays finite
    pub client_version: String, // CARGO_PKG_VERSION — self-reported, untrusted metadata
    pub platform: String,       // "ios" / "android" / "web" — self-reported
}

/// One crash report, posted on the launch after the crash.
pub struct HttpCrashReport {
    pub crash_id: Uuid,        // stamped at panic time — server dedupes on it
    pub client_version: String,
    pub platform: ClientPlatform,
    pub message: String,       // panic payload + location, truncated ~2000 chars
    pub occurred_at: DateTime<Utc>,
}
```

- `HttpUsageBatch` gains `#[serde(default)] pub client_errors: Vec<ClientErrorReport>`.
- **Version/platform ride ON the report** (decided 2026-07-28): the batch
  itself carries neither, and the only server-side source is the per-session
  app version on the refresh-token row (`ce8abcad`) — reachable but a join the
  ingest doesn't need. Self-reported like `HttpCrashReport` already does;
  it's untrusted debugging metadata, not authorization data.
- **`clamped()` must cover the new field** — it's the batch's untrusted-input
  defense and every other field has it. Server-side, regardless of client
  behavior: truncate the vec to `MAX_CLIENT_ERRORS_PER_FLUSH` (20, matching
  the client cap), re-truncate `message` to ~300 chars and `screen`/`action`/
  `kind`/`client_version`/`platform` to sane lengths (client-side truncation
  is untrusted), clamp `count` to `MAX_PER_FLUSH`.
- Handled errors ride the **authed** usage batch. Crashes get their **own
  unauthed endpoint** (`record_crash_route`) because the next launch may have
  no session, and a crash report must not depend on auth working.

### Client: handled errors

- `UsageBuffer` gains an errors map keyed `(screen, component, action, kind,
  message)` → count, capped at ~20 distinct entries per flush window (beyond
  the cap, increment nothing — an error loop must not grow memory or spam).
- New helper on the buffer: `report_error(screen, component, action,
  &ClientError)` — derives `kind`/`message`, truncates, increments, AND emits
  a `tracing::warn!` with the same fields (decided 2026-07-29): one call
  site, two sinks — local client logs and the server table always tell the
  same story. The screen/component consts live in one module in the
  telemetry dir (the closed vocabulary's single home), names derived from
  the screen/component module paths.
- Call it where errors are *shown*: the `toast.error(e.to_user_message(), ..)`
  sites. Incremental adoption is fine — start with deck edit/save, card
  add/remove, auth screens; a follow-up sweep can cover the rest. No side
  effects hidden in `to_user_message` itself.
- Skip `ClientError::Network` by default: it's mostly the user's connectivity,
  and it can't be reported over the same dead connection anyway. `Decode` and
  `Api` variants are the signal.
- **Soft copy rule**: the dedupe key includes `message`, so 4xx copy should
  stay static (no interpolated names/numbers) or one dynamic fragment
  fragments the dedupe. Current server copy already follows this; keep it
  in mind when authoring new validation messages.

### Client: crashes (exactly-once via disk)

- `std::panic::set_hook` (installed at app start, chaining the previous hook)
  writes a JSON `HttpCrashReport` — crash_id stamped now — to a fixed path in
  the app data dir (same dir family `theme_store.rs` uses). Write is the only
  thing the hook does: no network, no allocation-heavy work beyond the string.
- **Platform gate**: the hook + crash store are native-only
  (`#[cfg(not(target_arch = "wasm32"))]`), same platform-conditional shape as
  the session store (keyring on iOS, files on Android). The web preview and a
  future wasm build get a no-op store — a disk write isn't a thing there, and
  browser crashes are a different animal anyway.
- On startup, after the client is constructed: if the crash file exists, read
  it, POST to `record_crash_route`, and **delete the file only on 2xx**. Not
  sent → file stays for the next launch. Sent-but-response-lost can retry next
  launch; the server upserts on `crash_id`, so the user-visible guarantee is
  exactly one stored instance per crash.
- One file, last-crash-wins: a crash loop overwrites rather than accumulates.

### Server

- Migration (additive): `client_errors` table
  (`id, received_at, client_version, platform, screen, component, action, kind, message, count`)
  and `crash_reports` table
  (`crash_id PK, received_at, occurred_at, client_version, platform, message`).
  `INSERT ... ON CONFLICT (crash_id) DO NOTHING` gives the exactly-once store.
- `record_usage` handler/service/repo extended to insert `client_errors` rows
  in the same transaction as the counters.
- New `record_crash` handler (unauthed, mirroring `record_anonymous_event`).
  It MUST stay unauthed: crashes in logged-out or auth-broken states are
  exactly the signal wanted. Defense stack for the unauthed write (decided
  2026-07-29): governor **burst 2, ~1 req/60s per IP** (a legit client posts
  at most one per launch), body cap ~4 KB, `ON CONFLICT DO NOTHING` makes
  replays free, content is aggregate-only (nothing worth stealing), retention
  bounds whatever gets through. Re-truncate `message` server-side before
  insert (client-side truncation is untrusted).
- Retention via a new **`UpkeepService`** (decided 2026-07-29; named for the
  Magic phase — it IS the nightly "at the beginning of your upkeep" step): a
  normal domain service (port + service + sqlx impl in zerver's lib)
  constructed ONLY by the zervice bin — zerver's HTTP wiring never straps it
  on. One 5th zervice step, three prunes, one log line each:
  `prune_client_errors` / `prune_crash_reports` (rows older than ~90 days —
  the unauthed crash table is the one most worth bounding) and
  `prune_expired_sessions` (`DELETE FROM refresh_tokens WHERE expires_at <
  NOW()`, the final dusting for fully-dormant users the insert-time drive-by
  never revisits; owner call 2026-07-29). Renumber the `step N/4` log labels
  (and the "all N steps ok" summary) to `N/5`. The insert-time drive-by
  stays — this is belt-and-suspenders, not a replacement. This service is
  the standing home for future maintenance cleanups.
- **Grants live in `zcripts/server/sql/zervice_role.sql`, NOT the migration**
  (decided 2026-07-29): roles are cluster-global while migrations run
  per-database (sqlx::test spins ephemeral DBs — role creation would collide),
  passwords can't be committed, and the role only exists where provisioned.
  Schema = portable shape in migrations; roles/grants = per-cluster
  infrastructure in the one canonical role script. Ship-time additions:
  `GRANT SELECT, DELETE ON client_errors, crash_reports TO zervice` and the
  surgical session-sweep grant — `GRANT DELETE ON refresh_tokens` +
  column-scoped `GRANT SELECT (expires_at) ON refresh_tokens` (zervice can
  destroy expired sessions but can never READ value_hash/user_id; worst
  compromise = mass logout nuisance, zero data exposure). Make the script
  fully idempotent (existence-guarded `DO` block around CREATE ROLE) so
  first-run and every re-run are the same command; document the lifecycle
  (first-time / new-table re-run / dev parity) in `server.md`. **Dev parity**:
  dev setup scripts (`zcripts/dev-env/*/setup.sh`) also run the role script
  (fixed throwaway dev password, local cluster only) so a local zervice run
  as the scoped role proves the grants before any nightly prod alert can.
  A forgotten grant still fails loudly via the alert email.

### Reading it

- `zcripts` query alongside the DAU/WAU ones: errors by `(client_version,
  screen, kind)` per day; crashes by `(client_version, message-prefix)`.
  A decode-error spike on one version = contract drift alarm.

## Implementation steps

1. zwipe-core: `ClientErrorReport`, `HttpCrashReport`, extend `HttpUsageBatch`
   **and its `clamped()`** (+ `MAX_CLIENT_ERRORS_PER_FLUSH`), add
   `record_crash` path/route.
2. Migration + sqlx repo methods (`cargo sqlx prepare --workspace` after).
3. zerver: extend record_usage ingest; new record_crash handler + service port
   wiring; zervice prune step.
4. zwiper: buffer extension + `report_error`; panic hook + startup send/clear;
   first wave of toast-site call-ins.
5. Tests: contract round-trip with the `#[serde(default)]` back-compat case;
   `clamped()` truncates the vec / message / count (the untrusted-input lock);
   repo insert/dedupe (`ON CONFLICT` on crash_id); buffer cap behavior;
   handler test for the unauthed crash route.

## Verification

- Unit/integration per step 5; the CI gate:
  `cargo clippy -p zwipe-core -p zerver --all-targets -- -D warnings`.
- Black-box against prod after deploy (per usual): force a 422 in the app and
  see the `client_errors` row; `panic!()` behind a debug-only trigger, relaunch,
  see exactly one `crash_reports` row across two relaunches.
- Confirm an old client (1.7.3) still posts usage batches successfully
  (`client_errors` defaults to empty).

## Non-goals / later

- Breadcrumb trails, device metadata, session replay (vendor-SDK territory).
- Pre-auth *handled* errors (login-screen failures are 4xx the server already
  logs; transport failures there are unreportable anyway).
- Native (non-Rust) crash capture — the stores' own consoles remain the only
  window into those.
- Store data-safety label review: confirm the labels' existing "diagnostics"
  disclosure covers this before ship; adjust if not.
