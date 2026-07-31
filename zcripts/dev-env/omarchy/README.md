# Omarchy (Arch) Dev Environment

Brings a fresh Arch-based (Omarchy) machine up to "can run zwipe" for backend
and **web/desktop** frontend development. iOS builds are macOS-only — see the
[macOS guide](../macos/README.md) for those.

```bash
./zcripts/dev-env/omarchy/setup.sh   # first-time / fresh-machine setup
./zcripts/dev-env/omarchy/reset.sh   # wipe + reseed the local DB only
```

The script uses `pacman` and refuses to run unless `/etc/os-release` reports
`ID=arch`.

## What setup.sh does (Arch specifics)

Beyond the [shared end state](../README.md#shared-end-state), the Omarchy script:

- Installs `base-devel` plus `openssl` and `pkgconf` for the C build toolchain.
- Installs the **Dioxus desktop (WebKitGTK) deps** needed to render the
  frontend natively: `webkit2gtk-4.1`, `xdotool`, `libappindicator-gtk3`,
  `gtk3`, `glib2`, `librsvg`.
- Installs Postgres, runs `initdb -D /var/lib/postgres/data` on first install,
  and enables + starts it via `systemd`.
- Creates a Postgres role for your Unix user with `--createdb`, then creates
  the `zerver` database owned by that user.

All package installs use `--needed --noconfirm`, so re-running is safe and
won't re-fetch what's already present.

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
| `error: this script is for arch linux (omarchy) only` | Wrong distro — use the matching platform script. |
| Frontend build fails on `webkit2gtk` / `gdk` headers | WebKitGTK deps missing — re-run `setup.sh` or install the packages listed above. |
| `dx and dioxus versions are incompatible` | Pin `dx`: `cargo install dioxus-cli --version <Cargo.toml dioxus version> --locked --force`. |
| dx install fails on `auth-git2` / `credential_helper` | Missing `--locked` — cargo pulled `git2 0.21`; re-run the install with `--locked`. |
| `psql: could not connect` | Postgres not running — `sudo systemctl start postgresql`. |
