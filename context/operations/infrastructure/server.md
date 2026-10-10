# Ubuntu Server Setup

> **NOTE (2026-06-13): Prod no longer runs here.** Production migrated to a Hetzner VPS (`zerver-prod`, tailnet `<server-tailnet-ip>`, admin `ssh scadoshi@…`; root does not log in). (The old `context/plans/vps_migration.md` write-up no longer exists.) This home box was later rebuilt as scotland-server, Scotty's always-on maintenance box, so it is no longer a rollback. The checklist below remains the general rebuild/setup reference (it's what the VPS was built from); only the WiFi/netplan section is home-box-specific.

Repurposed desktop running Ubuntu Server (headless). Intel i5, 32GB RAM, x86_64. Backend served via Cloudflare Tunnel: no port forwarding, TLS handled by Cloudflare.

---

## Setup Checklist

- [ ] Boot with Ubuntu Server USB installer (headless install)
- [ ] Configure WiFi via netplan (see WiFi section below)
- [ ] Verify SSH is enabled on boot: `sudo systemctl enable ssh`
- [ ] SSH in from Mac as `scadoshi`, then lock SSH down (see SSH Access below)
- [ ] Install NetworkManager for `nmtui`/`nmcli`: `sudo apt install network-manager`
- [ ] Install Tailscale for stable SSH access (see Tailscale section below)
- [ ] Install PostgreSQL, create `zwipe` DB + user
- [ ] Create the service users, `/etc/zwipe` and `/var/log/zwipe` (see Service Users and Log Directory below)
- [ ] Install Rust, clone repo, build binaries
- [ ] Write the env files in `/etc/zwipe` (see .env below; the deploy reads `migrate.env` for migrations)
- [ ] Install sqlx-cli: `cargo install sqlx-cli --no-default-features --features rustls,postgres`
- [ ] Run initial migrations: `cargo sqlx migrate run --source zerver/migrations`
- [ ] Install `cloudflared`, configure tunnel to `api.zwipe.net`
- [ ] Install `zerver.service` from `zcripts/server/systemd/` and start it (see systemd Service below)
- [ ] Install the `zervice` units from `zcripts/server/systemd/` (`zervice.service`, `zervice.timer`, `zervice-alert.service`) into `/etc/systemd/system/`, and `zervice-alert.sh` as `/usr/local/bin/zervice-alert`. Nightly timer, NOT cron
- [ ] Place `/etc/zwipe/zervice.env` (see the zervice scheduling section below) and run `zcripts/server/sql/zervice_role.sql` to create the scoped Postgres role. Feed it on **stdin**, never with `-f`: `sudo -u postgres psql zwipe < ~/zwipe-src/zcripts/server/sql/zervice_role.sql`. The `postgres` user cannot traverse `/home/scadoshi`, so `-f` fails with a bare `Permission denied`; the redirect opens the file as you. Same trap applies to any script under the repo, `zcripts/metrics/errors.sql` included
- [ ] Add backup cron (5am daily): see `backups.md`
- [ ] Run `zervice` once manually to seed Scryfall card data
- [ ] Install the self-hosted GitHub Actions runner as the `runner` user (see `cicd.md`): this is what deploys code, runs migrations, and restarts zerver on every push to main
- [ ] Verify iOS app hits `api.zwipe.net` successfully

---

## WiFi (netplan)

The server connects over WiFi. Netplan is built into Ubuntu Server, so no extra packages are needed.

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

The config is persistent; WiFi reconnects automatically on boot.

**Optional:** Install NetworkManager for the friendlier `nmtui` and `nmcli` tools:
```bash
sudo apt install network-manager
```

---

## Tailscale

Tailscale provides a stable IP for SSH access regardless of local DHCP changes. Once installed, you can SSH via the Tailscale IP instead of the local network IP.

**Install:**
```bash
curl -fsSL https://tailscale.com/install.sh | sh
```

