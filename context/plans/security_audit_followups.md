# Security audit follow-ups

**Status: IN PROGRESS.** Tick items as they land. The audit itself (2026-10-10) and its first fourteen fixes merged as PR #28; this is what it left open, ordered by what to do first.

- [x] 2026-10-10 Server hand step: `zervice-alert.service` reinstalled as the `zervice-alert` user (which already existed), `alert.env` now `root:zervice-alert 640`; manual start exited 0 and the test email arrived.
- [x] 2026-10-10 Server hand step: `~/scripts/backup-db.sh` replaced with the script in `operations/infrastructure/backups.md` (private dump dir, password through `PGPASSWORD`); a hand run uploaded `zwipe-20261010.sql.gz`.
- [ ] Ranking signal poisoning: validate and cap the commander signals (below).
- [ ] Replaced-row retention keyed on `replaced_at`, not the token's own expiry (below).
- [ ] Follow-up migration: drop `refresh_tokens.revoked` + its index and the three `users` lockout columns. Tracked in `access_token_window.md`; listed here so it is not forgotten when that plan closes.
- [ ] DML-only runtime role for zerver (below).
- [ ] Register and change-email stop confirming that an email is taken (below).
- [ ] Android: `session.json` through the Keystore. Own task; the backup opt-out shipped in PR #28, so the file no longer leaves the device, but it is still plaintext on it.
- [ ] Merge Dependabot's action-pin PRs as they arrive, reading the pin diff each time.
- [x] 2026-10-10 `RESEND_API_KEY` stays the same value in `zerver.env` and `alert.env` (owner decision); rotate both files together.

**One sentence:** close the audit findings that needed a query change, a schema change or a product decision, which PR #28 could not carry.

## Ranking signal poisoning

`POST /api/metrics/usage` takes per-`(commander, card)` tallies (`added`, `shown`, `maybe`, `removed`) and per-commander select counts. The ids are any UUID the client sends, each field is clamped only to `HttpUsageBatch::MAX_PER_FLUSH` (10,000), a flush carries up to 1,000 signals, and the route allows 12 flushes a minute. The pooled net rate feeds the synergy `ORDER BY` for every user of that commander (`outbound/sqlx/metrics/mod.rs`, `commander_card_signal` and `commander_select_signal`), so one account can push a card up or down for everyone, and can write ~12k garbage rows a minute into the aggregate tables.

Do:

- In the upsert, `JOIN latest_cards` (or `card_profiles`) on both oracle ids and drop signals whose ids are unknown. Unknown ids today still insert rows.
- A per-signal ceiling that matches what a phone can do in a 30-second flush window: `shown` at most 200, and `added + maybe + removed <= shown`. Put it next to `MAX_PER_FLUSH` in `zwipe-core/src/http/contracts/metrics.rs` with a test.
- A per-user daily contribution cap, so a looping client or a bad actor saturates at, say, 5,000 `shown` per commander per day. Easiest as a `(user_id, commander_oracle_id, day)` counter table the ingest reads before it applies; alternative is to drop the aggregate-only design and store per-user rows that the rollup sums, which is a bigger change.
- `cargo sqlx prepare --workspace` and commit `.sqlx/`; this is why it did not ship in PR #28.

## Replaced-row retention

`helpers.rs` and the nightly upkeep prune `refresh_tokens` rows with `expires_at < NOW()` whatever `replaced_at` says, and separately prune replaced rows older than `REPLACED_TOKEN_RETENTION_DAYS` (7). A token rotated on day 12 of its 14 therefore keeps its replay evidence for 2 days, not 7; after that a replay reads as `NotFound` and the family survives.

Do: in both prune queries, keep a replaced row until `replaced_at + 7 days` regardless of `expires_at`. One query change in each place, then `cargo sqlx prepare`. The test in `tests/auth_refresh_families.rs` can plant a replaced row with `expires_at` in the past and `replaced_at` an hour ago and assert the replay still kills the family.

## DML-only runtime role

`CREATE DATABASE zwipe OWNER zwipe` and zerver connects as that owner (`server.md`), so the running server can `DROP TABLE`. The migration runner needs the owner; the server does not.

Do:

- `zcripts/server/sql/zerver_role.sql`, idempotent like `zervice_role.sql`: `CREATE ROLE zerver_app LOGIN`, `GRANT USAGE ON SCHEMA public`, `GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public`, `GRANT USAGE, SELECT ON ALL SEQUENCES`, and `ALTER DEFAULT PRIVILEGES FOR ROLE zwipe IN SCHEMA public GRANT ... TO zerver_app` so tables created by later migrations are covered without a re-run.
- Read every `REFRESH MATERIALIZED VIEW` and `TRUNCATE` in `zerver/src` first: those need ownership or an explicit grant, and anything missed turns into a 500 at runtime.
- `zerver.env` `DATABASE_URL` switches to the new role; `migrate.env` keeps `zwipe`. Run it on the dev DB and the full integration suite against it before the server.

## Email enumeration on register and change-email

Login and forgot-password hide whether an account exists; register answers `422 "user with that username or email already exists"` and change-email the same on a duplicate. Rate limited (5 per 12 minutes per IP) but still an oracle.

Decide first: the usual fix is to answer register as if it succeeded and send "you already have an account, here is a reset link" to the address, which changes the register flow's UX (no inline error; the user finds out by email). Username collisions can stay inline, since usernames are public. If that is not wanted, leave it and record the decision here.

## Not doing

- Account lockout stays per address. Brute force spread across many addresses gets five guesses per address per half hour on each account, which the password policy makes impractical; a global per-account lock is exactly the denial of service that was removed.
- `MIN_CLIENT_VERSION` defaulting to `0.0.0` on a rebuilt box is documented in `server.md` already.
- `vendor/tao` is relabelled 0.34.9 while being 0.37.0 code, which hides 0.37 advisories from `cargo audit`. Known, documented in `vendor/tao/VENDOR.md`, and goes away when the vendoring does.
