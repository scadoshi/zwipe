# Payoff — immediate vs long-term

Kept deliberately separate so scope decisions weigh near-term wins against speculative
accrual honestly.

## Immediate (ships with the first phases, Commander)

- **Drop a maintenance burden.** otags replace the hand-owned regex categories on any
  card that is tagged; we stop chasing corner cases the community already handles.
- **Accuracy on tagged cards** goes up over our ~70-80% heuristics, and we finally get a
  labeled set (cards with both otag + heuristic label) to measure the heuristics against.
- **Player-facing alignment.** The tags on a card in-app match what players see on
  Scryfall — less "why does Zwipe think this is X".
- **Richer filtering.** otag filter predicates sit right beside the existing three
  `mechanical_categories_*` predicates — more precise deck-building filters at low cost.
- **Better Commander serving.** The otag-level swipe rollup surfaces **deeper cuts** and
  fixes **cold-start for new cards** (see `purpose.md` §real payoff). This is the headline
  near-term win.

## Long-term (accrues over months, cross-format)

- **Non-EDH format serving** — the `(format, CI, otag)` dataset moat (`moat.md`). Weak at
  first, compounding, uncopyable.
- **MVP "more like this"** — otag overlap with starred cards, once the MVP feature and
  otag data coexist.
- **A unified tagging vocabulary** — `deck_tag` / `deck_other_tag` / `mechanical_category`
  collapse toward one otag-backed system instead of three overlapping hand-curated lists
  (`open-questions.md` §reconciliation).

## What is NOT a payoff (manage expectations)

- otags do **not** improve coverage on obscure cards; they can make it worse until
  heuristic backfill fills the gaps. Coverage is a cost to pay, not a benefit.
- Non-EDH serving is **not** launch-quality; treat any day-1 non-EDH serving as a data
  collector wearing a UI, not a finished feature.
