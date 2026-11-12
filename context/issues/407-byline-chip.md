# 407. The shared deck's owner sits on its own line, not as a chip

State: fixing | Owner: laptop | PR: - (pushed branch, hash in the composer's log) | Build seen: a54f213ec

## Report
Scotty, on zwipe.net's shared deck page (the phase 4 page from 370): the "by scadoshi" line isn't a chip and sits on its own line under the title. It should be a chip right after the deck name on the same line, with the other chips (card count, format, power, commander, tags, price) on the next line.

## Fix
laptop: components `DeckHead` draws `byline` as `by <owner>` in the kit's muted stat chip (`stat-chip stat-chip-muted deck-head-byline`, the size of the other chips) in a `.deck-head-name` group with the title, centered on the title's line and wrapping under it only when the line runs out. The facts' chips keep their own row below. The app's deck page passes no byline, so its head is unchanged.

## Checked (headless Chrome, local shared deck "Krenko goblins" by laptop370)
- 1400: "Krenko goblins" then the "by laptop370" chip on one line, the six fact chips on the next.
- 390: the same, the fact chips wrapping onto two lines.
- app.zwipe.net's deck page at 1400, before and after: the head is pixel-identical, and the whole page differs by 1 pixel of antialiasing.

## Verification
-
