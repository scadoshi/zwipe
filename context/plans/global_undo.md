# Plan: global undo — one per-deck mutation history across screens

**Status: PLANNED (2026-08-05), owner building next session.** Ships AFTER
1.7.5 (the release goes out with today's three separate histories; this plan
unifies them). Predecessor: `archive/deck_cards_undo.md` — its architecture
(UndoAction, UndoStore, apply_undo, stale guards) is the foundation, not
replaced.

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

**Entry identity.** `UndoAction` gains an `id: Uuid` (or a monotonically
increasing u64). `UndoLog` gains `remove(id)`. Screen-local histories
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

**Explicitly NOT recorded:**
- **Skips** (left-swipes) — their own system with its own clear-skips
  affordance; not deck mutations.
- **Bulk import (owner decision 2026-08-05): import CLEARS the deck's undo
  stack** instead of recording. An import effectively overwrites the deck —
  entries recorded before it are semantically void — and the screen takes
  enough deliberate steps that accidental imports aren't a real risk.
  Same clearing applies to Archidekt import and clone targets if they share
  the path. `UndoLog.clear()` on import success.
- Command-zone changes and deck-profile edits — unchanged scope from v1.

**Plumbing.** `UndoStore` is already app-level and per-deck; the add and
remove screens consume it via the same `use_context` the quick-add bar
uses. Their existing action histories keep working exactly as now for
stack-UI rewind; they only gain the id bookkeeping. Cap (100) and
park/restore semantics unchanged.

**Toasts.** Unchanged pattern — every button undo names the card. Gesture
undo keeps whatever feedback it has today (no new toasts mid-swipe).

## Verification script

- The motivating wrinkle: qty change → remove on remove screen → back →
  Undo re-adds at old qty → Undo again reverts the qty change. No "Already
  changed" anywhere.
- Right-swipe a card on the add screen → deck cards Undo removes it.
- Down-swipe (gesture undo) an add → deck cards Undo does NOT offer it
  ("Already changed"-free: the entry is gone, the next entry is offered).
- Button-undo an add on deck cards → return to add screen → down-swipe the
  same swipe → stack rewinds, no failed server call, card not double-deleted.
- Promote maybeboard → deck on the add screen → Undo moves it back.
- Remove + board-move on the remove screen → both undoable from deck cards.
- Import a list → undo stack is empty; toast history unaffected.
- Cross-deck isolation and park/restore still hold (deck A's stack
  untouched by deck B's session).

## Non-goals

- No visible history list (decided against in the v1 plan; unchanged).
- No persistence beyond app lifetime (unchanged).
- No undo for skips, imports, profile edits, command-zone printings.
