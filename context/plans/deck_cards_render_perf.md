# Deck-cards screen — cheap render-perf wins

**Status: Phase 0 SHIPPED 2026-07-15. Phase 1 tried and REVERTED same day (it
blanked on fast scroll). Client-only (zwiper), scoped to
`zwiper/.../inbound/screens/deck/card/view.rs` + `zwipe-components` CSS. No
backend, no schema, no cap change.** Phase 0 alone made a 500-card deck snappy on
the iOS simulator; the only residual is a ~1s one-time layout on first open,
judged acceptable for the niche large-deck case. True virtualization stays out of
scope (see Parked).

**One sentence:** killing the O(n²) per-render lookups (Phase 0) was the real
win; the WebView's own compositing handles scroll fine once the quadratic is
gone, so the extra `content-visibility` layer (Phase 1) was removed for causing
blank rows on fling.

**Related:** cap decision lives in `zerver` `MAX_CARDS_PER_DECK = 500`
(`zerver/src/lib/domain/deck/mod.rs`); staying at 500. This work makes a
*possible* future raise safe, but the raise is not part of it.

**Commits:** Phase 0 + Phase 1 landed together (`perf(deck): index deck entries
for O(1) row lookups; skip off-screen rows`), then Phase 1 reverted (`perf(deck):
drop content-visibility from card rows (blanked on fast scroll)`).

---

## Why

The deck-cards screen renders every active card as its own `CardRow` component in
a plain `for` loop, with no windowing. Two costs compound as the card count `N`
grows:

**Layer A — Rust/Dioxus side.** Per card, the render did 3–4 linear scans of
`deck_entries`:

- `qty_for(card_id)` → `.find()`
- board lookup → `.find()`
- mvp lookup → `.find()`
- per-group `qty_count` sum called `qty_for` again per card

That is O(N²). At N=500 it is already ~1M scans per render; at N=1,000 it is ~4M.
It re-ran on every re-render (qty change, filter toggle, group-by switch), not
just first paint. **This was the actual bottleneck.**

**Layer B — WebView side.** Dioxus mobile renders into a WebView, so every row is
a live DOM subtree the browser must lay out and paint. `CardRow` is text-only
(mana cost, type, keywords); card images load lazily on tap via `ImagePreview`,
so the cost is DOM node count, not image bandwidth. In practice, once Layer A was
fixed, normal compositing (paint only the visible tiles) handled scroll without
help — see Phase 1.

---

## Phase 0 — index `deck_entries` once (SHIPPED)

Added a `Copy` `DeckRowMeta { qty, board, mvp }` and a `row_meta` `use_memo`
keyed on `deck_entries` that indexes entries by printing id (`scryfall_data.id`),
first-occurrence-wins to match the old `.find()` semantics. The row render now
does O(1) map lookups for qty / board / mvp, and the per-group qty sum reads the
same map. O(N²) → O(N) build + O(1) per row. Behavior-preserving, no visual
change. The memo recomputes only when `deck_entries` changes, and since
qty/board/mvp edits already go through `deck_entries.write()`, rows refresh on the
same trigger as before.

**Result:** 500-card deck (`Render Stress 500` on the local test account) scrolls
smoothly; qty/board/mvp edits stay cheap. This was the whole win.

## Phase 1 — WebView skip off-screen rows (TRIED, REVERTED)

Added `content-visibility: auto` + `contain-intrinsic-size: auto 2.1rem` to
`.card-row` (excluding `.expanded` so paint containment couldn't clip its
scale/shadow). It worked, but on a **fast fling the rows scrolled into view
before the compositor painted them**, showing blank reserved-height rows. The
render-ahead margin of `content-visibility: auto` is not author-controllable from
CSS, so there is no knob to widen the buffer while keeping it.

Reverted because: Phase 0 removed the real (quadratic) cost, and without
`content-visibility` the WebView lays out all 500 rows once and normal
compositing paints only the visible screenful per frame — smooth scroll, **no
blanking**. The only tradeoff is a ~1s one-time layout when the 500-card deck
first opens, which is acceptable. `content-visibility` earns its keep at
thousands of heavy rows, not 500 cheap text rows.

---

## Parked (not doing unless a real deck size forces it)

- **Progressive chunked mount (preferred follow-up).** If the ~1s first-open ever
  bugs users, this is the lightest fix: a `render_budget: Signal<usize>` starting
  small, grown in chunks each tick by a `use_future`/coroutine (yielding so the
  browser paints between chunks) until it reaches the full row count; the render
  slices the flattened rows to the budget. First paint is near-instant, the rest
  streams in. Unlike `content-visibility` it keeps every mounted row in the DOM
  (no blanking); unlike windowing it never unmounts (no scroll tracking). One
  wrinkle: reset the budget only on **structural** changes (filter / group-by /
  board), not qty bumps, or a `+1` re-streams and flickers — reuse the existing
  `should_collapse_expanded` structural-vs-qty signal to gate the reset.
- **True virtualization / windowing.** Render only near-viewport rows via scroll
  detection (IntersectionObserver sentinels, not scroll-distance-to-index math).
  Fixes both layers and scales to thousands, but the correctness surface (grouped
  sections, expand-in-place rows, scroll anchoring, filter/group-by resetting the
  window) is not worth it for one niche large-deck user. Prefer chunked mount
  above first.
- **Raising `MAX_CARDS_PER_DECK`.** Cap stays at 500. This work makes a future
  raise cheaper to justify, but does not perform one.
