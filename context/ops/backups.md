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
2. Permissions: Object Read & Write
3. Scope: `zwipe-backups` bucket only
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

BACKUP_FILE="/tmp/zwipe-$(date +%Y%m%d).sql.gz"

pg_dump -U zwipe zwipe | gzip > "$BACKUP_FILE"
rclone copy "$BACKUP_FILE" r2:zwipe-backups/
rm "$BACKUP_FILE"

echo "backup complete: zwipe-$(date +%Y%m%d).sql.gz"
```

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

```bash
# List available backups
rclone ls r2:zwipe-backups/

# Download the one you need
rclone copy r2:zwipe-backups/zwipe-20260329.sql.gz /tmp/

# Decompress
gunzip /tmp/zwipe-20260329.sql.gz

# Restore (this replaces all data in the zwipe database)
psql -U zwipe zwipe < /tmp/zwipe-20260329.sql

# Clean up
rm /tmp/zwipe-20260329.sql
```

If restoring to a fresh server, create the database first:

```bash
sudo -u postgres createuser zwipe
sudo -u postgres createdb -O zwipe zwipe
psql -U zwipe zwipe < /tmp/zwipe-20260329.sql
```

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
