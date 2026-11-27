# Server

Files installed on the production VPS, and the database roles it runs as.

- `systemd/` holds the units: `zerver.service` (the API), `zervice.service` and `zervice.timer` (the nightly Scryfall sync, 04:00 UTC), and `zervice-alert.service` with `zervice-alert.sh`, which emails when a nightly run fails.
- `sql/zervice_role.sql` grants the `zervice` role what the sync needs, including ownership of the materialized views it refreshes. Run it again after a migration recreates one of them.

## Synergy worker role (prototype)

Not used yet: the synergy worker hasn't run anywhere, so this is a prototype kept for when it does.

`sql/synergy_worker_role.sh` creates the least-privilege Postgres role the synergy worker (separate private service, see `zwipe-context plans/synergy-data-layer.md`) connects as. Run it once per database, after the synergy tables migration.

```bash
# prod (ubuntu server; uses sudo -u postgres automatically)
zcripts/server/sql/synergy_worker_role.sh                 # database defaults to zwipe

# dev (mac; current user is the superuser)
zcripts/server/sql/synergy_worker_role.sh zerver          # or whatever your dev db is named
```

It prints the worker's `DATABASE_URL` once; copy it into the worker's `.env`. Re-running is safe: regenerates the password and reconverges the grants.

### What the role can and cannot do

| Table | Privileges | Why |
|---|---|---|
| `synergy_requests` | SELECT, INSERT, UPDATE, DELETE | its own work queue |
| `commander_synergy` | SELECT, INSERT, UPDATE | the cache it fills (upsert); no delete |
| `decks` | SELECT | demand discovery |
| `scryfall_data` | SELECT | commander identity resolution |
| everything else | None | denied, including CREATE on the schema |

Verified by test (2026-06-11, dev): all four allowed paths work; reads on `users`, writes on `decks`, deletes on `commander_synergy`, and `CREATE TABLE` all fail with permission errors.

How they're installed is in `zwipe-context operations/infrastructure/server.md`.
