# Scope — every system touched

Grounded in a codebase map (2026-07-11). Paths are relative to repo root. There is
currently **no `otag`/`oracle_tags` code anywhere** in `zwipe-core/` or `zerver/` — this
is greenfield, built alongside the existing systems below, not on top of them.

## Backend — ingest (new)

**New Scryfall bulk source.** The ingest pattern to copy lives in
`zerver/src/lib/inbound/external/scryfall/bulk.rs` — the `BulkEndpoint` enum + its
`amass()` two-step (GET metadata → read `download_uri` → GET data), fetching through
`zerver/src/lib/inbound/external/scryfall/planeswalker.rs`.

- Add an `OracleTags` variant to `BulkEndpoint` resolving to `/bulk-data/oracle-tags`
  (Oracle Tags now appears in Scryfall's bulk-data manifest, so the same two-step works).
- Parse **line-by-line** (`.lines()` + `serde_json::from_str` per line), *not* the current
  whole-array `serde_json::from_value::<Vec<_>>` — the tag file is large jsonl and is keyed
  **tag → oracle_id[]**, so fan each tag out to its oracle ids.

**New table.** `zerver/migrations/` (naming `YYYYMMDDHHMMSS_snake_case.sql`, forward-only;
latest is `20260711...`). Add e.g. `20260712000000_create_card_otags.sql`:
- `card_otags(oracle_id UUID, otag TEXT, ...)` keyed by `oracle_id`, GIN/btree indexed for
  "cards with otag X in color identity Y". Mirror the style of
  `20260611120000_create_synergy_tables.sql` / `20260706200000_create_card_signal_rollup.sql`.
- Likely a rollup matview later (like `card_signal_rollup`) for perf at hundreds-of-otags ×
  110k+ cards (`open-questions.md` §volume/perf).

**New repo port + impl.** Follow `CardRepository::batch_delta_upsert` and the `*_with_tx`
helper in `zerver/src/lib/outbound/sqlx/card/helpers/upsert_card.rs` — batched
`INSERT ... ON CONFLICT ... DO UPDATE`, batch sizing per the 65535-param limit logic in
`scryfall_data_fields.rs`.

**Wire into the job.** `zerver/src/bin/zervice.rs` `main` runs a fixed daily sequence
(sync → classify → refresh matviews → prune sessions). Add one otag-sync step + a
`tracing::info!` line, alongside the existing `scryfall_sync(...)` and
`refresh_card_signal_rollup()` calls.

## Backend — filtering (extend existing)

Parallels the three `mechanical_categories_*` predicates exactly:
- `zwipe-core/src/domain/card/models/search_card/card_filter/criteria/mod.rs` — add
  `otags_contains_any` / `_contains_all` / `_excludes` (`Option<Vec<String>>`), plus the
  matching getters/setters/builder/`matches.rs`/`query.rs`.
- `zerver/src/lib/outbound/sqlx/card/mod.rs` (~lines 912-926) — add otag jsonb/join
  predicates next to the existing `?|` / `@>` / `NOT ?|` category predicates.

## Backend — serving (extend existing)

The ranking query is `search_scryfall_data_deck_aware` in
`zerver/src/lib/outbound/sqlx/card/mod.rs`: score = **base** (commander synergy from
`commander_synergy`) + **signal** (`card_signal_rollup` net-rate) + **band jitter**, with a
wildcard deep-pool slot.

- Add an **otag-correlation term** with its own `W_OTAG`-style constant next to `W_SIGNAL`.
- `zerver/src/lib/domain/deck/services.rs` `search_deck_cards` loads the deck profile +
  commander synergy today; it would also load the deck's **selected otags** (JSONB
  `decks.otags`) to weight the new term.
- **The serve does NOT join `card_otags`.** It reads a **denormalized JSONB otag array on
  `card_profiles`** and tests overlap with `?|` — inline with the existing
  `mechanical_categories` predicate, no new join, no GROUP BY (see `open-questions.md` §6).
