# 491. In Stacks, revealing a tucked card takes a Back step

State: fixing | Owner: laptop | Type: issue | PR: - (pushed branch, hash in the composer's log) | Build seen: release 950ff3333

## Report
Scotty, via the composer: in Stacks, revealing a tucked card (the first tap) must take no browser history step. Back should only ever close or leave something that shows: an open details drawer, sheets, Cards, the page. A drawer opened over a revealed card keeps its own step. Applies to the app and phone web, with Android back and the browser's Back.

## Before
surfer (`before-491`, 950ff3333, 390 frame with touch emulation, Stacks, tucked Arcane Signet), 3 of 3 each:
- **A:** one tap reveals and pushes an entry; Back clears the reveal only and stays in Cards.
- **B:** two taps open the drawer; Back 1 closes the drawer, Back 2 clears the reveal.
- **C:** one entry per reveal session across cards.

Mine (local web, shared Atraxa deck, 390 touch), 3 of 3: the reveal takes `history.length` from 3 to 4, and Back only clears it.

## Cause
`components/src/deck_cards_view.rs`, `DeckStacks`: in tap mode it registered the reveal on the overlay back stack (`use_overlay_back_action(revealing, dismiss)`). That gives the reveal a web history entry (`overlay_steps`), and makes the OS back on the app clear it first.

## Fix
laptop: the registration is gone, so a reveal takes no step. The details drawer keeps its own step, as before. The test that pinned the reveal's step now pins that it takes none (`a_reveal_takes_no_back_step_and_only_the_details_drawer_takes_one`); it failed at depth 1 before the change.

## Checked
Local web at 390 with touch, same deck:
- **A, 3 of 3:** the reveal leaves `history.length` at 3, and Back leaves Cards.
- **B:** the second tap opens the drawer (3 to 4); Back 1 closes it, Back 2 leaves Cards.

## Verification
-
