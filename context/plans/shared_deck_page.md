# Shared deck page: one deck page component for zite and zwiper-web

Issue 370. Status: approved 2026-11-12; phase 1 shipped (#572); phase 2 in two PRs, the carts and stat panels (#592) then the cards view; phase 3 in #619 and #621, 4b-1 in #622; phase 4 (zite on the shared page, read only) in #624.

## Goal

zwipe.net's shared deck page (`/<user>/deck/<slug>/<token>`) matches app.zwipe.net's deck page, read-only, from one component both sites use with an editable flag, not a copy.

- **Phone width:** two areas, the deck profile and the cards, like the app.
- **Wide:** one page showing both, at the full width of the web deck page (350): list groups in two columns from 1400px, stacks filling the width.
- **Controls:** List and Stacks, Group by, Show, Boards (a shared deck sends every board), the filter, image, printings (view only), details, the group carts, flip and Meld.
- **Read-only keys:** `v`, `d`, `i`, `p`, `f`, `?` and Shift+H. No add, remove, move, star, quantity or undo.

## Where things are today

- **zite** `pages/shared_deck.rs` (850 lines) draws its own page from `components`' `DeckPage` pieces (`DeckHead`, `DeckControls`, `DeckBand`, `DeckFold`) with its own `CardRow` wrapper, filter and folds. It has no Stacks, no keys, no carts and no printings.
- **zwiper-web** `screens/deck/view/page.rs` with its siblings `stacks.rs`, `sections.rs`, `group_buy.rs` and `panels.rs`, plus `deck/cards/editor.rs` (`row_view`, `use_deck_editor`, `hover_keys.rs`) and `chrome/` (`page_keys`, `shortcuts`, `shortcuts_sheet`, `details_clear`). The page is tied to `Editor`, which owns the deck's entries and the API mutations.
- **Pure logic** already in `zwipe-ui`: `deck_cards` (sections, grouping, zones), `deck_cards::stacks` (`Reveal`, `Gesture`, `CardsView`), `filter`, `buy`.
- **Shared UI** in `components`: `DeckPage` and its parts, `CardRow`, `CardDetails` and `CardDetailsPanel`, `CardStack`, `CardBoardMoves` and `CardEditButtons`, `MeldSource`, `data_action`.

## What moves where

| Piece | From | To | Notes |
|---|---|---|---|
| Hover keys (`HoverKey`, `hover_key`, `hover_action`) | zwiper-web `deck/cards/hover_keys.rs` | zwipe-ui `deck_cards::keys` | pure; `hover_action` takes `Editable::{Yes, No}` and returns `None` for m, s, Shift+S, -, + when read-only |
| `card_key`, `CardKey`, `SWIPE_LETTERS`, `GLOBAL_KEYS`, the sheet rows and notes | zwiper-web `chrome/shortcuts.rs` | zwipe-ui `shortcuts` | pure; the rows take the editable flag, so the shared deck's sheet lists only read-only keys |
| Browser key plumbing (`use_page_keys`, `use_page_keys_through`, `use_global_keys`, the guards, `press_first_js`, `modal_hint_js`, `KEY_CARD_JS`, `PRESS_JS`) | zwiper-web `chrome/` | components `keys` | DOM scripts both sites need; no API calls |
| Shortcuts sheet (`ShortcutsSheet`, `use_offer_shortcuts`) | zwiper-web `chrome/shortcuts_sheet.rs` | components | same sheet on both sites |
| Deck cards view: List bands and rows, Stacks, the stack details drawer | zwiper-web `view/page.rs` (bands), `view/stacks.rs`, `editor.rs` `row_view` | components `deck_cards_view` | takes the sections, view, folds, `Reveal`, faces, and `actions: Option<DeckEditActions>` |
| `DeckEditActions` (`change_quantity`, `move_to_board`, `toggle_mvp`, `save_printing`, undo) | zwiper-web `Editor` callbacks | components, as a props struct | `None` hides every edit control and key: the read-only flag |
| Group carts dialog | zwiper-web `view/group_buy.rs` | components | link builders are already `zwipe_ui::buy` |
| Stat panels (Budget, Tags, Distributions, Mana, Draw odds) | zwiper-web `view/sections.rs` | components | zite draws its own today; one copy after the move |
| Printings sheet, view only | zwiper-web `cards/printing_sheet.rs` | components, fed by a `PrintingsSource` context like `MeldSource` | components never call the API; the host fetches `GET /api/card/{oracle_id}/printings`; Save shows only with `save_printing` |
| Image preview, card details sheet, Oracle tag examples | zwiper-web `cards/` | components where they are UI only | the examples' fetch goes behind an `ExamplesSource` context |
| Deck page CSS (350 full width, two list columns, stacks, details clear) | zwiper-web `web.css` | components `magic.css` (sheet regenerated) | zite takes the same rules; `.web-deck-page` becomes `.deck-page-wide` on both |
| `details_clear` script | zwiper-web `chrome/details_clear.rs` | components | shared |

`Editor` and `use_deck_editor` stay in zwiper-web: they hold the session, the mutations and undo, and build a `DeckEditActions` for the shared view. zite builds the sections from `HttpSharedDeck` with the same `zwipe_ui::deck_cards` functions and passes `actions: None`.

## Read-only gating

- **Hidden or inert with `actions: None`:**
  - controls: Quick add, the quantity stepper, Star, the board moves, Printing save, the Danger zone, Data import, Edit, the warnings' Add and Remove actions
  - keys: m, s, Shift+S, -, +, u and Ctrl+Z
  - the sheet's rows for those keys
- **Kept:** List and Stacks, Group by, Show (tokens, command zone, art), Boards, the filter, Image, Printings (view), Details, Flip, Meld, the carts, folds, and the keys v, d, i, p, f, ?, Shift+H, Esc.
- **One gate:** `actions.is_some()`, threaded as `Editable` into the pure key maps, so a test can pin every edit key to the editable deck only.

## Wire and API

No wire or schema change:
- `HttpSharedDeck` already carries the profile, the commander slots, every board's entries, the tokens, the owner and the share names.
- `GET /api/card/{oracle_id}/printings` and `GET /api/card/{id}` (meld) are public routes.
- Metrics are computed client-side, as zite does now.
- zite provides `MeldSource`, `PrintingsSource` and `ExamplesSource` from its `zwipe-client`.

## Phases (one PR each, no visible change until phase 4)

1. **Pure logic to zwipe-ui:** hover keys and the shortcut maps and rows, with `Editable`. zwiper-web adopts. Tests pin the read-only key set.
2. **The deck cards view to components:** List, Stacks, the drawer, rows and `DeckEditActions`, plus the group carts and the stat panels. zwiper-web adopts with `Some(actions)`. The HTML goldens show no change; check the web deck page headless before and after.
3. **Key plumbing, the shortcuts sheet, the printings and examples sources, and `details_clear` to components;** zwiper-web adopts.
4. **zite's shared deck page switches to the shared view, read-only:**
   - Stacks, every read-only control and key
   - full width with the CSS moved to components
   - zite provides the sources
   - 1280, 1920, 2560 and phone shots against the web deck page
5. **Phone layout, two areas.**

## Decisions

1. Phone areas on zite: an in-page Profile | Cards switch under the head, so the share link stays one URL (Scotty).
2. Shared deck stats: all five stat panels (composer).
3. Printings on zite: view only (composer).
