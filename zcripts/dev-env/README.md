# Dev Environment Setup

One-shot scripts for bringing a fresh machine up to "can run zwipe" — installs
toolchains, sets up Postgres, seeds the database. Plus matching `reset.sh`
helpers for wiping the local DB back to a clean state.

Each platform lives in its own directory with its own README covering the
OS-specific packages, prerequisites, and gotchas. **Start with the guide for
your platform:**

| Platform | Guide |
|---|---|
| macOS | [`macos/README.md`](macos/README.md) |
| Fedora | [`fedora/README.md`](fedora/README.md) |
| Omarchy (Arch-based) | [`omarchy/README.md`](omarchy/README.md) |

## setup.sh vs reset.sh

Every platform ships the same two scripts:

- **`setup.sh`** — first time on a new machine, or after a fresh OS install.
  Installs the Rust cargo tools (`dioxus-cli`, `sqlx-cli`), Postgres, and the
  OS build dependencies, then creates the `zerver` database and applies
  migrations. Idempotent — safe to re-run if something drifted.
- **`reset.sh`** — when you want a clean local DB. Drops + recreates the
  `zerver` database, regenerates the `.env` files, and re-applies migrations.
  Does **not** touch toolchains.

## Shared end state

The three `setup.sh` scripts differ only in package-manager calls (brew vs dnf
vs pacman) and platform build deps. They all converge on the same result:

1. **Cargo tools** — `dioxus-cli` (pinned to the `dioxus` crate version in
   `zwiper/Cargo.toml`) and `sqlx-cli`
2. **Postgres** — installed and started as a service
3. **`zerver` database** — created and owned by your local user (peer auth)
4. **Migrations** — `sqlx migrate run` against the fresh DB
5. **`zervice` role** — provisioned for local prod-parity of the SQL grants
6. **`.env` files** — written to `zerver/.env` and `zwiper/.env` with
   localhost defaults

After setup completes:

```bash
cargo run --bin zerver       # start the backend
cd zwiper && dx serve        # start the frontend (web hot reload by default)
```

macOS additionally provisions an iOS Simulator for `dx serve --ios` — see the
[macOS guide](macos/README.md).

## Related

- `zcripts/latency/` — probe scripts for measuring backend / tunnel latency
  once you have a running app.
