# Phase 1: serve the Universes Beyond franchise list

**Status: DONE 2026-09-22, rides 1.10.2.**

`GET /api/card/ub-franchises` returns the franchises the exceptions picker offers, so a new crossover release becomes selectable on a deploy instead of a store train.

## Why this one first

It is the clearest case. The picker writes `user_preferences.universes_beyond_exceptions`, which is a server round trip, so a compiled list can only ever render checkboxes the user cannot save.

It is also the one with a standing chore attached. `universe.rs` is hand-maintained and grows with every crossover set, and `todo.md` carries a per-release top-up task. Detection already updates on a zerver deploy, since zerver compiles the same table for its SQL predicates. The picker was the only half still waiting on a client build.

## Built already

- `zwipe-core`: `UbFranchiseView { slug, name }` and `selectable_franchise_views()` in `domain/card/models/scryfall_data/universe.rs`, sorted by display name so the server decides the order once. The view deliberately drops `UbFranchise.sets`: set codes drive server predicates and have no business on the wire.
- `zwipe-core`: `GET_UB_FRANCHISES_ROUTE` in `http/paths.rs` (added to the route-pinning test) and a one-line `public_get!` `GetUbFranchises` in `http/endpoints/card.rs`.
- `zerver`: `handlers/card/get_ub_franchises.rs`, built from the compiled table with no DB read, mirroring `get_card_roles`. Routed at `/api/card/ub-franchises`.
- `zwipe-client`: `get_ub_franchises()`.
- `zwiper`: a `ub_franchises` slot on `CatalogCache`, an `ensure_ub_franchises`, and entries in `prefetch_public` and `any_public_failed`.

## Also done

- `screens/profile/universes_beyond.rs` renders from the cache slot. The local `sort_by_key` went with it, since the server sorts.
- Failure renders no chips and is reported by the catalog-failure toast, which already covers every public catalog.
- The `any_public_failed` comment says nine catalogs now.

## Notes from the build

- `universe.rs` had no `serde` import; the view type needed one.
- `UbFranchiseView` owns its strings where `UbFranchise` used `&'static str`, so the picker's slug comparisons needed adjusting.
- zwiper has zero reads of `selectable_franchises` or `FRANCHISES` now.
