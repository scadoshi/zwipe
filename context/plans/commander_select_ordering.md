# Commander select — true popularity base + fresh ordering

**Status: BUILT 2026-07-07, dev-verified end-to-end. Architecture evolved
during build (owner call): commander search became a FIRST-CLASS endpoint
(`POST /api/card/search/commanders`) instead of piggybacking the deck-aware
search — see "As-built" below. Server + migration ready to deploy (migration
first, then re-run `zcripts/synergy-worker/setup-role.sh` on prod, then the
worker's first sweep populates); the client leg (swipe_select → new
endpoint, sort pin removed) rides the next release.**

## As-built (2026-07-07)

- **Dedicated endpoint**, not the deck-aware search: `POST
  /api/card/search/commanders` (authed, shares the card-search governor
  budget). The create flow has no deck yet, so the shuffle seed is
  **`{user_id}:{date}`** — per-user-per-day, deck-independent, mirroring the
  synergy serve's `{deck_id}:{date}`.
- **Engine**: `search_scryfall_data_deck_aware` gained `commander_seed:
  Option<String>`. When set → commander-select mode: token/emblem exclusion
  always applies; popularity base + banding + wildcard apply when no
  explicit sort (explicit sorts stay exact). All other callers pass `None`,
  byte-identical behavior.
- **Stack**: repo `search_commanders(request, user_id)` → CardService +
  ErasedCardService + blanket impl → handler
  (`handlers/card/search_commanders.rs`) → route → core path helper
  (`search_commanders_route()`) → zwiper client
  (`client/card/search_commanders.rs`) → `swipe_select.rs` (both fetch
  sites; EdhrecRank pin + import removed; filter-dot check now
  `sort().is_some()`). All four SwipeSelect modes
  (Commander/Partner/Background/SignatureSpell) route through it.
- **One trap fixed during build**: `commander_popularity`'s `name`/
  `oracle_id` columns made shared filters ambiguous under a plain join —
  the join is an aliased subquery exposing only `pop_decks`
  (`POPULARITY_JOIN` const). Any future join to this table must alias
  likewise.
- **Dev E2E (2026-07-07, live 3,325-row table)**: page 1 = true popularity
  head (Edgar Markov, Y'shtola, Yuriko, Atraxa, Krenko; zero 99-staples),
  deterministic; The Ur-Dragon led page 2 (band-1 spillover — the
  consumption-aligned 24-per-page math, nothing skipped); wildcards at
  index 17 were deep cuts, different per page and per user; second user =
  same band pool, different order, different wildcard; explicit `Name` sort
  exact; partner mode popularity-ordered; plain `/api/card/search`
  unaffected; token exclusion verified at SQL level (0 leaked). Clippy
  clean, 108 zerver tests pass, sqlx offline check green.

**What this builds, in one sentence:** Zwipe-select stops serving the same
`edhrec_rank` list on every open — commanders get ordered by how many decks
they actually helm (a `commander_popularity` table), dealt through the same
bands-of-25 shuffle the 99-serve already uses.

**Ownership boundary:** this repo owns the table (canonical migration here,
same split as the synergy tables), how zerver serves from it, and the
fallback chain when it's empty or partial. The synergy worker owns how the
table gets populated — that plan lives in the worker's own repo, not here.

## Why (measured 2026-07-06, dev DB)

- **Staleness.** The select stack is identical for every user on every
  open. The band shuffle shipped in 1.3.2 never reaches it because the
  client pins an explicit sort (the known caveat in
  [`suggestion_signal.md`](suggestion_signal.md) "Where it runs").
- **`edhrec_rank` measures the wrong quantity.** It ranks a card by decks
  *containing* it, not decks it *helms*. Within the legendary-creature
  pool, the head is 99-staples (Syr Konrad 252, Etali 260, Ragavan 269,
  Toski 380, Sheoldred 463...) while the format's most-helmed commanders
  sit hundreds deep: Krenko 1097, Yuriko 2421, Atraxa 2447, **The
  Ur-Dragon — EDHREC's #1 commander — 2575, with 281 legendary creatures
  outranking it (band 12)**, Edgar Markov 3095. This is not a noisy base
  to augment; it's the wrong base. (Augmenting with our own deck counts
  can't fix it either: that data is sparse and circular — users picked
  from this exact polluted ordering.)

## 1. Data — `commander_popularity` table (owned here)

Migration in `zerver/migrations/` (canonical schema lives in this repo;
worker writes, zerver reads — the `commander_synergy` split):

```sql
-- Decks-helmed popularity per commander. Written by the synergy worker,
-- read by zerver's commander-select ordering. Empty table = zerver falls
-- back to edhrec_rank (see the ORDER BY chain in outbound/sqlx/card/mod.rs).
CREATE TABLE commander_popularity (
    oracle_id  uuid PRIMARY KEY,
    name       text NOT NULL,
    decks      bigint NOT NULL,
    fetched_at timestamptz NOT NULL DEFAULT now()
);
```

Contract zerver assumes: full-pool coverage is NOT guaranteed (absent rows
are normal — fallback handles them); `decks` is a comparable magnitude
within the table; rows refresh on the worker's cadence (weeks-stale is
fine, popularity moves slowly). Population mechanics, source, cadence, and
seeding: the worker repo's plan (`zynergy` → `context/plans/commander-popularity.md`).

