# 354. Buying is a different control in every spot, and the wide deck page hides it

State: fixing | Owner: thinker | PR: - | Build seen: production, web wide

## Report
Composer for Scotty, 2026-11-11: on wide web the deck page seemed to have no Buy, which lives in the Budget widget (folded by default). Asked for, after 353:
- A cart button on the deck profile screen's action dock (web at every size, where the wide page also shows the cards, and the phone app), reusing the dock button component, opening the same Card Kingdom / TCGplayer dialog for the whole deck. Not on the separate deck cards screen, whose stacks have their own carts.
- Every buy entry point in web and app, where the app shows a "Buy" button and the web two retailer buttons (Budget's row and the featured spot included), becomes the cart icon opening the one store dialog, so web and app match. Budget loses its buy buttons; the dock cart replaces them.

## Cause
Each surface grew its own buy control: the web's Budget row and featured spots show the two retailer links, the app's Budget shows two buttons and its featured flavor a Buy button with a dialog.

## Fix
thinker, after 353: the shared `StoreDialog` and `CartButton` everywhere a card, a stack or the deck can be bought; the PR lists every spot changed.
