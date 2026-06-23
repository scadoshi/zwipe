# AI deck analysis — preset prompts

**Tier: premium (headline feature).** Real ongoing cost (API calls) — the
honest core of the subscription.

## Concept

The user picks a **preset prompt** against their deck — no free-text input to
the model, ever:

- "Suggest five cuts"
- "Make my deck better"
- "Find my win condition"
- "Tighten the curve"
- "Help me get to Bracket N" (the paid layer of `bracket-estimate.md`)

The server renders a prompt from a **server-side preset registry** (patchable
without app release), the deck list, the deck's tags, commander, and format.
Output is structured JSON → card suggestions, which can feed straight into a
**swipe stack of suggestions** — the premium feature lands inside the core
loop instead of a wall of text.

## Economics (estimated 2026-06-10)

Claude Haiku ≈ $1/MTok in, $5/MTok out → a full deck analysis (~100-card list
+ oracle text excerpts in, structured suggestions out) lands around
**$0.01/analysis**. At $3–5/month, a user would need hundreds of analyses a
month to be unprofitable; a soft rate limit (e.g. N/day) keeps the tail safe.
No fine-tuning — an all-purpose model with good prompting suffices; revisit
only if quality demands it.

## Safety / correctness design

- **Zero prompt-injection surface**: users pick presets and closed-vocabulary
  tags (`deck-tags.md`). No user-typed text reaches the model. **Never pass
  the deck name** — it's the one free-text field adjacent to the request.
- **Hallucination filter**: every suggested card name is validated against our
  own DB via the existing exact-name lookup (`find_cards_by_exact_names`).
  Names that don't resolve are silently dropped before the user sees them.
- Suggestions carry oracle_ids after validation, so the client renders real
  cards, never model text.

## Plumbing

- Lives in zerver (per `../monetization.md` technical path): paid users get
  the route, free users get a 402 via the entitlement flag
  (`iap-infrastructure.md`).
- Preset registry server-side: add/tune prompts without an app release.
- Hybrid with Recommander's stats layer is possible if the integration lands
  (see `status/backlog.md`) — statistical candidates in, LLM curation on top.

## Depends on

Deck tags (`deck-tags.md`) for prompt context; IAP entitlements to gate it.
