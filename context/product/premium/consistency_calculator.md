# Consistency calculator + opening-hand simulator

**Tier: DECIDED 2026-06-10 — split.** Headline consistency score, opening-hand
simulator, and mana math are **free** (zero cost, screenshot-able,
acquisition). The **per-tag breakdown** is **premium** (the intelligence
layer: which parts of your plan are inconsistent). Zero ongoing cost, so it's
a sweetener/anchor, not the subscription justification.

## Concept

Score a deck on **consistency, not quality** — honest math, no AI, no
hallucination risk, defensible to the decimal:

> "Your opening hand has a 71% chance of containing at least one card tagged
> Ramp."

For each deck tag (see `deck_tags.md`): count cards in the deck whose
mechanical categories include that tag, then closed-form hypergeometric for
P(≥1 in 7 draws). Extensions: P(by turn N), multivariate ("Ramp AND a draw
engine in the opener"), aggregate score across all the deck's tags. Examples
the math covers directly: odds of hitting draw cards, burn reach, sac outlets,
a land by turn 3 — whatever the deck's tags declare its plan to be.

Explicitly framed as "how often does your deck see its pieces," never "your
deck is good." Players obsess over exactly this ("do I sack out?" is a
frequency question).

## Opening-hand simulator

Same engine, sampling instead of closed form: shuffle, deal 7 with real card
images, London mulligan support, redraw button. Very demo-able and
screenshot-able — App Store listing material. Trivial to build once the deck
list is on screen.

## Mana math (adjacent, same file because same engine)

- Draw-probability calculator: "odds of a land by turn 3" — hypergeometric
  again.
- Mana base recommender from pip counts — the deck metrics screen already
  computes pip balance, so this is half-built.

## Implementation notes

- The math is a **pure function in zwipe-core** — no deps, fits the purity
  rules, shared by server and client.
- Can run entirely **client-side** on data the deck screen already has: no new
  endpoint, no server load, works offline.
- Depends on: deck tags + Layer 1 card categories (`deck_tags.md`). Without
  tags it degrades to generic land/curve math, which is still shippable as a
  free teaser.
