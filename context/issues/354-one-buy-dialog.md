# 354. Buying is a different control in every spot, and the wide deck page hides it

State: fixing | Owner: thinker | PR: - | Build seen: production, web wide

## Report
Composer for Scotty, 2026-11-11: on wide web the deck page seemed to have no Buy, which lives in the Budget widget (folded by default). Asked for, after 353:
- A cart button on the deck profile screen's action dock (web at every size, where the wide page also shows the cards, and the phone app), reusing the dock button component, opening the same Card Kingdom / TCGplayer dialog for the whole deck. Not on the separate deck cards screen, whose stacks have their own carts.
- Every buy entry point in web and app, where the app shows a "Buy" button and the web two retailer buttons (Budget's row and the featured spot included), becomes the cart icon opening the one store dialog, so web and app match. Budget loses its buy buttons; the dock cart replaces them.

## Cause
Each surface grew its own buy control: the web's Budget row and featured spots show the two retailer links, the app's Budget shows two buttons and its featured flavor a Buy button with a dialog.

## Fix
thinker: one cart, one store dialog (`CartButton`, `StoreDialog`, from 353), with zwipe-ui's `buy::deck_links` and `buy::card_links`:
- Web deck page, wide: a cart in the action dock after Share, for the whole deck.
- Web deck view, phone width: a cart in the dock beside Cards.
- Web Budget (wide fold and phone accordion) and its loading ghost: the TCGplayer and Card Kingdom links are gone.
- Web home featured flavor: the two retailer links become the cart (its ghost too).
- App deck view: Budget's Buy row is gone; a cart in the dock beside Cards.
- App home featured flavor: the Buy button becomes the cart, opening the shared dialog.
- zwipe.net guide: the Budget paragraph names the dock cart and the stacks' carts.
- zwipe.net featured flavor: the two retailer links become the cart (follow-up PR). orangui#36 moved the dialog's rules into their own `DIALOG_CSS`, which zwipe.net now loads, mounting the dialog host at its root.
- Removed with them: zwiper-web's `cards/buy_links.rs`, zwipe-ui's `Retailer`, `Action::Buy` and the two `BUY` labels, and the `.deck-buy-row` and `.web-buy-link` rules.
