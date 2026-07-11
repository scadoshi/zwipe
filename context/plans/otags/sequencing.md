# Sequencing — the phased build

Seven phases, each independently shippable, ordered so backend data lands before any
client sees it. **Every phase is additive** — see the per-phase "Wire & compat" line and
`compatibility.md`. No phase here requires a `MIN_CLIENT_VERSION` bump.

Cross-cutting reminders:
- **SQLx offline:** after any `query!`/`query_as!`/`query_scalar!` change, run
  `cargo sqlx prepare --workspace` from the workspace root and commit `.sqlx/`
  (never a crate-local `zerver/.sqlx/` — see `CLAUDE.md`).
- **Migrations:** `zerver/migrations/`, naming `YYYYMMDDHHMMSS_snake_case.sql`, forward-only.
  Pick timestamps later than the current latest (`20260711...`).
- **Shared types** (otag newtype, filter fields, deck otag field) go in `zwipe-core`, same
  pattern as `MechanicalCategory` — one contract, both sides.

---

## Phase 0 — Spike: confirm the bulk file shape (½ day, no code shipped)

Before anything, download the Oracle Tags bulk (`data.scryfall.io/oracle-tags/...`, ~17 MB)
and confirm:
- Keying (expect **tag → oracle_id[]**, so ingest inverts to per-card).
- **Whether it ships a per-tag description** (decides the Q5 definitions work — see
  `open-questions.md` §5).
- Exact JSON line schema (jsonl vs array; field names).

Output: a one-page note pinning the parse contract. Nothing else starts until this is known.

---

## Phase 1 — Ingest foundation (backend-only, invisible to clients)

**Goal:** every card carries its otag set, synced daily.

**Tables (new migration, e.g. `..._create_card_otags.sql`):**
- `card_otags(oracle_id UUID, otag TEXT, source TEXT DEFAULT 'scryfall', ...)` — normalized
  source of truth. Index for the inverse lookup (`otag`, and `oracle_id`).
- **Add `otags JSONB NOT NULL DEFAULT '[]'` + GIN index to `card_profiles`** (the
  denormalized serve projection; mirrors `card_profiles.mechanical_categories`). Same
  migration or a sibling.

**Backend:**
- `zerver/src/lib/inbound/external/scryfall/bulk.rs` — add `BulkEndpoint::OracleTags`
  (`/bulk-data/oracle-tags`); reuse `amass()` two-step + `planeswalker.rs` HTTP. Parse
  **line-by-line** (`.lines()` + `serde_json::from_str`), not whole-array `from_value`.
- `zerver/src/lib/domain/card/ports.rs` + `outbound/sqlx/card/helpers/upsert_card.rs` — new
  batched `INSERT ... ON CONFLICT` into `card_otags` (follow `batch_delta_upsert` + the
  65535-param batch-sizing in `scryfall_data_fields.rs`).
- `zerver/src/bin/zervice.rs` `main` — add two steps in the daily sequence: (1) fetch +
  upsert `card_otags`; (2) rebuild the `card_profiles.otags` JSONB via one
  `GROUP BY oracle_id` (`UPDATE card_profiles SET otags = agg FROM card_otags ...`). Add
  `tracing::info!` lines like the others.

**Wire & compat:** none. Server-internal only. No client, no contract, no bump.

**Exit:** `card_otags` populated, `card_profiles.otags` reflects it, sync runs clean nightly.

---

## Phase 2 — Heuristic backfill + provenance (backend-only)

**Goal:** fill serve/filter-critical otags on cards Scryfall left untagged (Q1 decision:
complement + seed).

**Backend:**
- Mirror `zwipe-core/src/domain/card/models/mechanical_category/classify.rs` for the
  **curated serve-critical otag subset (~40-80 only)** — not the full vocabulary.
- Extend the `classify_untagged_cards` path in `zerver/src/lib/domain/card/services.rs` (or
  a sibling step in `zervice.rs`) to write heuristic otags into `card_otags` with
  `source = 'heuristic'`, then rebuild the JSONB projection.
- **Free win:** cards carrying *both* an otag and our heuristic label form a labeled set —
  emit an accuracy metric (heuristic vs otag ground truth) to finally measure the ~70-80%.

**Wire & compat:** none. Internal.

**Exit:** coverage gap on the curated tier closed; provenance queryable; accuracy metric logged.

---

## Phase 3 — Filtering (first client-visible piece, fully additive)

**Goal:** players filter cards by otag; served cards surface their otags.

**Shared (`zwipe-core`):**
- `.../search_card/card_filter/criteria/mod.rs` — add `otags_contains_any` / `_contains_all`
  / `_excludes` (`Option<Vec<String>>`), plus getters/setters/builder/`matches.rs`/`query.rs`.
  All **`#[serde(default)]`**.
- `.../card/models/card_profile.rs` — add `otags: Vec<Otag>` to `CardProfile`, **`#[serde(default)]`**
  (so old clients reading a new server's `Card` don't choke, new clients reading old server
  get empty).

**Backend:**
- `zerver/src/lib/outbound/sqlx/card/mod.rs` (~912-926) — add otag jsonb predicates
  (`otags ?|`, `@>`, `NOT ?|`) beside the `mechanical_categories` ones, over
  `card_profiles.otags`.

