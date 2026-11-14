# 517. Shift+G leaves the cursor's card raised, its stack spilling over the Profile panels

State: fixing | Owner: thinker | Type: bug | PR: - (pushed to release) | Build seen: 37dbb9b58

## Report
Scotty, via the composer (web, deck page, wide): with a card picked by the keyboard cursor and raised out of its stack, Shift+G jumps to the bottom of the page but the card stays picked and raised, so its stack spills over the Profile panels below. Shift+G should let go of the card; check `g` `g` too and keep them consistent.

## Before
Headless Chrome at 1440x900, a fine pointer forced, the owner's deck page in Stacks with mocked reads, 3 of 3: `↓` twice raises Sol Ring (focused, `revealed`); Shift+G scrolls the page from 381 to 1432 and Sol Ring stays focused and raised.

## Cause
In Stacks a card the cursor focuses is raised through its focus (`components/src/card_stack.rs`: `onfocus` hovers it, which marks it `revealed`). orangui's page keys scroll the page and leave focus where it is, so the card stays raised; and the cursor's hand-back (`components/src/deck_cursor.rs`, `TRACK_JS`) focuses it again after any redraw while the last input was a key, so a blur alone would not hold.

## Fix
`g` `g` and Shift+G (with the page keys' `GG_WINDOW_MS` from orangui-core) let go of the cursor's card: it is blurred and the last input marked as not a key, so it stays let go. The cursor keeps the card's id, so the next arrow carries on from it. `g` `g` does the same as Shift+G: both move the page, not the cursor.

## Tests
- `a_page_jump_lets_go_of_the_cursor_card` (red before: the script had no such listener).

## After
Same setup, 3 of 3 each: Shift+G (381 to 1432) and `g` `g` (to 0) leave no card focused or raised; ArrowRight afterwards carries on from Sol Ring to the next column's card and brings it into view (2 of 2 each).

## Reopened
Scotty, via the composer: with real key presses, Shift+G still left the arrowed card raised, focused and outlined; `g` `g` cleared it.

Cause: the listener compared the raw `KeyboardEvent.key`, while orangui's page keys read a letter's case from Shift alone (`PRESSED_KEY`). A Shift+G whose `key` arrives lowercase (Caps Lock, some layouts) scrolled the page to the bottom yet read as a first `g` here, so nothing let go; with Caps Lock, `g` `g` arrives as `G` and let go on the first press, which is why it looked fine.

Fix: the listener reads the key through `PRESSED_KEY`, as the page keys do. Test `a_page_jump_reads_the_key_as_the_page_keys_do` (red before).

Check: headless Chrome at 1440x700, reduced motion so the page jumps at once, the deck page in Stacks over mocked reads, trusted CDP key events, `↓` twice then Shift+G sent as `key: 'g'` with Shift: before, the page went to the bottom with the card still focused, raised and outlined; after, 3 of 3, no card focused or raised. Shift+G as `G`, `g` `g`, and `G` `G` without Shift (Caps Lock) also let go, and so does Shift+G with the mouse resting over the stacks.
