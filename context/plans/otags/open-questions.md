# Open questions — decisions still to make

The original doc's #1 open question (**data source access**) is **RESOLVED**: Oracle Tags
is now a standard Scryfall bulk file (README §"what changed"). What remains:

## 1. mechanical_categories fate — DECIDED: retire the heuristic, otags supersede

**Settled 2026-07-11 (owner), reversing the earlier "complement + seed" call after
measuring live coverage post-Phase-1.** The community set is not merely as good as our regex
heuristic (`classify.rs`, ~70-80% accuracy) — it is markedly **more complete**. Per category,
counting distinct oracle_ids (otag column = the category's otag root expanded through the
ingested `parent_ids` hierarchy; heuristic-only = cards the heuristic tags that the subtree
misses):

| category | heuristic | otag subtree | heuristic-only |
|---|---|---|---|
| removal | 2,534 | 6,626 | 301 |
| tutor | 511 | 1,197 | 12 |
| draw | 4,277 | 6,526 | 201 |
| counterspell | 236 | 551 | 1 |
| ramp | 1,505 | 2,388 | 410 |
| evasion | 5,656 | 5,499 | 1,218 |

otags dominate removal/tutor/draw/counterspell (2-2.6x coverage, tiny residue), and the
recent-release lag we feared did not appear (13/13 latest-window cards already tagged). So:

- **Retire `classify.rs`** (the guesswork) and remove `classify_untagged_cards` from
  `zervice`. Owner's rule: don't hand-maintain a heuristic when the community keeps a gold
  standard.
- **`oracle_tag` is the canonical name** (DB/Rust/wire); `mechanical_categories` is retired
  as a concept but **kept on the wire as a deprecated translation** — the served `Card` emits
  *both* `oracle_tags` (canonical, granular) and `mechanical_categories` (legacy 24 coarse
  categories, derived from oracle_tag subtrees) so old clients don't break. Dual-emit now,
  drop the legacy key behind a `MIN_CLIENT_VERSION` floor later. Full mechanism in
  `compatibility.md` §Naming. Mirrors the Q2 "keep the vocabulary, back it with oracle tags"
  pattern.
- **Two fuzzy categories need multi-root mappings:** `evasion` (spans `flying` / `menace` /
  `can't-be-blocked` …) and `ramp` (`ramp` + `mana-producer`). Author with care.
- **Gate the switchover behind a filter-parity check** on the overlap set, and sample the
  `evasion`/`ramp` heuristic-only residue once (much of it is heuristic *false positives*,
  not otag gaps) before deleting `classify.rs`.
- The 24-category *vocabulary* can be fully sunset later behind a min-version floor if the
  new granular otag filters make it redundant; not now.
- **Phase 2 (heuristic backfill) is cut** — there is nothing to backfill.
- **Provenance** (`source` on `card_oracle_tags`) stays for the rare hand-added correlation but is
  no longer load-bearing.

## 2. Deck-tag reconciliation — DECIDED: demote `DeckTag`, one drives the other

**Settled 2026-07-11 (owner).** `DeckTag` does **not** go extinct — it is **demoted** to a
high-level archetype **container** that maps to card-level otags:

- Author a curated **`DeckTag` → otag-set correlation** (~120 archetypes, one-time,
  stable). E.g. `voltron → {auras, equipment, hexproof}`, `aristocrats → {sacrifice,
  tokens, death-triggers, drain}`.
- **One selection drives the other:** picking an archetype seeds the deck's otags, which
  the user can then refine. Archetype = the human-facing browse/display label; otags = the
  functional serving axis.
- This correlation does triple duty: archetype-seeds-otags, cold-start serving, and *would
  be* the migration tool if `DeckTag` is ever dropped later (reversible — not doing that
  now).

**Storage (also settled — see §6 for the perf reconciliation):**
- **`card_oracle_tags` (card → otags): normalized, indexed table** = source of truth + bulk-file
  landing + rollup/analytics source. Serving does **not** join it directly.
- **Serve path reads a denormalized JSONB otag array on `card_profiles`** (GIN `?|`, like
  `mechanical_categories`), built from `card_oracle_tags` by a nightly `GROUP BY`.
- **Deck-selected otags: JSONB `decks.oracle_tags` column** (matches existing `decks.tags`
  pattern). The serve reads it once in `search_deck_cards` and passes it as a param list —
  no bulk decks↔otags join, so no `deck_otags` table needed.

## 3. otag granularity — DECIDED: store full tree, curate at query time

**Settled 2026-07-11 (owner).** Storage and serving are separate decisions and compose:

- **Store the full tree everywhere** (every otag a card carries, at every level). Ingest
  takes what Scryfall gives; complexity is pushed to query time.
- **Curate at serve time.** A hand-picked serving list (~40-80 meaningful mid-tier otags,
  largely the same set named in the `DeckTag → otag-set` correlation from §2) selects which
  otags actually drive serving. Tweakable as we go — it's a query-time filter, not a
  storage shape.
- **Roll up swipe signal at FULL-tree granularity, not the curated tier.** This is the key
  to keeping §3's "learned tier" (option c) open: if the rollup only aggregated the
  hand-picked otags, we'd never gather discrimination data on the rest. Aggregate all
  otags; the curated list filters *over* the full rollup at query time. Then tweaking the
  list needs no re-aggregation, and a future data-driven pruning has full-granularity
  history to learn from.

