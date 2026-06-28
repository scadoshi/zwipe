# Draw-Odds / Consistency Charts

## Goal

Show a player the **probability of drawing what they need** from their deck:
"what are the odds I see at least one land / draw spell / sac outlet / GY-hate /
creature / artifact in my opening hand — or by turn N?" Same for mana-value
buckets ("odds of a 4-drop vs a 5-drop in my first 7"). A consistency calculator
that turns the deck's composition into plain-language odds.

This is a **high-impact differentiator**: most deck builders show a static mana
curve; almost none turn it into *draw probabilities*. Strong premium-tier
candidate (the existing Stats/curve panels are the free baseline; this is the
"power" analytics layer).

## The math — hypergeometric distribution

Drawing without replacement from a finite deck is exactly the hypergeometric
model. For a deck of `N` cards containing `K` cards of a category, drawing `n`
cards:

```
P(exactly k) = C(K, k) * C(N-K, n-k) / C(N, n)
P(at least 1) = 1 - C(N-K, n) / C(N, n)
P(at least t) = 1 - sum_{k=0}^{t-1} P(exactly k)
```

- **Opening hand:** `n = 7`.
- **By turn T:** `n = 7 + T` on the draw, `7 + (T-1)` on the play (decide default;
  let the user toggle play/draw).
- Compute `P(>=1)` by default; allow `P(>=t)` for a chosen threshold.

Implement with log-factorials or incremental ratios to avoid overflow on
`C(100, 7)`-scale numbers. Pure function in `zwipe-core` (no deps), fully unit
testable against known values.

## Inputs — we already have the data

Per deck, bucket the mainboard cards three ways and feed `K` per bucket:

1. **By mechanical category** — *reuse the existing classification* in
   `zwipe-core/.../card/models/mechanical_category/` (land, ramp, draw, removal,
   sacrifice, graveyard-hate, etc.). This is the headline view and the reason the
   feature is cheap: the category data already exists on `CardProfile`.
2. **By card type** — creature / artifact / enchantment / instant / sorcery /
   planeswalker / land (from `type_line`).
3. **By mana value** — 0,1,2,3,4,5,6,7+ buckets (from `cmc`).

`N` = total mainboard count (respect quantities; basics count as their copies).

## Where it lives

- **Computation:** a pure `zwipe-core` module (`draw_odds` / `hypergeometric`) so
  it's shared, tested, and client-side — no server, no new endpoint. The client
  already has the full deck (types, cmc, categories) loaded.
- **UI:** a new section on the deck profile / stats surface, and reachable from
  the in-build stats sheet (see `qol_bundle.md` item D). Likely a horizontal bar
  list: each category/bucket with its `P(>=1)` for the chosen draw window.
- **Controls:** opening-hand vs by-turn-N slider; play/draw toggle; optional
  threshold (`>=1` / `>=2`).

## Open questions / modeling caveats

- **Mulligans:** London mulligan changes effective odds; MVP ignores it (model a
  single keep of N). Note this as "raw odds, pre-mulligan."
- **No tutors/scry/draw:** hypergeometric is a *baseline* — it doesn't model card
  selection. Frame honestly ("odds from a random draw").
- **Play vs draw default:** pick one (commander is usually "on the draw"-ish in a
  pod) and let the user flip it.
- **Mana value edge cases:** X spells (cmc as printed), MDFCs (which face's cmc),
  lands (cmc 0 — exclude from the MV view or bucket separately).
- **Category overlap:** a card can be ramp *and* a creature; it counts in each
  bucket it belongs to. That's correct for independent `P(>=1 of category)`
  questions — just don't sum buckets to N.
- **Which buckets to surface by default** vs. behind a "more" expansion (there are
  many categories; lead with the most actionable: lands, ramp, draw, removal).

## Phasing

1. **Core math first:** the hypergeometric module + unit tests (known values:
   e.g. ~ P(>=1 land | 37/99, draw 7)). No UI — just the tested engine.
2. **Category odds view:** wire the three bucketings to the engine, render the
   opening-hand `P(>=1)` list on the deck stats surface.
3. **Interactivity:** by-turn slider, play/draw toggle, threshold selector.
4. **Polish / premium gating decision:** decide which depth is free vs premium.

## Effort & deploy

Client + core only (no server, no migration). Effort is **M–L**: the math is
small and well-defined, the bucketing reuses existing data, the UI/interactivity
is the bulk. Phase 1 (the tested engine) is small and de-risks the rest.
