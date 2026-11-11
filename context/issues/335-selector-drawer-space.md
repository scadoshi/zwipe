# 335. Selector drawers waste space above the chips

State: fixing | Owner: laptop | PR: - | Build seen: 7d3edf896

## Report
Scotty, 2026-11-10, side-drawer selectors (Format, Deck tags, Oracle tags): two things keep fewer chips on screen than there could be.

1. The chip list stops above the floating check (apply) button instead of running under it. Scotty is fine with chips scrolling under the button: more chips show, and a chip under it is one scroll away.
2. The hint box has a minimum height that is too large. With nothing selected it shows one line ("Tap a format to see its details here.") over a block of empty space. It should hug its text when empty, and grow up to its max height only when a selection gives it text to show.

Screenshots: `target/scotty-shots/335-format.png`, `target/scotty-shots/335-deck-tags.png` (scotland checkout).

## Repro
1. Open a deck's Format selector (or Deck tags, or Oracle tags) in the side drawer.
2. With nothing selected, the hint box is tall and mostly empty, and the chip list ends above the check button.

## Fix
- Chips under the check button: on the web the picker drawer reserved a 5rem band at its foot (`.web-picker-sheet` content padding) and faded the grid out above it, so the chip list stopped short of the checkmark. The band and the fade are gone: the grid now scrolls right down to the drawer's foot and under the checkmark, and its own trailing padding (`--content-pad-bottom: 5rem`) scrolls the last row clear of it. The app's picker screens already ran the grid under the dock this way.
- Hint box: the Format bar had a fixed 11rem height. It now hugs the hint while nothing is picked and takes its 11rem only once a format's details fill it (`:has(.tag-def-list)`), so after the first pick the next format's details still don't push the chips under the finger. The Deck tags and Oracle tags bars drop their 3.5rem minimum and hug their text. App and web share the same rules (`components/assets/screens/screens-b.css`, `zwiper-web/assets/sheets/07-screens.css`).
- CSS only, so there is no unit test; the check is the layout, measured in a headless Chromium harness built from the real stylesheets: empty Format bar 176px before, 68px after; with a format picked, 176px both; the web grid's bottom moved from 80px above the drawer's foot to its foot.
