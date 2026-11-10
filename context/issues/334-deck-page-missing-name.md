# 334. The web deck page shows no deck name

State: fixing | Owner: thinker | PR: - | Build seen: live 1.18.16

## Report
Scotty, 2026-11-09: the share page on zwipe.net (`/deck/<id>`) shows the deck name as a heading above the chips (card count, format, commander, price). The app's deck page on app.zwipe.net (`/decks/<id>`) shows the same chips with no heading.

## Diagnosis
thinker, 2026-11-09. Both pages draw components' `DeckHead`, which puts `name` in an `h1.deck-head-title` over the chips. zite passes the name. zwiper-web's wide deck page (`deck/view/page.rs`) stopped passing it in 306b95fd6, on the grounds that the tab and the nav carry it. On a wide window the nav shows its links, not the screen title, so only the tab named the deck. Below the breakpoint the web deck view is the app's stack: the folded nav shows the deck name and the profile card has a Name row. The native app's deck view shows a "Deck" header with a Name row in Profile. Neither lacks the name.

## Fix
- The wide page passes the deck name to `DeckHead`, so it gets the share page's own heading (same component and class). Its loading ghost gains a title bar in the same place.
- `ScreenFrame` takes `titled`. The deck view sets it on a wide window once the profile is in, so the frame's screen-reader `h1` with the same name stays out and the page has one `h1`.
- Phone width and the native app are unchanged, so nothing doubles.
