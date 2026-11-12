# 379. Quick add runs wider than the chip rows under it

State: fixing | Owner: laptop | PR: #581 | Build seen: production, web

## Report
Composer for Scotty, 2026-11-12: on the web deck page, the Quick add field runs much wider than the chip rows under it. It should end exactly at the right edge of the widest control row (the Group by row, ending at Card role), done structurally so it follows if the chips change. Phone width keeps the field full width.

## Cause
The controls block took the page's width, and Quick add's field grew to the shared 32rem field cap regardless of the rows under it.

## Fix
laptop, #581, `web.css`, from the 900px breakpoint, scoped to `.deck-page > .deck-controls`:
- the block is `width: fit-content` (capped at 100%), so it is as wide as its widest chip row;
- Quick add's row (a new `.quick-add` class) has `contain: inline-size`, so it takes no part in that width and stretches across it;
- the field drops the 32rem cap there, so it runs to the block's right edge.

Below 900px nothing changes, and the deck cards screen's own Quick add is untouched.

## Verification
-
