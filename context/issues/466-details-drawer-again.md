# 466. On a phone, a card's details from Cards should be the side drawer, not a full screen

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: 31b3ab105

## Report
Composer for Scotty, 2026-11-13: in the phone app, a featured card in the Cards header opens its details full screen; first scoped to the featured strip, then corrected: every card's details on a phone (app and phone web) go back to the side drawer from the right, not full screen; 449's full-screen details is withdrawn, since he only wanted the Cards to feel like a route. The drawer dims the page (424's phone rule) and closes three ways: a tap on the dim, the ×, or a drag to the right (new, matching how bottom sheets drag down to close). 457's route feel for Profile and Cards stays.

## Cause
`components/src/deck_cards_view.rs`: `DeckStackDrawer` passed `screen_on_phone: true` (449) for every card, whatever opened it, so below 900px every details opened as a full screen over a transparent backdrop.

## Fix
scotland:
- The screen form is gone: `DrawerSheet` and `CardDetailsSheet` lose `screen_on_phone`, `filter.css` its rules, and 457's hiding of the Cards under the details (web `body.rs` and `cards_layer.rs`, app `deck/card/view.rs`) goes with it.
- orangui 9039b70: `SheetDrag` takes a `Pull`, `Down` or `Right`; a pull to the right follows the pointer's x, measures the sheet's width, and moves it through `sheet-dragging-right`. `DrawerSheet` pulls a drawer at every width to the right below the kit's modal width (where it floats along the right edge), by its title bar, as a bottom sheet pulls down.

- Follow-ups after surfer's and droider's afters on 93c372e82:
  - (d) Each open and close of a stack card's details left a history entry behind. Cause: `components/src/deck_cards_view.rs`, the touch reveal gave up its back step while the details were open (`revealing` required `open().is_none()`), so the drawer took over the reveal's entry instead of pushing its own, and closing it re-registered the reveal at the same depth, popping nothing; on a slow phone the swap could also straddle the history's settle window. The reveal now keeps its step while anything is revealed; the drawer pushes and pops its own (test `a_revealed_card_s_details_take_a_back_step_of_their_own_over_the_reveal_s`, red on the old line). The reveal's own step stays for Scotty's call.
  - (e)/droider's real-touch fail: the title bar alone pulled, and on a real touch the browser took a sideways drag as a pan. The drawer now pulls from anywhere on it, and it and everything in it take `touch-action: pan-y` (each nested scroll container, the content and the card panel's body, is where a touch's allowed moves stop combining), so a sideways move pulls and a vertical one scrolls (`components/tests/drawer_pull.rs`).
  - (a) An Oracle tag's description scrolls into view as it opens, so its Examples button is never below the drawer's fold (`role_tags.rs`). Examples itself is a swipe surface, not a card details view, and stays full screen.
  - Droider's centered-panel finding was retracted with measurements: the drawer is the same on Cards, Add, Remove and the picker, in the app and on the web.
  - Droider's real-touch after on eec4e11be: drag to close passes everywhere, but the drawer's scroll broke. (1) Phone web: a vertical swipe on the details moves the body about 8px of the 53 it has. (2) App: the same swipe scrolls the Cards page behind the drawer (2488 to 2912). (3) Opening Other tags, then a tag, leaves the chips and Examples below the footer, 3 of 3 in the app and on the web.
    - Before, headless at 390x640 (Krenko): opening Other tags left the last chips at 492 to 539 below the body's bottom at 499, with the scroll at 0 of 63; opening a tag left the scroll at 0 of 141 and Examples at 612; no `scrollIntoView` ran at all (traced). A vertical touch on the body changed the sheet's class and style three times mid-gesture (`sheet-dragging-right` on, `--sheet-drag`, off on the pointercancel). CDP's synthetic scroll survived that, so (1) and (2) didn't reproduce headless.
    - Cause (3): the scroll went into `role_tags.rs`, the deck Roles chart's tags; the details panel's Card roles and Other tags are `CardRoleChips` (`card_role_chips.rs`), which has its own copy of the reveal and no scroll.
    - Cause (1) and (2), from the trace, not reproduced on a real touch here: `DrawerSheet` started the pull on the body's pointer down, so every vertical swipe re-rendered the sheet (an `!important` transform with transitions off, then back) while the browser was taking the gesture as a scroll, losing the scroll on a real device.
    - Fix: orangui 1f8f5a8 `SheetDrag::arm` readies a pull without touching the sheet and starts it only once a move past 8px runs more along the pull than across (`orangui_core::sheet_drag::intent`); a move across drops it. `DrawerSheet` arms on the body's pointer down (the title bar still starts at once). `CardRoleChips` scrolls an opened role's tags and an opened tag's description into view, by the helper `RoleTags` uses.
    - After, headless 390x640: opening Other tags scrolls 0 to 47 with every chip above 499; opening a tag scrolls to 118 with Examples ending at 494; a vertical touch scrolls 0 to 105 with no change to the sheet; a sideways drag still closes it and a short one springs back.

## Verification
- Phone web (390) and the app: open a deck's Cards, tap a stack card twice (or a featured card, or an app List row): the details slide in from the right with the page dimmed.
- A tap on the dim closes it and opens nothing behind; the × closes it; dragging its title bar a little to the right springs it back, past a quarter of its width (or a flick) closes it.
- A sideways drag on the drawer's body or its title bar past a quarter of its width closes it; a vertical swipe scrolls its content and nothing behind it moves. Open a card with Other tags, tap Other tags, then its last tag: each opening scrolls the body so the chips, then Examples, show above the footer. Open and close a stack card's details three times, by the dim and the ×: one Back settles the revealed card and the next closes the Cards.
