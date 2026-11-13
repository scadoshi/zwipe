# 474. A card's details drawer opens on the side away from the card's column

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: 635ec281a

## Report
Scotty, through the composer, 2026-11-13: (a) the drawer's side follows the card's column everywhere, the phone app included. In Stacks, a card in the right column(s) opens the drawer from the left and a card in the left column(s) opens it from the right; featured (header) cards open from the right; phone web the same, wide web as planned. On a phone, the drag to close follows the side: a left drawer drags left. (b), moving the dock, waits on Scotty.

## Before
Every card's details drawer opened from the right, on a phone (the app and the web) and on a wide page, whatever column the card sat in: `DrawerSheet` had one edge, the right (`filter.css` below 60rem, orangui's drawer at 60rem and up), and pulled only to the right (orangui `Pull::Right`). At 390 a card in the right column was covered by its own drawer.

## Cause
No side existed: nothing recorded which column a stack card sat in when its drawer opened, and the drawer had no left form.

## Fix
scotland:
- `zwipe_ui::deck_cards::columns::drawer_side(column, columns)`: a card in the right half of the columns opens a left drawer, the left half (and an odd count's middle column) a right one.
- `StackDetails` gains `side` and `opens_left` (made by `use_stack_details`). `DeckStacks` records the cards of the right half as it lays the columns; a click, a tap, the `d` key and the keyboard cursor set the side from it as a card opens, a featured card sets the right, and a close keeps the side so the drawer slides out the way it came.
- `DrawerSheet` and `CardDetailsSheet` take an `edge` (`DrawerEdge`), which adds `drawer-left` and pulls toward that edge. orangui c1e6a62: `Pull::Left` (`sheet-dragging-left`) and the wide `drawer-left`; `filter.css` the phone's left form.
- Droider's real-touch finding on bb316783f (phone web), a title-bar drag from the header's top 15 to 20px doing nothing 3 of 3, was withdrawn on f195aa92d: those drags started above the drawer, on the dim (a calibration offset). The title bar starts its pull at once and takes `touch-action: none` (`components/tests/drawer_pull.rs`), as it started on eec4e11be: it has nothing to scroll, so a drag there that sets off slightly downward still pulls instead of reading as a scroll.
## Verification
- Droider, real touch at 412 on f195aa92d (web and app): a right-column card opens from the left 3 of 3, a drag left closes it and a drag right doesn't; a left-column card and a featured card open from the right 3 of 3; a drag from 5px into the header closes 3 of 3. Not checked: a tap on a card in the other column with a drawer open (the phone drawer leaves about 38px of the page).
- 390, a deck's Cards in Stacks: a card in the left column opens its drawer from the right; one in the right column from the left, and a drag to the left closes that one (a short one springs back, a drag right does nothing). A featured card opens from the right.
- 1440: cards in the left three of six columns open on the right, the right three on the left; a click on another card swaps it in on its own side.
- Phone web and the app: a title-bar drag from the very top of the header closes the drawer.
