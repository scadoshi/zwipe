# Database Backups

Nightly PostgreSQL backups to Cloudflare R2 via `rclone`. The database is the only stateful data not replicated elsewhere; everything else is in GitHub.

---

## Prerequisites

- Cloudflare account (already have one for the tunnel)
- `rclone` installed on the server
- `pg_dump` available (comes with PostgreSQL)

---

## One-Time Setup

### 1. Create R2 Bucket

1. Log into Cloudflare Dashboard
2. R2 Object Storage → Create Bucket
3. Name: `zwipe-backups`
4. Region: auto (or nearest)
5. No lifecycle rule: `backup-db.sh` prunes to the newest 30 itself (see Retention below)

### 2. Create R2 API Token

1. R2 → Manage R2 API Tokens → Create API Token
2. Permissions: **Object Read & Write**, applied to the `zwipe-backups` bucket only. TTL: Forever (or put the expiry date somewhere you'll see it).
3. Save the **Access Key ID** and **Secret Access Key**: you won't see them again.
4. A bucket-scoped token can't create or inspect buckets, so rclone must not try: set `no_check_bucket = true` on the remote (step 3). Without it, every upload fails with `AccessDenied (403)` even though listing works.

### 3. Install and Configure rclone

```bash
sudo apt install rclone
rclone config
```

Interactive prompts:
```
n       (new remote)
r2      (name)
s3      (type: pick "Amazon S3 Compliant")
Cloudflare (provider)
        (paste Access Key ID)
        (paste Secret Access Key)
        (leave region blank)
        (endpoint: https://<ACCOUNT_ID>.r2.cloudflarestorage.com)
```

Your Cloudflare Account ID is on the R2 overview page in the dashboard.

Then turn off rclone's bucket check, which the bucket-scoped token isn't allowed to do:

```bash
rclone config update r2 no_check_bucket true
```

Replacing a token later is the same two values: `rclone config update r2 access_key_id <ID> secret_access_key <SECRET>`.

### 4. Test the Connection

```bash
echo "test" > /tmp/test-backup.txt
rclone copy /tmp/test-backup.txt r2:zwipe-backups/
rclone ls r2:zwipe-backups/
# Should show: test-backup.txt
rclone delete r2:zwipe-backups/test-backup.txt
rm /tmp/test-backup.txt
```

---

## Backup Script

Create `~/scripts/backup-db.sh`:

```bash
#!/bin/bash
set -euo pipefail

# Pull DATABASE_URL from zerver's .env (grep, not source; we don't want to
# eval other secrets like JWT_SECRET in this shell, and `source` would expand
# any $/backtick in their values).
ENV_FILE="/home/scadoshi/zwipe/.env"
DATABASE_URL=$(grep -E '^DATABASE_URL=' "$ENV_FILE" | cut -d= -f2-)
BACKUP_HEALTHCHECK_URL=$(grep -E '^BACKUP_HEALTHCHECK_URL=' "$ENV_FILE" | cut -d= -f2- || true)

BACKUP_FILE="/tmp/zwipe-$(date +%Y%m%d).sql.gz"
pg_dump "$DATABASE_URL" | gzip > "$BACKUP_FILE"
rclone copy "$BACKUP_FILE" r2:zwipe-backups/
rm "$BACKUP_FILE"

# Keep the newest $KEEP dumps in R2 (one a day, so ~30 days). Prune only after a
# successful upload, so a run of failed nights can never delete the last good copies.
KEEP=30
mapfile -t OLD < <(rclone lsf r2:zwipe-backups/ --files-only 2>/dev/null | grep -E '^zwipe-[0-9]{8}\.sql\.gz$' | sort | head -n -"$KEEP" || true)
for f in "${OLD[@]}"; do
  [ -n "$f" ] && { rclone deletefile "r2:zwipe-backups/$f" || echo "prune: could not delete $f"; }
done

echo "backup complete: zwipe-$(date +%Y%m%d).sql.gz"

# Ping Healthchecks.io only after a successful upload (set -e stops earlier on failure).
[ -n "$BACKUP_HEALTHCHECK_URL" ] && curl -fsS -m 10 --retry 3 "$BACKUP_HEALTHCHECK_URL" > /dev/null || true
```

**Retention:** the newest 30 dumps are kept (about 30 days at one a day, roughly 3.3 GB) and older ones are deleted, but only after a new upload succeeds. That is deliberate: an R2 lifecycle rule ("delete after 30 days") would keep deleting through an outage like the 2026 one and could empty the bucket.

**Alerting:** the last line pings the Healthchecks.io check "Zwipe Backups" (cron `0 5 * * *` UTC, 1 hour grace, email). `set -e` means it is only reached when the dump and the upload both succeeded, so a missing ping is the alert. The ping URL lives in the server's `.env` as `BACKUP_HEALTHCHECK_URL`, never in the repo. On a failed run the dump stays in `/tmp` (the `rm` is never reached), which is a useful last copy until the next reboot.

**Why `grep`, not `source`:** sourcing the whole `.env` would expand `$` and backticks in every value (e.g. a future `JWT_SECRET` containing shell-special characters). Pulling just the one line we need keeps the script ignorant of every other secret.

**Note:** `pg_dump` must receive the full connection URL as a positional argument, not via `-U`. Using `-U` with a URL causes PostgreSQL to treat the entire URL as a username and fail with peer authentication errors.

### Known noise: rclone 501 on attempt 1

Since ~2026-05-27, every run logs:

```
ERROR : zwipe-YYYYMMDD.sql.gz: Failed to copy: NotImplemented: Not Implemented
        status code: 501, request id: , host id:
ERROR : Attempt 1/3 failed with 1 errors and: NotImplemented: Not Implemented
ERROR : Attempt 2/3 succeeded
backup complete: zwipe-YYYYMMDD.sql.gz
```

Attempt 2 always succeeds and the backup lands correctly in R2. Root cause is rclone sending a checksum/multipart variant R2 returns 501 on. `--s3-upload-cutoff 1G` and `--s3-disable-checksum` were tried separately and together; neither silenced it. Likely fix when revisiting: confirm `provider = Cloudflare` is set in the `[r2]` block of `~/.config/rclone/rclone.conf`, or pin an older rclone version (`apt-cache madison rclone` to list, then `apt install rclone=<version>`).

**Severity: low.** Data is intact. Cost is one extra failed PUT per day and noisy logs. If monitoring greps for `ERROR`, exclude this script's output.

Make it executable:

```bash
chmod +x ~/scripts/backup-db.sh
```

Test it manually first:

```bash
~/scripts/backup-db.sh
rclone ls r2:zwipe-backups/
```

---

## Cron Schedule

```bash
crontab -e
```

Add:

```
0 5 * * * /home/<YOUR_USER>/scripts/backup-db.sh >> /var/log/zwipe/backup.log 2>&1
```

Runs at 5am daily, one hour after zervice (which runs at 4am).

Output goes to the same log directory as zerver logs.

---

## Restore from Backup

**This is destructive: it drops and recreates all tables.** Stop zerver first so nothing is writing to the database during restore.

```bash
# 1. Stop zerver
sudo systemctl stop zerver

# 2. List available backups
rclone ls r2:zwipe-backups/

# 3. Download the one you need
rclone copy r2:zwipe-backups/zwipe-20260329.sql.gz /tmp/

# 4. Decompress
gunzip /tmp/zwipe-20260329.sql.gz

# 5. Drop and recreate the database (clean slate)
sudo -u postgres dropdb zwipe
sudo -u postgres createdb -O zwipe zwipe

# 6. Restore (source .env for DATABASE_URL)
set -a && source ~/zwipe/.env && set +a
psql "$DATABASE_URL" < /tmp/zwipe-20260329.sql

# 7. Restart zerver
sudo systemctl start zerver

# 8. Clean up
rm /tmp/zwipe-20260329.sql

# 9. Verify
curl https://api.zwipe.net/health | jq
```

### Restore on a fresh server

If rebuilding from scratch, create the user first:

```bash
sudo -u postgres createuser zwipe -P   # prompts for password
sudo -u postgres createdb -O zwipe zwipe
set -a && source ~/zwipe/.env && set +a
psql "$DATABASE_URL" < /tmp/zwipe-20260329.sql
```

### Partial restore (single table)

If you only need to restore one table (e.g. user data got corrupted but cards are fine):

```bash
# Extract just that table's data from the dump
set -a && source ~/zwipe/.env && set +a
pg_restore --data-only --table=users /tmp/zwipe-20260329.sql | \
  psql "$DATABASE_URL"
```

**Note:** This only works if the backup was created with `pg_dump --format=custom`. The default plain-text format (which our script uses) requires manual editing of the `.sql` file to extract specific tables, doable but tedious. For most scenarios, a full restore is simpler and safer.

---

## Verify Backups Are Running

```bash
# Check last backup log entry
tail -5 /var/log/zwipe/backup.log

# Check what's in R2
rclone ls r2:zwipe-backups/

# Check cron is scheduled
crontab -l | grep backup
```

Healthchecks.io's "Zwipe Backups" check shows the last successful run at a glance.

**Incident, 2026-07-26 to 2026-10-06:** every nightly upload failed with `AccessDenied (403)` for 73 days and nothing noticed, because the script had no alert; the bucket was found empty. Fixed 2026-10-06 with a new bucket-scoped token, `no_check_bucket = true`, and the Healthchecks.io ping above. The dumps still in the server's `/tmp` (2026-09-27 onward) were uploaded, so history restarts there.

---

## Cost

R2: $0.015/GB/month, zero egress. A compressed dump is about 110MB (30 of them come to roughly 3.3GB), inside R2's 10GB free tier, so the monthly cost is $0.00.
