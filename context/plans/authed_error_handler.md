# Centralized authed-call error handling (facade)

**Status: PLANNED (expanded 2026-09-01 from the 2026-07-15 backlog sketch;
owner wants it soon — "best to do it as soon as possible"). Not started. Do it
as its own PR, not folded into a feature. Any code lands as 1.10.1+ (build 79 /
versionCode 42) — 1.10.0 is in review.**

**One sentence:** replace the per-call-site `ensure_fresh` + hand-rolled error
handling with one thin authed facade so every authed request refreshes, reports
telemetry, and surfaces failures the same way — killing the "some screens
toast, some swallow" inconsistency at its root.

---

## Why (unchanged from the sketch, still true)

Every authed call today is ~20 lines of the same ceremony:

```rust
let session_val = match session.ensure_fresh(client).await {
    Ok(v) => v,
    Err(e) => {
        usage_buffer.peek().report_error(screen::X, component::NONE, "op", &e);
        toast.error(e.to_user_message(), ToastOptions::default().duration(...));
        return;
    }
};
match client().the_call(args, &session_val).await {
    Ok(t) => { /* per-site success handling */ }
    Err(e) => { tracing::warn!(...); report_error(...); toast.error(...); }
}
```

Hand-rolled ceremony means some sites do it wrong: `add.rs` load-more is
silent on refresh failure, the otag examples screen was silent until a local
toast patch, and every new screen copies the block again (the 2026-09-01
Universes Beyond work added four fresh copies). The copy-paste is where the
next silent-failure bug comes from.

The 401-interceptor alternative was considered and rejected in the original
sketch (it would inject app state into the dumb HTTP adapter); the facade
reuses `ensure_fresh` as-is and keeps the layering. That decision stands.

## The facade

New module `zwiper/src/lib/inbound/components/auth/authed.rs`:

```rust
/// One handle bundling everything the ceremony needs. Copy (all Signals +
/// the toast handle), so closures capture it freely.
#[derive(Clone, Copy)]
pub struct Authed {
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    usage_buffer: Signal<UsageBuffer>,
    toast: /* the use_toast() handle type — verify; it is Copy (moved into
              closures everywhere today) */,
    screen: &'static str,
}

/// Reads the contexts once. `screen` is the telemetry vocabulary constant,
/// passed at hook time so call sites don't repeat it.
pub fn use_authed(screen: &'static str) -> Authed;

impl Authed {
    /// The uniform path: ensure_fresh → run → on any Err, report_error +
    /// tracing::warn + error toast (3000ms), returning None. Ok(T) → Some(T).
    pub async fn run<T, Fut>(
        &self,
        op: &'static str,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Option<T>
    where
        Fut: Future<Output = Result<T, ClientError>>;

    /// Same reporting, but hands the Err back instead of swallowing it —
    /// for optimistic-update sites that must revert state on failure
    /// (dark-mode toggle, UB toggle, MVP star, printing swap).
    pub async fn try_run<T, Fut>(&self, op: &'static str, f: ...)
        -> Result<T, ClientError>;

    /// Telemetry only, no toast — for background work where a toast would
    /// be noise (prefetch, load-more top-ups, usage flush). The current
    /// silent sites become *deliberately* quiet instead of accidentally.
    pub async fn run_quiet<T, Fut>(&self, op: &'static str, f: ...) -> Option<T>;
}
```

Call sites become:

```rust
let authed = use_authed(screen::PROFILE);
...
if let Some(prefs) = authed
    .run("get_preferences", |c, s| async move { c.get_preferences(&s).await })
    .await
{ ... }
```

Design points, resolved:

- **`component` vocabulary:** `report_error` takes (screen, component, action).
  Default `component::NONE` inside the facade; an `Authed::with_component()`
  or a `run_at(component, op, f)` variant covers the handful of sites that
  set one (check the vocabulary module for actual usage before deciding —
  if nearly all sites pass NONE, one extra variant beats a wider signature).
