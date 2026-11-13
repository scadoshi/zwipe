# 466. On a phone, a card's details from Cards should be the side drawer, not a full screen

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: 31b3ab105

## Report
Composer for Scotty, 2026-11-13: in the phone app, a featured card in the Cards header opens its details full screen; first scoped to the featured strip, then corrected: every card's details on a phone (app and phone web) go back to the side drawer from the right, not full screen; 449's full-screen details is withdrawn, since he only wanted the Cards to feel like a route. The drawer dims the page (424's phone rule) and closes three ways: a tap on the dim, the ×, or a drag to the right (new, matching how bottom sheets drag down to close). 457's route feel for Profile and Cards stays.

## Cause
`components/src/deck_cards_view.rs`: `DeckStackDrawer` passed `screen_on_phone: true` (449) for every card, whatever opened it, so below 900px every details opened as a full screen over a transparent backdrop.

## Fix
scotland:
- The screen form is gone: `DrawerSheet` and `CardDetailsSheet` lose `screen_on_phone`, `filter.css` its rules, and 457's hiding of the Cards under the details (web `body.rs` and `cards_layer.rs`, app `deck/card/view.rs`) goes with it.
- orangui 9039b70: `SheetDrag` takes a `Pull`, `Down` or `Right`; a pull to the right follows the pointer's x, measures the sheet's width, and moves it through `sheet-dragging-right`. `DrawerSheet` pulls a drawer at every width to the right below the kit's modal width (where it floats along the right edge), by its title bar, as a bottom sheet pulls down.

## Verification
- Phone web (390) and the app: open a deck's Cards, tap a stack card twice (or a featured card, or an app List row): the details slide in from the right with the page dimmed.
- A tap on the dim closes it and opens nothing behind; the × closes it; dragging its title bar a little to the right springs it back, past a quarter of its width (or a flick) closes it.
