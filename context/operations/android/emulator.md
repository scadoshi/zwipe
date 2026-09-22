# Android emulator: daily dev loop

Copy-paste commands for running, wiping, and serving the app to the `Pixel_9a` emulator. First-time machine setup (Android Studio, SDK, NDK) lives in [setup.md](setup.md); release/Play builds in [play-store/submission/build.md](play-store/submission/build.md).

---

## First time on a machine

A Mac that has built Android release bundles may still have no emulator: the release path needs `build-tools`, `ndk`, `platform-tools` and `platforms`, none of which include the emulator or any system image. Symptom is `$ANDROID_HOME/emulator/emulator: no such file or directory` with an otherwise healthy SDK.

Android Studio does not ship `sdkmanager` on the command line, so fetch `cmdline-tools` first. Pull the current filename from Google's manifest rather than guessing the build number, which changes:

```bash
export ANDROID_HOME="$HOME/Library/Android/sdk"
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"

# find the current mac zip
curl -s https://dl.google.com/android/repository/repository2-3.xml \
  | grep -o 'commandlinetools-mac-[0-9]*_latest.zip' | head -1

curl -L -o /tmp/ct.zip https://dl.google.com/android/repository/<that file>
unzip -q /tmp/ct.zip -d /tmp/ct
mkdir -p "$ANDROID_HOME/cmdline-tools"
mv /tmp/ct/cmdline-tools "$ANDROID_HOME/cmdline-tools/latest"

SDKM="$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager"
yes | "$SDKM" --licenses > /dev/null
"$SDKM" "emulator" "system-images;android-36;google_apis;arm64-v8a"

# The AVD is named Pixel_9a because every command below expects that name.
# There is no `pixel_9a` device profile in cmdline-tools, so it rides pixel_9.
"$ANDROID_HOME/cmdline-tools/latest/bin/avdmanager" create avd \
  -n Pixel_9a -k "system-images;android-36;google_apis;arm64-v8a" -d pixel_9
```

Set up on the work Mac 2026-09-22: emulator 37.1.11.0, android-36 google_apis arm64-v8a. Both `sdkmanager` and `avdmanager` print a harmless `line 173: test: : integer expression expected` on every run; ignore it.

---

## 0. Environment: run once per shell (or add to `~/.zshrc`)

The `JAVA_HOME` line is **mandatory**: Gradle's jlink transform dies on the system-default JDK 26 (fails in ~12s). Pointing `PATH` at the SDK lets you call `adb` / `emulator` directly.

```bash
export ANDROID_HOME="$HOME/Library/Android/sdk"
export PATH="$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$PATH"
export ANDROID_NDK_HOME=$(ls -d "$ANDROID_HOME"/ndk/* | head -1)
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"   # JDK 21
export BACKEND_URL="https://api.zwipe.net"
```

---

## 1. Launch the emulator

```bash
# List AVDs
emulator -list-avds

# Normal boot (keeps existing state/storage)
emulator -avd Pixel_9a -netdelay none -netspeed full &

# Fresh WIPE (clears /data — use when installs fail with INSUFFICIENT_STORAGE)
adb emu kill 2>/dev/null                                   # kill the running one first
emulator -avd Pixel_9a -wipe-data -netdelay none -netspeed full &

# Wait until fully booted before installing
adb wait-for-device
until [ "$(adb shell getprop sys.boot_completed | tr -d '\r')" = "1" ]; do sleep 2; done
echo "booted"; adb shell df -h /data | tail -1            # sanity-check free space
```

---

## 2. Put the app on the emulator: two loops

### A. `dx serve`: hot reload

```bash
cd ~/Developer/zwipe/zwiper
dx serve --platform android        # builds, installs, launches, hot-reloads on edits
```

Fastest for iterating on UI. Caveat: it sometimes reuses a stale install instead of reinstalling; if changes don't appear, fall back to the manual loop.

### B. Manual build + install: bulletproof

```bash
cd ~/Developer/zwipe/zwiper
dx build --platform android

APK=~/Developer/zwipe/target/dx/zwipe/debug/android/app/app/build/outputs/apk/debug/app-debug.apk
adb shell am force-stop com.scadoshi.zwipe                 # <-- kills the old process so new code loads
adb install -r "$APK"                                      # -r = keep app data; drop it for a clean install
adb shell am start -n com.scadoshi.zwipe/dev.dioxus.main.MainActivity
adb shell pidof com.scadoshi.zwipe                         # PID must CHANGE; if it's the same, it didn't reload
```

The debug APK is **large (~117 MB)**: unstripped native lib. If `install` returns nothing or fails, see Troubleshooting.

---

## 3. Handy `adb`

```bash
# Screenshot to your Mac
adb exec-out screencap -p > ~/Desktop/zwipe.png

# Screen recording (Ctrl-C to stop, then pull)
adb shell screenrecord /sdcard/zwipe.mp4
adb pull /sdcard/zwipe.mp4 ~/Desktop/

# Crash / app logs
adb logcat -d -b crash | tail -40
adb logcat -d | grep -iE 'FATAL|UnsatisfiedLink|AndroidRuntime' | tail

# App control
adb shell am force-stop com.scadoshi.zwipe                 # stop
adb shell pm clear com.scadoshi.zwipe                      # wipe app data
adb shell dumpsys activity activities | grep topResumedActivity   # what's foregrounded
adb shell dumpsys package com.scadoshi.zwipe | grep -E 'versionName|versionCode'
```

---

## 4. Troubleshooting (the stuff that actually bit us)

| Symptom | Cause | Fix |
|---------|-------|-----|
| `INSTALL_FAILED_INSUFFICIENT_STORAGE` / install returns blank | `/data` is full — the 117 MB debug APK extracts a ~425 MB `.so` | **Wipe the emulator** (§1). `df -h /data` to confirm. |
| Rebuilt but app shows old code; **PID unchanged** | `install -r` + `am start` resumed the running process | `adb shell am force-stop …` before `am start` (or `adb uninstall` + clean `adb install`) |
| Gradle aborts in ~12s (`JdkImageTransform` / `core-for-system-modules`) | Wrong JDK (system default 26) | Set `JAVA_HOME` to the Android Studio JBR 21 (§0) |
| Home-screen ASCII logo glyphs garbled | (historical) CDN font lacked block glyphs | Already fixed — full JetBrains Mono is self-hosted/bundled |

---

## 5. View the "Update required" gate screen

The min-version gate only renders when the build is below the server minimum. To force it for visual work, temporarily flip `zwiper/src/bin/zwipe.rs`:

```rust
// if upgrade_required.required() {
if true || upgrade_required.required() {   // forces the update screen — REVERT before shipping
```

The gate itself (server-driven `MIN_CLIENT_VERSION`) is documented in [`../../README.md`](../../README.md) (see "1.0.5, Min-Version Gate").