**Frontend (`zwiper`):**
- New otag include/exclude filter beside `.../deck/card/filter/category.rs`, hosted in
  `card_filter_sheet.rs`. Surface otags on the swipe card in `.../card/components/card_info.rs`.

**Wire & compat:** additive only. Old client omits the new filter fields (`#[serde(default)]`
→ `None`); old client tolerates the new response field. **No bump.**

**Exit:** otag filters work end-to-end; otags visible on served cards.

---

## Phase 4 — Deck otag selection + archetype demotion (additive)

**Goal:** a deck declares its strategy otags; picking an archetype seeds them (Q2 decision).

**Tables:** add `otags JSONB NOT NULL DEFAULT '[]'` + GIN index to `decks` (new migration,
mirrors `20260625000000_add_deck_tags.sql`).

**Shared (`zwipe-core`):**
- Authored **`DeckTag → otag-set` correlation** (~120 entries) as a curated map in
  `domain/deck/models/` (same home + style as `deck_tag.rs`; stable data, lives in code not
  DB).
- `http/contracts/deck.rs` — add `otags` to `HttpCreateDeckProfile` (`Option<Vec<String>>`,
  `#[serde(default)]`) and `HttpUpdateDeckProfile` (`Opdate<Vec<String>>`, `#[serde(default)]`);
  add to `HttpSharedDeck` (`#[serde(default)]`). `domain/deck/models/deck_profile.rs` +
  `requests/{create,update}_deck_profile.rs` — validate/carry the field (mirror `other_tags`).

**Backend:**
- `zerver/src/lib/outbound/sqlx/deck/{mod.rs,models.rs}` — persist/read `decks.otags` in
  create/update/get/clone (mirror the `other_tags` serializer + `clone_deck` column copy).

**Frontend (`zwiper`):**
- Generalize `.../deck/components/tag_select.rs` into a searchable otag picker (firehose +
  alphabetical + search, per Q5). Wire archetype selection to seed otags via the correlation.
  Hosts: `.../deck/create.rs`, `.../deck/edit.rs` (Opdate diff like `other_tags`).
- Scoped distribution view (selected + top-N present in decklist).

**Wire & compat:** additive only — `other_tags` already proves this exact pattern is
back-compat-safe (tests in `contracts/deck.rs`). **No bump.**

**Exit:** decks carry otags; archetype seeds them; picker + distribution ship.

---

## Phase 5 — Serving term (backend-only, ordering change)

**Goal:** fold otags into the serve (Q4 decision: one small `W_OTAG` term first).

**Backend:**
- `zerver/src/lib/outbound/sqlx/card/mod.rs` `search_scryfall_data_deck_aware` — add a
  `W_OTAG` **otag-correlation** term = overlap of `card_profiles.otags` with the deck's
  selected otags (`?|` inline, no new join). Keep `W_OTAG` **small** (revert lever).
- `zerver/src/lib/domain/deck/services.rs` `search_deck_cards` — load `decks.otags`; apply
  the cold-start ladder (selected otags → commander's popular otags → nothing / today's
  behavior).

**Wire & compat:** the serve **response shape is unchanged** — only ordering shifts. No
contract change, **no bump**. (Phase-2 otag *signal* term is deferred to Phase 7.)

**Exit:** decks with selected otags get otag-aware ordering; zero regression when otags absent.

---

## Phase 6 — Non-EDH signal collection (backend, ship dark)

**Goal:** start accruing the moat dataset *before* serving on it (Q7 decision).

**Tables:** new **generalized-context** per-otag signal table keyed
`(context, otag, shown/added/skipped/...)` where `context` = a commander **or** `(format,
CI)`; + a nightly rollup matview (mirror `card_signal_rollup`).

**Backend:**
- `zerver/src/lib/outbound/sqlx/metrics/mod.rs` — on each swipe, also credit **one row per
  otag** the deck has selected (derive `format`/`CI`/`otags` server-side from the deck, so
  likely **no client wire change** to `POST /api/metrics/usage`; confirm the handler
  `handlers/metrics/record_usage.rs` has deck context).
- `zerver/src/bin/zervice.rs` — refresh the new rollup nightly.

**Wire & compat:** signal derived server-side; ship dark. **No bump.** If it turns out the
client must send new context, hold it behind the min-version gate (last resort).

**Exit:** per-otag `(context)` signal accumulating for both Commander and non-EDH decks.

---

## Phase 7 — Non-EDH serving + otag signal term (deferred, data-hungry)

**Goal:** once Phase 6 data has volume, serve on it.

- Fold the **otag signal term** (Phase-2 of Q4) into `search_scryfall_data_deck_aware` for
  the deeper-cuts / new-card cold-start win, reading the generalized-context rollup.
- Enable non-EDH serving pivoting on `(format, CI, selected otags)`.

**Wire & compat:** ordering/behavior only; additive. Revisit only after the dataset matures
(`moat.md` — months, not launch).

---

## Dependency order

```
0 spike ─▶ 1 ingest ─▶ 2 backfill ─┐
                        1 ─▶ 3 filtering (needs card_profiles.otags)
                        1 ─▶ 4 deck otags ─▶ 5 serving term
                                    4 ─▶ 6 signal collection ─▶ 7 non-EDH serving
```

1 unblocks everything. 3, 4 are parallel after 1. 5 needs 4. 6 needs 4 (decks must have
otags to key on). 7 needs 6 to have run long enough to matter.
