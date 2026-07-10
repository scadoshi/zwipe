# Per-screen, per-deck filter persistence (zwiper)

**Status: SHIPPED-READY 2026-07-09, sim-verified by owner. Owner calls: in-memory only (restart
forgets), unbounded map (builders are tiny), Zwipe-select out of scope (a new
deck is a new context — no old filter should follow into it).**

**One sentence:** every swipe screen on every deck remembers its own filter —
independent contexts that persist for the session, replacing the shared
mutable filter and all its "move it across unless it looks like a default"
guesswork.

## Why

Today one app-global `Signal<CardQueryBuilder>` (provided in `session_upkeep`)
is shared by the Add, Remove, and Cards screens and all ~20 filter-UI modules.
Every cross-screen transition needs a hygiene hack: Add's Back button clears
the filter *if it looks like the screen default* (the land-target leak fixed
2026-07-08 was this guess going wrong), the Search↔Maybeboard source switch
does a clear/reapply dance, and the Cards screen toasts "Filter is active"
about a filter it never chose. Each screen has different use cases; their
filters do too. Persisting per context is simpler than guessing what the user
*would* want moved across.

## Design

- **`FilterStore`** (`screens/deck/card/components/filter_store.rs`, beside
  its precedent `add_stack_cache`): app-scoped context wrapping
  `Signal<HashMap<(FilterScope, Uuid), CardQueryBuilder>>` with
  `restore(scope, deck_id)` / `park(scope, deck_id, filter)`. In-memory,
  session-lifetime, no eviction.
- **`FilterScope`**: `Cards | Remove | AddSearch | AddMaybeboard`. Add's two
  sources become two scopes — the source toggle parks one and restores the
  other instead of mutating a shared filter.
- **Screens own their filter.** Each screen swaps `use_context()` for a local
  signal + provider:
  1. mount: `store.restore(scope, deck_id)` else *that screen's default*
     (Cards / Remove / AddMaybeboard: blank; AddSearch: blank, then the
     existing deck-load effect fills in deck context + auto-lands as today);
  2. `use_context_provider(local)` — Dioxus resolves contexts to the nearest
     provider, so the filter sheet and all ~20 filter modules bind to the
     screen's signal **unchanged** (`swipe_select` already proves this
     pattern);
  3. `use_drop`: park the final state back.
- The app-global `filter_builder` provider in `session_upkeep` is deleted
  once all three screens provide their own (all other consumers render under
  a screen or under `swipe_select`'s own provider).

## What dies

- Add Back-button "clear if default" (both the original and the 2026-07-08
  auto-lands fix — superseded: cross-screen contamination becomes
  structurally impossible).
- The Search↔Maybeboard clear/reapply dance (→ park/restore).
- Any future need for `is_empty_ignoring_…` default-detection on screen exit
  (the helpers stay for Reset and the filter dot, which are in-screen).

## What stays

- **Reset**: unchanged meaning — return *this* screen to *its* default; now
  also the user's exit from a remembered filter.
- **Filter dot**, `filter_reset_counter`, `last_search_filter` + add-stack
  parking (restoring the same filter on return makes parked stacks hit more
  often, not less).
- **"Filter is active" toast** on entering Cards — now truthful (it's the
  screen's own remembered filter); trivial to drop later if it annoys.

## Out of scope

- Zwipe-select (transient picker, keeps its own provider as today).
- Disk persistence (session-only by owner call).
- Eviction / deck-deletion cleanup (session memory, tiny values).

## Verification

Sim pass: filter on Add(deck A) → Cards shows its own (blank) filter → back
to Add(A) restores → Add(deck B) starts at fresh default → source toggle
round-trips both Add scopes → Reset still returns to default → land-target
deck: auto-lands default behaves, no leak into Cards.
