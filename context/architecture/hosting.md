# Hosting

**Current: a Hetzner VPS running Ubuntu Server, reached through a Cloudflare Tunnel (since 2026-06-13).**

## What runs there

- **zerver**: the Axum API, a systemd service, serving `api.zwipe.net` through the tunnel.
- **zervice**: the nightly sync (Scryfall cards, oracle tags, derived categories, materialized views, upkeep), run by a systemd timer at 04:00 UTC with a dead-man's-switch check so a missed run is noticed.
- **PostgreSQL 18**: on the same box, listening locally only. zervice connects as a scoped role that can touch the card catalog and nothing else.
- **A self-hosted GitHub Actions runner**: pushes to `main` that pass test and lint build the release binaries on the server, apply migrations and restart zerver (see [`../operations/infrastructure/cicd.md`](../operations/infrastructure/cicd.md)).
- **Nightly database backups** to object storage off the box ([`../operations/infrastructure/backups.md`](../operations/infrastructure/backups.md)).

The site (zite) is not here; it is static and served by GitHub Pages.

## Why this shape

- **No open inbound ports.** The tunnel dials out to Cloudflare, so there is nothing to port-forward and TLS terminates at Cloudflare. The tunnel targets `127.0.0.1`, not `localhost`, which resolves to IPv6 on this host (see [`../operations/infrastructure/cloudflare.md`](../operations/infrastructure/cloudflare.md)).
- **Admin access stays private.** SSH goes over a private network, never the public internet.
- **One box is enough.** Load is small; the database, the API and the sync fit comfortably together, and a single host keeps operations simple. The scoped zervice role and the local-only listener keep the pieces apart.
- **Same stack everywhere.** Ubuntu, systemd, PostgreSQL and cloudflared on every host it has had, so each move was a reinstall from the runbooks, not a redesign.

## History

- **Raspberry Pi 5** at first, until its memory and aarch64 cross-compiling got in the way.
- **A repurposed desktop at home (2026-03-27)** running the same stack.
- **Hetzner VPS (2026-06-13)**: off home power and home internet, with real uptime. The home box was kept as a cold rollback until the VPS settled, then rebuilt as scotland-server, the always-on maintenance box.

## Runbooks

Setup, deploys, the tunnel, backups and the zervice units are in [`../operations/infrastructure/`](../operations/infrastructure/).