**Authenticate (headless, no browser):**
```bash
sudo tailscale up
# Prints a URL: open it on your Mac/phone to authenticate
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

**After an Ubuntu release upgrade**, check `/etc/apt/sources.list.d/tailscale.list`. The upgrade comments out third-party sources it cannot migrate, and Tailscale then never updates. Put the line back with the new release's codename (`deb [signed-by=/usr/share/keyrings/tailscale-archive-keyring.gpg] https://pkgs.tailscale.com/stable/ubuntu <codename> main`) and run `sudo apt-get update && sudo apt-get install --only-upgrade tailscale`. Run the upgrade detached (`sudo systemd-run --collect sh -c "..."`) when you are connected over the tailnet, since restarting `tailscaled` drops the session.

---

## SSH Access

The server is administered as `scadoshi` over the tailnet. `scadoshi` has full sudo with a password; root does not log in. zerver, heron and scotland-server all follow the same rules.

### First login and the admin user

On a Hetzner VPS, add your Mac's public key when you create the server; it goes into root's `authorized_keys`, and `ssh root@<public-ip>` works once. On a home box, use the console. Either way, create the admin user first:

```bash
adduser scadoshi                         # asks for the sudo password
usermod -aG sudo,systemd-journal scadoshi
install -d -m 700 -o scadoshi -g scadoshi /home/scadoshi/.ssh
install -m 600 -o scadoshi -g scadoshi /root/.ssh/authorized_keys /home/scadoshi/.ssh/authorized_keys
```

Each key in `authorized_keys` ends with a label naming its device. Add one when you add a key, so an unknown key stands out.

Install Tailscale (section above), then log in from your Mac as `scadoshi@<server-tailnet-ip>` and check `sudo -v` before the next step.

### Lock it down

`/etc/ssh/sshd_config.d/10-hardening.conf`, the same file on all three boxes:

```
PermitRootLogin no
PasswordAuthentication no
KbdInteractiveAuthentication no
X11Forwarding no

Match Address 100.64.0.0/10,fd7a:115c:a1e0::/48
    PasswordAuthentication yes

Match all
```

Keys work from anywhere the box is reachable; passwords from the tailnet only. Leave the main `sshd_config` alone: the drop-in is read first and wins. Check and apply:

```bash
sudo sshd -t && sudo systemctl reload ssh
```

Then the firewall, which lets in nothing but the tailnet:

```bash
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow in on tailscale0
sudo ufw enable
```

Your session survives both. If the tailnet is ever down, Hetzner's web console (root's password) is the way back in.

### Alias

In `~/.ssh/config` on your Mac:
```
Host zerver
    HostName <server-tailnet-ip>
    User scadoshi
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

Use this when you need to reset all data but keep the schema, e.g. clearing test/dev data from production, or recovering from a corrupt state.

### Why not `sqlx database reset`?

`sqlx database reset` requires the `zwipe` user to have `CREATEDB` permission. Since we use a least-privilege user, it will fail with `permission denied to create database`. Use the postgres superuser instead.

### Steps

**1. Stop zerver** (releases the database connection):
```bash
sudo systemctl stop zerver
```

**2. Drop the database** (must be run as two separate commands; postgres won't accept both in one `-c` call):
```bash
sudo -u postgres psql -c "DROP DATABASE zwipe;"
sudo -u postgres psql -c "CREATE DATABASE zwipe OWNER zwipe;"
```

**3. Run migrations** (recreates all tables and indexes):
```bash
cd ~/zwipe-src
set -a; . <(sudo cat /etc/zwipe/migrate.env); set +a
cargo sqlx migrate run --source zerver/migrations
```

Migrations live in `zerver/migrations/`; `zwipe-src` must be cloned and up to date.

**4. Restart zerver:**
```bash
sudo systemctl start zerver
sudo systemctl status zerver
```

**5. Optionally reseed card data:**
```bash
sudo systemctl start zervice
```

This re-syncs all 35k+ cards from Scryfall. Takes a few minutes.

Run it through the unit, not by hand: the unit runs as the `zervice` user with `/etc/zwipe/zervice.env`. The three materialized views are **owned** by the `zervice` role, and `REFRESH MATERIALIZED VIEW` requires ownership rather than a grant, so a run with zerver's `DATABASE_URL` connects as `zwipe` and step 4 fails on all three with `permission denied for materialized view`. The card sync in steps 1 to 3 still succeeds, which makes the run look half-broken when the only thing wrong is the identity. Hit 2026-09-22.

---

## Change Database Password

```bash
# 1. Generate a new password
openssl rand -hex 24

