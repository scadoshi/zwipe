# In-universe filter + printing-aware set filters

**Status: SHIPPED end to end 2026-09-01 — a one-day plan-to-prod run. Server:
toast `75239e02`, const/table/overlay `6dc30b48`, view rebuild `b443b054`,
printing-aware set predicates `f4dcd650`, preference pipeline `98d67e41`.
Client (step 5 grew into a global preference, owner call): Profile Show/Hide
row + Exceptions chip sheet with the Secret Lair catch-all, `a31819f1`, shipped
as 1.10.0 (build 78 / vc41, both stores, submitted same day). Prod deploy
verified: migrations applied, `zervice_role.sql` re-run as postgres, manual
zervice run printed `oou sets overlay: 71 wholly-UB set codes`. Device smoke
test caught one hole — the deck form's typed commander picker rode the plain
search and bypassed the preference — fixed server-side `bd167f6b` (service
fetches the preference; HTTP regression test) and re-verified on device.
Build notes worth keeping: the view remap snapshots old picks and moves only
rows whose pick changed (printing-sheet choices survive, unlike the
06-06/08-12 remaps); 719 picks flipped in-universe; Sol Ring's pick
legitimately stays SLD (oval in-universe art) while its 74-set array fixes the
exclusion bug. Standing chore: top up `universe::FRANCHISES` per UB release.**

