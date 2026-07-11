# Payoff — immediate vs long-term

Kept deliberately separate so scope decisions weigh near-term wins against speculative
accrual honestly.

## Immediate (ships with the first phases, Commander)

- **Kill a maintenance burden outright.** We **retire `classify.rs`** (the ~70-80% regex
  heuristic) rather than complement it — measured coverage showed otags are 2-2.6x more
  complete on removal/tutor/draw/counterspell (`open-questions.md` §1). `mechanical_categories`
  survives as a wire field but is now **derived from otags**, gold-standard accurate, and no
  longer our guesswork to grow.
- **Player-facing alignment.** The tags on a card in-app match what players see on
  Scryfall — less "why does Zwipe think this is X".
- **Richer filtering.** New otag filter predicates sit beside the (now otag-derived)
  category filter — far more granular deck-building filters at low cost.
- **Better Commander serving.** The otag-level swipe rollup surfaces **deeper cuts** and
  fixes **cold-start for new cards** (see `purpose.md` §real payoff). This is the headline
  near-term win.

## Long-term (accrues over months, cross-format)

- **Non-EDH format serving** — the `(format, CI, otag)` dataset moat (`moat.md`). Weak at
  first, compounding, uncopyable.
- **MVP "more like this"** — otag overlap with starred cards, once the MVP feature and
  otag data coexist.
- **A unified tagging vocabulary** — `deck_tag` / `deck_other_tag` / `mechanical_category`
  all become otag-backed (the first two demoted per Q2, the last derived per Q1) instead of
  three overlapping hand-curated lists.

## What is NOT a payoff (manage expectations)

- **The two fuzzy categories are work, not a freebie.** `evasion` and `ramp` need
  multi-root otag mappings and a parity check before `classify.rs` is deleted
  (`open-questions.md` §1) — the retirement is safe but not zero-effort.
- Non-EDH serving is **not** launch-quality; treat any day-1 non-EDH serving as a data
  collector wearing a UI, not a finished feature.
