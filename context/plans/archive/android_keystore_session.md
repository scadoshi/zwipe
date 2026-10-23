# Android session file under the Keystore

**Status: PASSED on the emulator 2026-10-23; ready to merge.**

**One sentence:** on Android the session file in app-private storage is now AES-256-GCM ciphertext under a key the Android Keystore holds, instead of plain JSON.

## Why

`keyring` has no Android backend, so the session (access token, refresh token, user) is written to `/data/data/com.scadoshi.zwipe/files/session.json`. The backup opt-out in PR #28 keeps the file on the device, but anyone who reads the app's files dir (a rooted phone, a bad backup, a forensic pull) reads the tokens in the clear. iOS and desktop keep the session in the OS keyring; this gives Android the nearest thing it has.

## What changed

- `zwiper/src/lib/outbound/android_keystore.rs` (new): `seal(plain) -> blob` and `open(blob) -> plain`. On first use it generates an AES-256 key in the Android Keystore under the alias `zwipe-session` (purposes encrypt and decrypt, block mode GCM, no padding, no user authentication required, so it works with the screen locked) through JNI to `KeyGenParameterSpec.Builder` and `KeyGenerator`. Sealing and opening go through `Cipher` "AES/GCM/NoPadding"; the file holds `iv || ciphertext || tag` with a 12-byte iv and a 128-bit tag. The key never leaves the Keystore. A Java exception inside a call is cleared and comes back as the error text, so the VM stays usable.
- `zwiper/src/lib/outbound/session.rs` (Android arm only): `save` seals before writing. `load` reads the file and looks at the first byte: a `{` means plain JSON from a build before this one, which it parses and rewrites sealed, so an existing login survives the update. Anything else is opened through the Keystore; if that fails (the key is gone after a reinstall, or the file is corrupt) or the plaintext does not parse, the file is removed and the app starts logged out. No panics on any path. Expiry handling is unchanged.
- `zwiper/src/lib/outbound/mod.rs`: registers the module behind `#[cfg(target_os = "android")]`.

No new crates: `jni` and `ndk-context` were already Android dependencies. iOS, desktop and web are untouched.

Checked here: `cargo +nightly fmt --all --check`, host `cargo clippy -p zwiper --all-targets -- -D warnings`, and `cargo clippy -p zwiper --target aarch64-linux-android --features mobile -- -D warnings` against NDK r28c. The emulator run has not happened yet; that is the list below.

## Emulator test plan

Environment per [`../operations/android/emulator.md`](../../operations/android/emulator.md) section 0, with the `Pixel_9a` AVD booted. The app's Rust log goes to logcat under the `RustStdoutStderr` tag, so keep this running in a second terminal for the whole session:

```bash
adb logcat -s RustStdoutStderr | grep -E 'keystore|session'
```

Lines to expect: `android keystore: generating session key zwipe-session` once per install, `session: sealing a plaintext session file` once on the upgrade path, and `session: sealed file does not open, treating as logged out` only in the reinstall step. A `failed to save session` or `failed to load session` line at any other point is a failure.

The file check used throughout:

```bash
adb shell run-as com.scadoshi.zwipe cat files/session.json | head -c 64 | xxd
```

Ciphertext starts with the 12 random iv bytes; a plaintext file starts with `7b` (`{`).

1. Build this branch: `git checkout android-keystore`, then from `zwiper/` run `dx build --platform android`. The APK is `target/dx/zwipe/debug/android/app/app/build/outputs/apk/debug/app-debug.apk`; call it `$APK` below.
2. Fresh install and login: `adb uninstall com.scadoshi.zwipe` (ignore a failure if it is not installed), `adb install "$APK"`, `adb shell am start -n com.scadoshi.zwipe/dev.dioxus.main.MainActivity`. Log in. Logcat shows the `generating session key` line. Run the file check: the first byte is not `7b` and the bytes look random.
3. Kill and relaunch: `adb shell am force-stop com.scadoshi.zwipe`, then `am start` as above. The app opens straight to the logged-in home screen with no login prompt, and logcat shows no `session:` warning. `adb shell pidof com.scadoshi.zwipe` differs from before the stop.
4. Upgrade path: `git checkout main`, `dx build --platform android` from `zwiper/`, `adb uninstall com.scadoshi.zwipe`, `adb install "$APK"`, launch and log in. The file check shows `7b` first. Now `git checkout android-keystore`, `dx build --platform android`, `adb shell am force-stop com.scadoshi.zwipe`, `adb install -r "$APK"` (keeps app data), `am start`. The app is still logged in, logcat shows `sealing a plaintext session file` then `generating session key`, and the file check no longer starts with `7b`. Force-stop and `am start` once more to confirm the sealed file loads.
5. Lock the screen and relaunch: give the emulator a PIN so the lock is real, `adb shell locksettings set-pin 1234`, then `adb shell input keyevent KEYCODE_SLEEP`. With the screen off, `adb shell am force-stop com.scadoshi.zwipe` and `am start`. Logcat shows no `session:` warning while the screen is locked. Wake and unlock (`adb shell input keyevent KEYCODE_WAKEUP`, swipe up, enter 1234): the app is logged in. Afterwards `adb shell locksettings clear --old 1234`.
6. Uninstall and reinstall: `adb uninstall com.scadoshi.zwipe`, `adb install "$APK"`, `am start`. The app starts at the login screen with no crash (`adb logcat -d -b crash | tail` is empty for the package). The Keystore key went with the uninstall, so there was no file to open; `run-as ... cat files/session.json` says no such file. Log in again and the `generating session key` line appears a second time.
7. Corrupt file: while logged in, `adb shell am force-stop com.scadoshi.zwipe`, `adb shell run-as com.scadoshi.zwipe sh -c 'head -c 40 /dev/urandom > files/session.json'`, `am start`. Logcat shows `sealed file does not open, treating as logged out`, the app shows the login screen, and the file is gone.
8. Logout still clears the file: log in, log out from the settings screen, then the file check reports no such file.

If every step passes, tick the Android line in [`security_audit_followups.md`](../security_audit_followups.md), move this plan to `archive/`, and push the branch for a PR.
