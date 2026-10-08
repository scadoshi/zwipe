# Running Binaries Manually on the Server

Both binaries run as their own users (`zerver`, `zervice`) with env files in `/etc/zwipe` that `scadoshi` cannot read: `zerver.env` (JWT_SECRET, DATABASE_URL, etc.) and the much smaller `zervice.env`. Running through systemd keeps each binary on its own user, file and sandbox, so prefer it. A hand run under `sudo -u` is for debugging only.

---

## Run zervice manually (Scryfall sync + session cleanup)

```bash
sudo systemctl start zervice
journalctl -u zervice -f
```

zervice is a run-once binary: it syncs cards from Scryfall, cleans expired sessions, and exits. Useful after dropping/recreating the database to repopulate cards immediately instead of waiting for the nightly timer.

---

## Run zerver manually

Normally runs via systemd. Only use this for debugging:

```bash
# Stop the service first to avoid port conflicts
sudo systemctl stop zerver

sudo -u zerver bash -c 'set -a; . /etc/zwipe/zerver.env; set +a; exec /usr/local/bin/zerver'

# When done, restart the service
sudo systemctl start zerver
```

---

## Why `set -a` is needed

The env files use `KEY=VALUE` format without `export`. `set -a` tells bash to automatically export every variable that gets assigned, making them visible to child processes. `set +a` turns it back off.
