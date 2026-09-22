# Phase 1: serve the Universes Beyond franchise list

**Status: PART-BUILT 2026-09-22, uncommitted. Everything below is done
except the screen swap.**

`GET /api/card/ub-franchises` returns the franchises the exceptions picker
offers, so a new crossover release becomes selectable on a deploy instead of
a store train.

## Why this one first

It is the clearest case. The picker writes
`user_preferences.universes_beyond_exceptions`, which is a server round trip,
so a compiled list can only ever render checkboxes the user cannot save.

It is also the one with a standing chore attached. `universe.rs` is
hand-maintained and grows with every crossover set, and `todo.md` carries a
per-release top-up task. Detection already updates on a zerver deploy, since
zerver compiles the same table for its SQL predicates. The picker was the
only half still waiting on a client build.

## Built already

- `zwipe-core`: `UbFranchiseView { slug, name }` and
  `selectable_franchise_views()` in
  `domain/card/models/scryfall_data/universe.rs`, sorted by display name so
  the server decides the order once. The view deliberately drops
  `UbFranchise.sets`: set codes drive server predicates and have no business
  on the wire.
- `zwipe-core`: `GET_UB_FRANCHISES_ROUTE` in `http/paths.rs` (added to the
  route-pinning test) and a one-line `public_get!` `GetUbFranchises` in
  `http/endpoints/card.rs`.
- `zerver`: `handlers/card/get_ub_franchises.rs`, built from the compiled
  table with no DB read, mirroring `get_card_roles`. Routed at
  `/api/card/ub-franchises`.
- `zwipe-client`: `get_ub_franchises()`.
- `zwiper`: a `ub_franchises` slot on `CatalogCache`, an
  `ensure_ub_franchises`, and entries in `prefetch_public` and
  `any_public_failed`.

## Left to do

1. `screens/profile/universes_beyond.rs` renders from the cache slot instead
   of `selectable_franchises()`. The chips already sort server-side, so the
   local `sort_by_key` goes away with it.
2. Failure renders no chips. The catalog-failure toast added 2026-09-22
   already fires for any public catalog, so this needs no second error
   surface, and inventing one would mean two ways to say the same thing.
3. The `any_public_failed` doc comment says "eight public catalogs". Nine.

## Watch for

The slot is fetched but unread until step 1 lands, which is dead code in the
meantime. Do not commit the phase in that state.

`UbFranchiseView` needed `serde` imported into `universe.rs`, which had none.
