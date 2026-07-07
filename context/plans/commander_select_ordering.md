# Commander select — true popularity base + fresh ordering

**Status: PLANNED (2026-07-06, revised same day: popularity base promoted
from optional upgrade to first leg). Server legs shippable once the table
has data; the client leg rides the next release (1.4.0 batch).**

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
  seed)` with the same `deck_id:date` seed. Extract the banding wrapper
  into a shared closure/helper rather than duplicating it — two branches
  now band, only their score expression differs. No deck → pure base order.
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

- Signal term for select (which commanders get *added* after being shown —
  `commander_card_signal` doesn't capture this today; separate decision).
- Recency-windowed popularity (last year vs all time) if the worker ever
  provides it — schema would gain a column, ordering unchanged.
