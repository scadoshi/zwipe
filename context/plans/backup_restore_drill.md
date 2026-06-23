# Backup restore drill — prove the R2 dumps actually restore

**Status: written 2026-06-10, not yet run.** Nightly R2 backups have existed
since ~2026-03 (`operations/infrastructure/backups.md`) but a restore has never been tested.
"Backups exist" and "backups restore" are different facts — this drill turns
the first into the second. ~20 minutes, zero risk to prod (everything happens
in a scratch database), no VPS needed.

Re-run it occasionally (e.g. quarterly, or after changing Postgres versions,
the backup script, or rclone config). It is also Phase 0 step 6 of
`vps_migration.md` — if the migration happens first, that counts as the drill.

---

## What this proves

1. The R2 bucket actually contains recent, non-empty, non-corrupt dumps.
2. A dump restores cleanly into an empty database (no missing roles,
   extensions, or encoding surprises).
3. The restored data is *complete* — row counts and recent records match
   expectations, not just "psql exited 0".
4. You know the steps cold before doing them at 2am with prod down.

## Where to run it

Any machine with Postgres and rclone access — the Mac is ideal (dev Postgres
already set up via `zcripts/dev-env/macos/setup.sh`). Running it on the prod
server works too but use a scratch DB name; never touch the live `zwipe` DB.

---

## The drill

### 1. Fetch the latest dump from R2

```bash
# List what's there — confirm dumps are recent and plausibly sized (~5-10MB)
rclone ls r2:zwipe-backups/

# Pull the newest one
rclone copy r2:zwipe-backups/zwipe-YYYYMMDD.sql.gz /tmp/
gunzip /tmp/zwipe-YYYYMMDD.sql.gz
```

If the Mac has no rclone remote configured, either `apt`-style install + copy
the `[r2]` block from the server's `~/.config/rclone/rclone.conf`, or just
`scp` the dump from the server over Tailscale.

**Checkpoint:** newest dump is dated today-or-yesterday and is megabytes, not
bytes. A 20-byte dump means the backup script is silently broken — stop and
investigate (`/var/log/zwipe/backup.log` on the server).

### 2. Restore into a scratch database

```bash
createdb zwipe_drill   # or: psql -c "CREATE DATABASE zwipe_drill;"
psql -d zwipe_drill -f /tmp/zwipe-YYYYMMDD.sql 2>&1 | tee /tmp/restore-drill.log
grep -i "error" /tmp/restore-drill.log
```

**Checkpoint:** zero errors. Tolerable exceptions: "role does not exist"
noise if the dump references the server's `zwipe` role and the local Postgres
lacks it — fix by `createuser zwipe` first, or decide it's cosmetic (ownership
GRANTs only). Anything about missing extensions, failed COPY, or syntax is a
real failure.

### 3. Verify the data is actually there

```bash
psql -d zwipe_drill <<'SQL'
-- core tables exist and have plausible counts
select 'scryfall_data' t, count(*) from scryfall_data
union all select 'users', count(*) from users
union all select 'decks', count(*) from decks
union all select 'deck_cards', count(*) from deck_cards;

-- freshness: newest user and deck should be recent / recognizable
select email, created_at from users order by created_at desc limit 3;
select name, updated_at from decks order by updated_at desc limit 3;

-- migrations table came along (proves schema state restored, not just data)
select count(*) from _sqlx_migrations;
SQL
```

**Checkpoint:** ~35k+ scryfall rows, the real user count (~20 as of
2026-06), your own recent decks visible by name, and `_sqlx_migrations`
matching the count of files in `zerver/migrations/`.

### 3b. Compare against the live server — exact, not eyeball

Counting rows on both sides naively never matches: the dump is from 5am and
prod has moved since. **Bound both sides at the dump's cutoff time** and the
comparison becomes exact. Save this as `/tmp/drill-compare.sql`:

