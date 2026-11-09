# 322. Deck list rows: the tags drop under the art instead of beside it

State: fixing | Owner: thinker | PR: #486 | Build seen: 1.18.15

## Report
Scotty, 2026-11-09. On Decks, in the app and on the web, a row's command zone art sits beside the deck name, but the tags (color pips, card count, price, format, bracket, the command zone's cards, deck tags) wrap onto lines under the art. That leaves empty space to the right of the art. The name and the tags should share one column beside the art: the name on the first line, the tags wrapping beneath it.

## Fix
Branch nits/thinker-322, from scotland's WIP on nits/scotland-322. Each deck row is the art, then a `.deck-row-text` column holding the name over the chips, in the app's and the web's `DeckRow` and in both deck list skeletons. The art is top-aligned beside the column, and a long name wraps inside it. The CSS is the shared `components/assets/screens/components.css`, plus the web's copy of it in `zwiper-web/assets/sheets/07-screens.css`. At 1280 on the web a row is 25px shorter, with every chip beside the art. At 390 the chips wrap in the narrower column. There are no component tests for these rows, so there is no failing-first test.