**One sentence:** set filters currently run against the one printing
`latest_cards` happens to pick, so excluding a set can hide a card entirely
(Sol Ring's pick is a Secret Lair Drop row today); fix by precomputing each
card's full set membership as an array, teach the view's pick to prefer
in-universe printings, then an "exclude Universes Beyond" filter falls out
almost for free.

## Origin

- Ebany, Discord 2026-08-23: set-include "Universes Within" returns nothing
  (that part is actually Synergy intersecting the set down to zero — see the
  toast item at the bottom); exclude leaves UB/Secret Lair cards visible; asks
  for a UB filter, sticky filters, earliest-print preference. Owner verdict:
  build the universe filter (hand-maintained list accepted), fix the silent
  zero-results UX, park sticky filters until the empty-state story exists,
  decline the earliest-print setting (the pick preference below addresses the
  real complaint).
- Owner-spotted design flaw, same conversation: once default/sticky filters
  exist, pick-based set excludes hide staples forever. Verified live: `SELECT
  set_name, released_at FROM latest_cards WHERE name = 'Sol Ring'` → Secret
  Lair Drop, 2026-09-14. Excluding SLD today removes Sol Ring from search.

## Precedent

This is the third printing-shadowing bug, same shape as the two solved ones:
`20260606120000_latest_cards_prefer_real_printings.sql` (MTGA/promo picks) and
`20260812210000_latest_cards_prefer_english.sql` (foreign picks made 268 cards
vanish under the `lang=en` default). Both were one new ORDER BY tiebreaker plus
a remap pass repointing deck references at the new preferred printing ids. The
difference this time: no ORDER BY choice can beat an arbitrary exclusion list,
so filtering must stop consulting the pick at all — hence the array.

## Verified data (dev DB, 2026-09-01)

- `scryfall_data` holds 117,631 printings → 38,709 `latest_cards` rows
  (`DISTINCT ON (COALESCE(oracle_id, id))`).
- `security_stamp = 'triangle'` marks classic Universes Beyond: LTR, FIN/FIC/FCA,
  WHO (Doctor Who), PIP (Fallout), 40K, ACR (Assassin's Creed), and the UB
  Secret Lair drops (151 triangle rows inside `Secret Lair Drop`). 1,891
  triangle rows total.
- The stamp was retired for the newest tentpoles: **Marvel's Spider-Man (spm)
  and Avatar: The Last Airbender (tla, tle) carry oval/blank stamps.** So
  stamp-only detection leaks exactly the sets users complain about.
- Universes Within (slx) is 27 in-universe cards (oval), distinct names from
  their UB counterparts. Nothing special needed for them.

## Design

**OOU test (one expression, used everywhere):**
`security_stamp = 'triangle' OR set = ANY(<curated list>)`.

### 1. Curated OOU set list — the hand-maintained piece

- A const in **zwipe-core** (both sides need it: server predicates and client
  in-memory parity). Set *codes*, not names, with a comment per entry. Seed:
  `spm` + its token/art satellites, `tla`, `tle` + satellites — audit the full
  set list once at build time for other stampless UB (art series, minigames,
  front-cards satellites of the stamped sets are blank-stamped too; decide
  whether satellites matter, they mostly hold non-playable rows).
- zervice sync overlays the const into a small `oou_sets(code)` table (same
  pattern as `ORACLE_TAG_DESCRIPTIONS`), and the view reads that table at
  REFRESH. Adding a set = edit the const, deploy, nightly refresh re-picks. No
  migration per addition.
- **Maintenance chore (owner accepts):** on each UB set release, check its
  stamp; stampless → add the code. A few times a year, agent-automatable.

### 2. View rebuild (one migration, server-only)

- Add per-oracle set membership: join a `GROUP BY COALESCE(oracle_id, id)`
  aggregate of `scryfall_data` and expose `printing_set_names TEXT[]` (names,
  because the existing filter wire and `get_sets` vocabulary are `set_name`
  strings). GIN index on it.
- New ORDER BY tiebreaker after `lang`, before `released_at DESC`:
  `(sd.security_stamp = 'triangle' OR sd.set IN (SELECT code FROM oou_sets)) ASC`
  → in-universe printings win whenever one exists; UB-only cards tie and
  behave as today.
- Full remap pass (deck_cards + the four command-zone id columns), copied from
  the prefer_english migration.
- Consequence worth stating: after this, a card whose *pick* is OOU provably
  has no in-universe printing — the pick becomes a card-level truth the OOU
  filter can read directly.

### 3. Set filter predicates (server-only, `zerver/.../sqlx/card/mod.rs`)

- include: `set_name = ANY($sets)` → `printing_set_names && $sets`
  ("has a printing in") — also just better include semantics.
- exclude: `NOT (set_name = ANY($sets))` → `NOT (printing_set_names <@ $sets)`
  (card survives unless every printing it has is excluded). Sol Ring becomes
  un-hideable; SLD-only cards still correctly disappear.
- `get_sets` stays as-is: with "has a printing in" semantics, offering the full
  vocabulary is now correct rather than a trap.

### 4. OOU filter chip (client + core + server; needs a client release)

- New criterion in zwipe-core (builder setter/getter + criteria field), shape
  TBD: likely a single bool `exclude_universes_beyond` first; an include-only-UB
  variant can wait for demand.
- Server predicate on the picked row: `NOT (security_stamp = 'triangle' OR set
  = ANY(...))` — valid card-level test only after step 2's ORDER BY lands.
- In-memory parity in `matches.rs` via the card's own stamp/set + the shared
  const — same caveat, valid because the wire card IS the pick.
- UI: chip in the set filter screen; user-facing name TBD ("Universes Beyond"
  reads clearer than "out of universe").

### Open questions

- **In-memory parity for the new set semantics** (`matches.rs`, used by the
  maybeboard client-side filter): the wire card carries one printing, not the
  array. Options: ship `printing_set_names` on the wire (fat), or accept the
  maybeboard filter stays pick-based (small owned list, minor divergence).
  Leaning accept-divergence with a comment.
- Satellite sets (art series, tokens, minigames) in the curated list — probably
  irrelevant since those rows rarely survive `is_playable`, but check once.
- Whether the changelog frames this as a fix (set filters) + feature (UB chip)
  in the same release.

## Build order

1. **Standalone quick fix, no dependency on the rest:** no-results toast in
   `add.rs` (search Ok branch, after the deck-dedup filter). One composed
   message: base "No results for this filter", plus a suffix when synergy is
   on: "; tap Synergy to turn it off" (the chip is on the add screen top
   right, not in the filter sheet), or "; Synergy is warming up, so all cards
   were searched" while warming (a warming search runs against the full pool,
   so emptiness there is the filter's own doing and the copy says so).
2. Curated const + zervice overlay + `oou_sets` table.
3. View migration (array + ORDER BY + remap) — deployable alone; fixes art
   picks immediately.
4. Set predicates to array semantics + `cargo sqlx prepare --workspace`.
5. OOU chip end to end (this is the piece that forces the client release; owner bumped the workspace to 1.10.0).

Steps 2–4 are server-side only and improve every existing client the moment
they deploy. Version discipline per todo: any zwiper/zwipe-core change (steps
2's const and 5) invalidates current store artifacts → next cut is 1.10.0,
build 78 / versionCode 41.