# 2. Change it in PostgreSQL
sudo -u postgres psql -c "ALTER USER zwipe WITH PASSWORD 'NEW_PASSWORD';"

# 3. Update DATABASE_URL in all three files that carry it
sudoedit /etc/zwipe/zerver.env
sudoedit /etc/zwipe/migrate.env
nano ~/.config/zwipe-backup.env

# 4. Restart zerver
sudo systemctl restart zerver
sudo systemctl status zerver
```

URL-encode special characters in `DATABASE_URL` if needed (e.g. `<` → `%3C`). zerver reads `zerver.env`, the deploy's migrations read `migrate.env`, and the nightly backup reads `zwipe-backup.env`; miss one and that one fails.

---

## Log Directory

zerver and zervice write rolling daily logs to `/var/log/zwipe/`, and the backup cron appends `backup.log` there. All three share it through the `zwipe` group: the folder is setgid, and the units set `UMask=0002`, so every file stays group-writable whichever user created it.

```bash
sudo install -d -m 2775 -o root -g zwipe /var/log/zwipe
```

---

## Service Users

No service runs as `scadoshi`; only the nightly backup cron does (`backups.md`). Each service has its own login-less system user, and the deploy runner has its own `runner` user:

```bash
sudo groupadd --system zwipe
sudo useradd --system --no-create-home --shell /usr/sbin/nologin --user-group --groups zwipe zerver
sudo useradd --system --no-create-home --shell /usr/sbin/nologin --user-group --groups zwipe zervice
sudo usermod -aG zwipe scadoshi           # the backup cron writes backup.log
sudo useradd --create-home --shell /bin/bash runner
sudo install -d -m 700 -o runner -g runner /home/runner/deploy
```

Binaries live in `/usr/local/bin` (`zerver`, `zervice`, `zervice-alert`), owned by root. The runner installs new ones through its narrow sudo rule; see `cicd.md`.

---

## .env

Each process gets its own file in `/etc/zwipe`, owned by root and readable only by the user that needs it:

| File | Owner:group, mode | Read by | Holds |
|------|-------------------|---------|-------|
| `zerver.env` | `root:zerver 640` | zerver | everything below |
| `zervice.env` | `root:zervice 640` | zervice | `DATABASE_URL` (as the `zervice` role), `RUST_LOG`, `LOG_DIR`, `HEALTHCHECK_PING_URL` |
| `alert.env` | `root:zervice-alert 640` | zervice-alert | `RESEND_API_KEY`, `RESEND_EMAIL_FROM`, `SUPPORT_EMAIL_ADDRESS` |
| `migrate.env` | `root:runner 640` | the deploy runner | `DATABASE_URL` only |

Edit one with `sudoedit /etc/zwipe/zerver.env`, then `sudo systemctl restart zerver`. To use a value in an admin shell without making the file readable, source it through sudo: `set -a; . <(sudo cat /etc/zwipe/zerver.env); set +a`.

`/etc/zwipe/zerver.env`:
```
JWT_SECRET=<openssl rand -hex 32>
DATABASE_URL=postgres://zwipe:URL_ENCODED_PASSWORD@127.0.0.1/zwipe
BIND_ADDRESS=127.0.0.1:3000
ALLOWED_ORIGINS=https://zwipe.net
RUST_LOG=info,sqlx=warn,zwipe=debug,zerver=debug
RUST_BACKTRACE=1
RESEND_API_KEY=<from Resend dashboard>
RESEND_EMAIL_FROM=support@zwipe.net
# LOG_DIR omitted: defaults to /var/log/zwipe
# MIN_CLIENT_VERSION=0.0.0: the default, which DISABLES the client-version
# gate. A box rebuilt straight from this template ships with the gate off;
# set the real floor before it takes traffic.
# HEALTHCHECK_PING_URL omitted: optional; zervice pings it on a clean run
# so the monitor can alert on silence. Belongs in zervice.env.
# SUPPORT_EMAIL_ADDRESS + WEB_BASE_URL omitted: default to support@zwipe.net
# and https://zwipe.net. Set both here when switching the public domain.
# ACCESS_TOKEN_MINUTES omitted: defaults to 1440 (24 hours). Whole minutes,
# 1 to 1440; anything else refuses startup. Shorten it only once most devices
# run a client that refreshes ahead of expiry (context/plans/access_token_window.md).
```

### RUST_LOG directives

`RUST_LOG` is parsed by `tracing_subscriber::EnvFilter`. It accepts either a bare level (`info`) or comma-separated per-target directives. Production default above silences SQLx query spam while keeping `info` everywhere else and `debug` for our own crates.

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

**The iOS native app is not affected by CORS.** Native apps (Dioxus on iPhone, using `reqwest`) do not send an `Origin` header; CORS is a browser security mechanism. The iOS app will always reach the API regardless of what is in `ALLOWED_ORIGINS`.

For production:
```
ALLOWED_ORIGINS=https://zwipe.net
```

If you also need the web client (`dx serve` on your Mac) to hit the live API during development, add localhost as a second origin:
```
ALLOWED_ORIGINS=https://zwipe.net,http://localhost:8080
```

The value is parsed as `HeaderValue`: no trailing slashes, no wildcards.

---

## Migrations

`query_scalar!` and other SQLx macros verify SQL against real database tables **at compile time**. The database must exist and migrations must have run before `cargo build` will succeed.

```bash
cargo install sqlx-cli --no-default-features --features postgres

