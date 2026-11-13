# One deck route

Issue 430. One route for a deck in the app, on the web and for a shared deck, with the Profile as the page and the Cards as a layer over it on a phone. 438 builds the phone Profile's content on top of this.

## The rule

From Scotty:

- The owner's deck page and the shared deck page are the same thing, not two similar things. Owner mode adds the edit actions; read-only mode leaves them out. Both match the phone app.
- The deck route with no parameter is the Profile. `?view=cards` is the Cards screen. There is no `?view=profile`.
- On a phone (the app, and the web below 900px) Cards is a full-screen layer over the Profile, registered on the overlay stack (424), so Back and Esc close it, which drops the parameter. The Profile's dock gets a Cards button after a divider, as the app's owner dock has (#593); the layer has the app's back button at the top left.
- Wide web keeps both on one page, as today.
- `/decks/<id>/cards`, and any shared cards form, redirect to the deck route with `?view=cards`.
- Routing is client-only: no wire change.

## Routes

| Where | Profile | Cards | Old form, redirected |
|---|---|---|---|
| Web, owner | `/decks/:deck_id` | `/decks/:deck_id?view=cards` | `/decks/:deck_id/cards` |
| Web, shared | `/:owner/deck/:slug/:token` | `/:owner/deck/:slug/:token?view=cards` | none exists today |
| App | `/deck/get/:deck_id` | `/deck/get/:deck_id?view=cards` | `/deck/card/view/:deck_id` (the variant goes; its callers move) |

`DeckView` goes in zwipe-ui (shared, no UI crate): one variant, `Cards`, read from and written as `cards`. Each route carries `view: Option<DeckView>`, `None` being the Profile, so an unknown value falls back to the Profile. Dioxus writes a query route with no value as a trailing `?`; if that shows in the address bar, the web strips it on navigation (a one-line `replaceState`), checked in a test of the route's `Display`.

## One screen

One component per width does each part, for both modes:

- **`DeckScreen`** (zwiper-web) takes a `DeckSource`: the owner's editor or the shared deck, each filling the same signals the wide page's `DeckPageBody` already takes (listing, meta, group by, boards, show, view, folds, faces, details, filter, currency, stats, the store links), plus `actions: Option<DeckEditActions>` and the owner's extras (Quick add, add, remove, undo, the foot's edit panels). Owner and shared routes both render `DeckScreen`; the two separate narrow screens (the owner's `ViewDeck` stack and its `ViewDeckCard` screen, the shared page's Profile | Cards switch) fold into it.
- **Wide** renders `DeckPageBody` as today, from the same source.
- **Phone Profile** is the page: in 430 the current owner Profile (the app's structure), read-only for a shared deck; 438 settles its content and panel style. Its dock: owner = edit, clone, share | Cards; shared = filter, cart | Cards.
- **Phone Cards layer** is the 428 cards screen (View chip, touch Stacks, group price and cart) taking the source, with Quick add, add, remove and undo only in owner mode. It draws over the Profile as a full-screen layer with the app's `ScreenHeader` back button.

## Opening and closing the layer

- The Cards button pushes the route with `?view=cards`, so the browser's and the OS back pop it.
- The layer registers through `use_overlay_back_action` while `view` is `Cards`. Its close (back button, Esc, Android back) goes back one entry when the layer was opened from the Profile in this session, and otherwise (a deep link, a refresh) replaces the route with the Profile, so Back never leaves the deck by mistake.
- The layer keeps the Profile mounted under it, so closing it shows the Profile where it was.
- On wide web `?view=cards` shows the one page and scrolls to the cards.

## The app

The app's deck view takes `view` and draws its Cards screen (`deck/card/view.rs`) as the layer over it, on the same overlay stack. `Destination::DeckCards` becomes the deck route with `view: Some(Cards)`; the menu's section map and its test drop `ViewDeckCard`. Add and remove return to the Cards layer by pushing the deck route with the parameter, as they return to the Cards screen today. The leave guard and the app's history handling read the route as before.

## As built

- `DeckScreen` is `DeckPageBody`: both routes already fed it on a wide window, and now feed it at every width. On a phone it draws the host's `phone_profile` (the owner's app-style Profile; the shared deck passes none and keeps its panels until 438) or the page split to its profile, then the dock or the Cards layer (`DeckCardsLayer`).
- The web's history guard closes the top overlay on a browser back and puts the URL back. It now reads that URL after the close, so the layer, whose close moves the route, keeps the move. The layer's close checks the URL first: when the browser has already left the Cards it only records the Profile, so a browser Back never goes back twice.
- The router writes an empty query as a bare `?`; the trail history drops it.
- `DeckControls` takes the View row above Group by, the app's order, for the wide page, the layer and the app.

## Checks

- Web 390 and 1450, owner signed in and a shared deck signed out: Profile; Cards opens the layer and adds `?view=cards`; back button, Esc and browser Back each close it and drop the parameter; refresh on `?view=cards` opens the layer, and closing it lands on the Profile; `/decks/<id>/cards` lands on the layer; a shared link and its `?view=cards` open the same way.
- App: deep link into a deck, Cards and back, add then back to Cards, Android back closes the layer first.
- Wide: one page, the parameter scrolls to the cards.
- Tests: route parse and display round-trips (with and without the parameter, unknown values), the redirects, the app's `route(Destination)` map.

## Order of work

1. `DeckView` and the routes with the redirects, on the web and in the app.
2. `DeckSource` and `DeckScreen` on the web: the Cards layer from the 428 screen, the owner's and the shared deck's Profile under it, the dock's Cards button.
3. The app's Cards screen as the layer over its deck view.
4. Checks above, then 438 on top.
