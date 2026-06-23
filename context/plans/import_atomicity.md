# Deck import — make the whole operation atomic

**Status: PLANNED (low priority, deferred). Frontend can't be exercised right
now; the data-integrity half is the only non-low part. Decision: Option A — full
atomicity (limit check + writes in one transaction).**

This is review item **#7** (+ the related card-limit TOCTOU). The other 2026-06-19
hardening items shipped; this one was deferred deliberately.

---

## Problem

`import_deck_cards` (text) and `import_archidekt_deck` both do, in sequence on
**separate connections**:

1. count + post-import limit math (domain service, on the pool),
2. `bulk_create_deck_cards` — opens its **own** tx, upserts the batch, **commits**
   (`outbound/sqlx/deck/mod.rs:458`),
3. replace mode: `delete_deck_cards_not_in` per board — **separate** statements
   (`mod.rs:152`).

Two issues:

- **Non-atomic writes (Medium, data integrity):** insert commits, then deletes
  run after. A crash / failed delete in between leaves a **hybrid board** — new
  cards in, stale cards not removed.
- **Limit-check TOCTOU (Low):** the count + check (step 1) is separate from the
  write (step 2), so two concurrent imports can both pass the check then both
  insert, exceeding the deck card limit. Self-inflicted (own deck), bounded, and
  already throttled by the import rate limit — hence Low.

Decision (chosen): **Option A** — pull count + limit check + insert + replace-
delete into one transaction with a row lock, so the whole import is atomic. The
domain stays pure (no SQL/tx leaks in); the count *arithmetic* moves into the
adapter alongside the writes it guards, while the limit *value* (policy) stays in
the domain and is passed in.

(Option B — make only the writes atomic, keep the limit check as a domain
pre-check, accept the TOCTOU — was the lighter alternative. Rejected: user wants
everything atomic.)

---

## Target design

Replace `bulk_create_deck_cards` with one atomic method (rename for honesty):

```rust
// DeckRepository port
async fn apply_import_batch(
    &self,
    request: &ImportDeckCards,                       // carries user_id, deck_id, email_verified, mode
    batch: &[(Uuid, Uuid, i32, String)],             // (scryfall_id, oracle_id, qty, board)
    card_limit: i64,                                  // resolved by the domain (policy stays there)
) -> Result<Vec<DeckCard>, ImportDeckCardsError>;
```

Adapter body, all in **one** `tx`:

1. **Empty batch → `Ok(vec![])` before opening the tx.** Preserves "an import
   where nothing resolved never wipes a board."
2. `BEGIN`.
3. **Lock + ownership in one shot:** `SELECT id FROM decks WHERE id = $1 AND
   user_id = $2 FOR UPDATE`. No row → `Forbidden`. The `FOR UPDATE` serializes
   concurrent imports on the same deck (this is what closes the TOCTOU). Replaces
   the separate pool-based `owns_deck` call for this path.
4. **Count (in tx):** deck-board quantity — the `count_cards_in_deck` SQL, run on
   `&mut *tx` (don't call the pool method).
5. **Post-import total (in tx)** — preserve the exact current math
   (`services.rs:489-507`):
   - replace mode: if the batch has any `board = 'deck'` rows → sum of imported
     deck-board quantities; else → current count (unchanged).
   - add mode: `(count − overlap) + import_total`, where `overlap` = sum of
     existing quantities for the imported oracle_ids on the deck board (the
     `sum_quantities_for_oracle_ids` SQL, run on `&mut *tx`). Upsert replaces
     those quantities, hence the subtraction.
6. **Enforce:** if `post_import_total > card_limit` → return `LimitReached`
   (when `request.email_verified`) or `UnverifiedLimitReached` (otherwise). The
   tx rolls back on drop.
7. **Upsert** the batch (existing `INSERT … ON CONFLICT (deck_id, oracle_id) DO
   UPDATE SET quantity = EXCLUDED.quantity, board = EXCLUDED.board RETURNING …`).
8. **Replace reconcile (in tx):** if `request.mode.is_replace()`, group the
   batch's oracle_ids by board and, per board, `DELETE FROM deck_cards WHERE
   deck_id = $1 AND board = $2 AND NOT (oracle_id = ANY($3))`. This generalizes
   both importers — text (multi-board) and archidekt (single board) fall out of
   the batch's `board` column. Boards absent from the import are untouched.
9. `COMMIT`.

### Domain service changes (both importers)
- Drop the count/overlap/post-import math and the limit check (moves to step 5–6).
- Drop the replace delete loops (moves to step 8).
- Still: build the batch, resolve `card_limit = if email_verified
  { MAX_CARDS_PER_DECK } else { UNVERIFIED_MAX_CARDS_PER_DECK }`, call
  `apply_import_batch`, build the `ImportedCard` list from the result.
- This **dedups** the limit + replace logic that's currently copied across the
  two importers into one place.

### Cleanup
- Rename `bulk_create_deck_cards` → `apply_import_batch` (port + impl + 2 call
  sites).
- `sum_quantities_for_oracle_ids` and `delete_deck_cards_not_in` become dead
  (only the importers used them) → remove from port + impl.
- **Keep `count_cards_in_deck`** — still used by `create_deck_card`
  (`services.rs:153`).

---

## Semantics to preserve EXACTLY (frontend is untestable — do not regress)

- Empty / nothing-resolved import never deletes anything (step 1).
- Boards absent from the import are left untouched (per-board reconcile only).
- Replace deck-board total = imported deck-board qty; add mode =
  `(count − overlap) + import_total`.
- Limit values: verified `MAX_CARDS_PER_DECK`, unverified
  `UNVERIFIED_MAX_CARDS_PER_DECK`; error variants `LimitReached` vs
  `UnverifiedLimitReached`.
- Upsert: `quantity = EXCLUDED.quantity`, `board = EXCLUDED.board`.

---

## Files
- `domain/deck/ports.rs` — `DeckRepository` trait (rename + signature, remove 2
  dead methods).
- `outbound/sqlx/deck/mod.rs` — `apply_import_batch` body; remove
  `sum_quantities_for_oracle_ids`, `delete_deck_cards_not_in`.
- `domain/deck/services.rs` — both importers slimmed to build-batch + resolve-
  limit + call.

## Testing / risk
- The repo-tx path **can't be unit-tested without a live DB**. Either add a
  `#[sqlx::test]` against a test DB for `apply_import_batch` (concurrent-import
  case + replace + limit), or verify manually once the frontend is exercisable.
- Existing pure-logic tests stay green via `cargo test`; `clippy -D warnings`.
- Since the frontend can't be driven now, this is why it's deferred — the change
  reshapes the import write path.

## Related (could ride along, optional)
- `create_deck_card` (`services.rs:153`) has the **same** count-then-insert TOCTOU
  on the deck card limit. The same `FOR UPDATE` + count-in-tx pattern would fix
  it. Out of scope unless folding in is cheap at implementation time.
