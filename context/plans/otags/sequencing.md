# Sequencing — the phased build

Seven phases (0-6), each independently shippable, ordered so backend data lands before any
client sees it. **Every phase is additive** — see the per-phase "Wire & compat" line and
`compatibility.md`. No phase here requires a `MIN_CLIENT_VERSION` bump. (The old "Phase 2 —
heuristic backfill" was cut when Q1 flipped to retiring the heuristic; filtering absorbed the
retirement and the phases renumbered.)

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

**BUILT 2026-07-11.** Migrations `20260712000000_create_oracle_tags.sql` +
`20260712010000_rename_otag_tables_to_oracle_tags.sql` (the rename landed the canonical
`oracle_tag` naming — see `compatibility.md` §Naming).

**Tables (canonical names):**
- `oracle_tags(id UUID PK, slug UNIQUE, label, description, parent_ids UUID[], aliases
  TEXT[], updated_at)` — **catalog**: one row per oracle tag with its metadata + hierarchy.
  Added beyond the minimal plan because the bulk file carries descriptions + parent ids for
  free, and Phases 2-4 (definitions UI, category mapping, granularity curation) need them.
- `card_oracle_tags(oracle_id UUID, oracle_tag TEXT, source TEXT DEFAULT 'scryfall',
  PK(oracle_id, oracle_tag))` — normalized source of truth. Indexed on `oracle_tag` and
  `oracle_id`.
- **Deferred to Phase 2:** the denormalized `card_profiles.oracle_tags` JSONB serve
  projection. Nothing reads it until filtering/serving, and keeping Phase 1 to brand-new
  tables keeps it fully additive with zero risk to existing `card_profiles` queries.

**Backend (built):**
- `.../external/scryfall/oracle_tag.rs` — `OracleTag` / `Tagging` deserialize DTOs.
- `.../external/scryfall/bulk.rs` — `BulkEndpoint::OracleTags` (`/bulk-data/oracle-tags`) +
  `amass_oracle_tags()` (same two-step as `amass()`; the file is an 18 MB JSON **array** of
  tag objects, so whole-array `from_value` matches the existing pattern — no jsonl streaming
  needed).
- `.../outbound/sqlx/card/helpers/oracle_tags.rs` — `sync_oracle_tags(pool, tags)`: inverts
  tag→cards into card→otag rows, then in one tx full-replaces the catalog and the
  `source='scryfall'` correlations (heuristic rows preserved), batched under the 65535-param
  limit. Runtime `QueryBuilder`, so **no `.sqlx` regeneration needed**.
- `domain/card/ports.rs` + `services.rs` + `outbound/sqlx/card/mod.rs` —
  `CardRepository::sync_oracle_tags(&[OracleTag])` and `CardService::sync_oracle_tags()`.
- `zerver/src/bin/zervice.rs` — one new step after `scryfall_sync`, with a `tracing::info!`.

**Wire & compat:** none. Server-internal only. No client, no contract, no bump.

**Tests:** 5 unit (`oracle_tag.rs` deserialization incl. null/missing/unknown fields;
`oracle_tags.rs` correlation inversion incl. skip-missing-oracle_id) + 3 integration
(`tests/repo_oracle_tags.rs`, `#[sqlx::test]`: populate + null-description + source default;
idempotent full-replace of scryfall rows; heuristic-source rows preserved across re-sync).

**Exit: DONE ✅.** compiles + clippy clean (lib/bins/tests); all 8 tests green; live
`zervice` run verified against the dev DB — **4,494 otags, 227,732 correlations, 35,577
tagged oracle_ids**, descriptions present.

**Live finding — the firehose is noisy.** Top otags by frequency are dominated by
structural/trivia tags (`activated-ability`, `triggered-ability`, `alliteration`,
`unique-type-line`, `intervening-if-clause`) mixed with the useful functional ones
(`spot-removal`, `evasion`). Confirms the Q3 decision: serving must run on a *curated* tier,
never raw tag frequency, or cards would rank by whether their name alliterates. Feeds the
Phase 3/serving-tier + `DeckTag → otag` authoring (`open-questions.md` §3).

---

## Phase 2 — Filtering + retire the heuristic (first client-visible piece, additive)

**Goal:** players filter cards by otag; served cards surface their otags; `classify.rs` (the
guesswork) is retired and `mechanical_categories` is repopulated from otags. Q1 + Q6 decisions.

