# 422. The deck head's featured cards only open their image

State: fixing | Owner: thinker | PR: - | Build seen: production

## Report
Composer for Scotty, 2026-11-12: the deck head's featured cards (command zone and MVPs) should act like the same card in the List or Stacks, just up front. Web: hover floats (417); a click opens the card's details as a row or stack card does; every card shortcut works on a hovered featured card. App: a first tap floats it (421's float), a second opens the details sheet, a tap off clears. Read-only shared deck: the same, with read-only actions. Scotty's exception: the commander's quantity and Star do nothing, from buttons or keys; it changes only through Edit, while Image, Flip or Meld, Printings and details still work.

## Fix
thinker:
- zwipe-ui `deck_cards::featured_open`: a featured card opens its details while it is among the listed cards, else its image (filtered out, or its group hidden). The commander is its command zone row, whose details and hover keys already leave out the stepper, Star and board moves; `keys.rs` pins that.
- components `FeaturedCard`: `on_select` gets the card's id on a click, and the image carries `data-row`, which the hover keys' card lookup reads as a stack card's.
- Web deck page (owner and shared, one body since 418): a click opens the card in the Stacks details drawer, now mounted in List view too, so `d`, `i`, `p`, `v`, `-`, `+`, `s`, `m` and Shift+S act on a hovered featured card as on a stack card. zite's own share page gets the same wiring.
- App: a first tap reveals the featured card through the Stacks `StackDetails::tap`, which floats it (`floatable floating`, 421's classes), a second opens its details sheet, mounted in both views, and a tap off the cards clears it in both views.
- Headless Chrome, web owner and shared deck pages with a fixture (Sol Ring commander, Lightning Bolt MVP): a click or hover plus `d` opens the card's drawer in List and Stacks; the commander's shows Image and Printings only and its keys make no write; the MVP's shows the stepper, Unstar and the board moves and its keys write; the shared page's drawers show no edits. The app was not driven headless.

## Verification
-
