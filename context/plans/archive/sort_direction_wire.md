# Sort direction on the wire: `ascending: bool` to `direction`

**Status: DONE 2026-10-23. Phase 1 `494d2217`, Phase 2 cut as 1.13.0, Phase 3 the 1.13.0 floor, Phase 4 `893a1bb4`; a sort flipped in the live app afterwards.**

**One sentence:** `CardQuery` sends the sort direction as `"ascending": true|false` today; move it to `"direction": "ascending"|"descending"` by sending both for one release train, reading the new one first on the server, and dropping the bool once the client floor is past the release that sends it.

## Why

The code now carries the direction as `SortDirection { Ascending, Descending }` everywhere but the JSON, where `CardQuery` still writes the bool it always has. The bool is a leftover from before the enum existed: a reader of the JSON has to know that `ascending: false` means descending, and a client in another language would model it as a flag rather than a choice. The enum serialized by name says what it is. It is a small win, so it rides the normal release train and never gets its own.

## What the wire looks like

`CardQuery` (`zwipe-core/src/domain/card/models/search_card/card_filter/query.rs`) is the body of `POST` search requests: `search_card`, `search_commanders` and `search_deck_cards` in zerver take `Json<CardQuery>`. The app builds it from `CardQueryBuilder` and sends it through zwipe-client. zite also builds on zwipe-client; it deploys with the server on every push to main, so it is never behind.

Today:

```json
{ "order_by": "name", "ascending": false, "synergy": false, "include_skipped": false, ... }
```

End state:

```json
{ "order_by": "name", "direction": "descending", "synergy": false, "include_skipped": false, ... }
```

Absent `direction` means ascending, which is also what absent `ascending` means today (`default_ascending`), so the default never moves.

## The rule this follows

[`../development/api_evolution.md`](../../development/api_evolution.md): new request fields are additive and defaulted, server first, no gate. Removing a field the server still reads is a wire change and needs the propagation wait plus the `MIN_CLIENT_VERSION` gate. So this is one additive step, one client release, one gated cleanup. The precedent is the `mechanical_categories` to `card_roles` sunset in [`archive/server_driven_catalogs.md`](server_driven_catalogs.md) and the five-phase train in [`archive/patch_idempotent_updates.md`](patch_idempotent_updates.md).

## Phase 0: the enum, wire untouched. DONE

On main: `SortDirection` lives in zwipe-core beside `CardSortKey`. The builder holds `direction`, `Cards::sorted` and `sort_deck_entries` take it, the sort sheet flips it, and `CardQuery::new` takes it and stores `ascending: direction.is_ascending()`. The struct field and its serde attributes are unchanged, and a test pins `"ascending":true` and `"ascending":false` in the serialized body.

## Phase 1: core and server send and accept both. DONE

One commit in zwipe-core and zerver, deployed before any client build carries it.

- `SortDirection` gains `Serialize` and `Deserialize` with `#[serde(rename_all = "lowercase")]`, so it reads and writes as `"ascending"` and `"descending"`.
- `CardQuery` gains `#[serde(default)] direction: Option<SortDirection>` and keeps `#[serde(default = "default_ascending")] ascending: bool`. `new` sets both from the one `SortDirection` it is given, so a client on this core always sends both fields.
- The getter `direction()` resolves `self.direction.unwrap_or_else(|| SortDirection::from_ascending(self.ascending))`: the new field wins when present, the bool fills in for older clients. The `ascending()` getter is deleted here; zerver's `ASC`/`DESC` push matches on `direction()`.
- Tests in zwipe-core, one per shape the server can receive: both fields, `ascending` only (a pre-Phase-2 client), `direction` only (the end state), neither (the default), and the two disagreeing (`direction` wins). A zerver integration test (`search_reads_direction_from_either_wire_field`) posts the `ascending` only and `direction` only descending shapes to `search_card` and asserts the same first card, then the `direction: ascending` shape and asserts a different one.
- Nothing persists `CardQueryBuilder` or `CardQuery` on the device (confirmed 2026-10-22), so the server is the only reader and no second one is needed.

Old clients are untouched: they keep sending `ascending` alone and the fallback reads it. Deploy order is server first, as always.

## Phase 2: ship the client

The next cut after Phase 1 is deployed carries the new core, so the app sends both fields. No client code changes beyond the pin. Nothing to do in the stores beyond the normal release.

Record the version here when it is cut: **client sends both from 1.13.0** (iOS build 106, Android vc68, submitted 2026-10-22). Its Phase 3 floor is the same 1.13.0 as the email enumeration plans.

## Phase 3: raise the floor and wait

When Phase 2 is live on both stores and rolled out, set `MIN_CLIENT_VERSION` to that version in `/etc/zwipe/zerver.env` and restart zerver, the way [`access_token_window.md`](../access_token_window.md) did for 1.11.2. Any client below it gets the update screen and can no longer send a body without `direction`. Wait a few quiet days and watch `zcripts/metrics/errors.sql` for anything the gate caught.

## Phase 4: drop the bool. BUILT on `phase4-sort-direction`

Built 2026-10-23, not merged. Nothing uses `deny_unknown_fields`, so a body that still carries `ascending` parses and the bool is ignored; a core test (`a_stray_ascending_bool_is_ignored`) and the renamed zerver test (`search_reads_direction_by_name`) pin that. `CardQueryBuilder` now serializes its field as plain `direction` too, since nothing persists the builder, so the `as_ascending` adapter went with the bool.

One commit, after Phase 3 has been quiet:

- `CardQuery`: delete `ascending` and `default_ascending`; `direction` becomes `#[serde(default)] direction: SortDirection` (absent still means ascending, via the enum's `#[default]`).
- `SortDirection`: delete `from_ascending` and `is_ascending`; nothing needs the bool any more.
- Delete the `ascending` only and both-fields serde tests and the zerver test's `ascending` arm; keep the `direction` only and absent cases.
- Grep gate: `ascending` appears only in `SortDirection::Ascending`, its lowercase wire form, and the sort sheet's label.

The zwipe-core module doc for `card_filter` lists the wire fields (`order_by` / `ascending` / `synergy`); update it in Phase 1 to mention both and in Phase 4 to name `direction` alone.

## Verification

- Phase 1: the four-shape serde tests and the zerver integration test above; `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`; a 1.12.x client against the deployed server still sorts descending when asked to.
- Phase 3: `GET https://api.zwipe.net/api/client/min-version` returns the new floor; errors canary quiet.
- Phase 4: the `direction` only test is the proof; the grep gate holds; deploy, then a sort flip on the live app.
