# Oracle tags (otags) — plan index

**Status: BUILDING (updated 2026-07-11).** Phase 1 (ingest) is **shipped + committed**; all
7 open questions are resolved (`open-questions.md`). Note Q1 was **revised after Phase 1**:
measured coverage showed otags supersede our heuristic, so the plan now **retires
`classify.rs`** (was "complement + seed") and the old heuristic-backfill phase is cut. Scope
is grounded in a codebase map.

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

0. **Spike** — confirm the bulk file shape (keying, descriptions). ✅ done
1. **Ingest** — `otags` catalog + `card_otags` + daily `zervice` sync. ✅ **shipped**
2. **Filtering + retire heuristic** — `card_profiles.otags` projection, otag filter fields,
   otags on served cards; delete `classify.rs`, derive `mechanical_categories` from otags.
3. **Deck otags** — `decks.otags` + archetype→otag seeding + searchable picker.
4. **Serving** — one small `W_OTAG` correlation term in the ranking query.
5. **Signal collection** — generalized-context per-otag signal, shipped dark.
6. **Non-EDH serving** — deferred; serve on the accrued dataset once it matures.

Land 1-4 on Commander first (that is where the data and usage are); 5-6 accrue over time.
**Every phase is additive — no `MIN_CLIENT_VERSION` bump required.**
