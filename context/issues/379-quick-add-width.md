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

The label chip also sat off its field's center: the shared `filter.css` gave a compact field in a clear wrap 0.5rem under it, which made the row taller than the field, so the row's `align-items: center` centered the label on the taller box. A compact field inside a chip row now takes no bottom margin, so label and field share a center line, and `.quick-add-results` keep the 0.5rem gap with a top margin. The sheet is components', loaded by the app too, whose deck cards Quick add (and the Commander maybeboard's) has the same chip row and compact field, so the app gets the same fix.

## Verification
-
