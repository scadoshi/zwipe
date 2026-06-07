# Dev Environment Setup

One-shot scripts for bringing a fresh machine up to "can run zwipe" — installs
toolchains, sets up Postgres, seeds the database. Plus matching `reset.sh`
helpers for wiping the local DB back to a clean state.

Three platforms supported:

| Platform | Path |
|---|---|
| macOS | `macos/setup.sh`, `macos/reset.sh` |
| Fedora | `fedora/setup.sh`, `fedora/reset.sh` |
| Omarchy (Arch-based) | `omarchy/setup.sh`, `omarchy/reset.sh` |

## When to run

**`setup.sh`** — first time on a new machine, or after a fresh OS install.
Installs Rust toolchain, Dioxus CLI, sqlx-cli, Postgres, then creates the
`zwipe` database and applies migrations. Idempotent — safe to re-run if you
suspect something drifted.

**`reset.sh`** — when you want a clean local DB. Drops + recreates the
`zwipe` database, re-applies migrations, optionally re-seeds Scryfall data.
Does NOT touch toolchains.

## Quickstart

```bash
# macOS
./zcripts/dev-env/macos/setup.sh

# Fedora
./zcripts/dev-env/fedora/setup.sh

# Omarchy / Arch-based
./zcripts/dev-env/omarchy/setup.sh
```

After setup completes:

```bash
cargo run --bin zerver       # start the backend
dx serve                     # start the frontend (web hot reload by default)
```

## What each script does

The platform setup scripts have minor differences in package manager calls
(brew vs dnf vs pacman) but the end state is the same:

1. **Rust toolchain** via rustup, plus targets for whatever platforms you
   build for (wasm32-unknown-unknown for web, aarch64-apple-ios for iOS, etc.)
2. **Cargo tools** — `dioxus-cli`, `sqlx-cli`, `cargo-edit`
3. **Postgres** — installed and started as a service
4. **`zwipe` database** — created with the user from your `.env`
5. **Migrations** — `cargo sqlx migrate run` against the fresh DB
6. **`.env` template** — copied to `zerver/.env` if missing, with sensible
   localhost defaults

Each `setup.sh` is annotated with comments explaining what each step does and
how to skip it if you want to do something custom.

## Related

- `zcripts/latency/` — probe scripts for measuring backend / tunnel latency
  once you have a running app.
