# App card stacks

Issue 390. Status: approved 2026-11-11; phase 1 shipped (#595); phase 2, with the group carts folded in, built on 370 phase 2b's `DeckStacks` and `DeckStackDrawer`. Follows `card_stacks.md` (Phones) and sits beside `shared_deck_page.md` (370).

## Goal

The app's deck cards screen (`zwiper/src/lib/inbound/screens/deck/card/view.rs`) gets a List | Stacks switch. Stacks draws two stacks side by side, one per group, each about half the screen wide, with title strips as readable as the web's. The first tap on a card reveals it in place, a second tap on the revealed card opens the card details side sheet, and a tap anywhere else closes the reveal.

## Reused as is

- zwipe-ui `deck_cards::stacks`: `CardsView` and `VIEW_LABEL` for the switch, `Reveal` with `Gesture::Tap` and `Gesture::Close` (Tap already does reveal, then open, then close), `tucked` / `shifted` / `exposed`, and `DealCue` with the `DEAL_*` timing (#584).
- components `CardStack` and `StackCard`, with `CardProxy` for a card with no image (#589), the meld toggle, the quantity badge and the deal-in CSS, all already in `magic.css`.
- The app's `CardDetailsSheet` (`DrawerSheet`, from the right edge) as the details sheet, its footer holding components' `CardEditButtons` and `CardBoardMoves`, the way the web drawer's `stack_details` builds it. Printings and the image preview go through the screen's existing `open_printings` and `preview_card`.
- The group carts: `CartButton`, `StoreDialog`, and zwipe-ui `buy::group_title` / `cards_links`, as the web's `group_buy.rs` does (no token in a cart).
- The screen model's sections and folds (`display`, `Action::ToggleSection`, `SetSections`) and its existing per-card actions (`ChangeQuantity`, `MoveTo`, `ToggleMvp`). Nothing new in the model's API.
- The Art chip grays out while Stacks is on, as #577 does on the web.

## Built

1. **zwipe-ui `Gesture::Dismiss`**: a tap off every card clears the reveal and leaves the open card alone. Pure, with tests written first: tap reveals, tap again opens, Close keeps the reveal, Dismiss clears it, and a tap on another card moves the reveal.
2. **components `CardStack` in tap mode**: `on_hover` and `on_leave` become optional. With neither passed, the card button wires no mouseenter, mouseleave, focus or blur. In a webview a tap fires a synthetic mouseenter and a focus before the click, so a Hover would land first and the Tap would open the sheet on the first tap. In tap mode `on_select` also stops the click's propagation, so the screen's Dismiss doesn't see it. The web's call sites keep passing both handlers, with no visible change. A golden pins tap mode's markup.
3. **App stacks view** (`zwiper/.../deck/card/components/stacks.rs`): the stacks grid, each stack a group header (the app's `CollapsibleGroup` head: name and count, then a second line with the price chip and the cart, as the web's stack head wraps) over a `CardStack`. Order: command zone, main groups, then Maybeboard, Sideboard and Tokens when their chips are on, the same groups the List shows.
4. **The switch**: a `View` chip row (List, Stacks) above Group by, ghosted with the other rows until the deck lands.
5. **Remembered choice**: a `cards_view_store` in `zwiper/src/lib/outbound`, following `count_scope_store` (keyring on Apple, a file on Android). The List is the default.

## Touch handling

- A card's tap goes to `Gesture::Tap`. After it, the sheet shows when `reveal.open()` is `Some` (the web's `StackDetails::click` pattern, using Tap).
- The screen root's onclick sends `Dismiss`, so a tap on the chips, a header or the gaps between stacks closes the reveal. A header tap also folds its stack, as it does now.
- The sheet's ×, backdrop or the back gesture sends `Close` (the web's `use_stack_details_sync` effect, copied for the app): the card stays revealed, and one more tap reopens it.
- A scroll that starts on a card fires no click, so scrolling never reveals anything.
- `-webkit-tap-highlight-color: transparent` on `.card-stack-card`. `.card-stack:hover` z-index is harmless on touch: it sticks to the last tapped stack, which is the one that should sit on top.
- A card that leaves the list (removed, moved off a shown board, filtered out) closes the sheet, as on the web.
- A revealed card pushes the rows below it down instead of laying over them, and they slide back on tap off or Back, through the shared settle (Scotty, 2026-11-12). The web's hover keeps the overlap.

## Layout at 360 to 430px

- Grid: `grid-template-columns: repeat(2, minmax(0, 1fr))`, a 0.6rem column gap, and the screen's 16px gutters. The web's subgrid header rows come along, so the two stacks in a row start level.
- The column is about 159px wide at 360 and 197px at 430, against the web's 200px minimum. A tucked card shows its top 13% (`magic.css`), a strip of about 29px at 360 and 36px at 430, which holds the card's name line. A revealed card is about 222px tall at 360 and 274px at 430.
- Header at 159px: name and count on one line, cut short with an ellipsis, then the price chip and cart on the line under them. The fold arrow takes the left inset, as on the web.
- These rules live in the app's stylesheet under `.app-deck-stacks` for now. Phase 2 of 370 moves the deck page CSS to `magic.css`, and when it lands the app takes the shared rules and keeps only the two-column override.
- I'll check it by eye in headless Chrome at 360, 393 and 430 (the app's web build), and droider checks it on the emulator.

## Relation to 370

370 phase 2 (the deck cards view to components) hasn't started: no open PR, and release has only phase 1. The web's Stacks view is tied to `Editor` / `RowInfo`, so the app can't take it as is. I build on `CardStack` directly and keep my changes to components small (tap mode). When phase 2's `deck_cards_view` lands, the app view can move onto it if it takes the app's actions shape. Question: is that order fine with scotland, or should I wait for phase 2?

## Phases (one PR each, into release)

1. zwipe-ui `Gesture::Dismiss`, plus components `CardStack` tap mode with its golden. No visible change.
2. The app's Stacks view, switch, details sheet, Dismiss, deal-in, Art gray-out and the remembered choice, plus a changelog entry in the unshipped version. Model tests for the gestures; goldens for the stacks view; headless shots at 360, 393 and 430; droider on the emulator.
3. The group carts in the stack headers (split out so phase 2 stays reviewable; folds into 2 if you'd rather).

No wire or server change.
