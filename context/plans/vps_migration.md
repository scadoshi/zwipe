# VPS migration — moving prod off the home server

**Status: CUTOVER EXECUTED 2026-06-13.** `api.zwipe.net` now serves from a
Hetzner CPX31 (Hillsboro OR, Ubuntu 26.04, PG 18). The home server is stopped
(`zerver` only) but intact as the rollback for ~1–2 weeks. See "Cutover result"
below. Remaining: register CI runners on the VPS + remove home runners, stop
home `cloudflared`/`zynergy`, then decommission home after a clean window.

## Cutover result (2026-06-13)

- **Box:** Hetzner CPX31, tailnet `100.114.251.8`, hostname `zerver-prod`.
  Hardened: `scadoshi` key-only, ufw default-deny + `tailscale0` only (zero
  public ports). Postgres 18, Rust 1.96, sqlx-cli.
- **Services (systemd, boot-enabled, auto-restart):** `zerver` (→ `api.zwipe.net`),
  `zynergy` (synergy worker, `synergy_worker` least-priv role), `cloudflared`
  (tunnel `zwipe-vps`, UUID `2b5d54b3-f05a-47ad-9785-5f7348987618`).
- **Data:** restored from a fresh final `pg_dump` (home zerver stopped first).
  PG 17.10 → 18 clean, 0 errors. 24 users / 37 decks / 1,627 deck_cards /
  115,805 cards / 22 migrations. Verified: login + decks + writes on the live
  mobile client.
- **Cutover mechanism:** new tunnel + proxied-CNAME flip (Phase 1 below), not a
  shared tunnel. Downtime ≈ 2 min (stop home zerver → final dump → restore →
  `cloudflared tunnel route dns --overwrite-dns zwipe-vps api.zwipe.net`).
- **Rollback (still available):** flip `api.zwipe.net` CNAME back to the home
  tunnel `70ba169b-…` + `sudo systemctl start zerver` on home.

### Gotchas hit (fix-forward notes for any future move)

1. **Tunnel ingress must use `http://127.0.0.1:3000`, NOT `localhost`** on an
   IPv6-enabled host. zerver binds `0.0.0.0` (IPv4 only); `localhost` resolves
   to `::1` on the Hetzner box, so cloudflared intermittently hit `::1` →
   connection refused → ~20% **502s**. Home never showed this (no IPv6).
2. **`createuser synergy_worker` on the VPS before restoring** — the dump
   carries `GRANT`s to that role; without it psql logs `role "synergy_worker"
   does not exist` (cosmetic for zerver, but the worker needs the role anyway).
3. **Build JDK / unrelated:** N/A here, but note the box defaulted to Ubuntu
   26.04 + PG 18 (newer than the 24.04/PG17 the runbook assumed) — both fine.

---

### Post-cutover, same day (complete)

- **CI/CD runners** on the VPS: `zerver-prod` (`scadoshi/zwipe`) + `zynergy-prod`
  (`scadoshi/zynergy`), both boot-enabled; home runners removed from GitHub.
  Validated end-to-end: a push to `main` built + migrated + restarted zerver via
  `zerver-prod` → `api.zwipe.net` stayed 200.
- **Crons** moved to the VPS (zervice 4am + backup 5am); home crons disabled;
  cron daemon active+enabled; a manual backup run confirmed dump→R2 from the box.
- **Email** restored with a NEW Resend key (home was already off, so the old key
  couldn't be copied — generated fresh in the Resend dashboard). Verification
  email tested working end-to-end.
- **Sudo least-privilege**: `scadoshi` NOPASSWD scoped to only
  `systemctl {stop,start,restart} {zerver,zynergy}`; all other admin via
  `ssh root@100.114.251.8` (key-only, tailnet). Mac `zerver` alias repointed.
- **JWT_SECRET was rotated (not copied)** — fresh secret on the VPS. Existing
  access tokens died but refresh tokens (DB-stored) survived, so clients
  auto-refreshed; login confirmed seamless on mobile.

**Still open:** verify the first unattended cron runs (next morning); after 1–2
clean weeks, repurpose home (wipes its disk) + rotate the still-shared R2 keys.

---

## Original plan (kept for reference / future moves)

At ~20 users the home server was fine — free, behind the tunnel, backed up
nightly to R2. This plan was executed when the Android push made un-deferring
worthwhile. Todo entry: `context/progress/todo.md`.

## Context — what is actually moving

The home box (Ubuntu Server, i5/32GB, WiFi, `operations/infrastructure/server.md`) runs:

| Component | Notes |
|---|---|
| zerver | systemd unit, binds 0.0.0.0:3000, reached only via Cloudflare Tunnel |
| zervice | nightly cron 4am (Scryfall sync + session cleanup) |
| Postgres | local, least-privilege `zwipe` user |
| Backup script | nightly cron 5am → R2 via rclone (`operations/infrastructure/backups.md`) |
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
| PaaS (Fly/Railway/Render) | varies | **Rejected** — would rewrite the entire ops model (systemd units, crons, self-hosted runner, all of `operations/infrastructure/`). A plain VPS keeps every existing runbook valid; only hostnames change. |

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
round-trips (`../archive/latency_optimization.md`).

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

Follow `operations/infrastructure/server.md` top to bottom — it was written as a rebuild
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
6. **Restore the latest R2 dump** (per `operations/infrastructure/backups.md` "fresh server"
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
   new one on the VPS (`operations/infrastructure/cicd.md` "re-registering after a server
   rebuild" — exact steps exist). Add the `visudo` NOPASSWD systemctl line.
2. Push a trivial change to main; watch the deploy land on the VPS.
3. Run zervice manually once; next morning confirm the 4am log and the 5am
   backup in R2.
4. Enable provider snapshots (Hetzner: ~20% of instance cost).
5. Update docs: `operations/infrastructure/server.md` (VPS specifics, drop WiFi section),
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
