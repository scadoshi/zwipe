# 430. A deck's Profile and Cards are separate screens on narrow web and in the app, and the shared deck splits them with a chip row

State: fixing | Owner: laptop | PR: - (pushed branches, hashes in the composer's log) | Build seen: 0dd549305 | Type: issue

## Report
Scotty: the shared deck page at phone width uses a Profile | Cards chip row; it should work like the phone app's deck page, Profile as the page and a Cards button in the dock that opens the Cards with the app's back button. Then, approved: one deck route for the app, the web and the shared deck. No parameter is the Profile; `?view=cards` is the Cards, a full-screen layer over the Profile on a phone, on the overlay stack so Back and Esc close it and drop the parameter; wide web keeps one page; `/decks/<id>/cards` redirects. The owner's page and the shared page are the same thing, owner mode adding the edit actions. droider: the View chip sits in three places; take the app's (above Group by); a shared group header shows its price as the owner's does.

## Fix
laptop, per `context/plans/one_deck_route.md`.
- zwipe-ui: `DeckView` (`Cards`, read and written as `cards`); `DeckArea` goes.
- Web routes: `ViewDeck { deck_id, view }` at `/decks/:deck_id?:view` and `SharedDeck { .., view }`; `/decks/:deck_id/cards` redirects to `?view=cards`; `ViewDeckCard` goes. Add and remove go back to the Cards; edit, import and export to the Profile. The router writes an empty query as a bare `?`, which the trail history drops.
- Web screen: the owner's deck and the shared deck render `DeckPageBody` at every width. On a phone it draws the Profile (the owner's app-style Profile, or the shared deck's panels until 438) with a dock of the host's buttons, the cart and Cards, and with `?view=cards` the Cards layer (`DeckCardsLayer`, the 428 screen made generic: the deck's head, the controls, the List groups with price and cart or the touch stacks, a dock of add and remove, the filter and undo for the owner, the filter alone for a shared deck). The layer sits on the overlay stack. `use_cards_route` opens it by pushing the route and closes it by going back when it opened here, else by replacing the route with the Profile. The old narrow screens (the owner's cards screen and its featured strip) go.
- Web history guard: on a browser back with an overlay open it reads the screen's URL after closing the top overlay, so a layer the route holds open keeps the move its close makes.
- `DeckPage`'s Profile | Cards switch becomes `split`; `DeckControls` takes the View row above Group by, as the app has it, for the wide page, the layer and the app (which now uses the shared `CardsViewSwitch`).
- App: `ViewDeck { deck_id, view }` at `/deck/get/:deck_id?:view`, `/deck/card/view/:deck_id` redirects, `Destination::DeckCards` opens the layer; the deck view draws its Cards screen as a layer on the overlay stack, so the OS back closes it first.

## Checked (headless Chrome, app.zwipe.net local, 390)
- Owner, signed in: Profile has no parameter; Cards adds `?view=cards` and opens the layer; its back button, the browser's Back and Esc each close it back to the Profile; a load on `?view=cards` opens the layer and closing it replaces it with the Profile; `/decks/<id>/cards` lands on the layer.
- Shared deck: no chip row; the dock is filter, cart and Cards; the layer shows each group's price and cart.

## Verification
-
