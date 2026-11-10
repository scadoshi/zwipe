# Pip color on the wire: `natural` to `classic`

**Status: Phase 1 committed on main `c089dcc1f`, 2026-11-02, riding 1.17.1. Phases 2 and 3 done: the floor is 1.18.1 in production (2026-11-09). Phase 4 rebuilt on `server/pip-classic-only` from release, PR to release.**

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

## Phase 2: ship the client

The next cut after Phase 1 is deployed carries the new core, so the app reads both names. No client code beyond what Phase 1 changed.

Record the version here when it is cut: **client reads both from 1.17.1**.

## Phase 3: raise the floor and wait

When Phase 2 is live on both stores and rolled out, set `MIN_CLIENT_VERSION` to that version in `/etc/zwipe/zerver.env` and restart zerver. 1.15.0 through 1.17.0 get the update screen. Wait a few quiet days and watch `zcripts/metrics/errors.sql`.

## Phase 4: `classic` only. Rebuilt on `server/pip-classic-only`

First built 2026-11-02 on `server/pip-color-classic` (`32428253f`), which fell 1,889 commits behind release. Rebuilt 2026-11-09 on `server/pip-classic-only`, with the floor at 1.18.1:

- `Classic` is plain `Classic` under `rename_all = "lowercase"`, with no rename and no alias, so the wire writes and reads `classic`.
- `FromStr` drops the `natural` arm. The database has held only `classic` since Phase 1, so no migration.
- `PATCH` with `natural` is a 422.
- A session stored on a device before the update that still says `natural` no longer parses, and the app treats that as signed out (`zwiper/src/lib/outbound/session.rs`). That is accepted: those users sign in again, and nothing waits for old sessions to age out. A zite visitor whose localStorage says `"natural"` falls back to Theme, and `index.html` checks `"classic"` only.
- Tests: `classic_stores_and_writes_as_classic` and `the_earlier_natural_name_no_longer_reads` in core, three request-body tests in `http/contracts/user.rs`, and the zerver test PATCHes `classic`, expects `classic` back and expects `natural` to be a 422. The shipped-client parse tests now model the floor client, 1.18.1, which reads either name for each value, so every settings response still parses for every supported client.
- Grep gate holds: `natural` appears only in the tests that assert it is refused and in the floor client's enum.

Access token claims carry no settings (`user_id`, `username`, `email`, `exp`, `iat`, checked 2026-11-02), so tokens are not a concern.

## Verification

- Phase 1: the core and zerver tests above; `cargo clippy --workspace --all-targets -- -D warnings`. Done 2026-11-02: a local zerver took a `natural` PATCH and a `classic` PATCH, stored `classic` for both, and answered `natural` on PATCH, GET, login and refresh; every response parsed with the zwipe-core types pinned at 1.17.0 (`6241449f7`); `bogus` was a 422.
- Phase 3: `GET https://api.zwipe.net/api/client/min-version` returns the new floor; errors canary quiet.
- Phase 4: the zerver test is the proof; the grep gate holds; deploy, then flip Pips on the live app and on zite.
