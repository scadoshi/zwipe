# Completed Work — 2026 Q3

Archived from `context/progress/todo.md`. Everything below is shipped. Kept here so commit
hashes stay searchable.

---

## Bugs (fixed 2026-07-11 / 07-12)

- [x] **Printing shifting bug — FIXED 2026-07-11** (`900001bd`). The printing carousel drifted right as you swiped (worse on many-printing cards, reload cleared it): it translated the strip by `-index * page_width_px` off a one-time `window.innerWidth` read, so `index * (measured - actual)` error accumulated. Now positions by `-index * 100%` (percentage of the one-viewport-wide flex strip), which needs no measurement and can't drift. Device-verified on the 6.5" sim.
- [x] **Flip-card down-shift — FIXED 2026-07-11** (`02aab440`). Wasn't the button padding; the DFC-only `aspect-ratio: 5/7` wrapper rule made double-faced cards render a few px smaller/lower than single-faced ones. Now the button pins to an image-sized `.flip-face` (absolute, top-right) and DFC/single-face size identically. Sim-verified.
- [x] **Clone toast — DONE 2026-07-12.** Settled on `format!("Cloned as \"{name}\"")` (double quotes) at `clone_deck_dialog.rs:96`. The accent-1 colored-name idea was dropped: `dioxus_primitives` toast takes a plain `String`, and a custom Element toast body wasn't worth it — plain quoted text is fine.
- [x] **Keyword / Card-role reveal collapse — FIXED 2026-07-12** (`388a2274` KeywordChips, `ccb4d9b4` CardRoleChips). The shared `.keyword-reveal` (Keywords + Card roles) eased open but snapped shut on close: the reveal content was rendered only while open, so it was removed from the DOM before the grid-rows/opacity transition could play. Fixed by persisting the last-opened index in a separate `shown` signal (not cleared on close) so the content stays mounted through the collapse; the `open` class alone drives the animation both ways. Shared component, so app + zite both.
- [x] **Dual-color (hybrid) mana pip glyph misalignment — FIXED 2026-07-12** (`bb4bef05`). Hybrid/twobrid pips (`{2/W}`, `{W/U}`) rendered off-center on both zite + app. mana-font draws hybrids as two absolutely-positioned halves (`::before`+`::after`) tuned for its native border-less 1.3em box; our 1.5em inline-flex squircle broke the offsets. Fix: restore mana-font's box geometry for hybrids + `box-sizing: content-box` (border outside the offset reference). Verified on Reaper King. Detail: [`plans/hybrid_mana_pip_alignment.md`](../plans/hybrid_mana_pip_alignment.md) (shipped).