**Read side keys on `oracle_id`, so name-resolution is entirely the worker's
problem (verified 2026-07-07 first live sweep).** The worker's name→oracle_id
step hit two systematic classes the "expect a handful of split cards"
estimate understated — DFC commanders (EDHREC names the front face,
`scryfall_data` stores `A // B`; ~582 legendary DFCs) and same-name tokens
(216 commander names have a token sharing the name under a different
`oracle_id`). Both are fixed worker-side (front-face fallback tier; exclude
`token`/`double_faced_token`/`emblem` layouts). We inherit none of it: the
join is on the resolved `oracle_id`, never on name. Partner/Background combo
rows (`A // B`, ~3,202) are correctly skipped — they can't map to one
`oracle_id`, and each partner is ranked individually, so every *selectable*
single-card commander is still covered.

**One consistency requirement §2 must honor:** the popularity table excludes
token/emblem `oracle_id`s, so zerver's commander candidate pool must exclude
the same layouts. Measured 2026-07-07: 37 `token`/`double_faced_token`
legendary-creature rows sit in `latest_cards` today and could be offered as
commanders — a pre-existing gap, but fold the `layout NOT IN
('token','double_faced_token','emblem')` filter into the candidate query when
building the branch so serving and popularity agree on what a commander is.

## 2. Server — third ORDER BY branch

`search_scryfall_data_deck_aware` (`outbound/sqlx/card/mod.rs` ~810): the
ORDER BY chain today is `explicit sort → synergy band shuffle → nothing`.
The final else is currently **no ORDER BY at all** (which is why the client
pins a sort). Fill it:

- FROM gains `LEFT JOIN commander_popularity pop ON pop.oracle_id =
  latest_cards.oracle_id` (only when this branch will be taken — same
  conditional-FROM pattern as the signal rollup join).
- Base ordering: `pop.decks DESC NULLS LAST, edhrec_rank ASC NULLS LAST,
  name ASC` — decks-helmed first; `edhrec_rank` is the fallback for
  commanders the table doesn't cover (and gives plain no-sort searches a
  sane default order instead of none). Empty table = pure fallback = the
  revert lever.
- Band shuffle on top, exactly the synergy branch's machinery: when
  `deck_id` is present, `row_number()` over the base ordering, bands of
  `BAND_SIZE`, in-band order `hashtext(COALESCE(oracle_id::text,'') ||
  seed)` with the same `deck_id:date` seed. No deck → pure base order.
  **Post-wildcard note (2026-07-07):** the wildcard slot landed and already
  extracted `push_score(qb, scores)` and a `pool` CTE, but neither is a drop-in
  here — `push_score` emits the synergy base+signal expression, whereas this
  branch bands over `pop.decks` (a different column, and no `synergy_scores` in
  this path). This is a **new terminal branch** in the ORDER BY chain (today
  `sort → wildcard → synergy → nothing`; fill the `nothing`), with its own
  inline banding over `pop.decks DESC NULLS LAST, edhrec_rank ASC NULLS LAST,
  name ASC`. If the row_number→/BAND_SIZE→hash wrapper is worth sharing across
  the synergy and popularity branches, extract it as a helper then — but don't
  route through `push_score`; the score expressions genuinely differ.
- Perf: the popularity join is a few-thousand-row PK lookup, noise next to
  the measured window-sort cost (~150 ms worst-case firehose, real pools
  far smaller).

`.sqlx` untouched (QueryBuilder runtime queries), but run the prepare check
anyway before pushing, per the deploy rules.

## 3. Client — stop pinning the default sort

`zwiper/src/lib/inbound/screens/deck/components/swipe_select.rs`: three
`set_sort(CardSortKey::EdhrecRank)` sites (~97, ~126, ~287) stop setting a
sort, and the non-default detection (~387, `sort() !=
Some(CardSortKey::EdhrecRank)`) becomes `sort().is_some()`. The user can
still pick Rank explicitly in the filter sheet — explicit sorts stay exact,
as everywhere.

Check the same pattern anywhere else a screen pins `EdhrecRank` as its
default (grep before building; the caveat in suggestion_signal mentioned
commander-select specifically).

## 4. Rollout + compatibility

1. **Migration deploys first** — table exists empty, nothing reads it.
2. **Worker populates it** (worker repo's plan) — verify row counts and
   sane heads (Ur-Dragon-class commanders on top) via read-only psql.
3. **Server branch deploys** — old clients still send the explicit sort,
   so nothing changes for them; the branch is dormant until a client omits
   the sort. Verify on dev with a hand-built sortless request.
4. **Client change rides 1.4.0** — select gets popularity-based, per-deck
   per-day shuffled ordering the day users update.

Ordering matters: banding without the popularity base would just shuffle
the wrong cards — band 1 dealing Toski and Ragavan in varying order is not
the fix. Revert lever at every layer: client re-pins the sort; server
branch degrades to edhrec_rank when the table is empty (`TRUNCATE
commander_popularity` is a valid emergency switch-off).

## 5. Testing

- Dev-server end-to-end before commit (house rule): two decks, same day →
  different select hands; same deck twice → stable; explicit Rank sort →
  exact old ordering; known-popular commanders lead band 1.
- Integration harness (once [`integration_tests/`](integration_tests/overview.md)
  exists): seeded `commander_popularity` rows outrank higher-edhrec_rank
  commanders; NULL-popularity falls back; band determinism per (deck, day).

## Later

- First-party select signal (which commanders get *selected* after being
  shown — no signal table captures this today). Now its own focused plan:
  [`commander_select_signal.md`](commander_select_signal.md). Primary payoff
  is real least-shown weighting for the wildcard deep-slice, which currently
  falls back to shuffle-only.
- Recency-windowed popularity (last year vs all time) if the worker ever
  provides it — schema would gain a column, ordering unchanged.
