# Pip color on the wire: `natural` to `classic`

**Status: Phase 1 shipped; clients read `classic` from 1.17.1. Phase 2 is the 1.18.16 store build, the first app that writes `classic`. Phase 3, the floor at 1.18.16, waits on that build being live in both stores. Phase 4 is half done: the server writes and stores `classic` (#491), and the settings request keeps taking `natural` (#494) until Phase 3.**

**One sentence:** the pip setting's second value is renamed from `natural` to `classic`; the database stores `classic` now, the wire keeps writing `natural` because 1.15.0 through 1.17.0 can only parse that, and once the client floor is past a release that reads both, the server writes `classic` and stops accepting `natural`.

## Why

"Natural" does not say what it means. The option is Magic's classic pip fills, the same on every theme, so the code, the UI and the hints now call it Classic, and the type is `PipColor` rather than `PipStyle` since color is all it changes. The rename is free everywhere except the wire, where a shipped client holds the old name.

## What the wire looks like

`UserSettings` (`zwipe-core/src/domain/user/models/settings.rs`) carries `pip_style`. It rides in the session the server returns on login and refresh, in `GET` and `PATCH /api/user/settings`, and the app keeps it in the stored session. zite keeps the visitor's choice in localStorage under `zwipe.pips` as the serde JSON of the value.

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

Note, 2026-11-06: the type went back to `PipStyle` (and `ParsePipStyleError`, `pip_style_class`): a style describes the scheme better than colors. The field, key and column were `pip_style` throughout, so no wire change came of it. Read the names below with that swap.

## Phase 1: rename everywhere but the wire, read both. BUILT

- `PipStyle` is `PipColor`, `Natural` is `Classic`, `ParsePipStyleError` is `ParsePipColorError`, `pip_style_class` is `pip_color_class`, the CSS class is `pips-classic`. The toast reads "Pips in classic colors", and the Pips button, the zite toggle and both Pips hints say Classic.
- `Classic` is `#[serde(rename = "natural", alias = "classic")]`: the wire writes `natural` and reads either.
- `as_str` is `classic` (the database name) and `FromStr` reads `classic` or `natural`.
- Migration `20261102130000_pip_style_classic.sql` rewrites stored `natural` rows to `classic` and narrows the CHECK to `('theme', 'classic')`.
- zite's inline script in `index.html` adds `pips-classic` for a stored `"classic"` or `"natural"`.
- Tests: `classic_stores_as_classic_and_reads_either_name` and `the_wire_writes_the_name_shipped_clients_parse` in core, and `pip_style_setting_round_trips` in zerver, which checks that the row stores `classic`, a `natural` PATCH lands and the response says `natural`.

The field keeps its name, `pip_style`, on the wire and in the column. Renaming it is a separate wire change and not part of this plan.

## Phase 2: a store build that writes `classic`

Reading the new name is not enough: a client also sends the setting back in `PATCH /api/user/settings`, serialized with its own core. Clients from 1.17.1 read `classic`, but every build through the 1.18.16 web cut still serializes `natural`, because that core's serde writes the wire name. The first app that writes `classic` is the **1.18.16 store build**, cut from release after #491 merged, which carries the core that serializes `classic`.

## Phase 3: raise the floor to 1.18.16

When the 1.18.16 store build is live in both stores and rolled out, set `MIN_CLIENT_VERSION` to 1.18.16 in `/etc/zwipe/zerver.env` and restart zerver. Every build that still writes `natural` then gets the update screen. Wait a few quiet days and watch `zcripts/metrics/errors.sql`.

The floor at 1.18.1 (2026-11-09) covered reading `classic`, not writing it. Treating it as Phase 3 is what let the request refuse `natural` early and broke settings changes on every live build (fixed by #494).

## Phase 4: `classic` only. Rebuilt on `server/pip-classic-only`

First built 2026-11-02 on `server/pip-color-classic` (`32428253f`), which fell 1,889 commits behind release. Rebuilt 2026-11-09 on `server/pip-classic-only`, with the floor at 1.18.1:

- `Classic` serializes as `classic` under `rename_all = "lowercase"`, so the wire writes `classic`. Deserializing keeps `natural` as a read-only alias, for client-side copies written under it (see Stored data below).
- `FromStr` drops the `natural` arm: it takes only the current names, which is what the database holds.
- The settings request keeps taking `natural` until Phase 3. #491 first made `HttpUpdateSettings.pip_style` deserialize through `FromStr`, which 422'd `natural` from every live build; #494 put plain serde back, which takes `natural` through the alias and answers `classic`. **Only after Phase 3**: deserialize the request through `FromStr` again, and the zerver test expects a 422 for `natural`. The serde read alias for stored client copies stays until those copies have had a save; drop it then only if a release has passed since.
- Tests: `classic_stores_and_writes_as_classic`, `the_earlier_natural_name_no_longer_parses` (`FromStr`) and `a_stored_natural_still_reads_as_classic` (serde) in core; request-body tests in `http/contracts/user.rs` (takes `classic`, takes `natural` as shipped clients send it, refuses an unknown name, absent or null is unchanged); and the zerver round trip PATCHes `classic` and expects `classic` back, and PATCHes `natural` and expects 200 with `classic` back and stored. The shipped-client parse tests model the floor client, 1.18.1, which reads either name for each value.

### Stored data

- Postgres: `user_settings.pip_style` holds no `natural`. Phase 1's migration `20261102130000_pip_style_classic.sql` rewrote every `natural` row to `classic`, and the CHECK (`themed`, `classic` since `20261107120000`) refuses the old name. zerver writes the column with `as_str`, the current names.
- Clients: every store that serializes `PipStyle` with serde wrote the wire name, `natural`, before this phase: zite's `zwipe.pips`, zwiper-web's stored session, the app's stored session and its pending settings. They keep reading through the alias and write `classic` on their next save. zite's `pip_store` rewrites a stored `natural` as soon as it loads it, and its `index.html` still honors `"natural"` before first paint. No sign-out and no lost choice.

Access token claims carry no settings (`user_id`, `username`, `email`, `exp`, `iat`, checked 2026-11-02), so tokens are not a concern.

## Verification

- Phase 1: the core and zerver tests above; `cargo clippy --workspace --all-targets -- -D warnings`. Done 2026-11-02: a local zerver took a `natural` PATCH and a `classic` PATCH, stored `classic` for both, and answered `natural` on PATCH, GET, login and refresh; every response parsed with the zwipe-core types pinned at 1.17.0 (`6241449f7`); `bogus` was a 422.
- Phase 3: `GET https://api.zwipe.net/api/client/min-version` returns the new floor; errors canary quiet.
- Phase 4: the zerver test is the proof; the grep gate holds; deploy, then flip Pips on the live app and on zite.
