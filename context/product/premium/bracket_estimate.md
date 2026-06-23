# Bracket / power-level estimate

**Tier: FREE for the badge (decided in brainstorm 2026-06-10); the paid layer
is AI analysis.** Same split as prices: the number free, the intelligence
around it paid.

## Why the badge is free

- **Zero marginal cost** once built — Game Changers list lookup plus
  heuristics. No defensibility as a paid feature (anyone can copy a bracket
  checker).
- **Most shareable feature we could ship.** "Zwipe says my deck is Bracket 3"
  is a sentence spoken at pod tables — pre-game bracket talk happens at
  every Commander table now. A free bracket badge makes Zwipe the app people
  pull out at the LGS. That's acquisition, the free tier's whole job.

## The estimate

Inputs, roughly in WotC's own bracket terms:

- **Game Changers list membership** — count of GC cards in the deck.
- **Heuristics over mechanical categories** (`deck-tags.md` taxonomy):
  mass land denial, extra turns, tutor density, two-card-combo presence
  (combo detection may eventually want Commander Spellbook data — note, not
  scoped).

Output: bracket 1–5 with the contributing factors listed. Always framed as an
estimate — every pod argues about brackets, and arguing with the badge is
engagement, not failure.

## The premium layer

"You're Bracket 3 because of these two Game Changers and your tutor density —
here's what to swap to hit Bracket 2 for your playgroup." That's a preset in
`ai-analysis.md` ("Help me get to Bracket N"), costs real API calls, and is
the part people pay for.

## Operational note

The Game Changers list is WotC-maintained and changes a few times a year. It
must live in a **server-side table updatable without an app release** — same
patchable-without-review philosophy as the rest of the server-first decisions
(`dev/api_evolution.md`, `status/backlog.md` patch discipline). The client
only ever sees the computed estimate.
