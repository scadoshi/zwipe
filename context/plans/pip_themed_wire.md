# Pip style on the wire: `theme` to `themed`

**Status: Phase 4 merged (#492, 2026-11-09): the server writes `themed` and stores it. The settings request still takes `theme`, because every build through 1.18.16 sends it (the hotfix below). Phase 5, dropping that alias, waits on the floor reaching the first store build after 1.18.16.**

**One sentence:** the pip setting's first value is renamed from `theme` to `themed`; the database stores `themed` now, the wire keeps writing `theme` because 1.15.0 through 1.18.0 can only parse that, and once the client floor is past a release that reads both, the server writes `themed` and stops accepting `theme`.

## Why

The Pips button reads "Theme" beside "Classic", which names a thing (the theme) where its partner names a quality. "Themed" says what the option does: the mana symbols take the theme's colors. Scotty wants the name changed for real, not only on the button, so the code, the UI, the hints and the stored value all say Themed. The rename is free everywhere except the wire, where every shipped client holds the old name.

This is the same move as [`pip_color_wire.md`](pip_color_wire.md) (`natural` to `classic`), and it runs beside it: that plan's Phase 4 still waits, so the wire today writes `theme` and `natural`.

## What the wire looks like

`UserSettings` (`zwipe-core/src/domain/user/models/settings.rs`) carries `pip_style`. It rides in the session the server returns on login and refresh, in `GET` and `PATCH /api/user/settings`, and the app keeps it in the stored session. zwiper-web keeps the session, settings included, in localStorage under `zwiper-web.session`. zite keeps the visitor's choice in localStorage under `zwipe.pips` as the serde JSON of the value.

The setting shipped in 1.15.0. Every release from 1.15.0 through 1.18.0 parses `"theme"` for this value and fails on `"themed"`: 1.15.0 to 1.17.0 carry `PipStyle { Theme, Natural }` with `rename_all = "lowercase"`, and 1.17.1 to 1.18.0 carry `Theme` the same way beside `Classic` (`rename = "natural", alias = "classic"`). A session a client cannot parse is a broken sign-in, so the server must not write `"themed"` until 1.18.0 is below the floor.

Today (after Phase 1), unchanged from before it:

```json
{ "theme": "gruvbox", "dark_mode": true, "pip_style": "theme", ... }
```

End state:

```json
{ "theme": "gruvbox", "dark_mode": true, "pip_style": "themed", ... }
```

The settings' own `theme` field (the color theme's name) is a different field and does not change.

## The rule this follows

[`../development/api_evolution.md`](../development/api_evolution.md): reads go lenient first, the writer changes only once every reader understands the new value, and removing an old value is a wire change behind `MIN_CLIENT_VERSION`. The precedents are [`pip_color_wire.md`](pip_color_wire.md) and [`archive/sort_direction_wire.md`](archive/sort_direction_wire.md).

## Phase 1: rename everywhere but the wire, read both. BUILT on `nits/thinker-143`

- `PipStyle::Theme` is `PipStyle::Themed`. The Pips button in Settings (app and web) says Themed, the Pips hint says Themed, the toast reads "Pips in themed colors", and zite's toggle title says "Switch to themed pips". Themed draws no CSS class, as Theme drew none.
- `Themed` is `#[serde(rename = "theme", alias = "themed")]`: the wire writes `theme` and reads either. It stays the `#[default]`.
- `as_str` is `themed` (the database name) and `FromStr` reads `themed` or `theme`.
- Migration `20261107120000_pip_style_themed.sql` rewrites stored `theme` rows to `themed`, sets the column default to `themed` and narrows the CHECK to `('themed', 'classic')`.
- zite: `pip_store` reads the stored JSON through serde, so `"theme"` and `"themed"` both load; `index.html` only acts on the Classic value, and Themed draws nothing, so it needs no change. zwiper-web's stored session reads through the same serde.
- Tests: `themed_stores_as_themed_and_reads_either_name`, `every_settings_value_parses_as_shipped_clients_read_it` (a copy of the 1.15.0 to 1.18.0 enum reads every value the settings serialize to) and `the_wire_writes_the_name_shipped_clients_parse` in core; in zerver, `pip_style_setting_round_trips` checks that a `theme` PATCH lands, the row stores `themed` and the response says `theme`, and `settings_responses_parse_as_shipped_clients_read_them` parses every settings response with a copy of the 1.15.0 to 1.18.0 enum.

