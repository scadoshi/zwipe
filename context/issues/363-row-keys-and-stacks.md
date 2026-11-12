# 363. Playstyle and Activity have no card keys and no Stacks view

State: fixing | Owner: laptop | PR: - | Build seen: production, web

## Report
Composer for Scotty, 2026-11-11 (moved from thinker): the card-row keyboard shortcuts should work on Playstyle's Suggested commanders and Staples and on Activity's Most skipped. That means v, d, i and p; f where there is a filter; u and Ctrl+Z where undo applies; and Shift+H, all through the shared card_key map. Those screens also get the List / Stacks view toggle, one stack each since each is a single group, reusing the deck page's stack and toggle components. Web first, then a note on what the app would need.

## Cause
The card keys, the stacks and the View switch were built inside the deck page: its editor, its `stacks.rs` and its page. No other screen could reach them.

## Fix
laptop, web:
- **Moved, not copied,** into `inbound/cards/stack_view.rs`:
  - the deck page's `StackDetails`, `use_stack_details_sync` and drawer classes;
  - its View chip row, as `CardsViewSwitch`.
  - The deck page uses them from there.
- **Added there:**
  - `use_stack_details`;
  - `RowFaces`, the faces a screen's rows and stack share;
  - `CardsStack`, one `components::CardStack` over zwipe-ui's `Reveal`;
  - `StackDrawer`, the open stack card's `CardDetailsSheet` with Image, Printings and the screen's own row extras.
- **`inbound/cards/row_keys.rs`:** `use_card_row_keys` answers p, i, v and d on the card under the cursor (a row or a stack card), else the open one, through the stack drawer too. `v` and `d` read through `shortcuts::card_key`. Shift+H offers `ShortcutsScreen::CardRows`, whose rows are zwipe-ui's new `card_read_rows`, the same four keys the deck page lists first.
- **The web `CardRow`** carries `data-row` and reads and writes the screen's `RowFaces` when it has no face of its own.
- **Playstyle:**
  - the View switch under Show governs both lists;
  - Suggested commanders and Staples each draw one stack;
  - one drawer serves both, with a suggestion's reasons and actions or a staple's deck count.
- **Activity:** the View switch heads Most skipped, which draws one stack; its drawer shows "Skipped under".
- **Not added:**
  - `f`: these screens have no filter sheet; Show, Type and Color are chips.
  - `u` and Ctrl+Z: nothing on them undoes.
- **One shared preference:** List or Stacks is the deck page's stored choice, used by every card screen.

## What the app would need
- **Keys:** none. The app is touch only.
- **Stacks:** the app has no Stacks view anywhere yet, the deck page included. `components::CardStack` and zwipe-ui's `Reveal` (with `Gesture::Tap`) are platform-free. An app version needs:
  - the View chip row;
  - `CardStack` with taps in place of hover;
  - the app's card details sheet as the drawer, carrying the row's actions.
  - It would be best built for the deck page first, then reused on Playstyle and Activity the way this does on the web.

## Verification
-
