# Synergy worker database role

`setup_role.sh` creates the least-privilege Postgres role the synergy worker
(separate private service, see `context/plans/synergy-data-layer.md`) connects
as. Run it once per database, after the synergy tables migration.

```bash
# prod (ubuntu server; uses sudo -u postgres automatically)
./setup_role.sh                 # database defaults to zwipe

# dev (mac; current user is the superuser)
./setup_role.sh zerver          # or whatever your dev db is named
```

It prints the worker's `DATABASE_URL` once — copy it into the worker's `.env`.
Re-running is safe: regenerates the password and reconverges the grants.

## What the role can and cannot do

| Table | Privileges | Why |
|---|---|---|
| `synergy_requests` | SELECT, INSERT, UPDATE, DELETE | its own work queue |
| `commander_synergy` | SELECT, INSERT, UPDATE | the cache it fills (upsert); no delete |
| `decks` | SELECT | demand discovery |
| `scryfall_data` | SELECT | commander identity resolution |
| everything else | — | denied, including CREATE on the schema |

Verified by test (2026-06-11, dev): all four allowed paths work; reads on
`users`, writes on `decks`, deletes on `commander_synergy`, and `CREATE TABLE`
all fail with permission errors.
