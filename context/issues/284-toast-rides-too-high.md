# 284. Toasts sit far above the footer on short pages

State: merged | Owner: scotland | PR: #429 | Build seen: 1e03520ef

## Report
Scotty, app.zwipe.net Settings in a small browser window (screenshot): with the footer in view, the toast stack's bottom edge sits about 140px above the footer's top border. Toasts belong at the bottom left as low as they can go, stopping just above the footer (the same gap as the docks), never riding higher. Suspect: web.css's footer-rise animation on `.toast-container` (animation-range entry 0 to 100vh) overshoots on short pages; #383 only turned it off while a sheet is open. Repeated toasts in the shot are deliberate.

## Repro
1. Open app.zwipe.net/settings at 390 wide on a page short enough that the footer shows.
2. Toggle Universes Beyond to make a toast.
3. Compare the toast's bottom edge to the footer's top border.

Surfer, live v1.18.0 in Chrome (bundle dxhb2b58944e4504c50, hard reload): the toast stack stays fixed and never moves for the footer. 390x700: 72px from the bottom; with the page at the top the gap to the footer is +22px, scrolled to the bottom it's -155px (the toast covers Privacy policy, Community, Contribute and the Scryfall line). 1200x600: 16px from the bottom, gap -92px (covers the footer's left side and credits). Shots on halo: ~/Developer/surfer-shots/284-390x700-top.png, 284-390x700-bottom.png, 284-1200x600.png.

So the footer-rise misbehaves both ways: too high in Scotty's browser, not at all in Surfer's Chrome. The fix should not depend on `animation-timeline` support: measure the footer's top against the viewport (or similar) and keep the stack just above it, otherwise at the bottom left.

## Fix
scotland, #429: the stack rests 16px above the footer, measured by JS from the footer's top (no animation-timeline); #383's sheet behavior kept. Second cause: a closed sheet form's dock tripped base.css `body:has(.action-dock)` and lifted the stack 72px on Settings; web now ignores docks inside closed sheets. Shots at 390x700 and 1200x600, top and bottom. Verify: Surfer and Droid repeat the Repro at both sizes, top and bottom.