**Tables (the projection deferred from Phase 1) — BUILT 2026-07-11 (slice 1):**
- **`card_profiles.oracle_tags JSONB + GIN`** — migration `20260712020000_add_card_profiles_oracle_tags.sql`.
  Denormalized serve/filter projection; mirrors `card_profiles.mechanical_categories`.
- `CardRepository::refresh_card_oracle_tags` (`outbound/sqlx/card/mod.rs`) rebuilds it via one
  `WITH oracle_agg ... UPDATE card_profiles SET oracle_tags = jsonb_agg ...`, wired into
  `zervice.rs` after `sync_oracle_tags`. Verified live: 115,913 profiles updated, 111,783
  carry tags (e.g. Swords to Plowshares → `spot-removal`/`removal-exile`/`lifegain`/…). 2 new
  `#[sqlx::test]` regression tests.
- **Deploy hardening:** the whole `zervice` pipeline is **non-fatal per step** — every step
  (card sync, oracle tags + projection, classify, matview refreshes, session prune) logs
  `step N/5 …: starting/ok/FAILED (continuing)` and tallies failures, so one broken step
  can't skip the rest; the job exits non-zero if any step failed (for monitoring).

### STATUS — Phase 2 DONE + DEPLOYED (prod v1.6.0, 2026-07-12)

Shipped in the v1.6.0 push (all CI gates green; migration `20260712030000` ran on prod).
⚠ **Retirement + grouping populate on the next prod `zervice` run** — until then the new
`card_profiles` columns sit at defaults (no breakage). Only the `CardRole` wire/DB rename
(Phase M) and the `classify.rs` cleanup remain from this phase.

**BUILT + committed** (unpushed at time of writing; every commit additive → no client break;
tests + clippy + nightly-fmt green; `.sqlx` regenerated where needed):
- **Projection** `card_profiles.oracle_tags` + `refresh_card_oracle_tags` in zervice — `ac92b6a6`.
- **Derivation** `outbound/sqlx/card/helpers/derive_categories.rs` — `CATEGORY_ROOTS` (18 otag
  categories) recursive-CTE subtree expansion through `oracle_tags.parent_ids` + `Tokens` via
  `all_parts`; writes `card_profiles.mechanical_categories` — `3798090a`.
- **oracle_tag_gaps** `zwipe-core/.../mechanical_category/oracle_tag_gaps.rs` —
  `classify_oracle_tag_gaps` for the 4 stragglers (Pump/Stax/Protection/GraveyardHate), documented
  self-explaining header. Additive (`classify.rs` still present) — `8bdad628`.
- **Wiring** `CardService::derive_card_categories` (SQL derive 18+Tokens, then a Rust merge pass
  adding the 4 gaps) **replaces** `classify_untagged_cards` in `zervice` step 3 — `b7dbd9d0`.
  ⚠ **DEPLOYED (v1.6.0); populates on the next prod `zervice` run** (nightly / manual
  `./zervice`) — same wire shape, better values, reversible.
- **Display field** `CardProfile.oracle_tags: Vec<String>` (`#[serde(default)]`) on served cards;
  `.sqlx` regenerated + `prepare --check` passes — `ff57c776`.
- **Filter** `oracle_tags_contains_any/all/excludes` end-to-end (criteria → getters → in-memory
  `matches` → builder fields/default/clash/getters/setters/construction → SQL `?|`/`@>`/`NOT ?|`
  on `card_profiles.oracle_tags`) + card_filter_parity case — `4411204e`.
- **Catalog endpoint** `GET /api/card/oracle-tags` (nested under `/api/card` with the keyword/artist
  family, not the bare `/api/oracle-tags`) — serves all **4,494** tags as
  `OracleTag { slug, label, description, parent_slugs }` (new zwipe-core DTO; repo resolves
  `parent_ids`→parent slugs, slug-ordered). `CardRepository/CardService/ErasedCardService` +
  handler + route + `ClientGetOracleTags` zwiper client + `.sqlx` + read test. Live-smoke verified
  (4,494; ~29% carry a Scryfall description) — `f11cc1e3`.