The field keeps its name, `pip_style`, on the wire and in the column.

## Phase 2: ship the client

The next cut after Phase 1 is deployed carries the new core, so the app reads both names. No client code beyond what Phase 1 changed.

Record the version here when it is cut: **client reads both from 1.18.1**.

## Phase 3: raise the floor and wait

When Phase 2 is live on both stores and rolled out, set `MIN_CLIENT_VERSION` to that version in `/etc/zwipe/zerver.env` and restart zerver. 1.15.0 through 1.18.0 get the update screen. Wait a few quiet days and watch `zcripts/metrics/errors.sql`.

This floor also covers [`pip_color_wire.md`](pip_color_wire.md)'s Phase 4, which waits on 1.17.1 or later; both Phase 4s can merge after it.

## Phase 4: `themed` only. Rebuilt on `server/pip-themed-only`

First built 2026-11-07 on `server/pip-themed` (`ad0cffd5a`), which fell 828 commits behind release. Rebuilt 2026-11-09 on `server/pip-themed-only`, with the floor at 1.18.1:

- `Themed` serializes as `themed` under `rename_all = "lowercase"` and stays the `#[default]`, so the wire writes `themed`. Deserializing keeps `theme` as a read-only alias, for client-side copies written under it (see Stored data below).
- `FromStr` drops the `theme` arm: it takes only the current names, which is what the database holds.
- `HttpUpdateSettings.pip_style` first deserialized through `FromStr`, so `PATCH` with `theme` was a 422. That broke settings changes on every live build: through 1.18.16 the client serializes `PipStyle` with Phase 1's serde, which writes `theme`, so the floor at 1.18.1 covered reading the new name but not writing it. Hotfix 2026-11-09: the request deserializes with plain serde again, which takes `theme` through the alias. Responses keep writing `themed`, which those builds read.
- Tests: `themed_stores_and_writes_as_themed`, `the_earlier_theme_name_no_longer_parses` and `a_stored_theme_still_reads_as_themed` in core; request-body tests in `http/contracts/user.rs` (takes `themed`, refuses `theme`); and the zerver round trip expects `themed` by default, PATCHes `themed`, expects `themed` back and a 422 for `theme`. The shipped-client parse tests model the floor client, 1.18.1, which reads either name for each value, so every settings response still parses for every supported client.

### Stored data

- Postgres: `user_settings.pip_style` holds no `theme`. Phase 1's migration `20261107120000_pip_style_themed.sql` rewrote every `theme` row to `themed`, set the default to `themed` and narrowed the CHECK to (`themed`, `classic`), which refuses the old name. zerver writes the column with `as_str`, the current names.
- Clients: every store that serializes `PipStyle` with serde wrote the wire name, `theme`, before this phase: zite's `zwipe.pips`, zwiper-web's stored session, the app's stored session and its pending settings. They keep reading through the alias and write `themed` on their next save; zite's `pip_store` rewrites the stored value as soon as it loads it. Themed draws no CSS class, so zite's `index.html` needs nothing. No sign-out and no lost choice.

Access token claims carry no settings (`user_id`, `username`, `email`, `exp`, `iat`), so tokens are not a concern.

## Phase 5: the request stops taking `theme`

Phase 2 shipped a client that reads `themed` but still writes `theme`: the first client that writes `themed` is the first store build cut after Phase 4 merged (the next one after 1.18.16), since it carries Phase 4's core. When that build is the floor, `HttpUpdateSettings.pip_style` can take the current names only again (deserialize through `FromStr`), and the zerver test expects a 422 for `theme`. The serde read alias for stored client copies stays until those copies have had a save; drop it in the same change only if a release has passed since.

## Verification

- Phase 1: the core and zerver tests above; `cargo clippy --workspace --all-targets -- -D warnings`.
- Phase 3: `GET https://api.zwipe.net/api/client/min-version` returns the new floor; errors canary quiet.
- Phase 4: the zerver test is the proof; deploy, then flip Pips on the live app and on zite.
