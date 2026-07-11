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

## Naming

**Canonical name: `oracle_tag` / `oracle_tags`** (the granular tags) — DB (`oracle_tags`,
`card_oracle_tags`, `card_profiles.oracle_tags`), Rust (`OracleTag`), wire (`oracle_tags` +
`oracle_tags_*` criteria). Separately, the **coarse ~24 functional categories** survive as a
distinct concept but are renamed off our old word: **`MechanicalCategory → CardRole`**, and
the legacy wire field `mechanical_categories` is migrated to **`card_roles`** in a version-gated
phase (`compatibility.md` §Naming, `sequencing.md` Phase M). `otag`/`otags` appears only as
informal prose shorthand in these docs; concrete identifiers are all spelled out.

## The files

| File | Owns |
|------|------|
| `purpose.md` | What otags are, why they beat our heuristics, the swipe-at-otag insight, the data pipeline |
| `moat.md` | The non-EDH cross-format dataset moat (the long game) |
| `payoff.md` | Immediate vs long-term payoff, honestly separated |
| `scope.md` | Every backend + frontend file/table touched, grounded in the current code |
| `compatibility.md` | How to NOT break installed clients + the `oracle_tag` naming / wire translation |
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
1. **Ingest** — `oracle_tags` catalog + `card_oracle_tags` + daily `zervice` sync. ✅ **shipped**
2. **Filtering + retire heuristic** — `card_profiles.oracle_tags` projection, otag filter fields,
   otags on served cards; rename `MechanicalCategory→CardRole`, trim `classify.rs→oracle_tag_gaps.rs`
   (keep 4 stragglers), derive the other 20 categories + Tokens from otags/`all_parts`.
   (Wire rename `mechanical_categories→card_roles` is version-gated Phase M.)
3. **Deck otags** — `decks.oracle_tags` + archetype→otag seeding + searchable picker.
4. **Serving** — one small `W_ORACLE_TAG` correlation term in the ranking query.
5. **Signal collection** — generalized-context per-otag signal, shipped dark.
6. **Non-EDH serving** — deferred; serve on the accrued dataset once it matures.

Land 1-4 on Commander first (that is where the data and usage are); 5-6 accrue over time.
**Every phase is additive — no `MIN_CLIENT_VERSION` bump required.**
