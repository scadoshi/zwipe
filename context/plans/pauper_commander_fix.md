# Pauper EDH Commander Pool Fix

## The bug (confirmed real)

A PDH player reported the commander picker "only finds legendaries by default,
instead of all uncommon creatures." It's a genuine bug, and it's a **rarity
data-model** problem, not a wiring problem.

- **Frontend is correct:** `swipe_select.rs:74` passes the deck's format into
  `set_is_commander_in_format(format)`, so a PDH deck correctly reaches the
  `Format::PauperCommander` branch of the commander query.
- **The query is wrong:** `zerver/.../outbound/sqlx/card/mod.rs:625` filters the
  PDH commander pool with:
  ```sql
  (type_line ILIKE '%Creature%' AND rarity = 'uncommon')
  ```
  `rarity` is **per stored printing**, but PDH eligibility is "**has appeared at
  uncommon in any printing**." So this only matches cards whose *cached* printing
  is uncommon.

**Why it skews legendary:** legendary creatures routinely get uncommon printings
(Commander precons, etc.), so their stored printing is often `uncommon` and they
pass. Most *non-legendary* PDH-eligible creatures have their main/cached printing
at **common**, so `rarity = 'uncommon'` excludes them — leaving mostly legendary
uncommons. That's the "only legendaries" symptom exactly.

## The rules (verified)

PDH commander = **any uncommon creature, legendary or not** (per PDH Home Base).
The 99 are unique commons. Scryfall exposes this directly:
- `is:paupercommander` → the eligible **commander** pool (uncommon creatures).
- `f:paupercommander` → the legal **99** (commons).
These were built with Scryfall/Moxfield/Archidekt, so the term is stable.

## Fix options

### Option A — interim query fix (no migration, fast)
If the DB stores multiple printings per oracle (the `get_printings_by_oracle_id`
query at `card/mod.rs:778` implies it does — **confirm first**), change the PDH
commander predicate from "this printing is uncommon" to "**any** printing of this
oracle is uncommon":
```sql
type_line ILIKE '%Creature%' AND EXISTS (
  SELECT 1 FROM scryfall_data sd2
  WHERE sd2.oracle_id = <this card>.oracle_id AND sd2.rarity = 'uncommon'
)
```
- **Pro:** fixes the overwhelming majority immediately, no schema/ingest change.
- **Con:** "any uncommon printing" isn't *exactly* PDH's definition — it can
  include/exclude a few edge cases (MTGO promos with retroactively changed rarity,
  old German/French/Italian printings, bonus-sheet weirdness) that PDH Home Base's
  framework treats specially.

### Option B — correct fix (capture `is:paupercommander` at ingest)
Add an oracle-level boolean and populate it from Scryfall's authoritative term:
1. **Migration:** add `is_pauper_commander BOOLEAN NOT NULL DEFAULT false` (on the
   oracle/card-profile level).
2. **Ingestion (`zervice`):** during the Scryfall sync, query
   `/cards/search?q=is:paupercommander` (paginated, ~hundreds of cards), and set
   the flag for those oracle_ids. It's a *computed* Scryfall property, so it must
   be captured at ingest — it can't be derived from a single card's `rarity`.
3. **Query:** swap `card/mod.rs:625` to `type_line ILIKE '%Creature%' AND
   is_pauper_commander`.
- **Pro:** exactly correct, all edge cases handled by Scryfall.
- **Con:** migration + ingestion change + a re-sync to populate, then deploy.

### Recommendation
Ship **A** first as the immediate unblock for PDH testers, then follow with **B**
for correctness. Or, if we'd rather not touch it twice, go straight to **B** —
it's the right long-term model and the ingestion hook is small.

## The 99-card pool (verify, likely already fine)
The deck's format-legality filter should already handle `f:paupercommander` for
the 99 via the existing `legalities.paupercommander` field (present in
`zwipe-core/.../legalities.rs`). Confirm a PDH deck's *card* search (not commander)
already restricts to commons; if so, no change needed there — this bug is
commander-pool-only.

## Backward compatibility / deploy
- **No wire-contract change**, no client change — this is server query + (for B)
  ingestion/schema. Existing clients keep working; PDH commander results just get
  correct.
- Option B touches a deploy path (migration + `.sqlx` after the query change →
  `cargo sqlx prepare --workspace`, commit `.sqlx/`). Additive column, safe.
- Server-side unit (not a blind client worktree) — sequence + deploy with
  confirmation.

## Verification
- `Format::PauperCommander` commander search returns **hundreds** of uncommon
  creatures, not ~10 legendaries.
- Spot-check a known **non-legendary** uncommon commander (e.g. a vanilla-ish
  uncommon creature) now appears in the PDH commander pool.
- A regular Commander deck's commander pool is unchanged (legendary-gated).
- `cargo test --workspace` + `clippy` clean.
