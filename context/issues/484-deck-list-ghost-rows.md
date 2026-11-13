# 484. The Decks list's loading ghost shows three decks on a wide screen

State: fixing | Owner: thinker | Type: polish | PR: - (pushed to release) | Build seen: bf8444026

## Report
Scotty, via the composer: the Decks list's loading skeleton shows 3 deck rows, which looks silly on a big screen. Show enough ghost rows to fill a wide view (at least 5), and keep phones as they are.

## Before
Headless Chrome, signed-in fake session, the deck list and tags requests held so the ghost stays, 3 of 3 at 1440x900: three ghost tiles in two columns (two, then one), 88px tall, from y 282 to 459, with the footer at about 790. At 390x844: three tiles in one column.

## Cause
`zwiper-web/src/lib/inbound/screens/deck/skeletons.rs` `DeckListSkeleton` draws a fixed three tiles at every width, while the list is one wide column from 900px and pairs its rows in two columns from 1400px (`web.css`).

## Fix
- The web ghost draws ten varied tiles (`DECK_LIST_GHOST_TILES`), each the live row's classes plus `skeleton-deck-tile`.
- `web.css` shows three below 900px (the phone, unchanged), five from 900px to 1399px (one column), and all ten from 1400px (five rows of two).
- The app's ghost (`zwiper/.../deck/components/skeletons.rs`) keeps its three: the app is the phone layout.
- Test: `the_decks_ghost_fills_a_wide_list_and_keeps_the_phone_s_three` (red with the phone rule broken).

## After
Same setup: at 1440x900, 3 of 3, ten tiles in five rows of two, from y 282 to 726, the page still one screen tall; at 1024x768, five tiles in one column; at 390x844, three, as before.

## Follow-up: the ghost page scrolls while it loads
surfer, after the 950ff3333 batch: at 1440x900 the ghost page is 14 px taller than the screen, so a scrollbar shows while Decks loads (3 px at 390).
- Before: headless Chrome, the list held, 3 of 3 at 1440x900: the page's scroll screen (`.web-scroll-screen`) is 833 px tall in 819, with ten tiles; at 1024x768 with five tiles and at 390x844 with the phone's three, it overflows too.
- Cause: the screen's content (`flex: 1 0 auto`) grows to hold every ghost row, pushing the footer down; no fixed row count fits every window height.
- Fix: while a ghost tile shows, the screen's content takes the room above the footer and no more (`flex: 1 1 0; min-height: 0; overflow: hidden`), cutting off the rows that don't fit. The counts stay three, five and ten.
- Test: `the_decks_ghost_fills_a_wide_list_and_keeps_the_phone_s_three` checks the rule (red without it).
- After: no scroll on the screen at 1440x900 (3 of 3), 1440x700, 1024x768 or 390x844; the last row shows cut off under the footer.
