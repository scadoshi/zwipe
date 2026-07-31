# macOS Dev Environment

Brings a fresh Mac up to "can run zwipe on the iOS Simulator." Apple Silicon
(`arm64`) is the assumed and tested target.

```bash
./zcripts/dev-env/macos/setup.sh     # first-time / fresh-machine setup
./zcripts/dev-env/macos/reset.sh     # wipe + reseed the local DB only
```

## Prerequisites: full Xcode (not just Command Line Tools)

zwiper is a mobile app; `dx serve` builds for the iOS Simulator target
(`aarch64-apple-ios-sim`). That target needs the **full Xcode app** plus the
iOS Simulator SDK and runtime. The Command Line Tools alone are **not enough** —
a CLT-only machine builds fine right up until `cc` shells out to `xcrun` and
fails with:

```
xcrun: error: SDK "iphonesimulator" cannot be located
```

`setup.sh` guards against this: it verifies `xcrun --show-sdk-path --sdk
iphonesimulator` resolves and, if not, exits with the fix steps rather than
letting the failure surface later inside a Rust build.

### One-time Xcode setup

If `setup.sh` reports the SDK is missing, do this once, then re-run it:

```bash
# 1. install Xcode from the App Store (large download)
# 2. point the toolchain at it (CLT install leaves it aimed at CommandLineTools)
sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
sudo xcodebuild -license accept
# 3. install the remaining first-launch components (CoreSimulator, etc.)
sudo xcodebuild -runFirstLaunch
# 4. download the iOS platform + Simulator runtime (~7GB)
xcodebuild -downloadPlatform iOS
```

Verify both of these succeed before rebuilding:

```bash
xcrun --show-sdk-path --sdk iphonesimulator   # prints a path to an .sdk
xcrun simctl list runtimes                     # lists an "iOS <n>" runtime
```

> If `-downloadPlatform` warns about `CoreSimulator.framework ... no such
> file`, the first-launch components didn't finish installing — run
> `sudo xcodebuild -runFirstLaunch` and retry the download.

## What setup.sh does (macOS specifics)

Beyond the [shared end state](../README.md#shared-end-state), the macOS script:

- Requires the `iphonesimulator` SDK (exits early with fix steps otherwise).
- Provisions a **6.5" iPhone 11 Pro Max** Simulator device if one doesn't
  already exist (idempotent — re-running won't create duplicates).
- Installs Postgres via Homebrew (`postgresql@15`) and starts it as a service.
- Uses **peer auth** — the DB is owned by your macOS user, no password.

## Running on the iOS Simulator

`dx serve --ios` installs into whatever Simulator is **booted**, so boot one
first:

```bash
open -a Simulator                     # boots the default device
cd zwiper && dx serve --ios
```

To target the 6.5" device specifically:

```bash
xcrun simctl boot "iPhone 11 Pro Max"
open -a Simulator                     # bring the window forward
cd zwiper && dx serve --ios
```

The Simulator persists across reboots — you only create it once (setup.sh does
this for you); afterward just `boot` it.

### Managing Simulator devices

```bash
xcrun simctl list devices available        # what exists, and boot state
xcrun simctl list devicetypes | grep Max   # available device models
# create another size (runtime omitted → newest installed is used):
xcrun simctl create "iPhone 16 Pro Max" "iPhone 16 Pro Max"
xcrun simctl shutdown "iPhone 11 Pro Max"  # free resources when done
```

## dx / dioxus version pinning

`setup.sh` installs `dioxus-cli` **pinned** to the `dioxus` crate version in
`zwiper/Cargo.toml` (`DX_VERSION` in the script). An unpinned `cargo install
dioxus-cli` pulls the newest published version — including prereleases — which
produces this at serve time:

```
🚫 dx and dioxus versions are incompatible!
  • dx version: 0.8.0-alpha.1
  • dioxus versions: [0.7.10]
```

It still builds, but to align an already-installed `dx`, match it by hand:

```bash
cargo install dioxus-cli --version 0.7.10   # use the version from Cargo.toml
```

When you bump `dioxus` in `Cargo.toml`, bump `DX_VERSION` in `setup.sh` to match.

## reset.sh

Drops and recreates the `zerver` database, regenerates both `.env` files, and
re-applies migrations. It does **not** touch Xcode, Homebrew, the Rust
toolchain, or your Simulators — it's purely a local-database reset. It prompts
before dropping.

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `SDK "iphonesimulator" cannot be located` | CLT-only, no full Xcode — see [one-time Xcode setup](#one-time-xcode-setup). |
| `No iOS sdks installed` | Simulator runtime missing — `xcodebuild -downloadPlatform iOS`. |
| `No devices are booted` (exit 148) | Boot a sim first — `open -a Simulator`. |
| `CoreSimulator.framework ... no such file` | Run `sudo xcodebuild -runFirstLaunch`, then retry. |
| `dx and dioxus versions are incompatible` | Pin `dx` — see [version pinning](#dx--dioxus-version-pinning). |
