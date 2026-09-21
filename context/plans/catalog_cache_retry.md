# Catalog cache fails silently with no retry

**Status: PLANNED 2026-09-21 (found in the 2026-09-05 dead-backend pass).
Presentation is undecided: see the two options below. Client-only.**

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
