# 347. Add's left, right and up swipes play no fly-off

State: fixing | Owner: scotland | PR: #542 | Build seen: live 1.18.17 (918a1d47), Firefox and Chrome

## Report
Scotty, 2026-11-11, web, Firefox then Chrome: on Add, skipping, adding or maybeboarding a card (h, l, k, the arrows, the dock, a mouse drag) plays no fly-off; only Undo (down) animates. A card ghost shows under the top card, its edge sticking out at the left. Recording `target/scotty-shots/347-firefox-no-swipe-anim.mov`, still `347-firefox-ghost-under.png`.

## Cause
#530 (315). Surfer traced it on live: on every swipe the stack took `ghost-held` for about 0.4s, and the rule hiding the cards under a held top (`.swipe-stack.ghost-held > .swipe-stack-card:not(.top)`) also matched the card flying off, which is not the top. Its flight, about 0.3s, ran invisible every time. Undo plays its entrance on the top, so it showed.
- The stack held its ghost on an ordinary swipe because each new top restarted the paint grace (`TOP_PAINT_GRACE`), though it had loaded under the top.
- Every swipe rebuilt all ten stack cards (the key sat inside a block in the loop, not on the loop's node), so the new top also started with no image for a frame or two.
- The ghost was a skeleton with its own sizing, a little smaller and offset from the top card on a wide window, and 2px off on a phone.

## Fix
scotland, #542:
- The card flying off is exempt from the hiding (`:not(.top, .swipe-stack-exiting)`, screens-a.css).
- Only a top whose image arrives while it is the top holds the ghost for the paint grace (`zwipe_ui::stack::ghost_held`, its `painting` argument). A card that loaded under the top shows at once when it comes up, so a swipe never holds the ghost. A new pile still does (315's behavior).
- Stack cards, web and app, are keyed by card on the loop's own node, so a swipe drops the top and keeps the other nine mounted; a card the stack already had in shows at once.
- While a new pile reveals, the stack holds its own ghost (`hold`) instead of a separate skeleton beside it, and every ghost (held stack, pile on its way) takes the top card's box: the card image's 672 by 936 shape, its margins, and on a wide window the stack's room and the card's height cap.
- Tests first: `a_card_that_loaded_under_the_top_shows_at_once_when_it_comes_up` (zwipe-ui) and `a_card_flying_off_shows_while_the_stack_holds_its_ghost` (zwiper-web).
- Headless on web, Add, 1260 and 390: every swipe left the stack unheld with the flying card visible in every frame, and changed one card node (removed one, added one) instead of ten. The ghost's rect equals the top card's: 453,115 353 by 492 at 1260, 32,174 326 by 454 at 390, for the ghost while the pile is on its way, under the held stack, and the card itself.