cd ~/zwipe-src
set -a; . <(sudo cat /etc/zwipe/migrate.env); set +a
cargo sqlx migrate run --source zerver/migrations
```

**Matview ownership footgun (zervice least privilege, 2026-07-29; bit again 2026-08-14):** the three materialized views (`latest_cards`, `card_signal_rollup`, `otag_context_signal_rollup`) are OWNED by the scoped `zervice` role because `REFRESH` requires ownership (`zcripts/server/sql/zervice_role.sql`). A migration that drops/recreates one of them resets ownership to the migration user (`zwipe`) and the next nightly fails loudly (alert email + Healthchecks), exactly what the `latest_cards_prefer_english` rebuild did on the 2026-08-13 deploy. Hand fix: re-run the ledger (`sudo -u postgres psql -d zwipe < ~/zwipe-src/zcripts/server/sql/zervice_role.sql`).

**Standard footer for any matview-recreating migration**: it makes the rebuild self-healing while keeping grants out of dev/test clusters (conditional on the role existing, so the per-cluster principle holds):

```sql
-- Matview recreated above: hand ownership back to the zervice role where it
-- exists (prod); no-op on dev/test clusters that don't have the role.
DO $$ BEGIN
    IF EXISTS (SELECT FROM pg_roles WHERE rolname = 'zervice') THEN
        ALTER MATERIALIZED VIEW latest_cards OWNER TO zervice;
        GRANT SELECT ON latest_cards TO zwipe;
    END IF;
END $$;
```

(Considered and rejected 2026-08-15: pointing zervice at the main `zwipe` role. zervice parses ~1.7GB of untrusted Scryfall JSON nightly, so the scoped role caps that blast radius at card data and can never read session tokens; its worst failure mode is this loud, one-command-fix alert.)

---

## Build

Build directly on the server; no cross-compilation needed:

```bash
# Install build tools (gcc, make, etc.)
sudo apt install -y build-essential