- **Join-key note:** the `oracle_id`-vs-`scryfall_data_id` reconciliation moves to the
  **nightly sync** (build the `card_profiles` JSONB from `card_otags` by `oracle_id`), out
  of the hot path.

## Backend — signal keying for non-EDH (extend existing)

- Swipe signal is written in `zerver/src/lib/outbound/sqlx/metrics/mod.rs` (upserts into
  `commander_card_signal` / `commander_select_signal`) from `POST /api/metrics/usage`
  (`handlers/metrics/record_usage.rs`).
- The moat (`moat.md`) needs this broadened from commander-keyed to
  `(format, color_identity, otag_set)`-keyed — a new signal table + rollup, not a change to
  the existing commander tables. Additive; ship dark and start collecting early.

## Deck otags — storage (DECIDED)

Deck tags today are **denormalized JSONB on `decks`** (`decks.tags`, `decks.other_tags`
jsonb + `decks.power_level` text; migrations `20260625000000_add_deck_tags.sql`,
`20260630000000_...`), **no join table**. `DeckTag` (~120 variants) / `DeckOtherTag` (5
variants) live in `zwipe-core/src/domain/deck/models/`.

Decided (`open-questions.md` §2, §6): the deck's selected otags go in a **new JSONB
`decks.otags` column** (not a join table) — the serve reads it once and passes it as a
param list. `DeckTag` is **demoted to an archetype container**: a curated `DeckTag → otag-set`
correlation (authored, ~120 entries) seeds a deck's otags when an archetype is picked, and
`decks.tags` stays for browse/display. On the card side, **`card_otags` is the normalized
truth/rollup table**, while the serve reads a **denormalized JSONB otag array on
`card_profiles`** (GIN `?|`, built nightly from `card_otags`) — serving never joins
`card_otags` directly.

## Frontend — `zwiper` (mostly UI)

- **Deck otag selection UI.** The inline chip grid is
  `zwiper/src/lib/inbound/screens/deck/components/deck_fields.rs` (~lines 928-948, iterates
  `DeckOtherTag::all()`). For hundreds of otags, copy the **full-screen searchable picker**
  `zwiper/src/lib/inbound/screens/deck/components/tag_select.rs` instead of an inline grid.
  Hosts: `screens/deck/create.rs`, `screens/deck/edit.rs` (builds an `Opdate` diff).
- **Card filter UI.** New otag include/exclude filter beside
  `zwiper/src/lib/inbound/screens/deck/card/filter/category.rs`, hosted in
  `card_filter_sheet.rs`.
- **Surface otags on served cards.** `zwiper/src/lib/inbound/screens/deck/card/components/card_info.rs`
  is where a card's categories/otags show in the swipe UI. Serve client is
  `zwiper/src/lib/outbound/client/deck/search_deck_cards.rs`
  (`POST /api/deck/{deck_id}/card/search`).
- **Shared types** (`otag` newtype/enum + `CardQuery` filter fields) live in `zwipe-core`,
  same pattern as `MechanicalCategory`, so frontend and backend share one contract.

## mechanical_categories fate — DECIDED: retire the heuristic (Phase 2)

Per `open-questions.md` §1 (revised after Phase 1 coverage data), the heuristic is **retired**,
not complemented:
- **Delete** `mechanical_category/classify.rs` and the `classify_untagged_cards` /
  `get_unclassified_card_ids` / `update_mechanical_categories` heuristic plumbing in
  `zerver/src/lib/domain/card/{services.rs,ports.rs}` + `outbound/sqlx/card/mod.rs`.
- **Keep** the `card_profiles.mechanical_categories` column and its wire field, but
  **repopulate it from otag subtrees** via an authored `category → otag-root(s)` map (expand
  each root through `otags.parent_ids`) in the same nightly `zervice` pass that builds the
  `card_profiles.otags` projection. Non-breaking: served `Card` shape unchanged, values now
  gold-standard.
- **Gate** the deletion behind a filter-parity check on the overlap set; `evasion`/`ramp`
  need multi-root mappings. All of this lands in **Phase 2** (`sequencing.md`).
