# Plan: client error + crash reporting

**Status: planned (2026-07-28).** Server currently logs every error it produces
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
    pub screen: String,         // e.g. "deck_edit" — closed vocabulary, client-side consts
    pub action: String,         // e.g. "save_profile" — what the user was doing
    pub kind: String,           // ClientError variant + ApiError variant, e.g. "api_unauthorized", "decode", "network"
    pub message: String,        // truncated ~300 chars; 4xx messages are user-safe by contract
    pub count: u32,             // dedupe within a flush window
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

- `UsageBuffer` gains an errors map keyed `(screen, action, kind, message)` →
  count, capped at ~20 distinct entries per flush window (beyond the cap,
  increment nothing — an error loop must not grow memory or spam).
- New helper on the buffer: `report_error(screen, action, &ClientError)` —
  derives `kind`/`message`, truncates, increments.
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
  (`id, received_at, client_version, platform, screen, action, kind, message, count`)
  and `crash_reports` table
  (`crash_id PK, received_at, occurred_at, client_version, platform, message`).
  `INSERT ... ON CONFLICT (crash_id) DO NOTHING` gives the exactly-once store.
- `record_usage` handler/service/repo extended to insert `client_errors` rows
  in the same transaction as the counters.
- New `record_crash` handler (unauthed, mirroring `record_anonymous_event`),
  body-size-limited and rate-limited like its sibling — it's an unauthed write
  path; also re-truncate `message` server-side before insert (client-side
  truncation is untrusted).
- Retention: add a prune step to zervice (delete rows older than ~90 days)
  alongside `delete_expired_sessions`. Note this makes zervice 6 steps — the
  `step N/5` log labels (and the "all 5 steps ok" summary) renumber to `N/6`;
  cosmetic but the labels are read by eye in prod logs, keep them consistent.

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

- Unit/integration per step 5; `cargo clippy --workspace --all-targets -- -D warnings`.
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
