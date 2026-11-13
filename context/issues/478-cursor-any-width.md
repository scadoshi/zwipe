# 478. The deck page's keyboard cursor only works on the computer layout

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: production

## Report
Scotty, through the composer, 2026-11-13: the deck cursor's keys should work at any width when someone uses a keyboard (a narrow window, an iPad keyboard), with touch unchanged. Halo, ungating the page keys, notes the deck editor's narrow up/down row stepper is the cursor's narrow counterpart; there should be one behavior.

## Before
Headless, a deck's cards with a mouse and keyboard: at 800px j, j, ArrowDown, k move the cursor 0 of 4 in List and in Stacks; in List ArrowDown opens the first row instead (the old stepper), in Stacks it does nothing. At 1440 the same keys move it 4 of 4.

## Cause
- `components/src/deck_cursor.rs`, `NAV_JS`: the keys return unless `COMPUTER_QUERY` (a mouse with hover, from 900px) matches.
- `zwiper-web/src/lib/inbound/screens/deck/cards/editor.rs`: below that layout the arrows went to the older up/down row stepper (made to yield to the cursor on the computer layout by 440).

## Fix
scotland:
- `NAV_JS` drops the layout check: a key moves the cursor at any width, a keyboard's own proof; a touch never sends one, so touch is unchanged.
- The editor's row stepper is gone, so the arrows and `hjkl` are the cursor everywhere.

## Verification
- Headless, the same keys: at 800 4 of 4 in List and in Stacks, nothing opens a row; at 1440 4 of 4 as before.
- 390 with touch: a tap reveals, a second opens the drawer, as before; then j moves the cursor and the drawer follows it 2 of 2. The card the tap revealed stays revealed under the drawer.