# The project's .cargo/config.toml specifies linker = "x86_64-unknown-linux-gnu-gcc"
# for the x86_64 target (needed for cross-compiling from macOS).
# On the server, gcc is installed but under a different name: bridge the gap with a symlink:
sudo ln -s /usr/bin/gcc /usr/local/bin/x86_64-unknown-linux-gnu-gcc

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone and build
git clone <repo-url> ~/zwipe-src
cd ~/zwipe-src
cargo build --release --bin zerver --bin zervice

# Install binaries: output is in workspace root target/, not zerver/target/
sudo install -m 755 target/release/zerver target/release/zervice /usr/local/bin/
sudo install -m 755 zcripts/server/systemd/zervice-alert.sh /usr/local/bin/zervice-alert
```

This first build is by hand; every later one comes from the deploy runner.

---

## systemd Service

systemd is Ubuntu's service manager. A unit file tells it how to run zerver, so it starts automatically on boot and restarts itself if it crashes, instead of you running `./zerver` manually in a terminal.

The unit is versioned at `zcripts/server/systemd/zerver.service`; install that file rather than writing one:

```bash
sudo cp ~/zwipe-src/zcripts/server/systemd/zerver.service /etc/systemd/system/
```

It runs `/usr/local/bin/zerver` as the `zerver` user with `/etc/zwipe/zerver.env`, and is sandboxed the same way as heron's unit: `ProtectSystem=strict` and `ProtectHome=true`, with `/var/log/zwipe` the only writable path, plus `NoNewPrivileges`, `MemoryDenyWriteExecute` and the rest. A new path zerver needs to write must be added to `ReadWritePaths=`, or it fails with a read-only filesystem error.

**Register and start the service:**
```bash
sudo systemctl daemon-reload   # tells systemd to pick up the new file
sudo systemctl enable zerver   # start automatically on every boot
sudo systemctl start zerver    # start it right now
sudo systemctl status zerver   # verify it's running
```

What each command does:
- `enable`: registers zerver to start on boot
- `start`: starts it immediately without rebooting
- `Restart=always`: systemd brings zerver back no matter how it exits. It was `on-failure` until 2026-09-13: a startup DB race during an unattended libc upgrade made zerver exit cleanly and stay down for 53 hours (the 09-11 outage), so any exit now restarts. The versioned unit includes it.
- `status`: shows running state and the last few log lines

---

## zervice Scheduling (systemd timer: replaced cron 2026-07-29)

Unit files are versioned at `zcripts/server/systemd/` (`zervice.service`, `zervice.timer`, `zervice-alert.service`, `zervice-alert.sh`) and installed to `/etc/systemd/system/` (the script as `/usr/local/bin/zervice-alert`). zervice runs as the `zervice` user, sandboxed like zerver. Nightly at 04:00 UTC (+ up to 10 min jitter), `Persistent=true` so a missed window (reboot at 4am) fires on next boot.

**Least privilege (2026-07-29):** `zervice.service` reads `/etc/zwipe/zervice.env`: exactly `DATABASE_URL`, `RUST_LOG`, `LOG_DIR`, plus optional `HEALTHCHECK_PING_URL` (the bin's `ZerviceConfig` accepts nothing more; it holds no JWT/Resend secrets). The alert unit runs as the `zervice-alert` user (a member of `systemd-journal`, which is all a journal read needs) under the same sandbox block as zervice, and reads only `/etc/zwipe/alert.env`, the three Resend values. Creating that user is a hand step on the box, once: `sudo useradd --system --no-create-home --shell /usr/sbin/nologin zervice-alert && sudo chown root:zervice-alert /etc/zwipe/alert.env && sudo chmod 640 /etc/zwipe/alert.env`, then reinstall the unit and `sudo systemctl daemon-reload`. Test with `sudo systemctl start zervice-alert.service` and expect the email.

**Scoped Postgres role, the lifecycle.** `zervice.env`'s `DATABASE_URL` connects as the `zervice` role; `zcripts/server/sql/zervice_role.sql` is the canonical, IDEMPOTENT ledger of everything it may touch (card-sync tables, matview ownership, upkeep prunes, incl. the destruction-only session grant: `DELETE` + column-scoped `SELECT (expires_at)`, so it can dust expired sessions but never read them). Grants deliberately do NOT live in migrations (roles are per-cluster infrastructure; dev/test DBs differ). The lifecycle is one command for every case:

```bash
# first time, after adding a table zervice touches, or after a migration
# recreated a matview (ownership resets: next nightly run alerts loudly):
sudo -u postgres psql -d zwipe < ~/zwipe-src/zcripts/server/sql/zervice_role.sql

