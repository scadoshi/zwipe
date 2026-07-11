# Purpose — what otags are and why they matter

## The problem today

Our functional categorization of cards is `MechanicalCategory` — a 24-variant enum
(`zwipe-core/src/domain/card/models/mechanical_category/`) derived by our own regex
heuristics (`classify.rs`, self-documented at ~70-80% accuracy). It is:

- **Ours to maintain** — every corner case is a regex we write and own.
- **Brittle** — text-pattern derivation, not real semantics.
- **Internal-only** — it does not match the tags players see on Scryfall or elsewhere.

## What otags give us

Scryfall's **Oracle Tags** are hundreds of functional tags maintained by the community
(the Tagger project). Adopting them is:

- **Not our maintenance burden.** Corrections flow to us for free on the daily sync.
- **Aligned with what players already see** on cards when they search Scryfall.
- **A formalized vocabulary** our invented `deck_tag` / `deck_other_tag` feature can be
  backed by, instead of a hand-curated archetype list that overlaps otags anyway.

This is **not** a strict accuracy win — it is a precision-vs-coverage trade. Heuristics
give 100% coverage at ~70-80% accuracy; otags give higher accuracy on tagged cards but
**uneven coverage** (popular cards richly tagged, obscure cards thin or empty). So the
plan is complement/seed, not blind replace — see `open-questions.md`. Bonus: cards that
carry *both* an otag and our heuristic label become a free labeled test set to finally
measure that "~70-80%".

## The real payoff — swipe signal at the otag level

The strongest reason to build this is not filtering, it is **serving**. Today the pooled
swipe signal (`commander_card_signal` → `card_signal_rollup`) is **card-level and sparse**:
for any given commander, the long tail of cards has almost no swipes.

Roll that same signal up **to the otag level** and it gets dense: "for Atraxa, players
swipe right on `proliferate` / `+1/+1-counters` / `removal`." That unlocks:

1. **Deeper cuts** — serve *unseen* cards that carry the winning otags, not just the cards
   that already have swipe history. This is dimensionality reduction on a sparse signal
   using interpretable features.
2. **Cold-start for new cards** — a freshly released card inherits its otags' learned
   weight instantly, before anyone has swiped it.
3. **MVP "more like this"** — read the otags of a starred MVP card, serve overlap.

Blend otag correlation with the existing synergy-worker scores and swipe signal and the
serve gets richer than any single source.

## The data pipeline (now trivial — see README §"what changed")

- Oracle Tags is a **standard Scryfall bulk file**: `data.scryfall.io/oracle-tags/...`,
  ~17.2 MB, refreshed daily ~09:00 UTC (same cadence as our card sync).
- The bulk file is keyed **tag → oracle_id[]**, not card → tag[]. Ingest fans each tag
  out to its oracle ids and upserts a `card_otags(oracle_id, otag)` correlation.
- Because otags ride on `oracle_id`, and our Archidekt/text import already resolves cards
  to oracle ids, **imported decks inherit their cards' otags for free**.
- Note the decoy: Scryfall also ships **Art Tags** (38.5 MB) — illustration tags
  ("depicts a dog"), *not* functional. We want Oracle Tags only.

See `scope.md` for exactly where this slots into `zervice`.
