# Deck MVPs — client changes (zwiper)

All in phase 1 (1.4.0). Copy rules: sentence case, no em dashes.

## 1. API client — `zwiper/src/lib/outbound/client/deck_card/update_deck_card.rs`

The request body gains the `mvp: Option<bool>` field (comes free once the
contract updates; extend the call-site helper so screens can send
`mvp: Some(bool)` without touching quantity/board).

## 2. Star in the main row — `zwiper/src/lib/inbound/screens/deck/card/components/card_row.rs`

- `CardRow` props gain `mvp: bool` and an `on_toggle_mvp: EventHandler<()>`
  (or ride an existing callback bundle if one forms).
- Render a star at the row's leading edge, **visible without expanding**
  (owner call): filled star when MVP, outline when not. Text glyph (★ / ☆)
  fits the terminal aesthetic — no glow, no animation beyond the standard
  press state.
- Tap toggles; hit target ≥ 44pt.

## 3. Wiring — `zwiper/src/lib/inbound/screens/deck/card/view.rs`

- Deck response now carries `mvp_at` per card; `mvp = mvp_at.is_some()`.
- Toggle handler: `update_deck_card(deck_id, scryfall_data_id, mvp:
  Some(!current))`, then refresh the deck (or optimistic flip + rollback on
  error, matching however quantity edits behave today).
- **Cap error**: on 422, toast the server message verbatim ("This deck
  already has 3 MVPs"). No client-side pre-count needed beyond disabling
  nothing — let the server be the referee.
- Mainboard only: the star renders on `deck`-board rows; maybeboard/sideboard
  rows show none (server clears `mvp_at` on board moves, so state stays
  consistent).

## 4. One-time hint

`hints_shown` key `"deck_mvps"` via the existing `mark_hint_shown` machinery:
first visit to the deck cards screen after update shows a short hint —
"Star up to three MVPs: the cards that define this deck. Zwipe leans your
suggestions toward them." (Steering copy can soften to future tense until
phase 3 ships; confirm wording at build.)

## 5. Clone + export

- Clone: nothing client-side (server copies `mvp_at`).
- Text export (`export.rs` screen / core decklist formatter): mark MVP lines
  (proposed: trailing `*`). Import ignores the marker (strip it in the
  parser so round-trips are lossless). Confirm the exact marker at build —
  it must survive the plain-text importers of other tools.

## 6. zite guides

`zite/src/pages/guides/content.rs`: short section in the deck-management
guide ("Deck MVPs: star up to three cards that define your deck…"), shipped
with the 1.4.0 release notes pass. Store What's New copy: both
`form_fields.md` files at release time.
