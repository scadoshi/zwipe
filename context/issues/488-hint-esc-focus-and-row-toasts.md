# 488. Esc after the `?` hint sends focus to the deck title, and `+` and Shift+S raise no toast

State: fixing | Owner: thinker | Type: bug | PR: - (pushed to release) | Build seen: 045bd5c38

## Report
surfer's before-480 run (deck page at 1440): `+` and Shift+S give no toast while `-`, `m` and `s` do; Esc after the `?` hint sends focus to the deck title, not the card.

## Before
Headless Chrome at 1440x900, a fine pointer forced, the owner's deck page with mocked reads (writes failed, none sent), 3 of 3: the cursor on Sol Ring (`↓` twice), `?` opens the hint, Esc closes it, and focus lands on `main.web-main` (its text starts with the deck's name), not the card.

## Cause
- **Esc:** `HINT_FOCUS_JS` (`zwiper-web/src/lib/inbound/app.rs`) never hands focus back to the opener, to keep Enter off the `?` that opened it: it goes to the open drawer, else the opener's form or panel, else the page's main column. A card the cursor held is none of those, so it lost focus to the main column.
- **Toasts:** neither the web nor the app's model toasts a quantity change or a star, from a key or a button; a move and a removal toast. surfer's `-` toast was a removal ("Card removed"); `-` on a card with copies left is silent too.

## Fix
- **Esc:** a closing hint hands focus back to its opener when that is a card the deck cursor stops on (`components::deck_cursor::STOP_SELECTOR`, now public); every other opener goes where it went. Test: `a_closing_hint_hands_focus_back_to_the_card_the_cursor_held`.
- **Toasts:** by design, Scotty via the composer: the row shows the change, so `+`, `-` and the star stay silent.

## After
Same setup, 3 of 3: after Esc, focus is back on the Sol Ring row the cursor held.
