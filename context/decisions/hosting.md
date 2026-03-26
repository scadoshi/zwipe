# Hosting Decision

**Decided: Raspberry Pi 5 via Cloudflare Tunnel (2026-03-26)**

---

## What We're Running

- **Hardware**: Raspberry Pi 5, 4GB RAM, Debian 12 Bookworm, aarch64, local IP `10.0.0.78`
- **Backend**: `zerver` as a systemd service (`/home/scottyfermo/zwipe/zerver`)
- **Database**: PostgreSQL on the Pi — `zwipe` DB, `zwipe` user
- **Tunnel**: Cloudflare Tunnel → `api.zwipe.net` routes to `localhost:3000`
- **Nightly sync**: `zervice` via cron at 4am

## Why Pi + Cloudflare Tunnel

- Already owned hardware, $0/month hosting cost
- Cloudflare Tunnel avoids port forwarding, dynamic IP issues, and handles TLS at the edge
- POC approach — if load ever becomes a real problem, migrate to a VPS with same stack

## Key Config

- Tunnel ID: `70ba169b-8293-4a60-9b2d-e1f996a161db`
- Tunnel config: `/etc/cloudflared/config.yml` on Pi
- zerver .env: `/home/scottyfermo/zwipe/.env` on Pi
- DATABASE_URL uses `127.0.0.1` (TCP), not `localhost` (socket) — peer auth blocks socket for non-system users
- `<` and `>` in DB password URL-encoded as `%3C` / `%3E` in connection string

## Cross-Compilation (Mac → aarch64)

```bash
rustup target add aarch64-unknown-linux-gnu
brew install messense/macos-cross-toolchains/aarch64-unknown-linux-gnu
brew install zig && cargo install cargo-zigbuild
cargo zigbuild --release --bin zerver --bin zervice --target aarch64-unknown-linux-gnu
scp target/aarch64-unknown-linux-gnu/release/zerver zervice scottyfermo@10.0.0.78:~/zwipe/
ssh scottyfermo@10.0.0.78 "sudo systemctl restart zerver"
```

`cargo-zigbuild` required because reqwest uses `rustls-tls` (pure Rust TLS, no OpenSSL headers). `.cargo/config.toml` has the aarch64 linker configured.

## Full Step-by-Step Reference

See `context/project/shipping.md`.
