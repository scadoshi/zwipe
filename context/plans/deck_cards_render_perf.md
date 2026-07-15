# Deck-cards screen — cheap render-perf wins

**Status: PLANNED (not started). Client-only (zwiper), scoped to
`zwiper/.../inbound/screens/deck/card/view.rs`. No backend, no schema, no cap
change.** Two low-risk phases that flatten the deck-cards render cost so the
screen stays snappy as decks grow. True virtualization is explicitly **out of
scope** (see Non-goals).

**One sentence:** kill the O(n²) per-render lookups (Phase 0) and let the WebView
skip painting off-screen rows (Phase 1), so a fat deck renders cheaply without
rewriting the list into a windowed/virtualized component.

**Related:** cap decision lives in `zerver` `MAX_CARDS_PER_DECK = 500`
(`zerver/src/lib/domain/deck/mod.rs`); staying at 500 for now. This plan is what
makes a *possible* future raise safe, but the raise is not part of it.

---

## Why

The deck-cards screen renders every active card as its own `CardRow` component
in a plain `for` loop (`view.rs:976`), with no windowing. Two costs compound as
the card count `N` grows:

**Layer A — Rust/Dioxus side.** Per card, the render does 3–4 linear scans of
`deck_entries`:

- `qty_for(card_id)` → `.find()` (`view.rs:195`)
- board lookup → `.find()` (`view.rs:981`)
- mvp lookup → `.find()` (`view.rs:984`)
- per-group `qty_count` sum calls `qty_for` again per card (`view.rs:968`)

That is O(N²). At N=500 it is already ~1M scans per render; at N=1,000 it is
~4M. It re-runs on every re-render (qty change, filter toggle, group-by switch),
not just first paint.

**Layer B — WebView side.** Dioxus mobile renders into a WebView, so every row
is a live DOM subtree the browser must lay out and paint, on-screen or not.
`CardRow` is text-only (mana cost, type, keywords); card images load lazily on
tap via `ImagePreview`, so the cost is DOM node count, not image bandwidth.

Neither cost is a problem at real deck sizes today, but both scale badly and
both are cheap to fix. This is good-practice cleanup, done now while the code is
fresh, not a response to a reported complaint.

---

## Phase 0 — index `deck_entries` once (kills the O(n²))

Build an `id → DeckEntry` (or `id → &DeckEntry`) map with a `use_memo` keyed on
`deck_entries`, and read qty / board / mvp from it instead of scanning:

- Replace `qty_for` (`view.rs:195`) with a map lookup.
- Replace the two per-card `.find()` calls (`view.rs:981`, `view.rs:984`) with
  map lookups.
- Precompute per-group qty totals in the same memo (or a sibling memo) so the
  render stops re-summing via `qty_for` at `view.rs:968`.

Turns O(N²) into O(N) build + O(1) per-row reads. Contained to `view.rs`,
behavior-preserving, no visual change.

**Done when:** all per-row deck-entry access goes through the memoized map; no
`.find()` over `deck_entries` remains inside the row loop.

## Phase 1 — let the WebView skip off-screen rows

Add `content-visibility: auto` plus a `contain-intrinsic-size` estimate to the
card-row (and/or card-group) CSS. The WebView then skips layout and paint for
rows outside the viewport while keeping them in the DOM, so scrolling reveals
them with no Rust-side bookkeeping. Handles our variable-height rows (they expand
on tap) gracefully via the intrinsic-size estimate.

Supported in both iOS WKWebView and Android WebView at our min versions.

**Done when:** off-screen rows are skipped by the compositor (verify via a fat
test deck feeling flat to scroll), with no regression to expand-on-tap,
jump-to-card, or group headers.

**Note:** Phase 1 only addresses Layer B. Layer A (VNode build) still touches
all rows; Phase 0 is what keeps that cheap. Do both.

---

## Non-goals (parked / archive for now)

- **True virtualization / windowing** — rendering only near-viewport rows on the
  Rust side via scroll detection. It would fix both layers and scale to
  thousands, but the correctness surface (grouped sections, expand-in-place
  rows, scroll anchoring, filter/group-by resetting the window) is not worth it
  for a single niche large-deck user. Phase 0+1 should make 500 snappy and 1,000
  tolerable at a fraction of the cost. Revisit only if a real deck size makes
  the screen janky after 0+1.
- **Raising `MAX_CARDS_PER_DECK`** — cap stays at 500. This plan makes a future
  raise cheaper to justify, but does not perform one.

---

## Sequencing

Phase 0 and Phase 1 are independent and can land in either order or the same
change. Phase 0 is the higher-value one (fixes the quadratic cliff); Phase 1 is
near-free polish on top. Measure on a deliberately fat deck (~500 cards) before
and after.