**Live evidence (Phase 1 ingest, 2026-07-11):** the raw firehose is noisier than expected.
The most frequent otags are structural/trivia (`activated-ability` 9026, `triggered-ability`
7886, `alliteration` 4346, `unique-type-line` 2182, `intervening-if-clause` 2170) sitting
right alongside the useful functional ones (`spot-removal` 4979, `evasion` 4567). Serving on
raw frequency would be actively wrong. The curated serving tier is not optional polish — it
is load-bearing, and authoring it is real work (filtering ~4,494 tags down to the ~40-80
that matter).

## 4. Serving weight + cold-start — DECIDED: one term first, fallback ladder

**Settled 2026-07-11 (owner).** Phase the algorithm change:

- **Phase 1 — one new term.** Add a single `W_ORACLE_TAG` **otag-correlation** term to
  `search_scryfall_data_deck_aware` = how well a card's otags overlap the deck's selected
  otags (`card_oracle_tags` × `decks.oracle_tags`), scored beside `base` (synergy) and `signal`
  (rollup). Keep `W_ORACLE_TAG` **small at launch** (the revert lever, like `BAND_SIZE=1`) so it
  nudges rather than dominates; dial up as trust grows.
- **Phase 2 — the otag *signal* term** (deeper cuts + cold-start for new cards) comes
  later, once the full-tree rollup (§3) has data. Higher value, data-hungry.
- **Cold-start ladder** (before a deck has selected otags):
  1. Deck has selected otags → use them.
  2. No selected otags but has a commander → use the **commander's most-popular otags**.
  3. No commander (non-EDH, pre-moat) → otag term contributes nothing; fall back to today's
     synergy+signal behavior. **No regression** when otag data is absent.

## 5. UI for hundreds of otags — DECIDED: firehose + search, definitions bounded

**Settled 2026-07-11 (owner).**

- **Selection surface: show the full firehose, alphabetical, plus search.** Not a new
  problem — `tag_select.rs` already renders ~120 `DeckTag` chips with search; hundreds is
  "more of the same." Generalize that component for otags; picker is **in v1** (archetype
  seed from §2 fills the default set, the picker is for refining). Do **not** use the 5-chip
  inline `deck_fields.rs` grid for otags.
- **Distribution view: scoped**, never the full vocabulary — the deck's selected otags +
  top-N otags actually present in the decklist (`card_oracle_tags` over the cards). Answers "is my
  deck doing what I said" without the firehose.
- **Definitions: do NOT hand-write all hundreds.**
  1. **First check the bulk file** — it may ship a per-tag description; if so, carry it
     through ingest (free). On the Phase-1 ingest checklist.
  2. If not: **humanize the slug** for the display label (`unconditional-creature-removal`
     → "Unconditional creature removal") — deterministic, zero authoring. Hand-write prose
     **only for the curated serving tier** (~40-80, the same set from §2/§3). Long-tail
     otags show humanized label + optional example card / count. Slugs are self-descriptive
     enough that the undefined tail still reads fine.

## 6. Volume / perf — DECIDED: two representations, each for its query shape

**Settled 2026-07-11 (owner).** Scale is hundreds of otags × 110k+ cards × many-per-card
(~1-1.5M `card_oracle_tags` rows). "Table vs JSONB" is a false split — a JSONB column lives on an
indexed table too; the point is matching index shape to query shape, and there are two:

- **Serve path** ("does this card's otag set overlap the deck's ~3-10 selected otags?", per
  page, hot): **denormalized JSONB otag array on `card_profiles` + GIN**, tested with `?|` —
  no join, no GROUP BY, *identical to how `mechanical_categories` already serves*.
- **Inverse / aggregation** ("all cards with otag X", the Phase-2 `otag × commander` signal
  rollup): the **normalized `card_oracle_tags(oracle_id, otag)` table** — naturally relational.

**Keep both.** `card_oracle_tags` normalized = source of truth (also the natural landing spot for
the bulk file, which arrives tag→oracle_id[]) + rollup + analytics. The JSONB-on-
`card_profiles` copy = serve path, built by one `GROUP BY oracle_id` at the end of the
nightly sync. Write-time cost on a batch job, read-time win on every swipe. Phase-2 signal
aggregation is a nightly **matview** like `card_signal_rollup`, never computed in-request.

## 7. Non-EDH signal schema — DECIDED: per-otag keying, one generalized-context table

**Settled 2026-07-11 (owner).** Do **not** key on the literal otag *set* — set-cardinality
is astronomical, every context is unique, ~zero signal accumulates per key. Instead:

- **Key per individual otag: `(format, color_identity, otag)`.** Each swipe credits one row
  per otag the deck has selected (a `{ramp, removal, draw}` deck credits three rows). Dense
  (hundreds of keys, not astronomical), composes at serve time by summing the deck's
  selected otags' signals.
- **One generalized-context signal model, not a second pipeline.** This is the *same* shape
  as the Phase-2 commander otag-signal (`otag × context`) — the commander case is just
  "context = this commander," non-EDH is "context = (format, CI)." Generalize the context
  column so Commander and format signal live in one table/rollup, not two systems.
- **Start collecting early.** Turn on per-otag signal keying as soon as deck otag selection
  ships, before non-EDH serving is good, so the flywheel has months of head start
  (`moat.md`).
