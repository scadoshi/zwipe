# Deck MVPs — star the cards that define your deck

**Status: PLANNED (2026-07-06). Not started. Client + server — 1.4.0
candidate (first client-riding feature since 1.3.1).**

**What this builds, in one sentence:** each deck gets three MVP slots — the
user stars the cards that define the deck, the stars show at a glance in the
deck card list, and vested MVPs both steer that deck's own suggestions and
feed the strongest per-card signal the system collects.

**Why this shape (owner calls, 2026-07-06):**
- **Podium, not wallet.** 3 slots *per deck* (not N per day): starring a
  fourth means demoting one, which is the honesty forcing-function. No spend
  pressure, no waste anxiety, fully retroactive (you often only know the MVP
  after playing).
- **Honesty via selfish utility.** MVPs steer the deck's own serve ordering,
  so aiming them well benefits the owner directly; the global signal is
  exhaust from people steering their own decks.
- **Signal quality.** A vested MVP is the loudest per-(commander, card)
  datapoint available — several times an add's weight.

## Decisions (settled 2026-07-06)

- **Star lives in the deck cards screen, in the main row, visible before
  expansion** — MVPs identifiable at a glance. Star-in-place among the
  categories; no pinned MVP section; list order untouched.
- **Vesting 3 days, global signal only.** Long enough to outlive an impulse
  star that gets cut, short enough to feel responsive. **Deck steering is
  immediate** — no honesty risk in steering your own deck, and instant
  feedback teaches users the feature works.
- **Deck-list starring only at v1** — no mid-swipe entry point (MVPs are
  recognized after the fact).
- **Derive, don't collect**: no telemetry counter, no vesting job — vested
  MVPs are computed from live `deck_cards` rows at rollup time; unstar or
  removal self-retracts.

## Phases

1. **1.4.0 client + schema** — [`server.md`](server.md) steps 1–7 +
   [`client.md`](client.md). Collection starts.
2. **Signal weight** (server-only, later) — [`server.md`](server.md) step 8.
3. **Deck steering** (server-only, later) — [`server.md`](server.md) step 9.
4. **Artifacts** — MVPs on shared deck pages / weekly share cards (pairs
   with the badges backlog item).

Related: [`../suggestion_signal.md`](../suggestion_signal.md) (the ordering
this feeds), [`../wildcard_slot/`](../wildcard_slot/overview.md) (exposure
for deep cards; MVPs are the confirmation layer on what wildcards surface).
