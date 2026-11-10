# 335. Selector drawers waste space above the chips

State: reported | Owner: - | PR: - | Build seen: 7d3edf896

## Report
Scotty, 2026-11-10, side-drawer selectors (Format, Deck tags, Oracle tags): two things keep fewer chips on screen than there could be.

1. The chip list stops above the floating check (apply) button instead of running under it. Scotty is fine with chips scrolling under the button: more chips show, and a chip under it is one scroll away.
2. The hint box has a minimum height that is too large. With nothing selected it shows one line ("Tap a format to see its details here.") over a block of empty space. It should hug its text when empty, and grow up to its max height only when a selection gives it text to show.

Screenshots: `target/scotty-shots/335-format.png`, `target/scotty-shots/335-deck-tags.png` (scotland checkout).

## Repro
1. Open a deck's Format selector (or Deck tags, or Oracle tags) in the side drawer.
2. With nothing selected, the hint box is tall and mostly empty, and the chip list ends above the check button.
