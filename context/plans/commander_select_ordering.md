# Commander select — true popularity base + fresh ordering

**Status: PLANNED (2026-07-06). Side plan, small. Server + worker legs
shippable anytime; the client leg rides the next release (1.4.0 batch).**

**What this builds, in one sentence:** Zwipe-select stops serving the same
`edhrec_rank` list on every open — commanders get ordered by how many decks
they actually helm (a new popularity table the synergy worker keeps fresh),
dealt through the same bands-of-25 shuffle the 99-serve already uses.

**Why:**
- **Staleness.** The select stack is identical for every user on every
  open. The band shuffle shipped in 1.3.2 never reaches it because the
  client pins an explicit sort (the known caveat in
  [`suggestion_signal.md`](suggestion_signal.md) "Where it runs").
- **Wrong base.** `edhrec_rank` is a card's popularity *across all decks* —
  Sol Ring tops it. What select should rank by is popularity *as a
  commander* (decks helmed), which is a different number the current data
  doesn't hold.

## 1. Data — `commander_popularity` table

Migration (canonical schema lives in this repo, same split as the synergy
tables: worker writes, zerver reads):

```sql
CREATE TABLE commander_popularity (
    oracle_id  uuid PRIMARY KEY,
    name       text NOT NULL,
    decks      bigint NOT NULL,          -- decks helmed
    fetched_at timestamptz NOT NULL DEFAULT now()
);
```

**Worker leg (zynergy):** a new periodic fetch of the upstream commander
rankings, upserting the full table each cycle (small: a few thousand rows).
Fetch specifics stay in zynergy's own context docs, per the established
convention for upstream details. Cadence: weekly is plenty; popularity
moves slowly.

**Bootstrap option (zero new fetching):** the cached `commander_synergy`
payloads already imply a commander's helm count —
`max(potential_decks)` across its page is ≈ decks helmed. Coverage is only
commanders someone has already picked (~hundreds), so it's a seed, not the
source; the rankings fetch is what covers the whole legendary pool.

## 2. Server — third ORDER BY branch

`search_scryfall_data_deck_aware` (`outbound/sqlx/card/mod.rs` ~810): the
ORDER BY chain today is `explicit sort → synergy band shuffle → nothing`.
The final else is currently **no ORDER BY at all** (which is why the client
pins a sort). Fill it:

- FROM gains `LEFT JOIN commander_popularity pop ON pop.oracle_id =
  latest_cards.oracle_id` (only when this branch will be taken — same
  conditional-FROM pattern as the signal rollup join).
- Base ordering: `pop.decks DESC NULLS LAST, edhrec_rank ASC NULLS LAST,
  name ASC` — true commander popularity first, general-popularity fallback
  for anything the table doesn't cover (which also gives plain no-sort
  searches a sane default order instead of none).
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
default (grep before building; the add-screen caveat in suggestion_signal
mentioned commander-select specifically).

## 4. Rollout + compatibility

1. Migration + worker fetch deploy first — table fills, nothing reads it.
2. Server branch deploys — old clients still send the explicit sort, so
   nothing changes for them; the branch is dormant until a client omits
   the sort. Verify on dev with a hand-built sortless request.
3. Client change rides 1.4.0 — select gets popularity-based, per-deck
   per-day shuffled ordering the day users update.

Revert lever at every layer: client re-pins the sort; server branch falls
back to edhrec_rank when the table is empty.

## 5. Testing

- Dev-server end-to-end before commit (house rule): two decks, same day →
  different select hands; same deck twice → stable; explicit Rank sort →
  exact old ordering.
- Integration harness (once [`integration_tests/`](integration_tests/overview.md)
  exists): seeded `commander_popularity` rows outrank higher-edhrec_rank
  commanders; NULL-popularity falls back; band determinism per (deck, day).

## Later

- Signal term for select (which commanders get *added* after being shown —
  `commander_card_signal` doesn't capture this today; separate decision).
- Recency windows (popularity over the last year vs all time) if the
  upstream exposes them cleanly.
