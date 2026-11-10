# 328. Pip rings split on expanded and hovered card rows

State: fixing | Owner: thinker | PR: - | Build seen: live, DPR 1

## Report
Surfer, 2026-11-09, live web at DPR 1. On a deck's card list, an expanded row (and a hovered one, with a mouse) is scaled by 1.02 (`magic.css` `.card-row.expanded` and `.card-row:hover`). The scale puts every pip in the expanded cost on fractional pixels, so their 1px rings split across two. Also, the generic pip's ring is a fixed `#393835` on mana-font's `#beb9b2` fill, while colored pips derive their ink from their fill (OKLab l x 0.42), and on dark themes the generic ring reads as pulled in.

## Fix
thinker, on `nits/thinker-328`, `components/assets/magic.css` (shared by web and the app).
- Expanded and hovered rows no longer scale. They keep their tint, shadow and rounded corners, and the transform transition is gone with the scale. The other `scale()` transforms near pips (screen entrances, toasts, the modal entrance, the heat halo) are transient and end at `none`/`scale(1)`, so none stays on a row with pips.
- The generic pip's ink is derived from its fill like every other pip's: `--pip-fill` starts as mana-font's own `#beb9b2`, and the same 42% mix (or the relative-color formula where supported) gives the ink. Measured: the derived ink is `#373533` against the old `#393835`, practically the same color, so on dark themes the generic ring still sits close to the page color. The split ring was the visible defect, and the scale drop fixes it.
