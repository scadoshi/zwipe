# Android Development Setup

## Prerequisites

Install Android Studio (includes SDK, NDK, emulator):

```bash
brew install --cask android-studio
```

Open Android Studio, complete the setup wizard. Then install the NDK:
- Settings > Languages & Frameworks > Android SDK > SDK Tools tab
- Check "NDK (Side by side)" > Apply

## Environment

Add to `~/.zshrc`:

```bash
export ANDROID_HOME="$HOME/Library/Android/sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/<version>"
```

Find the version with `ls ~/Library/Android/sdk/ndk/`.

### JDK: must be 17/21, NOT the system default (gotcha)

The Android Gradle Plugin's `jlink`/`core-for-system-modules` transform **fails on
JDK 26** (Temurin 26 is the Homebrew default on this Mac). Symptom: Gradle aborts in
~12s with `Could not resolve ... core-for-system-modules.jar` /
`Execution failed for JdkImageTransform`. It is **not** slow — it never builds.

Fix: point the build at Android Studio's bundled JBR 21:

```bash
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
```

Required every Android build until Temurin 26 is no longer the default `java`
(`/usr/libexec/java_home -V` lists what's installed). Verified working with JBR
21.0.9 against Xcode-independent Gradle 9.x on 2026-06-22.

## Create an emulator (AVD)

One-time, in Android Studio: **Tools > Device Manager > Create Virtual Device >**
pick a phone (e.g. Pixel 9a) > download a system image > Finish.

## Running, wiping & serving to it

The day-to-day loop — launch/wipe the emulator, `dx serve` vs build-and-install,
`adb` helpers, and troubleshooting — lives in **[emulator.md](emulator.md)**,
kept separate so this page stays first-time-setup only.

## Notes

- Same Rust codebase as iOS — one `zwiper/` directory, different build targets
- `BACKEND_URL` is baked in at compile time via `env!()`
- Android uses WebView for rendering (vs iOS WKWebView) — CSS rendering may differ slightly
- Known issues tracked in `context/progress/todo.md` under Android section

## Releasing to the Play Store

This page is **dev/emulator** setup only. To build a signed release `.aab` and
submit it, follow [play-store-submission/build-and-submit.md](play-store-submission/build-and-submit.md)
— it covers the dx gotchas (hardcoded `targetSdk = 34` / `versionCode = 1`,
unsigned release output), signing with the upload key, the R8 + edge-to-edge
smoke test, and the Console rollout steps. Listing copy: [play-store-submission/form_fields.md](play-store-submission/form_fields.md).