- **Filter picker** `zwiper/.../deck/card/filter/oracle_tags.rs` — `OracleTags` accordion section:
  curated default grid (`CURATED_ORACLE_TAGS`, 48 = the 24 originals→best-populated real slug +
  functional fills, only those the catalog still serves), any/all toggle, exclude section, typeahead
  over the full catalog. Wired into `card_filter_sheet.rs` (import + active-indicator + clear). Note:
  filters on a card's **direct** slugs (not hierarchy-expanded), hence concrete slugs like
  `spot-removal` over the parent `removal` — `41512c59`.
- **Deploy hardening (earlier):** whole `zervice` pipeline is **non-fatal per step** with
  `step N/5 …: starting/ok/FAILED` logging + non-zero exit on any failure.

**Deliberately DEFERRED (NOT done):** the `MechanicalCategory → CardRole` rename and the
`mechanical_categories → card_roles` wire migration (Phase M). The retirement shipped **without**
the rename, so `MechanicalCategory` / the `mechanical_category/` module / `classify.rs` (24-branch)
all still exist. `classify.rs` is KEPT as the revert safety net until the retirement is proven by
a prod zervice run, then deleted (cleanup below).

### ▶ RESUME HERE (next session) — remaining Phase 2

1. ✅ **DONE** — `GET /api/card/oracle-tags` endpoint (`f11cc1e3`).
2. ✅ **DONE** — otag filter picker (`41512c59`).
3. ✅ **DONE** — card roles → oracle-tags drill-down (server-grouped) + naming alignment
   (`b404180d` backend, `6fc32c40` frontend). Owner chose server-side grouping over a client
   static map so the role↔tag mapping + noise filter update on **deploy**, not on mobile releases.
   **Backend:** `card_profiles.oracle_tags_by_role` (role → its tags) + `other_oracle_tags`
   (role-less functional tags, noise stripped), computed by `helpers/oracle_tag_groups.rs`
   (one recursive-CTE `UPDATE` over the hierarchy + `CATEGORY_ROOTS`; explicit noise list bound
   from core `NOISE_ORACLE_TAG_SLUGS`, the four patterns inlined). Refreshed in `zervice` step 2.
   Migration `20260712030000`; two `#[serde(default)]` `CardProfile` fields; `.sqlx` regen.
   ⚠ **DEPLOYED (v1.6.0); populates on the next prod `zervice` run** (like the retirement).
   **Frontend:** `zwipe-components/CardRoleChips` = roles as chips (accent-1 "Card roles" label),
   each easing open to its grouped tags (keyword-reveal animation, accent-3 chips); empty roles
   are plain chips; trailing "Other tags" bucket. Shown in the expanded card row + swipe eyeball
   (`CardRulesDialog`), opt-in via `show_classification` (portfolio unaffected). Flat
   `OracleTagChips` removed (superseded). **Naming:** Deck Tags (deck) / Card roles (chart
   "Role distribution" + filter) / Oracle tags (full name). Known: a few flavor tags (e.g.
   `bible-reference`) leak into "other" — tune server-side via the noise list.
4. **Cleanup (only after retirement proven in a prod zervice run):** delete `classify.rs`
   (24-branch) + `classify_untagged_cards` from `ports.rs`/`services.rs`/`ErasedCardService`.
5. **`CardRole` rename** (65 refs, all crates) + **Phase M** (`mechanical_categories → card_roles`
   version-gated wire migration) — see `open-questions.md` §1 and Phase M below. Owner confirmed
   (2026-07-11) the coarse axis stays `card_roles` (NOT folded into the granular `oracle_tags`
   name). NB the frontend **display labels** already say "Card roles" / "Role distribution"
   (done in `6fc32c40`); Phase M is the remaining **wire/DB** rename (`oracle_tags_by_role` keys +
   `mechanical_categories` field/column → `card_roles`), version-gated.

### The authored map (in `derive_categories.rs` `CATEGORY_ROOTS` + `oracle_tag_gaps.rs`)

*18 otag-mapped* (root(s) expand through `oracle_tags.parent_ids`): removal→`removal`,
tutor→`tutor`, draw→`card-advantage`, counterspell→`counterspell`, evasion→`evasion`,
lifegain→`lifegain`, mill→`mill`, burn→`burn`, wipe→`sweeper`, blink→`flicker`, drain→`drain-life`,
sacrifice→`sacrifice-outlet`, untap→`untapper`, recursion→`recursion`+`reanimate`,
ramp→`ramp`+`mana-producer`, counters→`gains/gives/repeatable-pp-counters`+`counters-matter`,
copy→`copy`+`clone`, anthem→`anthem`+`keyword-anthem`+`power-boost-to-all`+`toughness-boost-to-all`.
*Tokens* → `all_parts` component=token (4,614 cards). *Finisher* → DROPPED (owner-approved).
*Pump/Stax/Protection/GraveyardHate* → heuristic (`oracle_tag_gaps.rs`) — no clean otag concept;
parity confirmed keeping their simple regex beats a lossy otag mapping.

