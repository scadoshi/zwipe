# Settings rename on the wire: `preferences` to `settings`

**Status: Phase 1 deployed to production, 2026-11-05. Phase 2 rides 1.18.0. Phase 3 waits for the floor.**

**One sentence:** every identifier, module, table and doc that said preference says setting, and the two places a shipped client can see the old name, the route `/api/user/preferences` and the login reply's `preferences` key, keep answering under both names until `MIN_CLIENT_VERSION` is 1.18.0.

## The rule this follows

[`../development/api_evolution.md`](../development/api_evolution.md): a renamed route or key is a wire change, so the server goes lenient first, the client switches once the server is deployed, and the old name goes only behind the version floor. The precedent is [`pip_color_wire.md`](pip_color_wire.md), which is in flight on the same floor.

## What a 1.17 client needs

- `GET` and `PATCH /api/user/preferences`, since `zwipe-client` 1.17 calls that path.
- A `preferences` key on the login and refresh reply, since 1.17's `Session` is a plain derive with that field and fails to parse without it.
- Its stored session, written with a `preferences` key, must still load after the update to 1.18.0, or the user is signed out.

## Phase 1: the server is lenient. ON MAIN

- `zwipe-core/src/http/paths.rs`: `SETTINGS_ROUTE` is `/api/user/settings`; `LEGACY_SETTINGS_ROUTE` is `/api/user/preferences`, and both have `Get` and `Patch` rows in `FIXED_ROUTES`, so `zerver/tests/route_coverage.rs` proves the legacy path answers 401 without a token.
- `zerver/src/lib/inbound/http/routes.rs`: `LEGACY_SETTINGS_SEGMENT` (`/preferences`) is routed to the same `fetch_settings` and `update_settings` handlers as `/settings`.
- `zwipe-core/src/http/contracts/auth.rs`: `HttpSession` is the login and refresh reply, the `Session` flattened plus the same settings under `preferences`. `zerver/src/lib/inbound/http/handlers/auth/authenticate_user.rs` and `refresh_session.rs` answer with it.
- `zwipe-core/src/domain/auth/models/session.rs`: `Session` serializes with `settings` only and deserializes through `SessionWire`, which takes `settings`, `preferences` or both. A serde alias alone would not do: serde refuses a document that carries a field under both its name and its alias, and the reply carries both.
- `zerver/migrations/20261104120000_rename_user_preferences_to_user_settings.sql` renames the table and its three named constraints, and `20261105120000_rename_user_settings_not_null_constraints.sql` renames the `user_preferences_*_not_null` constraints Postgres 18 creates for every NOT NULL column, by prefix so it runs on any version. The primary key's index follows its constraint. Columns keep their names.
- Tests: `legacy_settings_names_still_answer` in `zerver/tests/user_flows.rs`, and `serializes_settings_under_the_new_key_only`, `loads_a_session_stored_under_the_legacy_key`, `loads_a_reply_carrying_both_keys` and `refuses_a_session_with_neither_key` in `session.rs`.
- Telemetry keeps its stored names: `screen::PROFILE_SETTINGS` is still the string `profile_preferences`, and the Config screen's error ops are still `load_preferences` and `update_preferences`, so the usage rows stay one series.

Verify against the local server: run zerver on `:3000`, then `curl -s -X POST localhost:3000/api/auth/login -H 'content-type: application/json' -d '{"identifier":"<user>","password":"<pw>"}' | jq '{settings, preferences}'` shows the two keys equal, and `curl -s localhost:3000/api/user/preferences -H "authorization: Bearer <token>"` answers the same row as `/api/user/settings`. For a true 1.17 check, build zwiper at `6241449f7` (the 1.17.0 cut) from a throwaway clone with `BACKEND_URL=http://localhost:3000 SQLX_OFFLINE=true dx build --platform ios`, install it on the simulator, sign in, open Settings, flip Dark/Light and Pips, then update to the 1.18.0 build without uninstalling and confirm the session is still signed in.

Done 2026-11-04 for the client half: `zwipe-client` at `6241449f7` (1.17.0), run as a throwaway test from a `git archive` of that commit, logged in against this server, read the login reply, called `fetch_preferences` and `update_preferences` on `/api/user/preferences`, and every reply parsed with the 1.17.0 types. The simulator walk is still to do.

