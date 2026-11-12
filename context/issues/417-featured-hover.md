# 417. The deck head's featured cards don't answer the mouse

State: fixing | Owner: laptop | PR: - (pushed branch, hash in the composer's log) | Build seen: 52da46ebc

## Report
Composer, for the web deck page's featured card strip (the commander and MVP cards in the shared `DeckHead`, so the read-only share route gets it too): on hover the card should lift a few pixels, grow about 3 to 4 percent and sway gently (a slow rotate of a degree or two) while hovered, then ease back on leave. Pointer devices only, nothing under reduced motion, transform only, smooth in Firefox.

## Fix
laptop: `.deck-head-card-image` (the image's box inside `.deck-head-card`, whose deal-in animation owns that element's `transform`) uses the individual `translate`, `scale` and `rotate` properties. Under `(hover: hover) and (pointer: fine)` with no reduced motion, hover lifts it 0.3rem and scales it to 1.035 over 0.3s, and it sways up to 1.25 degrees on a 4.5s pendulum. The turn is a phase (a registered number the sway animation runs from -1 to 1, paused off hover) times a reach (a registered number that eases 0 to 1 over 0.45s), so on leave the card rights itself from wherever the sway was, and a new hover picks the sway up from rest.

## Checked
- Chrome (headless, app.zwipe.net local, "Krenko goblins" head): hover eases the lift and scale in over 0.3s and the turn in from 0; on leave from 1.2 degrees the turn eases to 0 in about 0.4s with no snap; with reduced motion emulated nothing moves.
- Firefox 160 (headless over WebDriver BiDi, a page with the same CSS): the same curves; on leave from 0.97 degrees it eases to 0 in about 0.4s, and a new hover eases in from 0. A first try that put the reach in the keyframes snapped in Firefox, hence the phase and reach split.
- Frame strip and a short clip of the hover and leave sent with the hash.

## Verification
-