# first time only: set the password (interactive, never in a file):
sudo -u postgres psql -d zwipe -c "\password zervice"
```

Dev parity: the dev setup/reset scripts run the same file against the local `zerver` DB (throwaway password `zervice`), so a local run as the scoped role proves new grants before prod's nightly can fail on them.

Why systemd over cron: `EnvironmentFile=` replaces the fragile `SHELL=/bin/bash` + `source .env` dance (a dash-vs-bash `source` failure silently ate weeks of runs in mid-2026), early-startup failures land in the journal (`journalctl -u zervice`) instead of a side-channel log, a non-zero exit marks the unit **failed** visibly in `systemctl status zervice`, and `systemctl list-timers zervice*` answers last-ran/next-run at a glance.

**Failure alerting:** `zervice.service` carries `OnFailure=zervice-alert.service` On any failed scheduled run, systemd fires the alert unit, which emails the last 15 journal lines to `SUPPORT_EMAIL_ADDRESS` via Resend (`alert.env` holds the creds; the script sets a User-Agent because Cloudflare 403s python-urllib's default). Tested live 2026-07-29. Note it only fires for systemd-launched runs; a hand-run binary failing alerts nobody. Manual alert test: `sudo systemctl start zervice-alert.service`.

Operate it:

```bash
sudo systemctl start zervice          # run now, journaled
systemctl status zervice              # last result at a glance
journalctl -u zervice --since today   # full output
systemctl list-timers zervice*        # last / next scheduled run
```

The old crontab entry is removed, **for real as of 2026-08-05**. Incident note: this line originally claimed the removal on 2026-07-29, but the user crontab entry survived the migration, so every night ran zervice TWICE (cron at 04:00:01 running the old home-folder binary silently, the timer at 04:00:4x per its jitter). Six nights later the two instances' bulk `card_profiles` UPDATEs interleaved into a Postgres deadlock (40P01) and the systemd instance's step 2 failed, firing the alert. Diagnostics that cracked it: `grep -c "zervice running v" $LOG_DIR/zervice.YYYY-MM-DD.log` (two banners = two instances; both write the shared daily file) and the Postgres deadlock `DETAIL:` block in `/var/log/postgresql/`, which named both queries. The cron instance was invisible to alerting (`OnFailure=` only covers systemd-launched runs) and to `journalctl -u zervice`. If the nightly ever double-runs again, count the banners first.

`/var/log/zwipe/zervice-cron.log` is obsolete (the journal covers early startup); zervice's own rolling files at `$LOG_DIR/zervice.YYYY-MM-DD.log` are unchanged. Both planned follow-ups shipped: the dead-man's switch (`HEALTHCHECK_PING_URL`) and the least-privilege split (`context/archive/zervice_least_privilege.md`).

zervice is a run-once binary: it syncs cards from Scryfall, cleans expired sessions, and exits. Logs are written to `$LOG_DIR/zervice.YYYY-MM-DD.log` (default: `/var/log/zwipe/`).

Run it once to seed card data:
```bash
sudo systemctl start zervice
journalctl -u zervice -f
```

---

## Cloudflare Tunnel + DNS

See [cloudflare.md](cloudflare.md) for full tunnel setup, DNS records, and domain config.

---

## Verify

```bash
curl https://api.zwipe.net/
# {"message":"zerver","status":"ready","version":"<workspace version>"}
```

