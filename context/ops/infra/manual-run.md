# Running Binaries Manually on the Server

`zerver` and `zervice` need environment variables from `~/zwipe/.env` (JWT_SECRET,
DATABASE_URL, etc.). The systemd service handles this automatically, but manual runs
require sourcing the `.env` first.

---

## Run zervice manually (Scryfall sync + session cleanup)

```bash
cd ~/zwipe
set -a && source .env && set +a
./zervice
```

zervice is a run-once binary — it syncs cards from Scryfall, cleans expired sessions,
and exits. Useful after dropping/recreating the database to repopulate cards immediately
instead of waiting for the nightly cron.

---

## Run zerver manually

Normally runs via systemd. Only use this for debugging:

```bash
# Stop the service first to avoid port conflicts
sudo systemctl stop zerver

cd ~/zwipe
set -a && source .env && set +a
./zerver

# When done, restart the service
sudo systemctl start zerver
```

---

## Why `set -a` is needed

The `.env` file uses `KEY=VALUE` format without `export`. `set -a` tells bash to
automatically export every variable that gets assigned, making them visible to child
processes. `set +a` turns it back off.
