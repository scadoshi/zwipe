# 322. Deck list rows: the tags drop under the art instead of beside it

State: fixing | Owner: thinker | PR: #486, #489 | Build seen: 1.18.15

## Report
Scotty, 2026-11-09. On Decks, in the app and on the web, a row's command zone art sits beside the deck name, but the tags (color pips, card count, price, format, bracket, the command zone's cards, deck tags) wrap onto lines under the art. That leaves empty space to the right of the art. The name and the tags should share one column beside the art: the name on the first line, the tags wrapping beneath it.

## Fix
Branch nits/thinker-322, from scotland's WIP on nits/scotland-322. Each deck row is the art, then a `.deck-row-text` column holding the name over the chips, in the app's and the web's `DeckRow` and in both deck list skeletons. The art is top-aligned beside the column, and a long name wraps inside it. The CSS is the shared `components/assets/screens/components.css`, plus the web's copy of it in `zwiper-web/assets/sheets/07-screens.css`. At 1280 on the web a row is 25px shorter, with every chip beside the art. At 390 the chips wrap in the narrower column. There are no component tests for these rows, so there is no failing-first test.

## App report, 2026-11-09
Scotty built main at bb566fc4b to his phone and Decks still showed the tags under the art. Droid confirmed the live web shows the new layout at 411px. The app's code at bb566fc4b is right:
- Its one `DeckRow` (`zwiper/src/lib/inbound/screens/deck/list.rs`) has the `.deck-row-text` column.
- It loads the shared rule through `components::SCREENS_CSS`, which concatenates `screens/components.css`.
- Nothing in `zwiper/assets/main.css`, `magic.css` or orangui's sheets touches `.deck-list-row`, `.deck-row-text`, `.deck-row-chips` or `.deck-list-name`.

The deck list's DOM under the app's exact stylesheet stack (themes, components, magic, app, screens, filter, main.css, toast) renders the new layout in Chromium (the chips start at x=84, beside the art ending at x=75) and in WebKitGTK, AppleWebKit/605.1.15 (the column at x=100, beside the art ending at x=91). So the phone's build most likely didn't contain the change, from a stale build or an install that didn't replace the app. The in-app changelog listing 1.18.16's "Decks shows each deck's name and tags together beside its art" is a quick check that a build has it.

## Inline name and tags, 2026-11-09
Scotty, after seeing live web: the tags go on the same line as the name, flowing right after it and wrapping onto the next lines, still in the column beside the art. CSS only, in `components/assets/screens/components.css` and the web's copy:
- `.deck-row-text` is a wrapping row instead of a column.
- `.deck-row-chips` is `display: contents`, so each chip is an item of the name's line.
- A name longer than the column wraps inside it, and the chips follow on the next line.

The same markup serves the app, the web and both skeletons.
