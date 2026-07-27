# Deck import — make the whole operation atomic

**Status: SHIPPED 2026-07-27 (`048e99dc`).** Implemented as designed (explicit-
fields signature, ownership-before-empty-return, ride-along included). Verified
by 5 new `#[sqlx::test]` cases in `zerver/tests/import_atomicity.rs` — including
the concurrent-race test proving the `FOR UPDATE` serialization — plus a live
end-to-end pass against the real dev catalog: real text imports, overlap math,
replace reconcile, over-limit rollback, and the real Archidekt Satya deck
(83/83 set equality after replace). Server-only, no wire change, no migration.

This was review item **#7** (+ the related card-limit TOCTOU). The other
2026-06-19 hardening items shipped; this one was deferred deliberately.

---

## Problem

`import_deck_cards` (text) and `import_archidekt_deck` both do, in sequence on
**separate connections**:

1. count + post-import limit math (domain service, on the pool) —
   text importer `services.rs:579-615`, archidekt `services.rs:782-815`,
2. `bulk_create_deck_cards` — `owns_deck` on the pool, then its **own** tx,
   upserts the batch, **commits** (`outbound/sqlx/deck/mod.rs:657-710`),
3. replace mode: `delete_deck_cards_not_in` per board — **separate** statements
   (`mod.rs:190`; text loop `services.rs:626-642`, archidekt `:833-838`).

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

Replace `bulk_create_deck_cards` with one atomic method (rename for honesty).
**Signature revised 2026-07-27:** taking `&ImportDeckCards` would force the
archidekt importer to keep its current hack (it builds a `carrier:
ImportDeckCards { lines: Vec::new(), .. }` at `services.rs:818-825` just to
satisfy the parameter). Take the fields explicitly and the hack dies:

```rust
// DeckRepository port
async fn apply_import_batch(
    &self,
    user_id: Uuid,
    deck_id: Uuid,
    mode: ImportMode,
    batch: &[(Uuid, Uuid, i32, String)],             // (scryfall_id, oracle_id, qty, board)
    card_limit: i64,                                  // resolved by the domain (policy stays there)
    email_verified: bool,                             // picks LimitReached vs UnverifiedLimitReached
) -> Result<Vec<DeckCard>, ImportDeckCardsError>;
```

Adapter body, all in **one** `tx`:

1. `BEGIN`.
2. **Lock + ownership in one shot:** `SELECT id FROM decks WHERE id = $1 AND
   user_id = $2 FOR UPDATE`. No row → `Forbidden`. The `FOR UPDATE` serializes
   concurrent imports on the same deck (this is what closes the TOCTOU). Replaces
   the separate pool-based `owns_deck` call for this path.
   **Ordering decision (2026-07-27):** ownership runs BEFORE the empty-batch
   early return. Implementation finding: over HTTP a foreign deck actually
   404s at the importers' `get_deck_profile` pre-check (existence-hiding)
   before the repo is reached, so the in-tx `Forbidden` is defense-in-depth
   for future callers rather than the HTTP-visible behavior. No user-visible
   change; the empty-batch/foreign edge is covered by
   `import_atomicity::foreign_deck_import_is_rejected_and_writes_nothing`.
3. **Empty batch → `Ok(vec![])`** (after the lock, before any write). Preserves
   "an import where nothing resolved never wipes a board."
4. **Count (in tx):** deck-board quantity — the `count_cards_in_deck` SQL, run on
   `&mut *tx` (don't call the pool method).
5. **Post-import total (in tx)** — preserve the exact current math
   (text `services.rs:584-603`, archidekt `:787-801`):
   - replace mode: if the batch has any `board = 'deck'` rows → sum of imported
     deck-board quantities; else → current count (unchanged).
     (The archidekt importer special-cases this as `if board == Board::Deck {
     import_total } else { card_count }` — equivalent, since its batch is
     single-board; the generalized batch-filter form subsumes it.)
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
- Rename `bulk_create_deck_cards` → `apply_import_batch` (port `ports.rs:148` +
  impl `mod.rs:657` + 2 call sites `services.rs:620`, `:826`; the archidekt
  `carrier` struct at `:818-825` dies with the signature change).
- `sum_quantities_for_oracle_ids` (`ports.rs:87`, `mod.rs:207`) and
  `delete_deck_cards_not_in` (`ports.rs:158`, `mod.rs:190`) become dead
  (only the importers used them) → remove from port + impl.
- **Keep `count_cards_in_deck`** (`ports.rs:81`, `mod.rs:177`) — still used by
  `create_deck_card` (`services.rs:162`); its SQL is reused inside the tx.

---

## Semantics to preserve EXACTLY (each line becomes a test assertion)

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
- `domain/deck/ports.rs` — `DeckRepository` trait (rename + explicit-fields
  signature, remove 2 dead methods).
- `outbound/sqlx/deck/mod.rs` — `apply_import_batch` body; remove
  `sum_quantities_for_oracle_ids`, `delete_deck_cards_not_in`.
- `domain/deck/services.rs` — both importers slimmed to build-batch + resolve-
  limit + call; archidekt loses the `carrier` construction.
- `zerver/tests/` — new `#[sqlx::test]` cases (see below).

## Testing (2026-07-27: the suite exists — use it)
- Existing coverage to keep green:
  `deck_cards.rs::import_resolves_known_cards_and_reports_the_rest` (the happy
  path through the real router) and `archidekt_import.rs` (live-fetch parse).
- New `#[sqlx::test]` cases for `apply_import_batch` (pattern:
  `tests/common` `TestApp` + seeded cards, same as `metrics_flows.rs`):
  1. **Limit rollback**: an over-limit import writes NOTHING (the tx rolls
     back; today the insert would have committed before a replace-delete
     failure could strand a hybrid board).
  2. **Replace reconcile**: board becomes exactly the imported list; a board
     absent from the import is untouched; empty batch deletes nothing.
  3. **Add-mode overlap math**: re-importing existing oracle_ids lands
     `(count − overlap) + import_total`, not a double count.
  4. **Ownership**: foreign deck → `Forbidden`, including with an empty batch.
  5. **Concurrency (the TOCTOU)**: two imports racing the same deck both
     inside the limit individually but not jointly — one must get
     `LimitReached`. Drive with two sequential calls on separate connections
     if true interleaving is awkward; the `FOR UPDATE` serialization is the
     thing under test.
- `cargo clippy -p zwipe-core -p zerver --all-targets -- -D warnings` (the CI
  gate), nightly fmt.
- New in-tx queries: if written as `query!`/`query_scalar!` macros, run
  `cargo sqlx prepare --workspace` from the root and commit `.sqlx/`
  (workspace root ONLY — see CLAUDE.md). The upsert stays `QueryBuilder`
  (runtime, no offline data). All on sqlx **0.9** now.

## Ride-along (folded in per owner, 2026-07-27)
- `create_deck_card` (`services.rs:162-179`) has the **same** count-then-insert
  TOCTOU on the deck card limit. Apply the same `FOR UPDATE` + count-in-tx
  pattern inside the repo's `create_deck_card` (which already checks `owns_deck`
  at `mod.rs:300` — replace that call with the locking select). Same error
  variants (`CreateDeckCardError::{LimitReached, UnverifiedLimitReached}`);
  the limit value moves in as a parameter like the import path.
