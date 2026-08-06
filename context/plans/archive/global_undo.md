# Plan: global undo — one per-deck mutation history across screens

**Status: DONE — BUILT 2026-08-05 (`a7763093`) and DEVICE-VERIFIED the same
day: the owner ran the full sim run sheet below on a real phone against the
live server, all nine scenarios passed. Rides 1.7.6.** Predecessor:
`archive/deck_cards_undo.md` — its architecture (UndoAction, UndoStore,
apply_undo, stale guards) is the foundation, not replaced.

**As-built deviations from the mechanics below (rationale in
`undo_log.rs`):**
- **No entry ids after all.** The u64 id decision became moot: the swipe
  screens record their stack actions BEFORE the server call resolves, so an
  id minted at record time had nothing to attach to. Reconciliation instead
  matches by predicate — `UndoStore::take_newest(deck_id, matches)` pops
  the newest entry whose (variant, card) matches, which picks the same
  entry an id would have in every reachable ordering (newest-first per
  card). Screen histories (`AddAction` etc.) are untouched.
- **Skip detection is entry + world state.** Gesture undo takes its entry
  back; entry gone AND the world already reversed (card absent from the
  deck / already back on its board) means the button consumed it — skip
  the server call and just rewind the stack UI. Entry gone but the
  mutation still standing (cap eviction) falls through to the normal
  reversal. On a failed reversal the taken entry is pushed back.
- Recording from the add/remove screens goes through `UndoStore::push`
  straight into the parked stack (those screens never coexist with the
  deck cards screen's live log).

## The wrinkle that motivated this

Recording is deck-cards-screen-only today, so a mutation made through any
other door (remove screen, add screen swipes) is invisible to the undo
stack. Observed: change a qty on deck cards → remove that card on the
remove screen → back on deck cards → Undo → "Already changed" (the stale
guard correctly refuses to apply a qty inverse to a card that no longer
exists — but the user legitimately expected the removal to be undoable).

## Design (agreed 2026-08-05)

Separate two concepts that are currently fused:

- **Mutation undo** — "reverse the last change to my deck." ONE button, on
  deck cards, walking ONE per-deck global stack, newest-first. Unchanged in
  UI; what changes is that the stack now hears about every mutation.
- **Gesture undo** — the swipe screens' down-swipe: "take back my last
  swipe." Keeps its exact current feel and stays LOCAL (it never pulls from
  the global stack — no "qty of ABC back to ×1" surprises mid-swiping).
  The addition: when a gesture undo reverses a deck mutation, it also
  cancels that mutation's entry in the global stack.

Key payoff: per-card dependency order is automatically correct when a
single button walks newest-first (the newest action on a card is always
undone before older actions on it), and cross-card entries are independent
— so the wrinkle sequence just works: Undo #1 re-adds the removed card,
Undo #2 reverts the qty change. The stale guard stays but demotes to a
safety net for cross-device edits and true races.

## Mechanics

