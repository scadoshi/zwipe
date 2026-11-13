# 437. At phone width the Examples controls wrap, the eye on a row of its own

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: production

## Report
Composer for Scotty (from surfer), 2026-11-13: at phone width the Oracle tag examples' controls wrap and the details eye drops to its own full-width row. Put them on one action bar: the direction buttons (Next, Back one), a vertical rule, then the eye, in the app's dock style.

## Fix
scotland:
- `SwipeDock` takes an optional `tail`: a surface's own one or two controls, drawn after an `action-dock-divider` inside the swipes' dock, so the set is one bar at every width. The examples pass their eye as the tail instead of a second dock.
- The narrow rules that leave room for a stacked pair (the floating details drawer's lift, the screen's 8.6rem of bottom room) apply only to a pair of two docks; a one-dock pair keeps the kit's offset and the 5rem room. Add, Remove and Commander swipe keep two docks.

## Verification
- Phone width (412): open a card's tag, then Examples: one bar at the bottom right with the two arrows, a rule and the eye.
- Wide (1440): one bar with Next, Back one, a rule and the eye.
- Add and Remove cards at phone width: their two docks still stack as before.
