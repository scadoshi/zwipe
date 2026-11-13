# 480. The deck page's shortcuts sheet leaves out keys that work

State: fixing | Owner: thinker | Type: bug | PR: - (pushed to release) | Build seen: 045bd5c38

## Report
Scotty, via the composer: the deck page's shortcuts sheet (Shift+H, and the hint's keyboard button) doesn't list `/`, which focuses Quick add. Add its row, and check every other key the page answers is listed, and nothing listed that doesn't work.

## Before
Headless Chrome at 1440x900, a fine pointer forced, the owner's deck page with mocked reads (every write failed, none sent), 3 of 3:
- Shift+H lists 23 rows; none has `/`, Home or End.
- `/` puts the cursor in Quick add (`input#quick-add`).
- After `↓` puts the cursor on a card, End moves it to the last card and Home to the first (8 cards).

## Cause
- `zwipe_ui::shortcuts::deck_cards_rows` lists the cursor's arrows and `hjkl` but not Home and End, which the cursor (orangui `grid_cursor::MOVE_KEYS`, installed by `components::use_deck_hover_keys` on both deck pages) answers while a card has focus from a key.
- It lists no `/` row; the editor's `screen_key` (`zwiper-web/.../deck/cards/editor.rs`) answers `/` on the owner's page. The shared deck has no Quick add and doesn't answer `/`.
- Every other key the page answers is listed: `p i v d m s Shift+S - + u Ctrl+Z f Enter`, the arrows and `hjkl`, and the page keys. `=` (as `+`) and capitals are aliases. Every listed key acts (the web test `the_deck_cards_sheet_lists_only_that_screen_s_keys`).

## Fix
- `deck_cards_rows` adds `Home End = First or last card` on both pages and `/ = Quick add` on the owner's page only. `/` is `zwipe_ui::shortcuts::QUICK_ADD_KEY`, which the deck editor and the Commander maybeboard read too.
- Tests: `every_key_the_deck_page_answers_is_listed` (red before the rows), `a_read_only_deck_lists_only_the_keys_that_read` updated; the web test knows `/`.

## After
Same setup, 4 of 4 runs that loaded: the sheet lists `Home End = First or last card` after the moves and `/ = Quick add` after Filter (25 rows); `/`, Home and End act as before. One more run caught the page while the dev server rebuilt and loaded nothing.

## Not checked
- The shared deck's sheet headless: my fixture doesn't render its Cards. The rows come from the same function with `Editable::No`, covered by the unit test.
