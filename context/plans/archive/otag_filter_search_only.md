# Oracle-tag filter — search-only (drop the curated grid)

**Status: DONE 2026-07-15 (implemented + runtime-tested, archived), decision FINAL.
Client-only (zwiper), scoped to one component.** The card filter's oracle-tag section stops showing a curated grid
of default tags up front. With ~4,500 tags and no non-arbitrary "common" cut, a
chip wall just dirties the screen. Users search and click, exactly like keywords,
oracle words, types, and artists already do.

**One sentence:** delete the `CURATED_ORACLE_TAGS` default grid from the filter's
`OracleTags` component and replace it with a plain selected-chips row (mirroring
the exclude section), keeping the search-to-add input as the only way in.

**Related:** the discovery story this leans on is the dictionary +
[`otag_example_cards.md`](otag_example_cards.md) — browse/learn there, search/apply
here. Component: `zwiper/.../inbound/screens/deck/card/filter/oracle_tags.rs`.

---

## Why

Consistency is the feature. Every other high-cardinality filter (keywords, oracle
words, types, artists) is search-and-click; oracle tags being the one exception (a
grid) is a special case users must relearn. And "common" over 4,500 tags is
editorial guesswork — any grid long enough to be useful is too long to scan; any
short enough to scan is arbitrary. Search scales; a chip wall doesn't. Discovery
for people who don't yet know what they want is the **dictionary's** job, not the
filter's. Owner's call: the selectors aren't worth the screen space. Final.

## Scope — the card filter ONLY

`CURATED_ORACLE_TAGS` (`zwipe-core/.../card/models/oracle_tag.rs:30`) has **two**
consumers:

1. `filter/oracle_tags.rs:111` — the card filter grid. **This is what we remove.**
2. `deck/components/oracle_tag_select.rs:75` — the deck *strategy-tag* picker
   (writes tags into the deck, a different feature). **Leave untouched.**

So the const stays in `zwipe-core` (still used by #2). Only the `filter/` component
stops importing and rendering it. Whether the deck strategy picker should also go
search-only is a **separate** decision, explicitly out of scope here.

## Plan (all in `filter/oracle_tags.rs`)

The catch: today the currently-selected include slugs are shown *inside* the grid
(`grid_slugs` appends `selected` at lines 116-120). Remove the grid and selected
chips would vanish. So this is "replace the grid with a selected-chips row," not
just "delete the grid." The exclude section (lines 238-261) is already exactly the
shape to copy: removable chips + search-to-add, no curated grid.

1. **Drop the import** of `CURATED_ORACLE_TAGS` (line 14); keep `OracleTag`.
2. **Delete `grid_slugs`** construction (lines ~109-120) and the **grid render**
   (lines ~149-172).
3. **Add a selected-include chips row** in its place, mirroring excludes
   (lines 238-261): for each slug in `selected`, a removable `chip selected` with a
   `chip-remove` "×" that rewrites via `write_selected(&mut fb, mode(), current
   without slug)`. Use `label_for` for display. This preserves the existing
   any/all `mode` toggle and the clear-all "×" in the label row (lines 127-146).
4. **Keep** the search-to-add block (lines 175-208) and its input (210-221) as-is.
   One tweak: its filter currently excludes `grid_slugs` (line 184) — drop that
   clause, keep the `!selected.contains` guard so already-picked tags don't re-list.
5. **Excludes section**: unchanged (already search-only).

Net: includes and excludes become structurally identical (selected chips +
search), which is also a small readability win.

## Verify

- Selecting via search still writes `oracle_tags_contains_any` / `_contains_all`
  correctly; any/all toggle still flips between them; clear-all still empties.
- A pre-existing selection loaded from `FilterStore` renders as removable chips
  (no grid needed to surface it).
- `filter_reset` still clears the search inputs (effect at lines 97-101).
- `cargo +nightly fmt` + clippy clean before any push.

## Not doing

- Not removing `CURATED_ORACLE_TAGS` from `zwipe-core` (deck strategy picker still
  uses it).
- Not touching the deck strategy `OracleTagSelect` picker.
- No backend/wire change — this is pure client render.
