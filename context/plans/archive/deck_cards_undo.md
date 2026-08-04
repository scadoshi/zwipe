# Plan: deck-cards undo (single button, in-memory)

**Status (2026-08-04): DONE.** All four phases complete:

- ✔ Phase 1 — quantity debounce, `319747c7`. Verified live against zerver
  logs (+9 burst → one PUT, net-zero burst → no call, delete on crossing).
- ✔ Phases 2–3 — undo built per Architecture below (`undo_log.rs`, five
  recording points, `apply_undo`, conditional ActionBar Undo button).
- ✔ Phase 4 — verified on device (owner UI pass, 2026-08-04), committed as
  `3885049e` with the 1.7.5 changelog bullet.

Remaining ideas live in Parked below (MVP restore, command-zone printing,
the e2e-harness scenarios). No open work in this plan.

## Decisions already made (don't relitigate)

The design walked through several shapes before landing. Outcomes:

- **In-memory only.** No DB persistence (privacy surface + it's really a
  separate "deck changelog" feature), no local-disk persistence (versioning
  tax for marginal benefit). Stack dies with the app; that also flushes
  entries that could conflict with edits from elsewhere.
- **One Undo button, no list.** A "recent actions" list UI was considered
  and dropped — visuals only, not worth it. Repeated taps walk the stack
  top-down, one action per tap.
- **Screen-scoped to deck cards** (`screens/deck/card/view.rs`). No
  deck-profile actions (name/tags edits were considered and dropped).
- **Toasts carry the card name** — with no list, the toast is the only
  place the user learns what changed: "Re-added Goldspan Dragon".
- **v1 action set** (the screen's full mutation inventory, triaged):

| Action | Undo | In v1? |
|--------|------|--------|
| Card add (quick-add) | delete it | yes |
| Card remove (qty → 0) | re-create with old board + pre-burst qty | yes — the prize case |
| Qty change | inverse net delta | yes, **coalesced per debounce burst** |
| Board move | move back to old board | yes (missed in first draft; ranked #2 value) |
| Printing change, regular card | restore old printing id | yes |
| Printing change, command zone | — | no — different plumbing (deck profile), rare, low regret |
| MVP star | — | no — one tap reverses it; undo would fight the server vesting clock |
| Filter changes | — | no — not deck mutations |

- **Coalescing rides the debounce**: one burst = one server call = one undo
  entry. That was half the argument for building the debounce first.
- **MVP restore on remove-undo is skipped in v1** (re-create can't set it;
  a follow-up update could, best-effort — parked).
- **Undo checks current state first**: if the card was since removed/changed
  so the inverse no longer applies, consume the entry and toast "Already
  changed" instead of firing a doomed request.
- **Undo must not record onto the stack** (no ping-pong): the mutation
  closures gain a `record: bool` and undo calls them with `false`, or undo
  applies its own direct path where reuse doesn't fit (re-create).

## Architecture

New file `zwiper/src/lib/inbound/screens/deck/card/components/undo_log.rs`:

```rust
pub enum UndoAction {
    Added { card_id, card_name },                       // undo: delete, NO record_removal telemetry
    Removed { entry: DeckEntry, baseline: i32 },        // undo: re-create board + baseline qty
    QuantityChanged { card_id, card_name, net: i32, baseline: i32 }, // undo: apply -net
    MovedBoard { card_id, card_name, from: Board },     // undo: move back
    PrintingChanged { old_card: Card, new_id: Uuid },   // undo: with_printing(old id) + entry swap
}
pub struct UndoLog(pub Signal<Vec<UndoAction>>);        // cap 100, drain front on overflow
```

Provided as context by `View` so `QuickAdd` (the one external recorder) can
push. Everything else records inside `view.rs`.

Recording points:

- `QuickAdd` `add_card` success arm → `Added`.
- `change_quantity`, removal tap → `Removed` (capture the `DeckEntry` clone
  *before* the retain; `baseline` from the pending-burst map, else
  `current_qty`).
- Debounce flush, update arm, net ≠ 0 → `QuantityChanged` (name looked up
  from entries at flush; entry exists because the removal case didn't fire).
  Note: recorded at flush, so a burst isn't undoable during its 300ms
  window — accepted.
- `move_to_board` → `MovedBoard` with the pre-move board (already captured
  for rollback).
- `PrintingSheet` `on_save`, regular-card arm only → `PrintingChanged`
  (old card is in hand pre-swap).

Applying (an `apply_undo` closure in `view.rs`, popped entry):

- `Added` → optimistic retain + `delete_deck_card`. Skip-consume if the
  entry is already gone.
- `Removed` → optimistic push of the saved entry + `create_deck_card`
  (`HttpCreateDeckCard::new(&card.scryfall_data, baseline, board)` — board
  wire strings are lowercase `display_name`, parser verified). Adopt the
  returned `DeckCard` (fresh ids). Roll back the push on server error.
  Skip-consume if a card with that scryfall id is already back in entries.
- `QuantityChanged` → `change_quantity(card_id, -net, false, record: false)`
  — rides the same debounce; flush posts the inverse. Skip-consume if the
  entry no longer exists.
- `MovedBoard` → `move_to_board(card_id, from, record: false)`. Skip-consume
  if gone.
- `PrintingChanged` → replicate the on_save regular arm with the old card
  (update `with_printing`, swap entry back). Skip-consume if the entry no
  longer holds `new_id`.

UI: conditional `Button { "Undo" }` in the ActionBar, rendered only when
the stack is non-empty (same pattern as the conditional "Reset filter").

Toast copy (plain, name-first): "Removed {name}", "Re-added {name}",
"{name} back to ×{n}", "{name} back to {board}", "{name} printing restored",
"Already changed".

## Phases

1. ✔ **Qty debounce** — done, `319747c7`.
2. **UndoLog scaffolding** — `undo_log.rs` (enum, newtype, cap, push
   helper), module registration, context provision in `View`, `record: bool`
   threaded through `change_quantity` / `move_to_board` (call sites pass
   `true`).
3. **Recording + applying** — the five recording points, `apply_undo`, the
   ActionBar button, toasts.
4. **Verify + commit** — manual script below, then one commit
   (`feat(deck-cards): undo …`), changelog bullet for 1.7.5.

## Verification script (phase 4)

- Quick-add a card → Undo → card gone, "Removed {name}", no removal
  telemetry recorded for it.
- Remove a ×3 card via minus-mashing → Undo → card back at ×3 on its old
  board, "Re-added {name}".
- +4 burst on one card → wait for flush → Undo → back to original, one
  inverse server call after the debounce.
- Move a card Main → Side → Undo → back on Main.
- Change a printing → Undo → old art back, server holds old printing id.
- Stale path: remove card A, manually re-add it from the add screen, then
  Undo the removal → "Already changed", no duplicate row, no failed request.
- Multiple undos in a row walk backwards correctly; button disappears when
  the stack empties; leaving the screen and returning starts a fresh stack.

## Parked / related

- Recent-actions list UI, DB-backed history (→ future "deck changelog"
  feature with its own design), local-disk persistence: all explicitly
  dropped, see Decisions.
- MVP restore on remove-undo; command-zone printing undo.
- `patch_idempotent_updates.md` interaction: once quantities go absolute,
  `QuantityChanged` undo becomes "set back to baseline" — strictly simpler
  and retry-safe. Revisit this file's qty arm during that migration's
  phase 2.
