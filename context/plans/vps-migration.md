# VPS migration — moving prod off the home server

**Status: planned, deliberately deferred (2026-06-10).** At ~20 users the
home server is fine — free, behind the tunnel, backed up nightly to R2.
Execute this plan when a trigger fires: meaningful user growth (Android
launch lands, premium revenue starts), the home server's decommission date
becomes real, or a reliability scare. Until then it's a ready-to-run runbook.
Todo entry: `context/status/todo.md`.

## Context — what is actually moving

The home box (Ubuntu Server, i5/32GB, WiFi, `ops/infra/server.md`) runs:

| Component | Notes |
|---|---|
| zerver | systemd unit, binds 0.0.0.0:3000, reached only via Cloudflare Tunnel |
| zervice | nightly cron 4am (Scryfall sync + session cleanup) |
| Postgres | local, least-privilege `zwipe` user |
| Backup script | nightly cron 5am → R2 via rclone (`ops/infra/backups.md`) |
| cloudflared | tunnel to `api.zwipe.net` — **no port forwarding exists** |
| GitHub Actions runner | self-hosted, builds + migrates + restarts on push to main |
| Tailscale | SSH access (`100.91.55.16`) |

**Not moving:** zite (GitHub Pages), R2 backups (cloud-side, carries over
verbatim), DNS/email (Cloudflare). The database is the only state.

Two facts make this migration gentle:
1. **DNS points at a tunnel, not an IP.** Cutover = repoint the
   `api.zwipe.net` CNAME from the old tunnel to a new one. Instant, proxied,
   instantly reversible.
2. **Nightly R2 dumps already exist.** Restoring one onto the VPS doubles as
   the disaster-recovery rehearsal we've never run.

---

## Options

### Provider

| Option | Price ballpark | Notes |
|---|---|---|
| **Hetzner CPX (recommended)** | €8–15/mo | x86, US locations (Ashburn VA / Hillsboro OR), best price/perf, snapshots cheap |
| DigitalOcean | $24–48/mo for same specs | Slicker dashboard, managed-PG upsell, pricier |
| Vultr | between the two | Fine, no standout |
| PaaS (Fly/Railway/Render) | varies | **Rejected** — would rewrite the entire ops model (systemd units, crons, self-hosted runner, all of `ops/infra/`). A plain VPS keeps every existing runbook valid; only hostnames change. |

### Size and architecture

The constraint is not serving traffic (tiny) — it's that **CI builds the Rust
workspace in release mode on the box**. Guidance:

- **x86_64, not ARM.** The repo's `.cargo/config.toml` linker config, the gcc
  symlink workaround, and the runner binary are all x64. ARM is cheaper but
  converts a copy-paste migration into a porting exercise. Not worth it.
- **8GB RAM** comfortable for release builds (4GB + an 8GB swapfile works but
  builds slow). 3–4 vCPU. ~80GB+ disk — the DB is small (compressed dump is
  ~5–10MB); cargo target/cache dirs are the real disk users.
- Concretely: Hetzner CPX31 (4 vCPU / 8GB / 160GB, US region) or the DO/Vultr
  equivalent.
- Future option if downsizing later: move builds to GitHub-hosted runners and
  scp artifacts — bigger workflow change, out of scope here.

### Managed Postgres?

Skip. It costs as much as the instance again, and co-located Postgres +
nightly R2 dumps + provider snapshots is proportionate to current scale.
Revisit if the DB ever outgrows the box or needs HA.

### Region

US, near the user base (Hetzner Ashburn, or your majority coast). Cloudflare
terminates TLS at the edge regardless; origin RTT still matters for API
round-trips (`ops/latency-optimization.md`).

### JWT_SECRET — copy or rotate?

- **Copy (recommended):** migration is invisible; every session survives.
- **Rotate:** logs out every user (access tokens 24h, refresh 14d all die).
  Cleaner, but pointless pain unless the old box's disposal is uncertain.

Decision can wait until cutover day, but **either way the home disks get
wiped at decommission** — that `.env` is production auth (JWT secret, DB
password, Resend key, R2 keys).

### SSH exposure

Recommend **Tailscale-only**: ufw default-deny inbound (the tunnel is
outbound-only, so the box needs zero public open ports), SSH reachable only
over the tailnet. Strictly better posture than the home server ever had.

---

## Migration plan

### Phase 0 — build the new box (no downtime, no deadline)