- **Toast duration:** standardize 3000ms for errors (today's dominant value).
  Success toasts stay at call sites — they're per-site copy, not ceremony.
- **Auth failures:** `ensure_fresh` already clears the session on auth
  rejection and the AuthGate redirects. The facade still toasts (matches
  today's behavior); the redirect happens regardless.
- **Non-`ClientError` calls:** everything authed returns `ClientError` today;
  if a stray site doesn't, adapt it rather than genericizing the facade.
- **`ensure_fresh` stays public.** `session_upkeep`, `signal_logout`, the
  catalog cache, and the hint recorder have shapes that don't fit a
  screen-scoped facade; they keep the raw call. The facade is for screens.

## Inventory (2026-09-01: 66 sites, 25 files)

Counted by `ensure_fresh(client)` + `ensure_fresh(auth_client)` (same signal,
two local names — the rename is cosmetic):

| File | Sites | Notes |
|---|---|---|
| deck/card/view.rs | 12 | biggest win; several optimistic patterns → try_run |
| deck/card/add.rs | 11 | load_more is silent today → run_quiet, deliberately |
| deck/commander_maybeboard.rs | 6 | uses the `auth_client` name |
| deck/view.rs | 5 | |
| deck/card/remove.rs | 5 | |
| profile/mod.rs | 3 | two optimistic toggles → try_run |
| deck/components/more_buttons.rs | 3 | |
| profile/components/email_verification.rs | 2 | |
| deck/edit.rs | 2 | |
| deck/card/components/quick_add.rs | 2 | |
| 15 more files | 1 each | incl. the auth/profile change flows, import/export, create/list/clone, examples, preferences, universes_beyond, hint_dialog, session_upkeep, signal_logout |

`session_upkeep` and `signal_logout` are the two likely keep-as-is sites (see
above), so the migration target is ~64 sites.

## Migration plan

1. **Land the facade + convert one small screen** (`oracle_tag_examples.rs`,
   1 site — the screen whose bug motivated all this) in the same commit.
   Proves the shape end to end.
2. **Convert the profile cluster** (mod, preferences, universes_beyond,
   change_*, email_verification, delete dialog — ~11 sites) — exercises
   run, try_run (both toggles), and success-toast coexistence.
3. **Convert the deck screens** in descending size: card/view, add,
   commander_maybeboard, view, remove, then the 1–2 site stragglers.
4. **Sweep:** grep `ensure_fresh(` outside auth/ components must return only
   the sanctioned holdouts; add that grep to the conversion checklist.

Each phase compiles warning-free and is a separate commit; the old and new
patterns coexist safely throughout. During conversion, every currently-silent
site gets an explicit decision: `run` (toast) or `run_quiet` (deliberate) —
that per-site audit is half the value of the migration.

## Testing

The facade itself is UI-signal-bound, so no unit harness; correctness rides
on: (a) behavior parity per converted site — same toasts, same telemetry op
strings, verified by reading the diff against the old block; (b) keeping op
strings IDENTICAL to today's `report_error` action names so the client_errors
dedupe keys and any dashboards stay continuous; (c) an owner smoke pass per
phase (the phase-2 profile cluster is the easy one to eyeball). The existing
zerver-side tests are unaffected — this is client-only.

## Sizing

Facade: small (one module, ~150 lines with docs). Migration: the real cost,
~64 sites across 25 files — mechanical but each site needs the toast/quiet
decision and op-string check. Realistic as 2–3 sittings following the phases
above. Client-only → rides the next client release (1.10.1+); no server or
wire changes.

## Open questions for the owner

- Should the currently-silent `add.rs` load-more toast on failure after all,
  or stay quiet? (Plan assumes quiet: a background top-up failing mid-swipe
  with a toast would interrupt the gesture; the pile just stops growing and
  a retry happens on the next fetch anyway.)
- `use_authed(screen)` per screen vs a screen-less handle with `screen` passed
  per call: plan assumes per-screen (matches how `screen::` constants are used
  today, one per file).