```sql
-- Set to the dump's date at 05:00 server time (the backup cron hour)
\set cutoff '''2026-06-10 05:00:00'''

select 'users' t, count(*) n,
       md5(string_agg(id::text, ',' order by id)) ids
  from users where created_at < :cutoff
union all
select 'decks', count(*),
       md5(string_agg(id::text, ',' order by id))
  from decks where created_at < :cutoff
union all
select 'deck_cards', count(*),
       md5(string_agg(deck_id::text || oracle_id::text || quantity, ',' order by deck_id, oracle_id))
  from deck_cards
union all
select 'scryfall_data', count(*), null
  from scryfall_data
union all
select '_sqlx_migrations', count(*),
       md5(string_agg(version::text, ',' order by version))
  from _sqlx_migrations;
```

Run it on both sides and diff:

```bash
# on the home server (over ssh/Tailscale), against live prod DB
psql "$DATABASE_URL" -f /tmp/drill-compare.sql > /tmp/compare-prod.txt

# locally, against the restored scratch DB
psql -d zwipe_drill -f /tmp/drill-compare.sql > /tmp/compare-drill.txt

diff /tmp/compare-prod.txt /tmp/compare-drill.txt && echo "MATCH"
```

**Checkpoint:** `MATCH`. The md5-of-ordered-ids columns catch corruption that
counts alone miss (same row count, mangled content).

Honest caveats on the bounding:
- `deck_cards` has no `created_at` to bound on, so rows added/changed after
  5am show as a small diff — expected if anyone (including you) used the app
  since the dump. Same for prod-side deletes of pre-cutoff rows. A clean way
  to get a perfect run: do the drill in the morning before touching the app.
- `scryfall_data` churns only when zervice syncs (4am, before the 5am dump),
  so its plain count should match exactly unless the drill spans a sync.
- If a diff appears, explain every line before calling it corruption —
  "user 21 registered at 9am" is fine; differing md5s over the *bounded* set
  is not.

### 4. (Optional, strongest form) Boot zerver against it

Point a local zerver at the scratch DB and log in with your real account —
the only end-to-end proof that auth rows survived intact:

```bash
cd zerver   # run from zerver/ so dotenvy finds .env
DATABASE_URL=postgres://localhost/zwipe_drill cargo run --bin zerver
# then: log in from a dev client / curl the login endpoint with your creds
```

### 5. Clean up

```bash
dropdb zwipe_drill
rm /tmp/zwipe-YYYYMMDD.sql /tmp/restore-drill.log
```

The dump file contains every user's email and password hash — do not leave
it in /tmp or commit it anywhere.

---

## Record the result

After running, note the date + outcome here:

| Date | Dump tested | Result | Notes |
|---|---|---|---|
| 2026-06-13 | `zwipe-20260613.sql.gz` (fresh manual dump, ~19:53) | **PASS** | Restored onto the Hetzner VPS (`100.114.251.8`) as VPS-migration Phase 0 step 6. PG 17.10 → PG 18 restore clean. Counts: scryfall_data 115,805, users 24, decks 37, deck_cards 1,627, commander_synergy 111, _sqlx_migrations 22. Only errors were `role "synergy_worker" does not exist` (GRANTs only — the zynegry least-priv role isn't on the VPS yet; cosmetic for zerver). zerver booted against the clone, `/health` green, `scadoshi` auth row intact (verified, valid argon2 hash). **Cutover prereq surfaced:** `createuser synergy_worker` on the VPS before the zynegry worker runs. |

## Known issues to watch for

- **rclone 501 on attempt 1** (`operations/infrastructure/backups.md`) — known noise, attempt
  2 succeeds; not a drill failure.
- **Plain-format dump**: the script uses plain `pg_dump | gzip`, so
  single-table restores aren't practical (also documented in backups.md). If
  the drill ever motivates it, switching the script to `--format=custom`
  enables `pg_restore --table=...` — but that changes the restore commands
  above too.
- **Postgres major-version skew** between the server and the drill machine is
  usually fine for restores (dumps are forward-compatible), but note it in
  the results table if versions differ.
