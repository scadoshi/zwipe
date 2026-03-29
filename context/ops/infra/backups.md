# Database Backups

Nightly PostgreSQL backups to Cloudflare R2 via `rclone`. The database is the only
stateful data not replicated elsewhere — everything else is in GitHub.

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
5. Set lifecycle rule: delete objects after 30 days (keeps retention automatic)

### 2. Create R2 API Token

1. R2 → Manage R2 API Tokens → Create API Token
2. Permissions: **Admin Read & Write** (Object Read & Write alone is insufficient — rclone needs bucket list operations)
3. Scope: Apply to all buckets (bucket-scoped tokens may fail with 403 even with correct naming)
4. Save the **Access Key ID** and **Secret Access Key** — you won't see them again

### 3. Install and Configure rclone

```bash
sudo apt install rclone
rclone config
```

Interactive prompts:
```
n       (new remote)
r2      (name)
s3      (type — pick "Amazon S3 Compliant")
Cloudflare (provider)
        (paste Access Key ID)
        (paste Secret Access Key)
        (leave region blank)
        (endpoint: https://<ACCOUNT_ID>.r2.cloudflarestorage.com)
```

Your Cloudflare Account ID is on the R2 overview page in the dashboard.

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

set -a
source ~/zwipe/.env
set +a
BACKUP_FILE="/tmp/zwipe-$(date +%Y%m%d).sql.gz"

pg_dump "$DATABASE_URL" | gzip > "$BACKUP_FILE"
rclone copy "$BACKUP_FILE" r2:zwipe-backups/
rm "$BACKUP_FILE"

echo "backup complete: zwipe-$(date +%Y%m%d).sql.gz"
```

**Note:** `pg_dump` must receive the full connection URL as a positional argument — not
via `-U`. Using `-U` with a URL causes PostgreSQL to treat the entire URL as a username
and fail with peer authentication errors.

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

Runs at 5am daily — one hour after zervice (which runs at 4am).

Output goes to the same log directory as zerver logs.

---

## Restore from Backup

**This is destructive — it drops and recreates all tables.** Stop zerver first so nothing
is writing to the database during restore.

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

**Note:** This only works if the backup was created with `pg_dump --format=custom`.
The default plain-text format (which our script uses) requires manual editing of the
`.sql` file to extract specific tables — doable but tedious. For most scenarios, a full
restore is simpler and safer.

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

---

## Cost

R2: $0.015/GB/month, zero egress. A compressed Postgres dump of 35k cards + users is
~5-10MB. Monthly cost rounds to $0.00.
