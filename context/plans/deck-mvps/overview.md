# Deck MVPs — star the cards that define your deck

**Status: Phase 1 BUILT + SHIPPED TO MAIN 2026-07-07 (`abaaec0e`; server
steps 1–7 + client), API-verified end-to-end on dev and simulator-verified
(star reworked during the pass: indicator on starred rows only, Star/Unstar
button in the expanded row — an outline star on every row was 97% noise). Phases 2 (signal weight) and 3
(steering) remain server-only follow-ups. As-built deltas: export carries NO
MVP marker (a trailing `*` would corrupt pastes into Archidekt/Moxfield —
cross-tool safety won; MVPs travel via clone); the star renders inside the
name cell (the row grid is fixed-width columns); the one-time `deck-mvps`
hint fires only for users who already saw the deck-cards hint, and new users
get a star bullet inside that hint instead (two dialogs on one visit would
bury both). Dev E2E matrix: 3 stars OK, 4th → 422 "This deck already has 3
MVPs" verbatim, maybeboard star → 422, re-star preserves the vesting clock,
board move off mainboard clears the star and frees the cap, unstar clears,
clone inherits MVPs with original timestamps.**

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
   with the badges backlog item). **Shipped on the shared deck page 2026-07-07**
   (`d8f7dd4e` + `e5ed5e33`): the payload already carried `mvp_at`, so starred
   cards render the warning-gold ★ inline, and the page opens with a featured
   row of the commander + the MVPs as full art (each labeled) — the personality
   statement this plan envisioned. Weekly share cards remain.

Related: [`../suggestion_signal.md`](../suggestion_signal.md) (the ordering
this feeds), [`../../archive/wildcard-slot/`](../../archive/wildcard-slot/overview.md) (exposure
for deep cards; MVPs are the confirmation layer on what wildcards surface).
