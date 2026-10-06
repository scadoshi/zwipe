# Ubuntu Dev Environment

Brings a fresh Ubuntu machine up to "can run zwipe" for backend and **web/desktop** frontend development. iOS builds are macOS-only, see the [macOS guide](../macos/README.md) for those.

```bash
./zcripts/dev-env/ubuntu/setup.sh   # first-time / fresh-machine setup
./zcripts/dev-env/ubuntu/reset.sh   # wipe + reseed the local DB only
```

Run both from the repo root. The scripts use `apt` and refuse to run unless `/etc/os-release` reports `ID=ubuntu`.

## What setup.sh does (Ubuntu specifics)

Beyond the [shared end state](../README.md#shared-end-state), the Ubuntu script:

- Installs `build-essential`, `libssl-dev` and `pkg-config` for the C build toolchain.
- Installs the **Dioxus desktop (WebKitGTK) deps** needed to render the frontend natively: `libwebkit2gtk-4.1-dev`, `libxdo-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libgtk-3-dev`.
- Installs **Postgres 18** (`PG_VERSION` in the script), the major prod and CI run. Ubuntu 26.04 ships it; older releases default lower, so the script adds the postgresql.org apt repo to get it. If that repo has no build for your release yet, it falls back to Ubuntu's default `postgresql` and says so.
- Enables and starts Postgres via `systemd`, creates a Postgres role for your Unix user with `--createdb`, then creates the `zerver` database owned by that user.

`apt-get install` skips what's already present, so re-running is safe.

## Running

```bash
cargo run --bin zerver       # backend
cd zwiper && dx serve        # frontend, web hot reload by default
cargo run --bin zervice      # nightly sync, run once to load the card catalog
```

zervice reads its env explicitly rather than from `.env` (see `ZerviceConfig`), so source it first: `set -a; . zerver/.env; set +a`.

`dx serve` defaults to web. To render the native desktop shell instead, use `dx serve --platform desktop`; the WebKitGTK packages above are what make that work.

## dx / dioxus version pinning

`setup.sh` installs `dioxus-cli` **pinned** to the `dioxus` crate version in `zwiper/Cargo.toml` (`DX_VERSION` in the script). When you bump `dioxus` in `Cargo.toml`, bump `DX_VERSION` here and in the other platforms' scripts.

## reset.sh

Drops and recreates the `zerver` database, regenerates both `.env` files, and re-applies migrations. It does **not** touch toolchains or system packages, and prompts before dropping.

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `error: this script is for ubuntu only` | Wrong distro, use the matching platform script. |
| `warning: postgresql-18 unavailable for this release` | The postgresql.org repo hasn't published for your Ubuntu release yet; you got Ubuntu's default major instead. Fine for local work, but prod runs 18. |
| Frontend build fails on `webkit2gtk` / `gdk` headers | WebKitGTK deps missing, re-run `setup.sh` or install the packages listed above. |
| `dx and dioxus versions are incompatible` | Pin `dx`: `cargo install dioxus-cli --version <Cargo.toml dioxus version> --locked --force`. |
| dx install fails on `auth-git2` / `credential_helper` | Missing `--locked`; cargo pulled `git2 0.21`. Re-run the install with `--locked`. |
| `psql: could not connect` | Postgres not running: `sudo systemctl start postgresql`. |
