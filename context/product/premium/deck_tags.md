# Deck tags — the foundation everything else consumes

**Tier: free (infrastructure).** Not a premium feature itself — the join key
that makes four premium features possible. Likely the next build step
(noted 2026-06-10).

## Concept

Two tag layers drawing from **one closed vocabulary** (the mechanical-category
taxonomy, see `../../plans/mechanical-category.md`):

- **Card tags** — what a card *does*. Already planned:
  `card_profiles.mechanical_categories`, Layer 1 oracle-text heuristics
  (~70–80% accuracy), AI classification later (backlog Layers 2–3).
- **Deck tags** — what a deck *wants to do*. User-declared intent, picked
  from the same vocabulary via a chip/picker UI on the deck.

The intersection is the product: "cards in this deck that serve this deck's
plan" is computable, per tag, with no AI.

## Who consumes it

| Consumer | How |
|---|---|
| Consistency calculator | P(opening hand contains a card matching each deck tag) — pure hypergeometric |
| AI analysis | Tags constrain preset prompts ("this is a Treasure/sacrifice deck — suggest cuts that don't serve that") |
| Smart stack ordering | Boost cards whose categories match the deck's tags |
| Bracket estimate | Some categories (mass land denial, extra turns, tutors) map straight onto bracket criteria |
| Taste profile | Aggregation bucket: ~50 categories instead of 35k cards |

## Security property

Users **pick tags from a list — they never type free text that reaches a
model**. The closed vocabulary is what keeps the AI features
prompt-injection-free (see `ai_analysis.md`). Preserve this: no free-text
custom tags, at least not any that get forwarded to an LLM.

## Implementation sketch

- Schema: additive — a `tags` column (text[] or jsonb) on decks, or a
  `deck_tags` join table. New request fields `#[serde(default)]` per
  `development/api_evolution.md`, so old clients are unaffected.
- Validation: tags validated server-side against the canonical taxonomy enum
  in zwipe-core (shared so the client picker and server agree).
- UI: chip picker on the deck edit screen, same chip idiom as the import
  screen's From/Mode/Board rows.

## Sequencing

1. Deck tags schema + picker UI (small, additive).
2. Layer 1 card heuristics from the mechanical-category plan.
3. Consumers (consistency calculator first — days, not weeks, once 1–2 exist,
   and it costs zero API calls).
