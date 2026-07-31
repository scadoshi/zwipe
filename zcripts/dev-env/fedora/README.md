# Fedora Dev Environment

Brings a fresh Fedora machine up to "can run zwipe" for backend and **web/desktop**
frontend development. iOS builds are macOS-only — see the
[macOS guide](../macos/README.md) for those.

```bash
./zcripts/dev-env/fedora/setup.sh    # first-time / fresh-machine setup
./zcripts/dev-env/fedora/reset.sh    # wipe + reseed the local DB only
```

The script uses `dnf5` and refuses to run if `/etc/os-release` isn't Fedora.

## What setup.sh does (Fedora specifics)

Beyond the [shared end state](../README.md#shared-end-state), the Fedora script:

- Installs the C toolchain and build tooling — `gcc`, `gcc-c++`, `make`,
  `cmake`, `openssl-devel`, `pkg-config`.
- Installs the **Dioxus desktop (WebKitGTK) deps** needed to render the
  frontend natively: `webkit2gtk4.1-devel`, `libxdo-devel`,
  `libappindicator-gtk3-devel`, `gtk3-devel`, `glib2-devel`, `librsvg2-devel`.
- Installs Postgres (`postgresql-server`), runs `postgresql-setup --initdb` on
  first install, and enables + starts it via `systemd`.
- Creates a Postgres role for your Unix user with `--createdb`, then creates
  the `zerver` database owned by that user.

## Running

```bash
cargo run --bin zerver       # backend
cd zwiper && dx serve        # frontend — web hot reload by default
```

`dx serve` defaults to web. To render the native desktop shell instead, use the
desktop target (`dx serve --platform desktop`); the WebKitGTK packages above are
what make that work.

## dx / dioxus version pinning

`setup.sh` installs `dioxus-cli` **pinned** to the `dioxus` crate version in
`zwiper/Cargo.toml` (`DX_VERSION` in the script), so `dx` and the crate stay in
lockstep and you don't hit the "dx and dioxus versions are incompatible"
warning. When you bump `dioxus` in `Cargo.toml`, bump `DX_VERSION` to match.

## reset.sh

Drops and recreates the `zerver` database, regenerates both `.env` files, and
re-applies migrations. It does **not** touch toolchains or system packages, and
prompts before dropping.

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `error: this script is for fedora only` | Wrong distro — use the matching platform script. |
| Frontend build fails on `webkit2gtk` / `gdk` headers | WebKitGTK deps missing — re-run `setup.sh` or install the `*-devel` packages listed above. |
| `dx and dioxus versions are incompatible` | Pin `dx`: `cargo install dioxus-cli --version <Cargo.toml dioxus version>`. |
| `psql: could not connect` | Postgres not running — `sudo systemctl start postgresql`. |
