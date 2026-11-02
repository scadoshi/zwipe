# Pip color on the wire: `natural` to `classic`

**Status: Phase 1 built on main, uncommitted, 2026-11-02. Phases 2 to 4 not started.**

**One sentence:** the pip preference's second value is renamed from `natural` to `classic`; the database stores `classic` now, the wire keeps writing `natural` because 1.15.0 through 1.17.0 can only parse that, and once the client floor is past a release that reads both, the server writes `classic` and stops accepting `natural`.

## Why

"Natural" does not say what it means. The option is Magic's classic pip fills, the same on every theme, so the code, the UI and the hints now call it Classic, and the type is `PipColor` rather than `PipStyle` since color is all it changes. The rename is free everywhere except the wire, where a shipped client holds the old name.

## What the wire looks like

`UserPreferences` (`zwipe-core/src/domain/user/models/preferences.rs`) carries `pip_style`. It rides in the session the server returns on register, login and refresh, in the access token's claims, in `GET` and `PATCH /api/user/preferences`, and the app keeps it in the stored session. zite keeps the visitor's choice in localStorage under `zwipe.pips` as the serde JSON of the value.

The setting shipped in 1.15.0, and 1.15.0 through 1.17.0 carry `PipStyle { Theme, Natural }` with `rename_all = "lowercase"` and nothing else, so it parses `"theme"` and `"natural"` and fails on `"classic"`. A session it cannot parse is a broken sign-in, so the server must not write `"classic"` until 1.17.0 is below the floor.

Today (after Phase 1):

```json
{ "theme": "gruvbox", "dark_mode": true, "pip_style": "natural", ... }
```

End state:

```json
{ "theme": "gruvbox", "dark_mode": true, "pip_style": "classic", ... }
```

## The rule this follows

[`../development/api_evolution.md`](../development/api_evolution.md): reads go lenient first, the writer changes only once every reader understands the new value, and removing an old value is a wire change behind `MIN_CLIENT_VERSION`. The precedent is [`archive/sort_direction_wire.md`](archive/sort_direction_wire.md).

## Phase 1: rename everywhere but the wire, read both. BUILT

- `PipStyle` is `PipColor`, `Natural` is `Classic`, `ParsePipStyleError` is `ParsePipColorError`, `pip_style_class` is `pip_color_class`, the CSS class is `pips-classic`. The toast reads "Pips in classic colors", and the Pips button, the zite toggle and both Pips hints say Classic.
- `Classic` is `#[serde(rename = "natural", alias = "classic")]`: the wire writes `natural` and reads either.
- `as_str` is `classic` (the database name) and `FromStr` reads `classic` or `natural`.
- Migration `20261102130000_pip_style_classic.sql` rewrites stored `natural` rows to `classic` and narrows the CHECK to `('theme', 'classic')`.
- zite's inline script in `index.html` adds `pips-classic` for a stored `"classic"` or `"natural"`.
- Tests: `classic_stores_as_classic_and_reads_either_name` and `the_wire_writes_the_name_shipped_clients_parse` in core, and `pip_style_preference_round_trips` in zerver, which checks that the row stores `classic`, a `natural` PATCH lands and the response says `natural`.

The field keeps its name, `pip_style`, on the wire and in the column. Renaming it is a separate wire change and not part of this plan.

## Phase 2: ship the client

The next cut after Phase 1 is deployed carries the new core, so the app reads both names. No client code beyond what Phase 1 changed.

Record the version here when it is cut: **client reads both from 1.17.1**.

## Phase 3: raise the floor and wait

When Phase 2 is live on both stores and rolled out, set `MIN_CLIENT_VERSION` to that version in `/etc/zwipe/zerver.env` and restart zerver. 1.15.0 through 1.17.0 get the update screen. Wait a few quiet days and watch `zcripts/metrics/errors.sql`.

## Phase 4: the server writes `classic` only

A short branch, merged after Phase 3 is quiet:

- `Classic` becomes plain `Classic` under `rename_all = "lowercase"`: no rename, no alias. The wire writes and reads `classic`.
- `FromStr` drops the `natural` arm. The database has held only `classic` since Phase 1, so the column needs no migration.
- zite: a visitor whose localStorage still holds `"natural"` would fall back to Theme. Either map it in `pip_store::load` before parsing or accept the reset; it is one preference on one site. Drop the `"natural"` check in `index.html` either way.
- Tests: `classic_stores_as_classic_and_reads_either_name` becomes a check that `natural` no longer parses, `the_wire_writes_the_name_shipped_clients_parse` asserts `"classic"`, and the zerver test PATCHes `classic`, expects `classic` back and expects `natural` to be a 4xx.
- Grep gate: `natural` appears nowhere in the pip code, the migration history aside.

Access token claims carry no preferences (`user_id`, `username`, `email`, `exp`, `iat`, checked 2026-11-02), so tokens are not a concern. The device is: the app stores the session, preferences included, and a build from Phase 2 on writes `"natural"` there. The server dropping `natural` is safe because it never reads that file, but a later app build on a core without the alias would fail to read a session stored before it. So Phase 4 removes `natural` from what the server accepts, not from what the app can read back. Either keep the alias on the client's stored-session path, or confirm that an unreadable stored session falls through to a refresh rather than a sign-out before dropping it.

## Verification

- Phase 1: the core and zerver tests above; `cargo clippy --workspace --all-targets -- -D warnings`. Done 2026-11-02: a local zerver took a `natural` PATCH and a `classic` PATCH, stored `classic` for both, and answered `natural` on PATCH, GET, login and refresh; every response parsed with the zwipe-core types pinned at 1.17.0 (`6241449f7`); `bogus` was a 422.
- Phase 3: `GET https://api.zwipe.net/api/client/min-version` returns the new floor; errors canary quiet.
- Phase 4: the zerver test is the proof; the grep gate holds; deploy, then flip Pips on the live app and on zite.
