# Ubuntu Server Setup

> **NOTE (2026-06-13): Prod no longer runs here.** Production migrated to a
> Hetzner VPS (`zerver-prod`, tailnet `100.114.251.8`, admin `ssh root@…`) —
> see `context/plans/vps_migration.md`. This home box is powered off but kept
> intact as the rollback. The checklist below remains the general
> rebuild/setup reference (it's what the VPS was built from); only the
> WiFi/netplan section is home-box-specific.

Repurposed desktop running Ubuntu Server (headless). Intel i5, 32GB RAM, x86_64.
Backend served via Cloudflare Tunnel — no port forwarding, TLS handled by Cloudflare.

---

## Setup Checklist

- [ ] Boot with Ubuntu Server USB installer (headless install)
- [ ] Configure WiFi via netplan (see WiFi section below)
- [ ] Verify SSH is enabled on boot: `sudo systemctl enable ssh`
- [ ] SSH in from Mac, set up key auth
- [ ] Install NetworkManager for `nmtui`/`nmcli`: `sudo apt install network-manager`
- [ ] Install Tailscale for stable SSH access (see Tailscale section below)
- [ ] Install PostgreSQL, create `zwipe` DB + user
- [ ] Create `/var/log/zwipe/` log directory
- [ ] Install Rust, clone repo, build binaries
- [ ] Configure zerver `.env` (must include `DATABASE_URL` — CI/CD sources this for migrations)
- [ ] Install sqlx-cli: `cargo install sqlx-cli --no-default-features --features rustls,postgres`
- [ ] Run initial migrations: `cargo sqlx migrate run --source zerver/migrations`
- [ ] Install `cloudflared`, configure tunnel to `api.zwipe.net`
- [ ] Start `zerver` systemd service
- [ ] Add `zervice` nightly cron (4am daily)
- [ ] Add backup cron (5am daily) — see `backups.md`
- [ ] Run `zervice` once manually to seed Scryfall card data
- [ ] Install self-hosted GitHub Actions runner (see `cicd.md`) — this is what deploys code, runs migrations, and restarts zerver on every push to main
- [ ] Verify iOS app hits `api.zwipe.net` successfully

---

## WiFi (netplan)

The server connects over WiFi. Netplan is built into Ubuntu Server — no extra packages needed.

**Find the wireless interface name:**
```bash
ip link show
# Look for wlp3s0 or similar (not lo, not enp*)
```

**Create or edit the netplan config:**
```bash
sudo nano /etc/netplan/50-cloud-init.yaml
```

```yaml
network:
  version: 2
  wifis:
    wlp3s0:
      dhcp4: true
      access-points:
        "YOUR_SSID":
          password: "YOUR_PASSWORD"
```

**Apply and verify:**
```bash
sudo chmod 600 /etc/netplan/50-cloud-init.yaml
sudo netplan apply
ip addr show wlp3s0
# Should show an inet line with an IP address
```

The config is persistent — WiFi reconnects automatically on boot.

**Optional:** Install NetworkManager for the friendlier `nmtui` and `nmcli` tools:
```bash
sudo apt install network-manager
```

---

## Tailscale

Tailscale provides a stable IP for SSH access regardless of local DHCP changes. Once installed,
you can SSH via the Tailscale IP instead of the local network IP.

**Install:**
```bash
curl -fsSL https://tailscale.com/install.sh | sh
```

**Authenticate (headless — no browser):**
```bash
sudo tailscale up
# Prints a URL — open it on your Mac/phone to authenticate
```

**Verify:**
```bash
tailscale status
# Shows this machine and any other devices on your tailnet
tailscale ip -4
# Shows your stable Tailscale IP (100.x.x.x)
```

**SSH via Tailscale from your Mac:**
```bash
ssh scadoshi@<tailscale-ip>
```

Tailscale runs as a systemd service (`tailscaled`) and starts automatically on boot.

---

## SSH Access

Everything below is done over SSH. Get onto the server first.

### Find the server's IP (from the server directly on first boot)

The Ubuntu Server installer leaves you at a login prompt with the IP shown on screen.
If you miss it or need it later:

```bash
ip addr show | grep 'inet ' | grep -v 127.0.0.1
# Look for something like: inet 192.168.1.XXX/24
```

Or check your router's DHCP client list — the server will appear as a connected device.

### First-time access — fix "Permission denied (publickey)"

Ubuntu Server disables password authentication by default. You'll get this error
immediately if you just try to `ssh` in cold. Fix it once from the physical console
(plug in a keyboard/monitor briefly, or use the server's existing display):

**On the server (physically):**
```bash
sudo nano /etc/ssh/sshd_config
# Find and change (or add) these two lines:
#   PasswordAuthentication yes
#   KbdInteractiveAuthentication yes
# Save: Ctrl+O, Enter, Ctrl+X

sudo systemctl restart ssh
```

Now SSH with your password works from your Mac. Set up key auth immediately so
you never need the console again:

**On your Mac:**
```bash
# Generate a key if you don't have one
ssh-keygen -t ed25519 -C "zwipe-server"
# Accept defaults

# Copy the public key to the server (enter your server password once)
ssh-copy-id scadoshi@192.168.1.XXX
```

**Back on the server — re-disable password auth (security):**
```bash
sudo nano /etc/ssh/sshd_config
# Set back to:
#   PasswordAuthentication no
#   KbdInteractiveAuthentication no

sudo systemctl restart ssh
```

From now on `ssh scadoshi@192.168.1.XXX` works without a password and
password-based login is blocked.

### SSH in from your Mac

```bash
ssh scadoshi@192.168.1.XXX
```

### Add a friendly alias (optional, saves typing)

In `~/.zshrc` on your Mac:
```bash
alias zwipe-server='ssh scadoshi@192.168.1.XXX'
```

---

## PostgreSQL

```bash
sudo apt update && sudo apt install -y postgresql postgresql-contrib
sudo systemctl enable postgresql
sudo systemctl start postgresql

sudo -u postgres psql -c "CREATE USER zwipe WITH PASSWORD 'YOUR_DB_PASSWORD';"
sudo -u postgres psql -c "CREATE DATABASE zwipe OWNER zwipe;"
```

Test connection:
```bash
PGPASSWORD='YOUR_DB_PASSWORD' psql -U zwipe -h 127.0.0.1 -d zwipe -c '\l'
```

**Use `127.0.0.1` (TCP), not `localhost` (Unix socket).** Peer auth blocks socket connections for non-system users. TCP uses password auth.

**Special characters in password:** `<` and `>` must be URL-encoded as `%3C` / `%3E` in `DATABASE_URL`.

---

## Wipe and Rebuild the Database

Use this when you need to reset all data but keep the schema — e.g. clearing test/dev data
from production, or recovering from a corrupt state.

### Why not `sqlx database reset`?

`sqlx database reset` requires the `zwipe` user to have `CREATEDB` permission. Since we use
a least-privilege user, it will fail with `permission denied to create database`. Use the
postgres superuser instead.

### Steps

**1. Stop zerver** (releases the database connection):
```bash
sudo systemctl stop zerver
```

**2. Drop the database** (must be run as two separate commands — postgres won't accept both in one `-c` call):
```bash
sudo -u postgres psql -c "DROP DATABASE zwipe;"
sudo -u postgres psql -c "CREATE DATABASE zwipe OWNER zwipe;"
```

**3. Run migrations** (recreates all tables and indexes):
```bash
cd ~/zwipe-src/zerver
DATABASE_URL=postgres://zwipe:YOUR_PASSWORD@127.0.0.1/zwipe sqlx migrate run
```

Migrations live in `zerver/migrations/` — `zwipe-src` must be cloned and up to date.

**4. Restart zerver:**
```bash
sudo systemctl start zerver
sudo systemctl status zerver
```

**5. Optionally reseed card data:**
```bash
cd ~/zwipe && ./zervice
```

This re-syncs all 35k+ cards from Scryfall. Takes a few minutes.

---

## Change Database Password

```bash
# 1. Generate a new password
openssl rand -hex 24

# 2. Change it in PostgreSQL
sudo -u postgres psql -c "ALTER USER zwipe WITH PASSWORD 'NEW_PASSWORD';"

# 3. Update DATABASE_URL in ~/zwipe/.env
nano ~/zwipe/.env

# 4. Restart zerver
sudo systemctl restart zerver
sudo systemctl status zerver
```

URL-encode special characters in `DATABASE_URL` if needed (e.g. `<` → `%3C`).
No cron or CI changes required — both source the same `.env`.

---

## Log Directory

zerver writes rolling daily logs to `/var/log/zwipe/`. The app calls `create_dir_all` on startup
(idempotent), but `/var/log/` is root-owned — create it once:

```bash
sudo mkdir -p /var/log/zwipe
sudo chown $USER /var/log/zwipe
```

---

## .env

Located at `~/zwipe/.env`:
```
JWT_SECRET=<openssl rand -hex 32>
DATABASE_URL=postgres://zwipe:URL_ENCODED_PASSWORD@127.0.0.1/zwipe
BIND_ADDRESS=0.0.0.0:3000
ALLOWED_ORIGINS=https://zwipe.net
RUST_LOG=info,sqlx=warn,zwipe=debug,zerver=debug
RUST_BACKTRACE=1
RESEND_API_KEY=<from Resend dashboard>
RESEND_EMAIL_FROM=support@zwipe.net
# LOG_DIR omitted — defaults to /var/log/zwipe
```

### RUST_LOG directives

`RUST_LOG` is parsed by `tracing_subscriber::EnvFilter`. It accepts either a bare level
(`info`) or comma-separated per-target directives. Production default above silences
SQLx query spam while keeping `info` everywhere else and `debug` for our own crates.

Useful tweaks:
- Bump app-only verbosity temporarily: `RUST_LOG=info,zwipe=trace,zerver=trace`
- Silence hyper/h2 noise: append `,hyper=warn,h2=warn`
- One-off via systemd override (no `.env` edit needed):
  ```bash
  sudo systemctl edit zerver
  # In the editor:
  # [Service]
  # Environment="RUST_LOG=info,sqlx=debug"
  sudo systemctl restart zerver
  ```
  The override file wins over `EnvironmentFile=` from `.env`. Remove with `sudo systemctl revert zerver`.

### ALLOWED_ORIGINS

`ALLOWED_ORIGINS` is a comma-separated list of browser origins permitted by the CORS policy.

**The iOS native app is not affected by CORS.** Native apps (Dioxus on iPhone, using `reqwest`)
do not send an `Origin` header — CORS is a browser security mechanism. The iOS app will always
reach the API regardless of what is in `ALLOWED_ORIGINS`.

For production:
```
ALLOWED_ORIGINS=https://zwipe.net
```

If you also need the web client (`dx serve` on your Mac) to hit the live API during development,
add localhost as a second origin:
```
ALLOWED_ORIGINS=https://zwipe.net,http://localhost:8080
```

The value is parsed as `HeaderValue` — no trailing slashes, no wildcards.

---

## Migrations

`query_scalar!` and other SQLx macros verify SQL against real database tables **at compile time**.
The database must exist and migrations must have run before `cargo build` will succeed.

```bash
cargo install sqlx-cli --no-default-features --features postgres

cd ~/zwipe-src/zerver
DATABASE_URL=postgres://zwipe:YOUR_DB_PASSWORD@127.0.0.1/zwipe sqlx migrate run
```

---

## Build

Build directly on the server — no cross-compilation needed:

```bash
# Install build tools (gcc, make, etc.)
sudo apt install -y build-essential

# The project's .cargo/config.toml specifies linker = "x86_64-unknown-linux-gnu-gcc"
# for the x86_64 target (needed for cross-compiling from macOS).
# On the server, gcc is installed but under a different name — bridge the gap with a symlink:
sudo ln -s /usr/bin/gcc /usr/local/bin/x86_64-unknown-linux-gnu-gcc

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone and build
git clone <repo-url> ~/zwipe-src
cd ~/zwipe-src
cargo build --release --bin zerver --bin zervice

# Deploy binaries — output is in workspace root target/, not zerver/target/
mkdir -p ~/zwipe
cp target/release/zerver target/release/zervice ~/zwipe/
```

---

## systemd Service

systemd is Ubuntu's service manager. A unit file tells it how to run zerver — so it starts
automatically on boot and restarts itself if it crashes, instead of you running `./zerver`
manually in a terminal.

**Create the file:**
```bash
sudo nano /etc/systemd/system/zerver.service
```

Paste in the following, save with `Ctrl+O` → Enter → `Ctrl+X`:

`/etc/systemd/system/zerver.service`:
```ini
[Unit]
Description=zerver - zwipe backend
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=scadoshi
WorkingDirectory=/home/scadoshi/zwipe
EnvironmentFile=/home/scadoshi/zwipe/.env
ExecStart=/home/scadoshi/zwipe/zerver
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

**Register and start the service:**
```bash
sudo systemctl daemon-reload   # tells systemd to pick up the new file
sudo systemctl enable zerver   # start automatically on every boot
sudo systemctl start zerver    # start it right now
sudo systemctl status zerver   # verify it's running
```

What each command does:
- `enable` — registers zerver to start on boot
- `start` — starts it immediately without rebooting
- `Restart=on-failure` — if zerver crashes, systemd brings it back automatically
- `status` — shows running state and the last few log lines

---

## zervice Cron (Nightly Scryfall Sync + Session Cleanup)

```bash
crontab -e
```

Add **two lines** — `SHELL=/bin/bash` plus the scheduled invocation:

```cron
SHELL=/bin/bash
0 4 * * * set -a && source /home/scadoshi/zwipe/.env && set +a && /home/scadoshi/zwipe/zervice >> /var/log/zwipe/zervice-cron.log 2>&1
```

**Why `SHELL=/bin/bash` is required:** cron defaults to `/bin/sh` (dash on Ubuntu), and
`source` is not a builtin in dash — it's a bash-ism. Without `SHELL=/bin/bash` the whole
line silently fails at `source`, config never loads, tracing never inits, and **no log
file is produced** (the rolling appender hasn't been built yet). That's how weeks of
zervice runs went missing in mid-2026 before the 2026-05-26 fix.

**Why the stderr redirect (`>> /var/log/zwipe/zervice-cron.log 2>&1`):** captures any
early-startup failure (bad `.env`, missing binary, dash-vs-bash issue) that happens
before tracing's file appender takes over. Without it cron sends failure output to a
mail spool nobody reads. Belt-and-suspenders: zervice's own logs go to
`$LOG_DIR/zervice.YYYY-MM-DD.log` once it boots; the cron log captures everything from
schedule-fire to that point.

zervice is a run-once binary — it syncs cards from Scryfall, cleans expired sessions,
and exits. Logs are written to `$LOG_DIR/zervice.YYYY-MM-DD.log` (default: `/var/log/zwipe/`).

Run manually first to seed card data:
```bash
cd ~/zwipe
set -a && source .env && set +a
./zervice
```

---

## Cloudflare Tunnel + DNS

See [cloudflare.md](cloudflare.md) for full tunnel setup, DNS records, and domain config.

---

## Verify

```bash
curl https://api.zwipe.net/
# {"message":"zerver","status":"ready","version":"0.1.0"}
```

