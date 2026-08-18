# Testing on a physical Android device

Set up 2026-08-17 to root-cause the ndk-context crash. Turned a bug that had
survived five releases into a seconds-per-iteration loop, so reach for this
before shipping another blind fix.

## Connecting

Tooling lives in the SDK but is not on `PATH`:

```bash
ADB=~/Library/Android/sdk/platform-tools/adb
AAPT=~/Library/Android/sdk/build-tools/34.0.0/aapt2
$ADB devices -l
```

On the phone: **Settings → About phone →** tap Build number ×7, then
**Developer options → USB debugging → ON**, then accept the authorization
prompt.

**Diagnosing "no devices found":** `system_profiler SPUSBDataType` returns
empty on this Mac (a sandbox quirk) — it is NOT evidence the cable is bad. Use
`ioreg -p IOUSB -w0 | grep -i pixel` instead. If the phone shows there but
`adb` sees nothing, USB debugging is off. The "Use USB for" mode reverting to
*No data transfer* is normal and does not block ADB.

## Installing a test build

The store build is Play-signed, so a locally-built APK will not install over it
(`INSTALL_FAILED_UPDATE_INCOMPATIBLE`) — uninstall first. That wipes local app
state (session, hints, prefs); decks are server-side and survive a re-login.

Fastest loop that carries **release** native libs in a debug-signed APK:

```bash
dx bundle --release --platform android --package-types aab --package zwiper
zcripts/android/patch_bundle.sh          # icons + back handler + manifest
# dx regenerates build.gradle.kts too — re-apply, and bump versionCode above
# whatever is installed or the install is rejected as a downgrade
perl -i -pe 's/compileSdk = 34/compileSdk = 36/; s/targetSdk = 34/targetSdk = 36/;
             s/versionCode = 1$/versionCode = <NEXT>/' \
  target/dx/zwipe/release/android/app/app/build.gradle.kts
cd target/dx/zwipe/release/android/app
ANDROID_HOME=~/Library/Android/sdk \
JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home" \
  ./gradlew :app:assembleDebug --console=plain
$ADB install -r app/build/outputs/apk/debug/app-debug.apk
```

Gradle needs both env vars: without `JAVA_HOME` it reports "Unable to locate a
Java Runtime" (use Android Studio's bundled JBR), and without `ANDROID_HOME`
it fails with "SDK location not found".

**Verify the patches actually shipped before testing** — this is the check that
distinguishes "the fix does not work" from "the fix was not in the build":

```bash
unzip -o "$APK" 'classes*.dex' -d /tmp/apkchk
strings /tmp/apkchk/*.dex | grep -c 'zwipe:back'      # back handler
strings /tmp/apkchk/*.dex | grep -c '^killProcess$'   # onDestroy kill
$AAPT dump xmltree --file AndroidManifest.xml "$APK" | grep -iE 'launchMode|configChanges'
```

## Driving the app

The app is one WebView Activity, so `dumpsys` cannot tell you which *screen*
is showing — screenshot and look instead.

```bash
$ADB exec-out screencap -p > shot.png     # then read the image
$ADB shell wm size                        # 1080x2400 on the Pixel 6
$ADB shell input tap <x> <y>              # coordinates in DEVICE pixels
$ADB shell input swipe 540 1800 540 700 400
$ADB shell input keyevent KEYCODE_BACK    # hardware/nav-bar back
$ADB shell pidof com.scadoshi.zwipe       # pid change = process restarted
```

Screenshots come back scaled for viewing; multiply by the ratio the viewer
reports before feeding coordinates to `input tap`.

## Useful triggers

```bash
$ADB shell cmd uimode night yes|no                    # config-change recreate
$ADB shell settings put global always_finish_activities 1   # don't keep activities
$ADB shell am start -n com.scadoshi.zwipe/dev.dioxus.main.MainActivity
$ADB shell monkey -p com.scadoshi.zwipe -c android.intent.category.LAUNCHER 1
```

`am start -n` is an **explicit component start** and behaves differently from
the launcher intent — that distinction is what exposed the ndk-context crash.
`pidof` before and after is the cheapest signal for "did the process die".

## What this has been used for

- **ndk-context crash**: root-caused and fixed, verified fixed on device
  ([`../../plans/archive/android_ndk_context_crash.md`](../../plans/archive/android_ndk_context_crash.md))
- **Back-swipe overlay fixes**: verified with `KEYCODE_BACK` across the format
  picker, a bottom sheet, and the nested picker→dictionary case
  ([`../../plans/archive/back_swipe_audit.md`](../../plans/archive/back_swipe_audit.md))
