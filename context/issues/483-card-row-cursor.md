# 483. The Commander maybeboard, Activity and Playstyle have no keyboard cursor

State: fixing | Owner: laptop | Type: feature | PR: - (pushed branch, hash in the composer's log) | Build seen: release bf8444026

## Report
Scotty, via the composer: the deck page's keyboard cursor (440) on the other card lists: the Commander maybeboard, Activity's card rows and Playstyle's card rows. The arrows and h j k l move, Enter opens, and the card keys act on the cursor's card. Reuse `components::deck_cursor`; don't copy it. Plan approved with two changes:
- the hover image follows the keyboard cursor on these screens;
- the keys work at any width, as 478 makes them on the deck page.

## Before
Local web, signed in, 1440. On the maybeboard (5 commanders), ↓, j, ↓: focus stays on the page, no row is reached, and no preview shows. 3 of 3 presses.

## Cause
- `components::deck_cursor` stops only on the deck page's rows and stacks (`.web-deck-row`, `.card-stack-slot`), and only the deck page mounts it. These screens draw zwiper-web's `CardRow` (`.web-card-row[data-row]`), and their card keys (`cards/row_keys.rs`) read only the hovered card.
- A `.web-card-row` is `display: contents`. It has no box, no `offsetParent` and an empty rectangle, so the cursor's visibility check and position reads skipped it.

## Fix
laptop:
- **`deck_cursor`:**
  - adds `.web-card-row[data-row]` to its stops, except a deck row's own inner card row, which would otherwise be a second stop for the same card and hold the deck page's cursor in place (caught by a regression check on the deck page);
  - measures a `display: contents` stop by its first child;
  - counts an expanded card row as an open row, so an open row follows the cursor;
  - opens a shut card row the way it opens a deck row.
- **`use_card_list_cursor()`:** the same cursor, marked so that after a key, focusing a row fires that row's own hover enter, and leaving it fires the leave. The cursor card's image shows in the screen's hover stack as hovering does; moving the mouse hands the preview back.
- **Screens:** the maybeboard, Activity and Playstyle mount it.
- **Card keys:** `row_keys` acts on the cursor's card when a key moved last (`deck_cursor::keyed_js`), else on the hovered card.
- **Shortcuts:** the screens' sheet lists ↑ ↓, j k, Home/End and Enter before the card keys (`zwipe_ui::shortcuts::card_rows_rows`), and j and k no longer scroll there.

## Checked
Local web at 1440, signed in. On the maybeboard (5 rows) and Playstyle (4 rows):
- ↓, j, ↓ and k move the cursor row by row, and the preview follows each row;
- Enter opens the row, j with it open opens the next one in its place, and Enter closes it;
- `i` opens the cursor card's image;
- End and Home go to the last and first rows;
- a mouse move hands the preview back to hover.

On release with 478, the maybeboard does the same at 600px with a keyboard. The deck page's cursor still moves down rows and across columns.

## Verification
-
