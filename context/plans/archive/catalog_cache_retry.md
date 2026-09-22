# Catalog cache fails silently with no retry

**Status: DONE 2026-09-22 (`68b4444a`). Rides 1.10.2.**

Shipped as a toast with no button, fired once per sheet opening when any
public catalog is `Failed`: "Filter lists didn't load, trying again".

No retry button, and no auto-retry loop, because neither was needed. Each
picker's own effect calls its `ensure_*` on mount and that refetches a failed
cell, so **opening the sheet already is the retry** and the wording is true at
the moment it appears. A button would have duplicated what just happened on
its own.

The check lives in the sheet, not the pickers, so one failure gives one
message rather than eight. Deck tags are excluded from it: they warm only once
a session exists, so an unloaded deck-tag catalog is ordinary rather than a
failure.

Left alone deliberately: `deck/card/view.rs` reads the oracle-tag catalog for
row note chips with no failure branch. A missing chip is invisible rather than
wrong, and the user has no action to take, unlike an empty picker that reads as
a real "no matches" answer. The oracle-tag dictionary already had its own
failure toast and keeps it.

**One sentence:** when a filter catalog fails to load, the picker shows an
empty list that is indistinguishable from "no matches", and nothing retries
until the app is killed.

## What actually happens

`CatalogCache` prefetches eight public catalogs at launch (artists, sets,
keywords, keyword reminders, oracle words, card types, card roles, oracle
tags) plus deck tags once a session exists. Each lives in a `CatalogSlot`
holding a `CatalogCell`: `Loading`, `Loaded`, or `Failed`.

Consumers read it as `cell().read().loaded()`, and `loaded()` returns `None`
for **both** `Loading` and `Failed`. Every picker then falls back to an empty
list (`unwrap_or_default()`, or `unwrap_or(&[])` in `oracle_tags.rs:102`). So
a failed fetch renders exactly like a successful fetch that found nothing.
The user sees an empty picker with no explanation.

## The retry already works

Worth knowing before designing anything: **no cache-layer work is needed.**
`CatalogSlot::refresh` decides `needs_fetch` with

```rust
match &*current {
    CatalogCell::Loaded { .. } => current.is_stale(),
    _ => true,          // Loading or Failed
}
```

so calling `ensure_artists(client)` again on a `Failed` cell refetches, and
single-flight already stops duplicate calls. The `ensure_*` functions are
idempotent retry entry points that exist today. All that is missing is
something in the UI that calls one.

## Shape: one app-level affordance (owner, 2026-09-21)

The eight public catalogs stay cached together. They also fail together, and
a failure means something has gone badly wrong (backend unreachable, offline
at launch), not that one list is missing. So the affordance is one
app-level thing, not an empty state repeated in up to eight pickers.

On whether a toast can carry a retry button: `ToastOptions` supports
`permanent(bool)`, so a toast can stay until dismissed rather than
auto-expiring. It does **not** support a custom action; the primitive takes
title, description, type, duration and permanent, and close is its only
interactive element (`dioxus-primitives` toast, pinned rev `02801f27`). A
retry button needs either a small extension to the primitive or a
toast-shaped element of our own. That is the main build cost here, since the
retry itself is free.

Still open: whether it is a toast or the place the app already reports
app-wide state. Decide when building.

## Notes for whoever builds it

- Distinguish `Failed` from `Loading` at the consumer. Today both collapse to
  `None`; the picker needs to tell "still loading" (skeleton) from "failed"
  (retry affordance) from "loaded and genuinely empty" (no matches).
- `CatalogCell::Failed` carries no error. If the copy should distinguish
  offline from server error it needs to carry the `ClientError`; if the copy
  is generic, leave it as is.
- Deck tags are the one authed catalog and warm only once a session exists, so
  "empty" there can legitimately mean "not signed in yet".
- Client-only, and UI, so it rides a client build and waits for a visual
  review before committing.

## Verification

Reproduce by launching with the backend unreachable (point `.env` at a dead
port), open a filter sheet, confirm the picker is empty with no explanation.
After: the failure is visible and one tap repopulates it without restarting
the app.