## Phase 2: 1.18.0 clients use the new names only

Nothing beyond what is on main: `zwipe-client` calls `SETTINGS_ROUTE`, and the app stores its session with a `settings` key. A session a 1.17 client stored on the phone or in the browser carries `preferences`, and 1.18.0 loads it through `SessionWire` (test `loads_a_session_stored_under_the_legacy_key`), so the update signs nobody out. `SessionWire` does this job instead of `#[serde(alias = "preferences")]`: serde ignores field attributes on a `try_from` type, and on the wire struct an alias rejects the login reply, which carries both keys. The access token's `UserClaims` hold only `user_id`, `username`, `email` and `exp`, so tokens issued before the deploy are untouched by the rename. Record the version here when it is cut: **client uses the new names from 1.18.0**.

Deploy order is server first, as always. The server on main already answers both names, so there is no window where a 1.18.0 build can meet a server that lacks `/api/user/settings`.

## Phase 3: the server narrows

When 1.18.0 is live on both stores and the rollout has settled, set `MIN_CLIENT_VERSION=1.18.0` in `/etc/zwipe/zerver.env` and restart zerver, then wait a few quiet days on `zcripts/metrics/errors.sql`. Then, in one commit:

- `zwipe-core/src/http/paths.rs`: delete `LEGACY_SETTINGS_ROUTE` and its two `FIXED_ROUTES` rows.
- `zerver/src/lib/inbound/http/routes.rs`: delete `LEGACY_SETTINGS_SEGMENT` and its `.route(...)`.
- `zwipe-core/src/http/contracts/auth.rs`: delete `HttpSession`; the two handlers answer `Json<Session>` again.
- `zwipe-core/src/domain/auth/models/session.rs`: delete `SessionWire`, `MissingSettings` and the `#[serde(try_from)]`, so `Session` is a plain derive again. A session a 1.17 app stored no longer loads, which is fine: 1.17 cannot sign in under the floor anyway.
- Delete the five tests named under Phase 1.
- Update this plan's status and move it to `archive/`.

Gate: `cargo +nightly fmt --check`, `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` with `DATABASE_URL`, `RUSTDOCFLAGS="-D warnings" SQLX_OFFLINE=true cargo doc --workspace --no-deps --lib`, and `grep -rn preferences --include='*.rs' . --exclude-dir=target --exclude-dir=vendor` hitting only the changelog entries and the telemetry strings.

## zwiper-web

The web crate `zwiper-web` lives only on `feat/web`, so this branch leaves it alone. It takes the rename when `feat/web` merges main: its `domain/preferences.rs`, its imports of `zwipe_core::domain::user::models::preferences` and its `zwipe_client` preferences calls move to the settings names then. It stores its session in `localStorage` through the same `Session`, so a stored web session with the old key still loads. Phases 1 to 3 hold for it unchanged.

## Other migrations on the same floor

- [`pip_color_wire.md`](pip_color_wire.md): Phase 3 raises the floor to a build that reads both pip names (1.17.1 or later; 1.18.0 qualifies), and Phase 4, built on `server/pip-color-classic`, writes `classic` only. Raising the floor to 1.18.0 for this plan satisfies that plan's Phase 3 too, so merge `server/pip-color-classic` in the same window as Phase 3 above.
- [`access_token_window.md`](access_token_window.md): one open item, Android's plaintext `session.json`, which is its own task and not tied to the floor.
- `zerver/migrations/`: nothing else pending. Every migration up to `20261105120000` is applied locally and deploys with the next zerver push.

## Release day, in order

1. Confirm main is green (Security audit, Deploy zerver, Deploy zite) so prod already answers both names.
2. Cut 1.18.0 with the changelog entry, and record the version under Phase 2 here.
3. Submit to both stores; nothing on the server changes on submission day.
4. Once both stores show 1.18.0 live and the rollout has settled, raise `MIN_CLIENT_VERSION` to 1.18.0 and restart zerver.
5. After a few quiet days, land Phase 3 above and pip Phase 4 together, deploy, and archive both plans.
