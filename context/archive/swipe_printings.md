# Printings while swiping (add / remove / commander)

**Status: SHIPPED 2026-07-11 (`999ef00a`, "view printings while swiping"). All
three screens (add, remove, commander) done; remove is view-only as decided.
Client-only — no server or contract changes. The spec below records what was built.**

## Goal

Let users view — and where safe, pick — a card's printings from the swipe
screens (add cards, remove cards, commander select), from the existing
eyeball/details flow, without a trip to the deck view screen.

## The reusable piece

The deck **view** screen already has the full printing flow:
`PrintingSheet(card, open, on_save)`
(`zwiper/.../deck/card/components/printing_sheet.rs`) — a bottom sheet that
fetches printings by `oracle_id` (`GET /api/card/{oracle_id}/printings`), shows a
carousel, and enables Save only when the selection changed. Crucially it
**persists nothing itself** — it calls `on_save(new_card)` and the host decides.
So we reuse `PrintingSheet` on the swipe screens with a per-screen `on_save`.

**Identity model (drives everything):** every `Card` carries `scryfall_data.id`
(printing-specific) and `scryfall_data.oracle_id` (stable). Deck rows are keyed
server-side by **`scryfall_data_id`** (`/api/deck/{deck_id}/card/{scryfall_data_id}`
for both update and delete).

## Collision with the live otags work (none — timing cleared by owner)

otags is server + shared-type + separate-frontend work; this is client-only. Owner
confirmed (2026-07-11) the otags agent won't reach frontend changes soon, so this
builds freely. `create.rs` is still avoided anyway (commander printing lives in
`swipe_select.rs`, keeping deck-creation untouched). Files touched:
`printing_sheet.rs`, `card_stack.rs`, `card_info.rs`, `add.rs`, `remove.rs`,
`swipe_select.rs`.

## Mount point (all three screens)

The **"Printings" button lives inside the eyeball `CardRulesDialog`**
(`card_info.rs`), as a footer action — the owner's original UX and a match for the
deck-view pattern (expand row → Printing button). `CardRulesDialog` gains a
`#[props(default)] on_printings: Option<EventHandler<()>>`; when set it renders a
"Printings" button that fires the handler. Each swipe screen renders its own
screen-level `PrintingSheet` (driven by a signal + the focused `current_card()`)
and wires `on_printings` to open it (`read_only` per screen). The swipe screens
don't import `PrintingSheet` today — they will.

## Add screen (`add.rs`) — select-and-swipe-to-commit ✅

- Focused card = `stack.current()` (a `CardStack`). Swipe-right →
  `create_deck_card(HttpCreateDeckCard::new(&card.scryfall_data, ...))` — sends
  both ids, both derived from the card's `scryfall_data`.
- `on_save(new_card)` = **replace the focused card in the stack** with the chosen
  printing. Nothing persists; swipe-right then adds that printing.
- Needs a "replace current" on `CardStack` (it has `replace(Vec)` / `append` /
  `park` but no replace-current) — add `replace_current(card)` or mutate
  `cards[index]`.
- Flow: browse printings → pick → the swipe card re-skins → swipe right adds it.

## Commander select (`swipe_select.rs` / `create.rs`) — same as add ✅

- Focused card = `cards[current_index]`; swipe-right → `on_select(card)` → stored
  in the slot signal; deck creation persists `commander_id = card.scryfall_data.id`
  (printing id). Same for partner / background / signature spell.
- `on_save(new_card)` = swap the focused (or stored slot) card to the chosen
  printing; swipe-right chooses that printing.
- Safe: `SignatureSpell` mode keys off the commander's `color_identity`, identical
  across printings — no refetch churn.

## Remove screen (`remove.rs`) — view-only (DECIDED 2026-07-11)

- Focused card = `stack.current_wrapping()`; swipe-right →
  `delete_deck_card(deck_id, card.scryfall_data.id)` — keyed on the **exact
  printing id**.
- Hazard (owner-spotted, code-confirmed): swapping the printing locally then
  removing would target a NEW printing id that isn't a row on the deck →
  404 / mismatch.
- **Decision: printings are view-only here.** Open the same bottom sheet in
  read-only mode — no Save, `on_save` unused — so the focused card's
  `scryfall_data.id` never changes and swipe-right always deletes the real deck
  printing. Zero mismatch risk, minimal code.
- Rationale: changing a printing is an *edit*, and that already lives on the deck
  **view** screen (expand row → Printing → `update_deck_card`); remove stays
  destructive-only. Rejected "persist-the-change-on-save then remove": it fires a
  deck mutation from the remove screen and has a browse-then-skip footgun (a card
  you *keep* gets silently re-printed).

## Pieces to build (all client-side)

1. **`PrintingSheet` gains a `#[props(default)] read_only: bool`** — hides/disables
   the Save button and the change tracking, so the remove screen can open it
   purely to browse. Default `false` keeps the view screen unchanged.
2. **`CardStack::replace_current(Card)`** — swaps the focused card in place
   (the stack has `replace(Vec)`/`append`/`park`, not this). Used by add.
3. **A "Printings" button** in the `CardRulesDialog` footer (`card_info.rs`), so
   every eyeball dialog can open the sheet. Opens `PrintingSheet { card:
   current_card(), read_only: <screen-specific> }`. (Alternatively a button in the
   ActionBar beside the eyeball — owner preference; the dialog footer mirrors the
   view screen's expand-row → Printing.)
4. **Per-screen `on_save` wiring:**
   - `add.rs`: `on_save(new)` → `stack.replace_current(new)` (then swipe-right adds
     the new printing; nothing persists on select).
   - `swipe_select.rs`/`create.rs`: `on_save(new)` → swap the focused/slot card to
     `new` (then swipe-right selects that printing).
   - `remove.rs`: open with `read_only: true`; no `on_save`.
5. Import `PrintingSheet` into the three swipe screens (only `view.rs` does today).

## Ship / verify

Client-only, no server change (add/commander/Option A). Option B reuses the
existing `update_deck_card` path (also no server change). Verify per
`context/development/commit_guidelines.md` (nightly fmt, clippy, tests), then
`dx serve`: add screen — pick a printing, confirm the added card is that printing;
remove — confirm swipe-right removes the real card regardless of what was browsed;
commander — confirm the chosen printing lands as `commander_id`.