---

## Phase 3 — Deck otag selection + archetype demotion (additive)

**Goal:** a deck declares its strategy otags; picking an archetype seeds them (Q2 decision).

**Selection UX (owner-decided 2026-07-11).** Give decks **direct oracle-tag selection**, not just
Deck Tags/roles — not everything maps to a Deck Tag, so power users need the granular axis.
Layered so beginners aren't dumped into 4,494 tags:
- **Deck Tag = easy on-ramp** — picking an archetype **seeds** its oracle tags (the authored
  `DeckTag → otag-set` map). Most users never open the raw list.
- **Direct otag selection = advanced refinement** on top of the seed. Hint it as *advanced*
  ("~4,500 of them, expect some sifting").
- **Hint/help page** = the card drill-down's structure at catalog scale: a **grouped** view
  (reuse `oracle_tags_by_role` grouping + the curated ~48, tags under their role) + a **raw**
  searchable list of everything else (off `GET /api/card/oracle-tags`, all 4,494).
- NB: Card **roles** are NOT a deck-level pick (they're computed from the deck's cards → the
  distribution chart). The two things a user *selects* at deck creation are Deck Tags + Oracle tags.

**Tables:** add `otags JSONB NOT NULL DEFAULT '[]'` + GIN index to `decks` (new migration,
mirrors `20260625000000_add_deck_tags.sql`).

**Shared (`zwipe-core`):**
- Authored **`DeckTag → otag-set` correlation** (~120 entries) as a curated map in
  `domain/deck/models/` (same home + style as `deck_tag.rs`; stable data, lives in code not
  DB).
- `http/contracts/deck.rs` — add `oracle_tags` to `HttpCreateDeckProfile` (`Option<Vec<String>>`,
  `#[serde(default)]`) and `HttpUpdateDeckProfile` (`Opdate<Vec<String>>`, `#[serde(default)]`);
  add to `HttpSharedDeck` (`#[serde(default)]`). `domain/deck/models/deck_profile.rs` +
  `requests/{create,update}_deck_profile.rs` — validate/carry the field (mirror `other_tags`).

**Backend:**
- `zerver/src/lib/outbound/sqlx/deck/{mod.rs,models.rs}` — persist/read `decks.oracle_tags` in
  create/update/get/clone (mirror the `other_tags` serializer + `clone_deck` column copy).

**Frontend (`zwiper`):**
- Generalize `.../deck/components/tag_select.rs` into a searchable otag picker (firehose +
  alphabetical + search, per Q5). Wire archetype selection to seed otags via the correlation.
  Hosts: `.../deck/create.rs`, `.../deck/edit.rs` (Opdate diff like `other_tags`).
- Scoped distribution view (selected + top-N present in decklist).

**Wire & compat:** additive only — `other_tags` already proves this exact pattern is
back-compat-safe (tests in `contracts/deck.rs`). **No bump.**

**Exit:** decks carry otags; archetype seeds them; picker + distribution ship.

### Grounded touchpoints (scouted 2026-07-12) — build in 4 additive slices

**Key correction:** `decks.other_tags` is NOT free-text — it's a second **curated enum**
(`DeckOtherTag`, `deck_other_tag.rs`). So `decks.oracle_tags` is a **new shape for decks**:
`Vec<String>` slugs (free strings from the `oracle_tags` catalog), NOT an enum. Store/decode as
`Vec<String>` (no `TryFrom` enum filter). Everything else mirrors `other_tags` exactly.

