# 434. On a phone, a featured card floats on the first tap instead of opening, and the strip clips it

State: fixing | Owner: thinker | Type: issue | PR: - | Build seen: -

## Report
Composer for Scotty, 2026-11-12: in the phone app, featured cards (commander and MVPs) should open card details on the first tap. Unlike stack cards nothing covers them, so the float-first tap has no purpose, and the strip's frame cuts the floated card's top edge. Drop the tap-to-float step for featured cards on touch (the web's hover float stays), and make sure nothing else in the strip clips a card. Extended: on the web at phone width, a featured card on the owner's deck opens its details on one tap, as the app does (it opened the image).

## Fix
thinker:
- App: the featured strip's tap is `StackDetails::click`, which opens the details at once, and the strip no longer floats a card.
- Web at phone width (the deck cards screen): a featured tap goes through `featured_open` like the wide deck page, opening the details drawer, or the image for a card no listed row has. The tap stops at the card so the screen's tap-off doesn't close it.
- The strip's row (shared CSS) pads its clip by the card shadow's reach and gives it back with negative margins, so no card's shadow or edge is cut and the row sits where it did.

## Verification
- App and web at 390px, owner deck: tap the commander or an MVP once: its details open. A card with no row (filtered out) opens its image.
- Checked headless on the web at 390px (deck cards screen, fake session, writes failed): one tap on Sol Ring opened its drawer; the strip shows both cards whole with their shadows.
- The featured cards show their full shadow at the strip's top and side edges.
