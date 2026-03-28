# Ubuntu Server Setup

Repurposed desktop running Ubuntu Server (headless). Intel i5, 32GB RAM, x86_64.
Backend served via Cloudflare Tunnel — no port forwarding, TLS handled by Cloudflare.

---

## Setup Checklist

- [ ] Boot with Ubuntu Server USB installer (headless install)
- [ ] SSH in from Mac, set up key auth
- [ ] Install PostgreSQL, create `zwipe` DB + user
- [ ] Create `/var/log/zwipe/` log directory
- [ ] Configure zerver `.env`
- [ ] Install sqlx-cli, run migrations
- [ ] Install Rust, clone repo, build binaries
- [ ] Install `cloudflared`, configure tunnel to `api.zwipe.net`
- [ ] Start `zerver` systemd service
- [ ] Add `zervice` nightly cron
- [ ] Run `zervice` once manually to seed Scryfall card data
- [ ] Verify iOS app hits `api.zwipe.net` successfully

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
RUST_LOG=info
RUST_BACKTRACE=1
RESEND_API_KEY=<from Resend dashboard>
RESEND_EMAIL_FROM=hello@zwipe.net
# LOG_DIR omitted — defaults to /var/log/zwipe
```

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

## zervice Cron (Nightly Scryfall Sync)

```bash
crontab -e
# Add: 0 4 * * * /home/scadoshi/zwipe/zervice >> /home/scadoshi/zwipe/zervice.log 2>&1
```

Run manually first to seed card data:
```bash
cd ~/zwipe && ./zervice
```

---

## Cloudflare Tunnel

Cloudflare Tunnel creates an outbound-only encrypted connection from the server to Cloudflare's edge — no port forwarding, no firewall rules, TLS handled by Cloudflare. Requests to
`api.zwipe.net` from any network (home, mobile, anywhere) route through the tunnel.

### Install cloudflared

```bash
curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64.deb \
  -o cloudflared.deb
sudo dpkg -i cloudflared.deb
```

### Authenticate (one-time — headless, no browser)

The server has no display. `cloudflared tunnel login` detects this and prints a URL
to the terminal instead of opening a browser. Copy that URL and open it on your Mac
or phone to complete the OAuth flow. The server polls and writes `~/.cloudflared/cert.pem`
automatically once you approve.

```bash
cloudflared tunnel login
# Prints something like:
# Please open the following URL and log in with your Cloudflare account:
# https://dash.cloudflare.com/argotunnel?callback=...
#
# Open that URL on your Mac/phone — select the zwipe.net zone
# Terminal will confirm: "You have successfully logged in."
```

### Create the tunnel (first time only)

```bash
cloudflared tunnel create zwipe
# Prints a UUID — note it, e.g. 70ba169b-8293-4a60-9b2d-e1f996a161db
# Writes ~/.cloudflared/<UUID>.json (credentials file)
```

### Create config file

```bash
mkdir -p ~/.cloudflared
nano ~/.cloudflared/config.yml
```

`~/.cloudflared/config.yml`:
```yaml
tunnel: 70ba169b-8293-4a60-9b2d-e1f996a161db
credentials-file: /home/scottyfermo/.cloudflared/70ba169b-8293-4a60-9b2d-e1f996a161db.json

ingress:
  - hostname: api.zwipe.net
    service: http://localhost:3000
  - service: http_status:404
```

Replace the UUID with the one printed by `tunnel create`.

### Add DNS record in Cloudflare dashboard

1. dash.cloudflare.com → zwipe.net → DNS
2. Add record:
   - Type: `CNAME`
   - Name: `api`
   - Target: `<UUID>.cfargotunnel.com` (e.g. `70ba169b-8293-4a60-9b2d-e1f996a161db.cfargotunnel.com`)
   - Proxy status: Proxied (orange cloud — required)

Or do it via CLI (requires tunnel login already done):
```bash
cloudflared tunnel route dns zwipe api.zwipe.net
```

### Install as systemd service

```bash
# Copy credentials and config to /etc/cloudflared for the service
sudo mkdir -p /etc/cloudflared
sudo cp ~/.cloudflared/config.yml /etc/cloudflared/
sudo cp ~/.cloudflared/70ba169b-8293-4a60-9b2d-e1f996a161db.json /etc/cloudflared/

# Update credentials-file path in /etc/cloudflared/config.yml to /etc/cloudflared/...
sudo nano /etc/cloudflared/config.yml
# Change: credentials-file: /etc/cloudflared/70ba169b-....json

sudo cloudflared service install
sudo systemctl enable cloudflared
sudo systemctl start cloudflared
sudo systemctl status cloudflared
```

### Re-deploying to a new machine

If moving to a new server, the tunnel UUID and credentials file already exist in Cloudflare —
you don't need to recreate them. Just:
1. `cloudflared tunnel login` (re-authenticate)
2. Copy the existing `<UUID>.json` credentials from the old server (or re-download via `tunnel token`)
3. Write `config.yml` with the same UUID
4. Install the service as above

**Note:** `nano` and other terminal programs may fail over SSH from Ghostty with
`Error opening terminal: xterm-ghostty`. Fix: `TERM=xterm-256color nano ...`

---

## Verify

```bash
curl https://api.zwipe.net/
# {"message":"zerver","status":"ready","version":"0.1.0"}
```

