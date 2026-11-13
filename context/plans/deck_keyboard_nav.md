# Keyboard navigation of the deck page's cards (440)

Status: v1 built 2026-11-13 with Scotty's five picks (the recommendations under Decisions). Owner: scotland. Two changes from the plan as written: every card was already a Tab stop (List lines are `role=button` with `tabindex=0`, stack cards are buttons), so v1 moves focus between them and keeps that Tab order instead of a roving `tabindex`; and up and down read the order from the cards as drawn (columns in page order, folded groups out) rather than rebuilding it from the sections, so they never disagree with the screen. The decisions stay in `zwipe_ui::deck_cards::cursor`; the browser side is `components/src/deck_cursor.rs`.

The web deck page (owner and shared, List and Stacks) gets a keyboard cursor: the arrow keys and `h` `j` `k` `l` move it between cards, Enter or `d` opens the card under it, and every card key (`p` `i` `v` `m` `s` Shift+S `-` `+`) acts on it as it acts on a hovered card today. A mouse user sees nothing new; a keyboard user can work the whole page without touching the mouse.

## 1. The cursor and the hovered card

**One keyed card.** The page already has one card the keys act on (431's rule): the card the details drawer shows, else the card under the mouse, else the open List row. The cursor slots into that order as "the card under the mouse or the cursor, whichever moved last":

1. the card the details drawer shows (431, unchanged);
2. the cursor card, when the last input was a key; the hovered card, when it was the mouse;
3. the open List row.

"Last input" is a page flag set to `key` by an arrow, `hjkl`, Home or End, and to `mouse` by a real mouse move over the cards. A `mousemove` with no movement, which browsers fire when the page scrolls under a still mouse, does not count, so scrolling the cursor into view never hands the keys back to whatever card slid under the mouse. A mouse move that lands on a card also moves the cursor there, so the next arrow continues from where the mouse is rather than jumping back to a stale spot.

**What it looks like.**
- List: the cursor row shows the kit's focus ring (the same `:focus-visible` outline buttons get). It does not show the hover preview: that preview follows the mouse to the far corner and would read as noise for a keyboard user. An expanded row shows its details as it does today.
- Stacks: the cursor card reveals in its stack exactly as a hovered card does (the reveal's `Hover` gesture, so the stack opens around it and it floats as 435 has it), plus the focus ring. Moving on lets it settle as a mouse leaving does.
- With reduced motion the float and sway are off already; the scroll into view is instant instead of smooth.

**Real focus, roving tabindex.** The cursor is DOM focus. The cursor card (a List row's line, a stack card) carries `tabindex="0"` and every other card `-1`, so Tab enters the cards once, at the cursor, and Tab again leaves them. Real focus gives three things for free: the browser's focus ring, screen readers announcing the card, and 429's sheet focus handing focus back to the cursor card when Printings or the drawer closes. `aria-activedescendant` was the alternative; it needs a single focused container with a listbox or grid role, which the List's groups and the Stacks' columns are not, so it would mean inventing ARIA structure the page doesn't have.

The first arrow on a page with no cursor yet puts it on the first card in view (top left of the visible part) instead of moving, so the first press never scrolls the page away.

## 2. Movement

Up and down come from a pure model of the page's order in `zwipe-ui` (`deck_cards::cursor`), built from the same sections, folds and column split the page draws with, so they're unit tested without a browser. Left and right need where cards sit on screen, which only the browser knows (a revealed stack card is taller, an expanded row pushes the rest down), so they read the cards' rects at the moment of the key and pick from them with a pure function.

**List.**
- `↓` `j` / `↑` `k`: the next or previous row in its column, crossing into the next or previous group in that column. Folded groups are skipped; their rows aren't stops.
- Below 1400px the List is one column (left groups, then right groups, as it draws), so `↓` runs through every row in page order.
- From 1400px the groups sit in two columns: `↓` and `↑` stay in their column, and stop at its ends.
- `←` `h` / `→` `l`: from 1400px, to the other column's row whose vertical center is nearest the cursor row's. In one column they do nothing (no surprise group jumps).
- Group headers are not stops in v1 (see Out of scope).

**Stacks.**
- `↓` `j` / `↑` `k`: the next or previous card in the stack; past a stack's last card, the first card of the next stack down in the same column; past its first, the last card of the stack above. Folded stacks are skipped.
- `←` `h` / `→` `l`: to the neighboring column's card whose vertical center is nearest the cursor card's. "Nearest" is by the cards' rects as drawn, so columns of different heights work: past the end of a shorter column, its last card. A column with no visible card is skipped over to the next one.

**Edges stop.** No wrapping: at the first card, `↑` does nothing; at the last, `↓` does nothing; the same at the outer columns. Wrapping makes a long list feel lost and fights the scroll position.

**Home / End**: the first and last card in reading order, taken only while the cursor shows (the last input was a key), so a reader scrolling with Home and End keeps that.

**PageUp / PageDown / Space** stay the browser's (scroll); the cursor follows on the next arrow from the first card in view if it scrolled out.

**Enter or `d`** on the cursor card: List, the row expands or collapses (as a click on its line does); Stacks, the card pins and its details drawer opens, or closes if open (as a click does). Shift+Enter, Ctrl or Alt combos pass through.

## 3. Key conflicts

| Key | Today | With the cursor |
|---|---|---|
| `↑` `↓` `←` `→` | browser scrolls the page | move the cursor; the page scrolls by keeping the cursor in view |
| `h` | unused on the deck page (Shift+H is the shortcuts sheet) | left; Shift+H unchanged (431's Caps Lock fix keeps `h` and `H` apart) |
| `j` `k` `l` | unused on the deck page | down, up, right |
| Enter | the app's "submit the one form panel" when nothing is focused | open; the deck page has no form panel today, and the cursor's handler takes Enter first and prevents the default |
| `d` | open or close the hovered card's details | the same, on the keyed card (cursor or hovered) |
| `p` `i` `v` `m` `s` Shift+S `-` `+` `=` | act on the hovered card | act on the keyed card |
| `f` `u` `/` `?` Shift+H, Ctrl+Z | page keys | unchanged |
| Alt+`←` `→` | browser back/forward | unchanged: any Ctrl, Alt or Meta combo passes through |
| Shift+arrows | text selection | unchanged: passes through |

Where the keys don't reach the cursor at all:
- **Typing.** Nothing moves while focus is in an input, textarea, select or contenteditable (Quick add, the filter's fields): the page guard already refuses keys there. Esc leaves Quick add (431), and the arrows work again.
- **Modal sheets and dialogs keep their keys.** Printings (`←` `→` `h` `l` Enter), the Oracle tag examples (`←` `↓` `h` `j`), the filter, the shortcuts sheet and every dialog keep every key, as the guard does now. The arrows never reach the page behind a modal.
- **The details drawer is the one exception**, as it is for the card keys (431): it is not modal on a wide window, and the arrows pass through it. Moving the cursor with the drawer open swaps the drawer to the new card (as clicking another card does), so the drawer always shows the keyed card and 431's rule holds without a second keyed card.
- **Screen readers.** In a screen reader's browse mode the arrows are the reader's and never reach the page; in focus mode (entered by Tab onto a card) they move the cursor, and the reader announces each card as it takes focus. Each card gets an accessible name: name, quantity, and board where it isn't the main board (for example "Lightning Bolt, 2, sideboard").

## 4. Scroll, focus, motion

- **Scroll into view**: `scrollIntoView({ block: 'nearest' })` on the cursor card, inside the page's own scroller, with `scroll-margin` on the cards so the sticky header and the bottom docks never cover the cursor. Smooth, or instant under `prefers-reduced-motion`.
- **Focus handoff**: Printings, the image preview and the drawer already hand focus back to their opener (429); with the cursor that's the cursor card, so a keyboard user lands back where they were. A card the action removes (`-` at one copy, `m` or `s` off the shown board) hands the cursor on per section 5.
- **Reduced motion**: the reveal's float is already off; the cursor adds no animation of its own.

## 5. When the page changes under the cursor

The cursor is a card id, not a position, so it survives anything that keeps the card on screen.

| Change | Cursor |
|---|---|
| View switch (List ⇄ Stacks) | stays on the same card, scrolled into view; if the card isn't in the new view, the first card in view |
| Grouping change (Type, Mana value, Color, Card role) | stays on the same card |
| Filter or board chip (Main, Maybe, Side) hides the card | the next card after it in the old order that's still shown, else the one before, else none |
| Its group or stack folds | the first shown card after the folded group, else the last before it |
| The card leaves (`-` at one copy, moved to another board, Undo) | the next card in order, else the one before; Undo that brings a card back doesn't move the cursor |
| Printing change (the card's id changes) | follows to the new id (the save already hands focus to the new row's Printings, 429) |
| The page reloads its deck | stays on the same id if present |

## 6. Phones and touch

Unchanged. The cursor is only set up on the computer layout (a fine pointer with hover, as the shortcuts sheet's keyboard button is gated, `use_computer`), so a phone or tablet with no mouse never registers the listener, never gets `tabindex` changes and never shows a ring. A tablet with a keyboard but no fine pointer is out of scope for v1.

## 7. Tests for surfer

Run each on the owner deck and a shared deck, List and Stacks, at 1440 (two List columns) and 1000 (one List column), with a keyboard only unless noted.

**Getting in**
1. Load the deck page, press `↓`: the cursor lands on the first card in view, nothing scrolls.
2. Tab from the controls into the cards: focus lands on the cursor card (or the first card); Tab again leaves the cards.
3. Scroll down with the wheel, press `↓`: if the cursor scrolled out, it lands on the first card in view.

**List**
4. `↓` through a whole group into the next one; `↑` back. Fold a group; `↓` skips its rows.
5. At 1440, `→` from a left-column row lands on the right column's nearest row; `←` back. `↓` at the bottom of a column does nothing. At 1000, `←` and `→` do nothing.
6. Enter on a row expands it; Enter again collapses it; `d` does the same.
7. `↓` past an expanded row moves to the next row and leaves the expansion as is.

**Stacks**
8. `↓` through a stack, into the stack below; `↑` back up into the stack above's last card.
9. `→` and `←` across columns of different heights: lands on the nearest card by height; past a short column's end, its last card.
10. The cursor card reveals and floats as a hovered one does; moving on lets it settle.
11. Enter opens the drawer on the cursor card; arrows with the drawer open swap the drawer to each new card; Enter or `d` closes it; Esc closes it and leaves the cursor where it was.

**Card keys on the cursor**
12. `p`, `i`, `v`, `m`, `s`, Shift+S, `-`, `+` act on the cursor card with the mouse resting on a different card (the cursor moved last).
13. Move the mouse over another card: the keys act on that one; press `↓`: the cursor continues from the hovered card.
14. Scroll the cursor into view so the page slides under a still mouse: the keys still act on the cursor card.
15. Commander and shared exceptions hold (422/431): `+` on the commander and edits on a shared deck do nothing.
16. `-` at one copy removes the card: the cursor goes to the next card. `m` on a main board card with only Main shown: the cursor goes to the next card. Undo puts the card back; the cursor stays put.

**Changes under the cursor**
17. Switch List to Stacks and back: the cursor stays on the same card.
18. Change Group by: same card. Filter it out: the next shown card.
19. Change a printing with Enter in Printings: focus and the cursor land on the new printing's row.

**Conflicts**
20. Type in Quick add: arrows move the text caret, not the cursor. Esc, then `↓`: the cursor moves.
21. Open Printings: `←` `→` step printings, the cursor doesn't move. Close it: focus is back on the cursor card.
22. Open the filter, the shortcuts sheet, a dialog: arrows do nothing behind them.
23. Alt+`←` still goes back in the browser; Shift+arrows don't move the cursor.
24. Caps Lock on: `h` still moves left (not the shortcuts sheet); Shift+H opens the sheet.

**Access and motion**
25. With a screen reader (VoiceOver or NVDA) in focus mode: each move announces the card's name, quantity and board.
26. With reduced motion: no float, the scroll jumps instead of gliding.
27. Phone width and a touch device: no ring, no `tabindex="0"` on cards, arrows do nothing new; taps work as before.

**Shortcuts sheet**
28. Shift+H lists "↑ ↓ ← →, h j k l: Move between cards" and "Enter, d: Open the card", and the note reads "Move to a card, or hover it, then press its key".

## Build order

1. `zwipe-ui` `deck_cards::cursor`: the order model (List columns and groups, Stacks columns and stacks, folds), up/down/Home/End, and the nearest-by-rect pick for left/right, all unit tested.
2. `components`: `use_deck_cursor` (the cursor id, the last-input flag, the key listener for arrows, `hjkl`, Home, End and Enter under the page guard with the drawer pass-through), roving `tabindex` and accessible names on List rows and stack cards, scroll into view, the Stacks reveal on the cursor card.
3. The keyed-card lookup (`KEY_CARD_JS`) reads the cursor between the drawer and the hover, by the last-input flag; the drawer follows the cursor.
4. Section 5's hand-offs on filter, fold, board, removal and printing changes.
5. The shortcuts sheet's rows and note, `keyboard_shortcuts.md`, the changelog.

## Decisions for Scotty

Each has a recommendation; none blocks starting step 1.

1. **Edges**: stop (recommended) or wrap to the other end.
2. **`←` `→` in a one-column List**: do nothing (recommended), or jump to the previous or next group.
3. **Arrows with the details drawer open**: the drawer follows the cursor (recommended), or the arrows close the drawer first.
4. **Featured cards in the deck head**: not stops in v1 (recommended; they repeat cards that are in the List or Stacks anyway), or the first row of stops above the cards.
5. **Home / End**: only while the cursor shows (recommended), or always.

## Out of scope for v1

- Group and stack headers as stops (Enter to fold, Space to select a group).
- Moving into an expanded row's buttons with the arrows (Tab reaches them).
- The phone layout and touch screens, and a tablet with a keyboard but no mouse.
- The other card screens (Playstyle, Activity, Commander maybeboard), which can take the same cursor once it's proven here.
