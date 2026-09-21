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

## Two shapes, still to choose

Because the eight public catalogs are prefetched **together** at launch, a
failure is normally correlated: offline at startup means every picker is
empty at once, not one of them.

**A. Inline empty state per picker** (the 2026-09-05 call). "Couldn't load,
tap to retry" inside the picker that is empty, calling that catalog's
`ensure_*`. Closest to where the user is looking, and retries only what is
missing. Downside: the common case shows the same message in up to eight
places.

**B. One toast with a retry action** (owner, 2026-09-21). One affordance for
a failure that is usually app-wide.

On whether B is possible: `ToastOptions` supports `permanent(bool)`, so a
toast can stay until dismissed rather than auto-expiring. It does **not**
support a custom action button; the primitive takes title, description, type,
duration and permanent, and its only interactive element is close
(`dioxus-primitives` toast, pinned rev `02801f27`). A retry button therefore
needs either a small extension to the primitive or a bespoke toast-shaped
element of our own. Not free, but not large.

A third possibility worth weighing: since the failure is app-wide, the
affordance could live where the app already reports app-wide state rather
than in either the pickers or a toast.

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
