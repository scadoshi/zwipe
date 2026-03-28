# Hosting Decision

**Decided: Ubuntu Server (headless) via Cloudflare Tunnel (2026-03-27)**

Previous host was a Raspberry Pi 5 (4GB RAM). Moved to a proper server — 32GB RAM, repurposed from a desktop build (GPU removed before OS install). Overkill for current load but gives real headroom and removes Pi's memory/aarch64 constraints.

---

## What We're Running

- **Hardware**: Ubuntu Server (headless, no desktop UI), x86_64, 32GB RAM
- **OS**: Ubuntu Server — no GUI, managed entirely via SSH
- **Backend**: `zerver` as a systemd service
- **Database**: PostgreSQL — `zwipe` DB, `zwipe` user
- **Tunnel**: Cloudflare Tunnel → `api.zwipe.net` routes to `localhost:3000`
- **Nightly sync**: `zervice` via cron at 4am

## Why Ubuntu Server (headless)

- 32GB RAM vs Pi's 4GB — real headroom for DB + backend under actual load
- x86_64 eliminates cross-compilation to aarch64 — `cargo build` on the server itself is viable, or cross-compile Mac → x86_64-unknown-linux-gnu
- No UI needed — everything managed via SSH. Desktop environment would be wasted RAM on a server
- Same stack (systemd, PostgreSQL, cloudflared) — migration is a clean reinstall, not a redesign

## Cross-Compilation (Mac → x86_64-unknown-linux-gnu)

```bash
rustup target add x86_64-unknown-linux-gnu
cargo zigbuild --release --bin zerver --bin zervice --target x86_64-unknown-linux-gnu
scp target/x86_64-unknown-linux-gnu/release/zerver zervice <user>@<server-ip>:~/zwipe/
ssh <user>@<server-ip> "sudo systemctl restart zerver"
```

Note: Update `.cargo/config.toml` linker config if it still points at aarch64 toolchain.

## Key Config (carry over from Pi — update IPs/paths as needed)

- Tunnel config: `/etc/cloudflared/config.yml` on server
- zerver .env: `~/zwipe/.env` on server
- DATABASE_URL uses `127.0.0.1` (TCP), not `localhost` (socket) — peer auth blocks socket for non-system users
- `<` and `>` in DB password URL-encoded as `%3C` / `%3E` in connection string

## Status (2026-03-27)

- [ ] Disassemble desktop, remove GPU, reassemble
- [ ] Boot with Ubuntu Server USB installer (headless install)
- [ ] Install PostgreSQL, create `zwipe` DB + user
- [ ] Install `cloudflared`, configure tunnel to `api.zwipe.net`
- [ ] Deploy zerver + zervice binaries (cross-compiled x86_64 from Mac or built on server)
- [ ] Configure zerver `.env`, run SQLx migrations
- [ ] Start `zerver` systemd service, add `zervice` cron
- [ ] Create `/var/log/zwipe/` for rolling log files
- [ ] Run `zervice` once manually to seed Scryfall card data
- [ ] Verify iOS app hits `api.zwipe.net` successfully

## Full Step-by-Step Reference

See `context/project/shipping.md` — update that file with x86_64 steps once migration is complete.
