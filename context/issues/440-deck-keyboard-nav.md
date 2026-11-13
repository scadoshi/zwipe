# 440. The deck page's cards can't be reached from the keyboard

State: fixing | Owner: scotland | Type: feature | PR: - | Build seen: production

## Report
Composer for Scotty, 2026-11-13: keyboard navigation of the web deck page's cards in List and Stacks, owner and shared: the arrows and `h` `j` `k` `l` move a cursor between cards, Enter or `d` opens the card, and the card keys act on the cursor card. Plan first (`context/plans/deck_keyboard_nav.md`); Scotty took the five recommended picks: edges stop, left and right do nothing in a one-column List, the details drawer follows the cursor, featured cards are not stops, Home and End only while the cursor shows.

## Fix
scotland:
- `zwipe_ui::deck_cards::cursor`: where a move lands, from a snapshot of the cards as drawn (columns, each card's vertical center and whether it is in view): up and down within a column, stopping at its ends; left and right to the nearest card by center in the next column with one; Home and End; the first card in view with no cursor; the next card, else the one before, when the cursor's card leaves.
- `components::deck_cursor`, called from `use_deck_hover_keys` so the owner's page and a shared deck both have it: a key listener under its own guard (no typing, no Ctrl, Alt or Meta, no modal but the details drawer, no image preview, a mouse with hover only) gathers the drawn cards, Rust picks the target, and the browser focuses its line or button and scrolls it into view. Enter is each card's own (a row expands, a stack card opens its drawer); the drawer, while open, swaps to each new cursor card.
- The keyed card (`KEY_CARD_JS`): the drawer's card, then the focused card when a key moved last, then the hovered card, then the open row. A mouse move that moves nothing (the page scrolling under it) is not mouse input.
- A redraw that drops focus to the page while a key moved last (Undo, a removed card) puts focus back on the cursor card, or the next one.
- The shortcuts sheet lists the moves and Enter; the note reads "Move to a card, or hover it, then press its key".

## Verification
Surfer's 28 tests in the plan's section 7.
