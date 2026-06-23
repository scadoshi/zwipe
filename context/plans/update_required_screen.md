# Update-required screen redesign — verify on Android

**Status: WIP on branch `wip/update-required-screen` (pushed to origin).** The
redesign is done and **verified correct at the binary level**, but **not yet
visually confirmed on the Android emulator** — local re-deploy was finicky (see
gotcha below), so the on-device look is unconfirmed.

## What's done

- **`zwiper` — `update_required.rs` redesigned** to the standard page layout:
  green `page-header` "Update required" → message in a `.card` box whose header
  is a **red, pulsing, all-caps "UPDATE REQUIRED"** (`--color-error` overridden to
  `#ff3030`, reusing the `update-required-title` keyframe) with a full-bleed `hr`
  (`box-rule`) under it → **store button after the card** → blank `util-bar` footer.
- **Store links route through `zwipe.net/download/*`** (platform `cfg` split) so the
  destination is controlled from the site, never an app update:
  - Android → `https://zwipe.net/download/android`, button **"Open Play Store"**
  - iOS → `https://zwipe.net/download/ios`, button **"Open App Store"**
- **`zite` — added `/download/ios`** (`pages/ios.rs`) that immediately redirects to
  the live App Store listing (meta-refresh + `eval` fallback + visible link).
  `/download/android` is still the "pending" landing page until the Play listing
  is public.

## Must-do before this ships

- [ ] **Visually verify the redesigned screen on the Android emulator** — header,
  red pulsing card title + `hr`, message, **"Open Play Store"** button, blank
  footer. The compiled `libmain.so` strings already confirm `Open Play Store` +
  `zwipe.net/download/android` (and *no* iOS strings), so the binary is right — we
  just never got a clean on-device capture.
- [ ] **Deploy `zite`** so `/download/ios` is live (it 404s until then).
- [ ] **Next iOS build** picks up the new `zwipe.net/download/ios` URL (the shipped
  iOS client still has the old direct App Store link — fine, it works).
- [ ] **Revert the temp force-route** before any real build/ship (see below).

## How to test the screen (temp force-route)

`zwiper/src/bin/zwipe.rs` has the gate forced on for review:

```rust
// TEMP (revert): `true ||` forces the update screen for visual review.
if true || upgrade_required.required() {
```

This routes the app straight to the update screen on launch. **Revert the
`true ||` before shipping.** (Alternative without the hack: set a high
`MIN_CLIENT_VERSION` on a test server and let the client poll detect it.)

## Emulator re-deploy gotcha (why it was finicky)

`adb install -r` + `am start` **resumed the old running process** (same PID)
instead of loading the new code — so rebuilds appeared to "not take." Also the
**122 MB debug APK** install is slow/flaky on the emulator. Reliable loop:

```bash
adb shell am force-stop com.scadoshi.zwipe
adb uninstall com.scadoshi.zwipe
adb install <app-debug.apk>          # NOT -r; clean install
adb shell am start -n com.scadoshi.zwipe/dev.dioxus.main.MainActivity
adb shell pidof com.scadoshi.zwipe   # PID should CHANGE — if same, it didn't reload
```

`dx serve --platform android` would be the smoother hot-reload loop if it
cooperates (it had its own stale-install quirks earlier this session).