- **Slice A — backend plumbing (mechanical, additive): ✅ DONE `08b485eb`.** Migration
  `20260712040000`, `DeckProfile.oracle_tags` + `deck_oracle_tags.rs` (dedupe + cap 30),
  create/update requests, HTTP contracts (create/update/shared), sqlx create/get/get_all/update/
  clone + decode, handlers, `.sqlx` regen, HTTP round-trip test in `deck_flows`. Not yet pushed.
  <details><summary>original checklist</summary>

  - Migration `decks.oracle_tags JSONB NOT NULL DEFAULT '[]'` + GIN (next ts after `20260712030000`).
  - `deck_profile.rs`: `#[serde(default)] pub oracle_tags: Vec<String>` (line ~35, beside `other_tags`).
  - `create_deck_profile.rs` / `update_deck_profile.rs`: carry `oracle_tags` (create: `Vec<String>`;
    update: `Option<Vec<String>>` domain / `Opdate` wire). Validation = just a `MAX_DECK_ORACLE_TAGS`
    cap (no enum parse — free slugs).
  - `http/contracts/deck.rs`: `HttpCreateDeckProfile.oracle_tags: Option<Vec<String>>`,
    `HttpUpdateDeckProfile.oracle_tags: Opdate<Vec<String>>`, `HttpSharedDeck.oracle_tags: Vec<String>`
    (all `#[serde(default)]`).
  - handlers `create_deck_profile.rs` (`.oracle_tags(body.oracle_tags.unwrap_or_default())`) +
    `update_deck_profile.rs` (`.oracle_tags(body.oracle_tags.into_option())`).
  - `outbound/sqlx/deck/models.rs`: `DatabaseDeckProfile.oracle_tags: Option<serde_json::Value>` +
    `serde_json::from_value::<Vec<String>>` in `TryFrom`.
  - `outbound/sqlx/deck/mod.rs`: `deck_oracle_tags_to_json` helper; add the column to **create**
    INSERT+RETURNING, **get**/**get_all** SELECT, **update** QueryBuilder branch + RETURNING, and
    **clone_deck** INSERT+SELECT copy. `.sqlx` regen (create/get use macros; update is QueryBuilder).
  </details>
- **Slice B — the seed map: ✅ DONE `7690f984`.** `DeckTag::oracle_tag_slugs(&self)` in `deck_tag.rs`
  maps ~50 common archetypes → curated slugs (all 107 owner-approved + validated against the live
  catalog; unmapped → `&[]`). `seed_oracle_tags(&[DeckTag])` (deck_oracle_tags.rs) unions + dedupes.
  Tuned over time via feedback. Not yet pushed.
- **Slice C — frontend: ✅ FIRST CUT `789a0b70` (visuals to tune).** New `OracleTagSelect` overlay
  (`deck/components/oracle_tag_select.rs`) — fetched catalog + curated default grid + full-catalog
  search + def-bar, cap 30; wired as an "Oracle tags" field in `deck_fields.rs` + `create.rs`/`edit.rs`
  (create `Option`, update `Opdate` diff). Seeding: a `use_effect` on `selected_tags` unions
  `seed_oracle_tags` for **newly-added** deck tags only (peek-based, additive; deselects + manual
  removals stick; edit inits `seeded_from` to the loaded tags so it doesn't re-seed). `CURATED_ORACLE_TAGS`
  promoted to zwipe-core, shared with the card filter. Compiles + clippy + fmt green; **not pushed**.
  UX polish landed (`baf23278`): reconcile-on-Done seeding (drop old seed, add new from final deck
  tags, keep manual picks; one info toast) instead of per-tap; Cancel buttons on the deck-tag,
  oracle-tag, and format pickers (snapshot on open; format restores its command-zone cascade);
  "Tags"→"Deck tags" rename; chip-wrap CSS. Deck **view** split the three tag rows out of the profile
  card into a collapsible **Tags** section (`DeckTagsSection`, count badge, collapsed by default); the
  create/edit **form** groups the tag fields at the bottom under a "Tags" heading, mirroring view order.
  `CURATED_ORACLE_TAGS` lives in zwipe-core, shared by the card filter + deck picker. Not pushed.
- **Slice D — the grouped/raw hint page** (polish; the picker's grouped view doubles as this).

**Exit:** decks carry otags; archetype seeds them; picker + distribution ship.

---

## Phase 4 — Serving term (backend-only, ordering change)

**Goal:** fold otags into the serve (Q4 decision: one small `W_ORACLE_TAG` term first).

**Backend:**
- `zerver/src/lib/outbound/sqlx/card/mod.rs` `search_scryfall_data_deck_aware` — add a
  `W_ORACLE_TAG` **otag-correlation** term = overlap of `card_profiles.oracle_tags` with the deck's
  selected otags (`?|` inline, no new join). Keep `W_ORACLE_TAG` **small** (revert lever).
- `zerver/src/lib/domain/deck/services.rs` `search_deck_cards` — load `decks.oracle_tags`; apply
  the cold-start ladder (selected otags → commander's popular otags → nothing / today's
  behavior).

**Wire & compat:** the serve **response shape is unchanged** — only ordering shifts. No
contract change, **no bump**. (The otag *signal* term is deferred to Phase 6.)

**Exit:** decks with selected otags get otag-aware ordering; zero regression when otags absent.

---

## Phase 5 — Non-EDH signal collection (backend, ship dark)

**Goal:** start accruing the moat dataset *before* serving on it (Q7 decision).

**Tables:** new **generalized-context** per-otag signal table keyed
`(context, otag, shown/added/skipped/...)` where `context` = a commander **or** `(format,
CI)`; + a nightly rollup matview (mirror `card_signal_rollup`).

**Backend:**
- `zerver/src/lib/outbound/sqlx/metrics/mod.rs` — on each swipe, also credit **one row per
  otag** the deck has selected (derive `format`/`CI`/`oracle_tags` server-side from the deck, so
  likely **no client wire change** to `POST /api/metrics/usage`; confirm the handler
  `handlers/metrics/record_usage.rs` has deck context).
- `zerver/src/bin/zervice.rs` — refresh the new rollup nightly.

**Wire & compat:** signal derived server-side; ship dark. **No bump.** If it turns out the
client must send new context, hold it behind the min-version gate (last resort).

**Exit:** per-otag `(context)` signal accumulating for both Commander and non-EDH decks.

---

## Phase 6 — Non-EDH serving + otag signal term (deferred, data-hungry)

**Goal:** once Phase 5 data has volume, serve on it.

- Fold the **otag signal term** (the deferred half of Q4) into `search_scryfall_data_deck_aware`
  for the deeper-cuts / new-card cold-start win, reading the generalized-context rollup.
- Enable non-EDH serving pivoting on `(format, CI, selected otags)`.

**Wire & compat:** ordering/behavior only; additive. Revisit only after the dataset matures
(`moat.md` — months, not launch).

---

## Phase M — `mechanical_categories → card_roles` wire migration (version-gated)

**Goal:** finish the rename off the wire — the coarse-category field becomes `card_roles`
everywhere, and the legacy `mechanical_categories` word disappears. Committed by owner
(2026-07-11). **Independent track:** can run any time after Phase 2 (once `CardRole` exists);
spans client upgrades, so it takes real calendar time. Full rationale: `compatibility.md`
§Naming.

**Steps (dual-emit → migrate clients → sunset):**
1. **Add `card_roles`** beside `mechanical_categories` — server **dual-emits** both (same
   `Vec<CardRole>` values) on `Card`/`CardProfile`; requests **accept both** `card_roles_*`
   and legacy `mechanical_categories_*` criteria (`#[serde(default)]`). Additive, no bump.
2. **Migrate clients** (`zwiper`, `zite`) to read `card_roles` + send `card_roles_*`; ship and
   let installs upgrade.
3. **Sunset** once a `MIN_CLIENT_VERSION` floor guarantees every install uses `card_roles`:
   drop the `mechanical_categories` response field + legacy criteria, and rename the DB column
   `card_profiles.mechanical_categories → card_roles`. This is the **one gated removal** in the
   whole feature; bump `MIN_CLIENT_VERSION`.

**Wire & compat:** additive through step 2 (no bump); step 3 is the single gated removal —
long after ship, when old installs have aged out.

---

## Dependency order

```
0 spike ─▶ 1 ingest ─▶ 2 filtering + retire heuristic (needs card_profiles.oracle_tags)
                        1 ─▶ 3 deck otags ─▶ 4 serving term
                                    3 ─▶ 5 signal collection ─▶ 6 non-EDH serving
                        2 ─▶ M card_roles wire migration (independent, version-gated)
```

1 unblocks everything. 2 and 3 are parallel after 1. 4 needs 3. 5 needs 3 (decks must have
otags to key on). 6 needs 5 to have run long enough to matter. **M** needs only 2 (the
`CardRole` rename) and runs on its own client-upgrade timeline.
