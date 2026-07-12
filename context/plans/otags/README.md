# Oracle tags (otags) — plan index

**Status: Phases 2–5A PUSHED 2026-07-12 (deploying to prod).** Phase 1 (ingest) + Phase 2
(retirement, `oracle_tags` filter, `GET /api/card/oracle-tags`, server-grouped card roles →
oracle-tags drill-down) live since v1.6.0. **Phase 3 (deck-level otag selection), Phase 4
(serving term), and Phase 5 Slice A (generalized-context signal, dark) all shipped in the
2026-07-12 push** — Phase 3 A/B/C (`08b485eb`, `7690f984`, `789a0b70`+polish), Phase 4
`W_ORACLE_TAG` serve term, Phase 5A `otag_context_signal` + rollup + additive `deck_id` wire.
Two additive migrations run on this deploy (`20260712040000_add_deck_oracle_tags`,
`20260712060000_create_otag_context_signal`); the Phase 5 rollup + retirement/grouping repopulate
on the next prod `zervice` run.

**▶ Landed since the push (committed, UNPUSHED — ~14 commits ahead of `origin/main`):**
- **Phase 5 Slice B (client) — DONE** (`1a857e67`): `zwiper` populates `CardSignalDelta.deck_id`
  and emits for commander-less decks. Non-EDH signal now flows once shipped.
- **Phase 5 wire made lenient — DONE** (`77801be6`): `CardSignalDelta.commander_oracle_id` is now
  `Option<Uuid>` (`#[serde(default)]`) — non-EDH decks omit it, EDH sends `Some`; the per-card
  commander tables skip commander-less signals (no nil pollution). Additive, no bump.
- **Phase 2 tail — `classify.rs` DELETED** (`f8ed0e36`): retirement proven on prod (88,304 profiles
  categorized from otags; Cultivate=ramp+tutor, Swords=lifegain+removal). Server-internal only,
  client-compatible.
- **Phase M — type rename `MechanicalCategory → CardRole` + Step 1 dual-emit DONE** (`4455fd20`,
  `a20d56ca`): responses emit `card_roles` beside `mechanical_categories`; criteria accept
  `card_roles_*` via serde alias. Additive, no bump. See `sequencing.md` Phase M for **Step 2**
  (client migration) the next agent starts on.
- **Cleanup:** `DeckServeContext` struct replaced the 8-arg serve signature (`9f783745`).

**▶ Still open:**
- **Phase M Step 2** (clients read/send `card_roles`) then **Step 3** (gated sunset + DB column
  rename + `MIN_CLIENT_VERSION` bump). Precise file-level plan in `sequencing.md` §Phase M.
- **Phase 5 test gap:** no test refreshes `otag_context_signal_rollup` asserting `net`/`shown` yet.
- **Phase 5S (gated):** sunset the legacy `commander_oracle_id` wire once `deck_id` is guaranteed.
- **Phase 6:** non-EDH serving on the accrued dataset (deferred, data-hungry).

All 7 open questions resolved; Q1 revised after Phase 1 (otags supersede the heuristic).

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
2. **Filtering + retire heuristic** — ✅ **DONE + committed**: retirement (otag-derived categories
   + `oracle_tag_gaps`), `oracle_tags` filter, `GET /api/card/oracle-tags` endpoint, the otag
   filter picker, and the server-grouped **card roles → oracle-tags drill-down** + UI naming
   (Card roles / Oracle tags / Deck Tags). Full status + commits in `sequencing.md` Phase 2 §STATUS.
   **▶ Remaining tail:** `classify.rs` delete (after a prod zervice run proves the retirement), then
   the `CardRole` wire/DB rename + Phase M (display labels already say "Card roles").
3. **Deck otags** — `decks.oracle_tags` + archetype→otag seeding + searchable picker.
4. **Serving** — one small `W_ORACLE_TAG` correlation term in the ranking query.
5. **Signal collection** — generalized-context per-otag signal, shipped dark. **Slice A
   (server + wire) BUILT 2026-07-12, unpushed** (`otag_context_signal` + rollup, `deck_id` added
   additively to `CardSignalDelta`, credit loop keyed on the swiped card's otags by commander OR
   `(format, CI)`); Commander accrues from existing clients immediately, non-EDH waits on the
   Slice B client update. **Phase 5S** later sunsets the legacy `commander_oracle_id` wire field
   once `deck_id` is guaranteed — the **first `MIN_CLIENT_VERSION` gate** the feature needs.
6. **Non-EDH serving** — deferred; serve on the accrued dataset once it matures.

Land 1-4 on Commander first (that is where the data and usage are); 5-6 accrue over time.
**Every phase is additive — no `MIN_CLIENT_VERSION` bump required.**
