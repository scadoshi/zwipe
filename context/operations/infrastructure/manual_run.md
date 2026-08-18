# Running Binaries Manually on the Server

`zerver` needs environment variables from `~/zwipe/.env` (JWT_SECRET, DATABASE_URL,
etc.). `zervice` reads the much smaller `~/zwipe/.env.zervice` instead. Its systemd
unit points `EnvironmentFile=` at that file, and sourcing the full `.env` for a manual
run would hand the sync binary every secret it was deliberately cut off from. The
systemd units handle this automatically; manual runs source the right file first.

---

## Run zervice manually (Scryfall sync + session cleanup)

```bash
cd ~/zwipe
set -a && source .env.zervice && set +a
./zervice
```

zervice is a run-once binary — it syncs cards from Scryfall, cleans expired sessions,
and exits. Useful after dropping/recreating the database to repopulate cards immediately
instead of waiting for the nightly timer.

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
