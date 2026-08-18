# Oracle tags (otags) — plan index

**Status (2026-07-25): BUILD PHASE COMPLETE.** Every buildable phase (0 through 5S, incl. both
adoption-gated sunsets) is shipped and live; only the data-gated **Phase 6** (serve on the
matured signal) and ongoing description authoring remain. Historical build log below.

**Original status (2026-07-12): Phases 2–5A PUSHED (deploying to prod).** Phase 1 (ingest) + Phase 2
(retirement, `oracle_tags` filter, `GET /api/card/oracle-tags`, server-grouped card roles →
oracle-tags drill-down) live since v1.6.0. **Phase 3 (deck-level otag selection), Phase 4
(serving term), and Phase 5 Slice A (generalized-context signal, dark) all shipped in the
2026-07-12 push** — Phase 3 A/B/C (`858fed10`, `b3a61b8a`, `dff8ce73`+polish), Phase 4
`W_ORACLE_TAG` serve term, Phase 5A `otag_context_signal` + rollup + additive `deck_id` wire.
Two additive migrations run on this deploy (`20260712040000_add_deck_oracle_tags`,
`20260712060000_create_otag_context_signal`); the Phase 5 rollup + retirement/grouping repopulate
on the next prod `zervice` run.

**▶ Update 2026-07-14: everything below is now PUSHED (server 1.7.0).** Also shipped this
batch: **Phase M fully sunset** (`mechanical_categories → card_roles`, incl. DB-column rename),
**Phase 5S dual-accept** (signal fully `deck_id`-driven server-side + legacy fallback; 1.7.0
client pushes `deck_id` only), the **oracle-tag dictionary**, and the **unified catalog cache**.
See [`../../progress/overview.md`](../../../progress/overview.md) top entry. History below:
- **Phase 5 Slice B (client) — DONE** (`e4b5a6a5`): `zwiper` populates `CardSignalDelta.deck_id`
  and emits for commander-less decks. Non-EDH signal now flows once shipped.
- **Phase 5 wire made lenient — DONE** (`ce35aa7f`): `CardSignalDelta.commander_oracle_id` is now
  `Option<Uuid>` (`#[serde(default)]`) — non-EDH decks omit it, EDH sends `Some`; the per-card
  commander tables skip commander-less signals (no nil pollution). Additive, no bump.
- **Phase 2 tail — `classify.rs` DELETED** (`ecfb9441`): retirement proven on prod (88,304 profiles
  categorized from otags; Cultivate=ramp+tutor, Swords=lifegain+removal). Server-internal only,
  client-compatible.
- **Phase M — type rename `MechanicalCategory → CardRole` + Step 1 dual-emit DONE** (`b11dc0cc`,
  `a00e5030`): responses emit `card_roles` beside `mechanical_categories`; criteria accept
  `card_roles_*` via serde alias. Additive, no bump.
- **Phase M Step 2 (client migration) DONE — built + green, NOT yet committed** (2026-07-12): read
  side (`zwiper`/`zite`/`zwipe-components` read `CardProfile.card_roles` — `card_info.rs`,
  `card_row.rs`, `deck_metrics.rs`, `group_cards.rs`) + send side (`criteria/mod.rs` flipped to
  `#[serde(rename = "card_roles_*", alias = "mechanical_categories_*")]`, so the wire now emits
  `card_roles_*` and still accepts the legacy key). `matches.rs` + `CardQueryBuilder` Rust names
  stay `mechanical_categories_*` (only the serde name is wire-visible). Sits in the working tree
  intermingled with two other concurrent efforts (mana-pip fix, education hints) — commit as-is.
- **Cleanup:** `DeckServeContext` struct replaced the 8-arg serve signature (`c67ad5a6`).

## Where we stand (2026-07-12) — the build phase is essentially DONE

**Everything buildable-now is built and shipped.** What remains is **data-gated**, not effort:

1. **Shipped.** Server + clients live since 1.7.0 (2026-07-14/15); the deck_id-only signal is
   accruing from every install.
2. **Adoption-gated sunsets — ALL DONE.** **Phase M Step 3** — ✅ **DONE 2026-07-14** (dropped
   `mechanical_categories` field/criteria, renamed the DB column to `card_roles`). **Phase 5S** —
   ✅ **fully DONE 2026-07-25**: steps 1+2 (2026-07-14), then step 3 behind the owner-set 1.7.0
   `MIN_CLIENT_VERSION` floor (2026-07-24) — legacy `commander_oracle_id` wire + server fallback +
   client commander resolution all removed; `deck_id` is the sole signal key.
3. **Data-gated payoff** — **Phase 6:** fold the otag-signal term into ranking + non-EDH serving on
   `(format, CI, otags)`. Needs months of accrued swipe volume ("REALLY drive serving").
4. **Tiny non-gated leftover — ✅ DONE 2026-07-27:**
   `otag_context_signal::rollup_refresh_computes_net_and_shown` posts a known
   batch, runs the zervice refresh, and asserts `net`/`shown`. Nothing
   non-gated remains.

So otags is now quiet and *waiting*: the only future work is the Phase 6 serving payoff once the
dataset has weight (description authoring COMPLETE 2026-08-06 — `../archive/otags/tag_descriptions_and_dictionary.md`).

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
phase (`../archive/otags/compatibility.md` §Naming, `sequencing.md` Phase M). `otag`/`otags` appears only as
informal prose shorthand in these docs; concrete identifiers are all spelled out.

## The files

Active (everything else is shipped and archived — see below):

| File | Owns |
|------|------|
| `sequencing.md` | The phased build + per-phase status — Phases 0-5S all ✅ DONE; Phase 6 is the open item |
| `moat.md` | The non-EDH cross-format dataset moat (the Phase 6 long game) |
| `payoff.md` | Immediate vs long-term payoff, honestly separated |
| `../archive/otags/tag_descriptions_and_dictionary.md` | Description authoring — DONE 2026-08-06, every populated tag covered (4,395) |

Archived to [`../archive/otags/`](./) (fully shipped, 2026-07-25 sweep):
`purpose.md`, `scope.md`, `compatibility.md`, `open-questions.md`, `dictionary_backend.md`,
`dictionary_client.md`, `hint_host.md`, `mapping_sweep_review.md`, `user_education.md`.
The related `../archive/catalog_session_cache.md` (catalog prefetch) also shipped.

## What changed on 2026-07-11

The original `otags.md` treated **data access as the critical open question** — otags
were assumed to live only behind Scryfall's undocumented Tagger GraphQL API. They now
ship as a standard bulk file (`Oracle Tags`, 17.2 MB, updated daily ~09:00 UTC, at
`data.scryfall.io/oracle-tags/...`). That collapses the highest-risk unknown into a
routine bulk ingest that mirrors our existing Scryfall sync. See `../archive/otags/purpose.md` §pipeline.

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
