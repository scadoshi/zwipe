# Oracle tags (otags) — plan index

**Status: PLANNING / decisions locked (updated 2026-07-11).** Concept validated with the
owner, and **all 7 open questions are resolved** (`open-questions.md`) — data access,
`mechanical_categories` fate, deck-tag reconciliation, granularity, serving weights, UI,
perf, and the non-EDH signal schema are all decided. Scope is grounded in a codebase map.
Not started; next step is breaking the sequencing below into tickets.

## One sentence

Ingest Scryfall's **Oracle Tags** (community-maintained functional tags: `removal`,
`ramp`, `card-advantage`, `tutor`, ...) via a daily `zervice` bulk sync, correlate every
card to its otags by `oracle_id`, let players select the otags that describe a deck's
strategy, and use that community-accurate tagging as a new axis for filtering, serving,
and cross-format swipe-signal collection.

## The files

| File | Owns |
|------|------|
| `purpose.md` | What otags are, why they beat our heuristics, the swipe-at-otag insight, the data pipeline |
| `moat.md` | The non-EDH cross-format dataset moat (the long game) |
| `payoff.md` | Immediate vs long-term payoff, honestly separated |
| `scope.md` | Every backend + frontend file/table touched, grounded in the current code |
| `compatibility.md` | How to NOT break already-installed mobile clients |
| `open-questions.md` | The 7 decisions, all resolved (2026-07-11) with rationale |
| `sequencing.md` | The phased build — per-phase files touched + additive-wire guarantee |

## What changed on 2026-07-11

The original `otags.md` treated **data access as the critical open question** — otags
were assumed to live only behind Scryfall's undocumented Tagger GraphQL API. They now
ship as a standard bulk file (`Oracle Tags`, 17.2 MB, updated daily ~09:00 UTC, at
`data.scryfall.io/oracle-tags/...`). That collapses the highest-risk unknown into a
routine bulk ingest that mirrors our existing Scryfall sync. See `purpose.md` §pipeline.

## Sequencing

Full phase-by-phase build (files touched + per-phase additive-wire guarantee) lives in
**`sequencing.md`**. In brief:

0. **Spike** — confirm the bulk file shape (keying, descriptions).
1. **Ingest** — `card_otags` table + `card_profiles.otags` projection + daily `zervice` sync.
2. **Backfill** — heuristics fill the curated serve-critical otags, with `source` provenance.
3. **Filtering** — otag filter fields + otags on served cards (first client-visible piece).
4. **Deck otags** — `decks.otags` + archetype→otag seeding + searchable picker.
5. **Serving** — one small `W_OTAG` correlation term in the ranking query.
6. **Signal collection** — generalized-context per-otag signal, shipped dark.
7. **Non-EDH serving** — deferred; serve on the accrued dataset once it matures.

Land 1-5 on Commander first (that is where the data and usage are); 6-7 accrue over time.
**Every phase is additive — no `MIN_CLIENT_VERSION` bump required.**
