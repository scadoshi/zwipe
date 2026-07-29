# Plan: session-prune drive-by + zervice least privilege

**Status: COMPLETE — both phases SHIPPED + verified in prod (2026-07-29),
plan archived.** Phase 1: systemd-path run 4/4 ok in 47s off `.env.zervice`,
alert chain live-tested twice. Phase 2: `zervice_role.sql` applied — boundary
verified as the role (catalog readable, all 3 matviews owned, `users`/
`refresh_tokens`/`decks` DENIED), then a full sync green as the scoped role.
Decided: zerver stays fully permissive as the `zwipe` owner role — the app-role
split is a deliberate non-goal for now (see "Zerver app-role split" in
`progress/backlog.md` for the rationale and what it would take).
**Addendum 2026-07-29 (later same day):** the nightly session sweep RETURNS
via `UpkeepService` in `plans/client-error-reporting.md` — belt-and-suspenders
over the insert-time drive-by (dormant users' expired rows), with a surgical
grant (`DELETE` + column-scoped `SELECT (expires_at)` on `refresh_tokens`) so
zervice still cannot read session data. AuthService stays out of zervice.

Two intertwined decisions from the systemd migration night (owner + assistant):

1. Session cleanup moves fully into zerver as a drive-by; zervice loses its
   only AuthService use and with it every auth/email capability.
2. Zervice gets a real privilege boundary: its own minimal config, its own
   env file, and (phase 2) its own Postgres role scoped to card-sync tables.

Context: the 5-per-user refresh-token cap is already enforced at insert
(`enforce_refresh_token_max`, `outbound/sqlx/auth/helpers.rs:68`), so the
sessions table is hard-bounded regardless of any sweeper. The nightly global
prune only uniquely covers dormant users' expired rows (≤5 each, never
scanned by auth). Redundant once the drive-by also purges expired.

## Phase 1 — drive-by prune + shed AuthService (one PR, server-only)

- **Extend the insert-time hygiene**: `enforce_refresh_token_max` becomes a
  single per-user delete covering `expires_at < NOW()` OR beyond-cap (rename
  to something like `prune_users_refresh_tokens`). Same call site, one
  statement, indexed by `user_id`.
- **zervice**: remove step 5 (prune) and the whole `AuthService`/`Resend`
  construction — zervice becomes CardService-only. Steps renumber `N/5` →
  `N/4` (log labels + "all N steps ok" summary).
- **Ports**: drop `delete_expired_sessions` from AuthService/repo if nothing
  else calls it (verify; it was zervice-only at time of writing).
- **`ZerviceConfig`**: new minimal config for the bin reading ONLY
  `DATABASE_URL`, `LOG_DIR`, `RUST_LOG`. The full `Config::from_env` demands
  JWT/Resend/etc. — zervice should fail to start *without* asking for
  secrets it has no business holding.
- **Server**: new `/home/scadoshi/zwipe/.env.zervice` with those three vars;
  `zervice.service` unit's `EnvironmentFile=` points at it (one line; update
  the versioned copy in `zcripts/server/systemd/`). The alert script keeps
  reading the MAIN `.env` (it legitimately needs the Resend creds) — its
  unit's `EnvironmentFile=` stays as-is.
- **Tests**: extend the auth integration suite — insert a session while the
  user holds expired tokens → expired ones are gone; cap behavior unchanged.
- `resend-verifications-cli.md` is parked (caveat recorded in that plan);
  no coordination needed here.

## Phase 2 — scoped Postgres role (server-side, after phase 1 settles)

`CREATE ROLE zervice LOGIN` granted only what the sync touches:

- INSERT/UPDATE/DELETE + SELECT: `scryfall_data`, `card_profiles`,
  `oracle_tags`, `card_oracle_tags`, the otag grouping table, and
  `zervice_metrics`.
- SELECT only: whatever the derive/refresh queries read beyond those
  (enumerate from the repo methods zervice actually calls during
  implementation — don't guess grants).
- NO access at all: `users`, `refresh_tokens`, `decks`, `deck_cards`,
  signal tables, everything else. A compromised zervice reads zero user data.

**The wrinkle — materialized views**: `REFRESH MATERIALIZED VIEW` requires
*ownership* of the view, not a grant. zervice refreshes `latest_cards`,
`card_signal_rollup`, and `otag_context_signal_rollup`. Options:
  a. `ALTER MATERIALIZED VIEW ... OWNER TO zervice` for the three (zerver
     only ever reads them). Caveat: any future migration that drops/recreates
     a matview resets ownership to the migration user — the migration must
     re-ALTER, and that's easy to forget. Add a note to the migrations
     section of `server.md` if this option is chosen.
  b. `SECURITY DEFINER` wrapper functions owned by the main user that zervice
     is granted EXECUTE on. More moving parts; only worth it if (a)'s
     re-ALTER footgun bites.
Start with (a) + the doc note.

- `.env.zervice`'s `DATABASE_URL` switches to the new role. Rollback is
  trivial (point it back at the main URL).

## Explicitly kept

- zervice stays a separate scheduled binary (systemd timer, `Persistent=true`,
  `OnFailure=` Resend alert) — the process-model decision is settled.
- No separate zervice crate: the boundary that matters is runtime (what main
  constructs + what env it receives), both closed by phase 1. A crate split
  would still link the same lib and buys ceremony, not privilege.

## Verification

- Phase 1: `cargo test -p zwipe-core -p zerver` (incl. the new drive-by
  integration case), CI clippy command, a local `zervice` run showing 4/4
  steps ok with the minimal env file.
- Phase 2: on the server, `psql` as the zervice role: confirm card tables
  writable, `SELECT count(*) FROM users` DENIED; then one full timer-path
  run green end to end.