Follow `ops/infra/server.md` top to bottom — it was written as a rebuild
checklist and ~all of it applies (skip the WiFi/netplan section; VPS has
real networking). Deltas and order:

1. Provision (Ubuntu LTS, x86_64, 8GB), add your SSH key at creation.
2. Harden: non-root user `scadoshi`, key-only SSH, `ufw default deny incoming`
   + allow only Tailscale (or SSH while bootstrapping), unattended-upgrades.
3. Install stack: Tailscale, Postgres (+user/db), Rust + build-essential +
   the gcc linker symlink, sqlx-cli, rclone (copy `~/.config/rclone/rclone.conf`
   — same R2 remote), cloudflared.
4. `scp` the `.env` over Tailscale; review every value (BIND_ADDRESS and
   ALLOWED_ORIGINS unchanged; MIN_CLIENT_VERSION present; JWT decision).
5. Clone repo, build `zerver`/`zervice`, install the systemd unit,
   create `/var/log/zwipe`.
6. **Restore the latest R2 dump** (per `ops/infra/backups.md` "fresh server"
   steps). This seeds the box AND is the backup-restore fire drill.
7. Create a **new** Cloudflare tunnel (e.g. `zwipe-vps`) and route a temp
   hostname `api-staging.zwipe.net` → `localhost:3000`. Smoke test from the
   Mac: health, register/login, deck CRUD, text import, min-version.
8. Install both crons (zervice 4am, backup 5am — keep the `SHELL=/bin/bash`
   line, see server.md for why). **Leave the backup cron disabled until
   cutover**: both boxes dumping to the same `zwipe-YYYYMMDD.sql.gz` name
   would silently overwrite each other in R2.

### Phase 1 — cutover (minutes of downtime; pick a quiet hour)

Do NOT run old and new as simultaneous connectors on one tunnel — requests
would round-robin across two databases (split-brain writes).

1. Stop zerver on the home box (writes cease; clients fail soft — min-version
   fails open, app surfaces error toasts).
2. Final `pg_dump` on home → copy to VPS over Tailscale → drop/recreate/
   restore (backups.md restore steps).
3. Start zerver on VPS; verify health locally on the box.
4. Flip DNS: `api.zwipe.net` CNAME → the new tunnel ID. Instant under
   Cloudflare proxy.
5. Verify prod from the outside: `/health`, login with a real account, load a
   deck, run an import, `GET /api/client/min-version`.
6. Disable the home box's crons and stop its cloudflared + zerver for good.
   **Leave the box intact** — it is the rollback for the next week or two.
7. Enable the VPS backup cron.

### Phase 2 — CI/CD + follow-through (same day or next)

1. GitHub → Settings → Actions → Runners: remove the old runner, register a
   new one on the VPS (`ops/infra/cicd.md` "re-registering after a server
   rebuild" — exact steps exist). Add the `visudo` NOPASSWD systemctl line.
2. Push a trivial change to main; watch the deploy land on the VPS.
3. Run zervice manually once; next morning confirm the 4am log and the 5am
   backup in R2.
4. Enable provider snapshots (Hetzner: ~20% of instance cost).
5. Update docs: `ops/infra/server.md` (VPS specifics, drop WiFi section),
   `cicd.md` (new Tailscale IP), `cloudflare.md` (new tunnel), delete the
   `api-staging` hostname.

### Phase 3 — decommission (after 1–2 clean weeks)

1. Rollback window closes — see below.
2. Remove the old runner entry, old tunnel, and old Tailscale device.
3. **Wipe the home disks.** They hold the full `.env`. If disposal/wiping is
   at all uncertain, rotate Resend API key, R2 keys, DB password, and (accept
   the logout) JWT_SECRET.

---

## Rollback

Within the window: flip the CNAME back to the old tunnel, restart home
zerver + crons. Caveat: **any data written on the VPS since cutover does not
exist in the home DB** — rollback is for "the VPS is broken within hours,"
not days. After meaningful divergence, fix forward instead; the nightly R2
dumps bound the damage either way.

---

## Decisions to make before starting

- [ ] Provider + region + size (default: Hetzner CPX31, Ashburn)
- [ ] JWT_SECRET copy vs rotate (default: copy, wipe disks later)
- [ ] SSH exposure (default: Tailscale-only, zero public inbound ports)
- [ ] Cutover hour (default: weekday early morning, traffic is lowest)
