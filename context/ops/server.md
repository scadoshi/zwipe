# Ubuntu Server Setup

Repurposed desktop running Ubuntu Server (headless). Intel i5, 32GB RAM, x86_64.
Backend served via Cloudflare Tunnel — no port forwarding, TLS handled by Cloudflare.

---

## Setup Checklist

- [ ] Boot with Ubuntu Server USB installer (headless install)
- [ ] Install PostgreSQL, create `zwipe` DB + user
- [ ] Create `/var/log/zwipe/` log directory
- [ ] Install Rust, clone repo, build binaries
- [ ] Run SQLx migrations
- [ ] Configure zerver `.env`
- [ ] Install `cloudflared`, configure tunnel to `api.zwipe.net`
- [ ] Start `zerver` systemd service
- [ ] Add `zervice` nightly cron
- [ ] Run `zervice` once manually to seed Scryfall card data
- [ ] Verify iOS app hits `api.zwipe.net` successfully

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

## Log Directory

zerver writes rolling daily logs to `/var/log/zwipe/`. The app calls `create_dir_all` on startup
(idempotent), but `/var/log/` is root-owned — create it once:

```bash
sudo mkdir -p /var/log/zwipe
sudo chown $USER /var/log/zwipe
```

---

## Build

Build directly on the server — no cross-compilation needed:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone and build
git clone <repo-url> ~/zwipe-src
cd ~/zwipe-src/zerver
cargo build --release --bin zerver --bin zervice

# Deploy binaries
mkdir -p ~/zwipe
cp target/release/zerver target/release/zervice ~/zwipe/
```

---

## Migrations

```bash
cargo install sqlx-cli --no-default-features --features postgres

cd ~/zwipe-src/zerver
DATABASE_URL=postgres://zwipe:YOUR_DB_PASSWORD@127.0.0.1/zwipe sqlx migrate run
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
RESEND_EMAIL_FROM=noreply@zwipe.net
# LOG_DIR omitted — defaults to /var/log/zwipe
```

---

## systemd Service

`/etc/systemd/system/zerver.service`:
```ini
[Unit]
Description=zerver - zwipe backend
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=scottyfermo
WorkingDirectory=/home/scottyfermo/zwipe
EnvironmentFile=/home/scottyfermo/zwipe/.env
ExecStart=/home/scottyfermo/zwipe/zerver
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable zerver
sudo systemctl start zerver
sudo systemctl status zerver
```

---

## zervice Cron (Nightly Scryfall Sync)

```bash
crontab -e
# Add: 0 4 * * * /home/scottyfermo/zwipe/zervice >> /home/scottyfermo/zwipe/zervice.log 2>&1
```

Run manually first to seed card data:
```bash
cd ~/zwipe && ./zervice
```

---

## Cloudflare Tunnel

```bash
curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64.deb \
  -o cloudflared.deb
sudo dpkg -i cloudflared.deb

cloudflared tunnel login

# Reuse existing tunnel credentials
sudo mkdir -p /etc/cloudflared
sudo cp ~/.cloudflared/config.yml /etc/cloudflared/
sudo cp ~/.cloudflared/70ba169b-8293-4a60-9b2d-e1f996a161db.json /etc/cloudflared/

sudo cloudflared service install
sudo systemctl enable cloudflared
sudo systemctl start cloudflared
```

`/etc/cloudflared/config.yml`:
```yaml
tunnel: 70ba169b-8293-4a60-9b2d-e1f996a161db
credentials-file: /etc/cloudflared/70ba169b-8293-4a60-9b2d-e1f996a161db.json

ingress:
  - hostname: api.zwipe.net
    service: http://localhost:3000
  - service: http_status:404
```

**Note:** `nano` and other terminal programs may fail over SSH from Ghostty with `Error opening terminal: xterm-ghostty`. Fix: `TERM=xterm-256color nano ...`

---

## Verify

```bash
curl https://api.zwipe.net/
# {"message":"zerver","status":"ready","version":"0.1.0"}
```
