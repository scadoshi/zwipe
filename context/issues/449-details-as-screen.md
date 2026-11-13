# 449. On a phone, a card's details from Cards slide in as a sheet instead of opening as a screen

State: withdrawn (466) | Owner: scotland | Type: feature | PR: - | Build seen: production

## Report
Composer for Scotty, 2026-11-13, testing the phone app: tapping a card on a deck's Cards (a featured card, a row, a stack card) opens its details as a side sheet that animates in. It should read as a route change: the details open as their own screen, no slide, no dim, Back returning to Cards where you were, with the grid tiles behind. With 448's part: the Cards layer gets the grid tiles too, and opens and closes with no slide either. Phone app and phone web the same; the wide web's drawer stays. Card keys, Undo, star, Fix and Printings keep working from the details.

## Fix
scotland (plan in the composer's log; no `?card=` route):
- `DrawerSheet`'s `screen_on_phone`, which the deck's card details drawer (`DeckStackDrawer`) passes: below the web's wide layout (899px, so the phone app and a phone's web) the drawer fills the window, comes and goes with no slide, corners or dim, holds its own still `GridCells` with the header and content clear over them, and has a back arrow in place of its ×. It is the same sheet with the same classes, on the overlay back stack with 431's history entry, so Android's back, the iOS edge swipe, the browser's Back and the arrow all close it onto Cards as left.
- 448's part: the Cards layer (the app's `CardsScreen`, the web's `DeckCardsLayer`) holds its own still `GridCells` in place of its solid fill, and its open and close run in a moment with nothing moving (the keyframes stay, since the closing one's end drops the layer).

- Withdrawn 2026-11-13 by Scotty (466): he wanted only the Cards themselves to feel like a route; a card's details on a phone go back to the side drawer from the right, which dims the page. The screen form (`DrawerSheet`'s `screen_on_phone` and its CSS) is gone; the Cards layer's grid tiles and instant open and close (448's part) stay.

## Verification
- Phone web (412) and the phone app: open a deck's Cards: there at once, grid tiles behind. Tap a stack card twice, a featured card, or a row in the app: the details fill the screen at once, tiles behind, a back arrow top left. Back (or the arrow) returns to Cards where you were; Back again closes Cards.
- From the details: star, the stepper, the board moves, Printings (over the details) and Undo work.
- Wide web: the drawer slides in from the right as before.
