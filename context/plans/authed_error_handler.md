# Centralized authed-call error handling (facade)

**Status: MIGRATION COMPLETE on the `authed-facade` branch 2026-09-06 (built
2026-09-04..06). ~60 sites converted across 22 files, net ≈ −900 lines of
ceremony; every remaining `ensure_fresh` caller is a sanctioned holdout (the
sweep grep is clean). Rides 1.10.1+ (build 79 / versionCode 42). Remaining
before merge: owner dead-backend pass over the deck screens (the profile
cluster passed 2026-09-05), then merge to main.**

**Final holdout list** (each carries an in-code comment):
- Infrastructure, by design: `session_upkeep`, `signal_logout`, `hint_dialog`
  (`open_and_record_hint`), `flush_loop`, and `ensure_session` itself.
- Variant-specific error copy: `email_verification` resend (downgrades
  `TooManyRequests` to an info toast, keeps the cooldown).
- **Staged compensation (6 sites)**: the swipe-undo arms in `add.rs` (3),
  `remove.rs` (2), and the maybeboard-promote undo — they compensate
  differently depending on whether the *refresh* or the *call* failed (full
  rewind vs deliberately leaving the action standing), which run/try_run
  collapses. Seven data points now argue for a future `run_staged` variant
  exposing the failure stage; design it from these real cases if the facade
  ever grows again.

Notable behavior upgrades made during the audit (the migration's other half):
printing-sheet saves were fully silent and now report + toast as
`change_printing`; load-more toasts (owner decision); the tokens fetch,
deck-context load, skip posts, and both typed-search dropdowns are now
*deliberately* quiet with telemetry instead of accidentally silent; deck
view/edit/export/list resources keep their `Result` types via `try_run` with
their duplicate error-watching effects deleted.

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
    screen: Screen,
}

/// Reads the contexts once. `screen` is the typed screen (owner decision
/// 2026-09-02: a hierarchical enum, e.g. `Screen::Auth(AuthScreen::Login)`,
/// `Screen::Deck(DeckScreen::CardAdd)`), passed at hook time so call sites
/// don't repeat it. The enum's `as_str()` MUST map to the existing `screen::`
/// vocabulary strings byte-for-byte — the wire format and `client_errors`
/// dedupe keys stay unchanged; the enum adds compile-time structure on top,
/// and room to carry more context later without touching the wire.
pub fn use_authed(screen: Screen) -> Authed;

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

    /// Telemetry only, no toast — for true background work where a toast
    /// would be noise (image/catalog prefetch, fire-and-forget flushes).
    /// Owner's default is the OTHER way: when in doubt, toast — a user who
    /// hits a dead end deserves a warning (see the load-more decision below).
    pub async fn run_quiet<T, Fut>(&self, op: &'static str, f: ...) -> Option<T>;
}
```

Call sites become:

```rust
let authed = use_authed(Screen::Profile(ProfileScreen::Main));
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
| deck/card/add.rs | 11 | load_more is silent today → gets a toast (owner decision below) |
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

## Dead-backend findings (owner smoke test, 2026-09-05)

Clicking through every screen against a dead backend after phase 2: profile,
deck list, and deck create all error loudly (converted + already-toasting
sites behave). Two pre-existing silent failures OUTSIDE the facade's scope
surfaced, tracked here so they aren't lost:

- **Catalog cache (otags / card roles): fails silently and never retries**,
  so pickers sit empty all session with no explanation. The cache stays a
  facade holdout, but it wants its own fix: an inline empty state in the
  pickers ("couldn't load, tap to retry"), not a toast.
- **Home flavor text quietly absent** on failure. Unauthed and decorative;
  probably fine, noted for completeness.

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

## Owner decisions (2026-09-02)

- **Load-more toasts on failure.** The silent-stops-growing behavior means a
  user reaches the end of the pile with no cards and no warning, so `add.rs`
  load-more converts with `run`, not `run_quiet`. The general lean: when in
  doubt, toast. `run_quiet` survives only for work the user never initiated
  and never waits on (prefetch, fire-and-forget flushes).
- **Typed screens.** `use_authed` takes a hierarchical `Screen` enum
  (`Screen::Auth(AuthScreen::Login)`-shaped) rather than the bare `&'static
  str` constants, capturing where a failure happened as structured data.
  Non-negotiable constraint: `Screen::as_str()` maps to the existing
  `screen::` vocabulary strings exactly, keeping the wire format and the
  `client_errors` dedupe keys continuous. Building the enum (one variant per
  existing constant, grouped auth/profile/deck/card) is a small prerequisite
  step before phase 1; the old constants can delegate to it during the
  transition.