**Entry identity (owner 2026-08-05: u64).** `UndoAction` gains a
monotonically increasing `id: u64` (in-memory only, orderable, no Uuid
needed). `UndoLog` gains `remove(id)`. Screen-local histories
(`AddAction`, the remove screen's action history) store the global id of
any deck mutation they caused.

**Bidirectional reconciliation** (the real engineering):
- Gesture undo reverses its mutation → `undo_log.remove(id)` so the button
  can't double-undo it.
- Button undo consumes an entry → a later gesture undo of that same swipe
  finds its id gone from the store and must SKIP its server mutation (just
  rewind the stack UI), or it would e.g. delete an already-deleted card.
  Rule: the global store is the source of truth for "is this mutation still
  outstanding"; both undo paths check it before acting.

**Recording inventory** (every deck-mutation door):

| Screen | Action | UndoAction |
|--------|--------|------------|
| deck cards | add / remove / qty burst / board move / printing | as today (five points) |
| quick add | add | as today (`Added`) |
| add screen | right-swipe add to deck | `Added` |
| add screen | up-swipe add to maybeboard | `Added` (board maybeboard) |
| add screen | maybeboard source: promote to deck | `MovedBoard` |
| remove screen | remove card | `Removed` (full entry + qty) |
| remove screen | move to board | `MovedBoard` |
| deck cards | command-zone printing change | `CommandZonePrintingChanged` (new) |

**Explicitly NOT recorded:**
- **Skips** (left-swipes) — their own system with its own clear-skips
  affordance; not deck mutations.
- **Bulk import (owner decision 2026-08-05): import CLEARS the deck's undo
  stack** instead of recording. An import effectively overwrites the deck —
  entries recorded before it are semantically void — and the screen takes
  enough deliberate steps that accidental imports aren't a real risk.
  Same clearing applies to Archidekt import and clone targets if they share
  the path. `UndoLog.clear()` on import success. **Disclose it (owner
  2026-08-05): the import screen's hint gets one brief line** — e.g.
  "Importing replaces the deck, so undo history starts fresh." — so the
  behavior is stated in-app, not a surprise. Keep it timeless per the hint
  rules.
- Deck-profile edits (name, tags, format, land/price targets, picking a
  different commander) — unchanged scope from v1. The ONE profile-backed
  action now included is the command-zone printing change (below).

**Command-zone printing undo (promoted from the v1 exclusion list, owner
2026-08-05).** The printing sheet's command-zone arm updates the deck
PROFILE (the slot id points at the chosen printing), not a deck_card row —
which is why v1 skipped it — but the inverse is fully in hand at save time:

- New variant: `CommandZonePrintingChanged { slot, old_card: Card,
  new_id: Uuid }`. `CommandZoneSlot` MOVES out of `view.rs` (private today)
  into `undo_log.rs` beside the variant (decided 2026-08-05): the store is
  shared infrastructure and must not import types from one screen's view
  module — screens depend on the store, never the reverse.
- Record in the printing sheet's `on_save` command-zone arm on server
  success (mirror of the regular-card arm).
- Apply: rebuild the matching `HttpUpdateDeckProfile` builder arm with the
  OLD printing id, call `update_deck_profile`, restore the slot's pinned
  signal (`commander_card` / partner / background / signature spell), bump
  the filter counter, toast "{name} printing restored".
- Stale check: the slot must still hold `new_id` (read the slot's signal).
  If the user since swapped or cleared the commander, skip-consume with
  "Already changed" — same guard semantics as everything else.
- Only record point is the deck cards screen (the deck-edit pickers choose
  a different commander, which stays excluded; only the printing swap of
  the current occupant is undoable).

**Plumbing.** `UndoStore` is already app-level and per-deck; the add and
remove screens consume it via the same `use_context` the quick-add bar
uses. Their existing action histories keep working exactly as now for
stack-UI rewind; they only gain the id bookkeeping. Cap (100) and
park/restore semantics unchanged.

**Toasts.** Unchanged pattern — every button undo names the card. Gesture
undo keeps whatever feedback it has today (no new toasts mid-swipe).

## Verification — run sheet (owner-executed on a real device against prod,
2026-08-05: ALL PASSED; automated tests cover the store primitives: cap
eviction + newest-first take, 3 tests green)

- [x] **The motivating wrinkle:** on deck cards, change a card's qty → go
  to the remove screen and remove that same card → back to deck cards →
  Undo re-adds it at the old board and pre-burst qty → Undo again reverts
  the qty change. No "Already changed" anywhere in the sequence.
- [x] **Add-screen swipes reach the button:** right-swipe a card on the
  add screen → deck cards Undo removes it (toast names the card).
  Up-swipe (maybeboard) → Undo removes it from the maybeboard.
- [x] **Gesture undo consumes the entry:** right-swipe an add, down-swipe
  it back immediately → go to deck cards → Undo does NOT offer that add
  (the next entry is offered; no "Already changed" detour).
- [x] **Button first, gesture second (the double-delete guard):**
  right-swipe an add → deck cards → Undo (card deleted) → back to the add
  screen → down-swipe that same swipe → stack rewinds visually, NO error
  toast, and the card is not double-deleted (server log quiet).
- [x] **Promote round-trip:** maybeboard source, right-swipe promotes to
  deck → deck cards Undo moves it back to the maybeboard. Then the mirror:
  promote → down-swipe (gesture) → deck cards Undo does not offer it.
- [x] **Remove screen:** remove a card + up-swipe another to a different
  board → both step back from deck cards Undo, in reverse order, with the
  removed card returning at its old qty and board.
- [x] **Command-zone printing:** change the commander's printing from the
  deck cards screen → Undo restores the old art in the featured strip and
  the profile slot. Then swap to a DIFFERENT commander and Undo the
  earlier printing change → "Already changed", slot untouched.
- [x] **Import clears:** make a few undoable changes → import a list →
  the Undo button is gone (stack cleared); the import hint shows the
  "undo history starts fresh" line.
- [x] **Isolation + parking:** build a stack on deck A, edit deck B,
  return to A → A's stack intact; B's separate. App restart forgets both
  (expected).

## Non-goals

- No visible history list (decided against in the v1 plan; unchanged).
- No persistence beyond app lifetime (unchanged).
- No undo for skips, imports, or profile edits (name/tags/format/targets/
  commander choice). Command-zone printing swaps ARE in scope (above).
