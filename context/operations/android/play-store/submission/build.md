# Build a signed release AAB (Android)

Produce a **signed release `.aab`** ready for the Play Console. To upload it,
continue to [publish.md](publish.md). Listing copy is in
[form_fields.md](form_fields.md); dev-env setup (JDK 21 gotcha, emulator) lives
in [../../setup.md](../../setup.md).

---

## ⚠️ Gotchas (read first — these will bite every release)

1. **dx hardcodes `targetSdk = 34` and `versionCode = 1`** in its generated Gradle
   template, and **regenerates that file on every `dx bundle`**. Google requires
   `targetSdk >= 35`, and every upload needs a **unique, incrementing** `versionCode`.
   → After `dx bundle`, you must **edit the generated `build.gradle.kts` and
   repackage with Gradle directly** (see steps 2–3). Don't re-run `dx bundle`
   after editing — it wipes the edit.
2. **dx does not sign Android release builds** (only Apple codesigning exists).
   The Gradle output `.aab` is **unsigned** — you sign it yourself with the
   upload key (step 4).
3. **R8 minification is ON in release** (off in debug). It can strip WebView/JNI
   classes the app needs. **Always smoke-test the release build** (step 5) before
   rollout — a debug build passing proves nothing about the release.
4. **targetSdk 35 enables edge-to-edge enforcement.** Verify the WebView layout
   isn't drawing critical content under the status/nav bars (checked in step 5).
5. **dx regenerates `MainActivity.kt` too** (bare `class MainActivity :
   WryActivity()`), which closes the app on the OS back gesture. Re-apply the
   back-navigation patch after `dx bundle` (step 1c) or back-swipe ships broken.

---

## Prerequisites (one-time)

- **Upload keystore** at `~/certs/zwipe-upload.jks` (alias `zwipe-upload`, PKCS12).
  Created once with `keytool -genkeypair ... -keystore ~/certs/zwipe-upload.jks
  -alias zwipe-upload -keyalg RSA -keysize 2048 -validity 9125`. The **password
  lives in the password manager** — never in this repo. `~/certs/` is outside the
  repo and covered by the [mac-restore](../../../ios/mac_restore.md) backup.
  *Losing the upload key is recoverable via Play's upload-key reset; losing the
  password isn't fun — keep it.*
- **bundletool** (`brew install bundletool`) — for the smoke test.
- Build env exported (see [../../setup.md](../../setup.md)):
  ```bash
  export ANDROID_HOME="$HOME/Library/Android/sdk"
  export ANDROID_NDK_HOME=$(ls -d "$ANDROID_HOME"/ndk/* | head -1)
  export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"  # JDK 21
  export BACKEND_URL="https://api.zwipe.net"
  ```

---

## 1. Build the release bundle

```bash
cd ~/Developer/zwipe/zwiper
dx bundle --release --platform android --package-types aab
```

This compiles the Rust lib, stages `libmain.so` into the Gradle project's
`jniLibs/`, and produces an AAB targeting SDK **34** (wrong — fixed next).
Generated Gradle project: `target/dx/zwipe/release/android/app/`.

## 1b. Regenerate launcher icons (dx ships its default droid)

dx's generated project uses the **default Android droid** for the launcher icon
(legacy `mipmap-*/ic_launcher.webp` + the adaptive `anydpi-v26` droid-on-green).
Regenerate them from the Zwipe source icon. Like the Gradle edits, this runs
**after `dx bundle`** (which wipes `res/`) and before the repackage:

```bash
zcripts/android/launcher_icons.sh
```

This rewrites the legacy webp at every density and the adaptive foreground (a
full-bleed `icon-1024.png`) + background (solid `#282828`, the icon's bg). Skip
it and the build ships the green droid — testers will notice.

## 1c. Patch the back-navigation handler (dx ships a no-op MainActivity)

dx generates a bare `MainActivity.kt` (`class MainActivity : WryActivity()`),
whose default back handling **closes the app** on the OS back gesture instead of
navigating. Overwrite it with the version that routes back into the Dioxus
router. Like the icons, dx **wipes this on every `dx bundle`**, so run it
**after `dx bundle`** and **before** the Gradle repackage:

```bash
zcripts/android/back_handler.sh
```

Skip it and the edge-swipe / hardware back closes the app from any screen (the
pre-2026-07-09 bug). See [`../../../../plans/back_swipe_gesture.md`](../../../../plans/back_swipe_gesture.md).
R8 keeps the handler (it's used), but the step-5 smoke test is the confirmation.

## 2. Bump targetSdk (and versionCode) in the generated Gradle

```bash
cd ~/Developer/zwipe/target/dx/zwipe/release/android/app
# targetSdk -> 35 (min for Play). compileSdk -> 36 (the installed platform here;
# any compileSdk >= targetSdk works as long as that platform is installed).
# Also bump versionCode for EVERY upload after the first (1 -> 2 -> 3 ...).
perl -i -pe 's/compileSdk = 34/compileSdk = 36/;
             s/targetSdk = 34/targetSdk = 35/;
             s/versionCode = 1/versionCode = <NEXT_CODE>/' app/build.gradle.kts
grep -nE 'compileSdk|targetSdk|versionCode|versionName' app/build.gradle.kts
```

> `compileSdk = 36` resolves to the installed `platforms/android-36.1`. If only
> API 35 is installed, use `compileSdk = 35`. If neither ≥35 is installed, add
> the platform (Android Studio → SDK Manager, or `sdkmanager "platforms;android-35"`
> — note this machine has **no `cmdline-tools`**, so the GUI is the easy path).

## 3. Repackage with Gradle directly (NOT `dx bundle`)

```bash
./gradlew :app:bundleRelease --console=plain
# unsigned output:
ls app/build/outputs/bundle/release/app-release.aab
```

Confirm the target landed:

```bash
bundletool dump manifest --bundle=app/build/outputs/bundle/release/app-release.aab \
  | grep -iE 'targetSdk|versionCode'   # -> targetSdkVersion="35"
```

## 4. Sign with the upload key

```bash
cd ~/Developer/zwipe
jarsigner -keystore ~/certs/zwipe-upload.jks \
  -sigalg SHA256withRSA -digestalg SHA-256 \
  -signedjar zwipe-<VERSION>.aab \
  target/dx/zwipe/release/android/app/app/build/outputs/bundle/release/app-release.aab \
  zwipe-upload
# enter the keystore password when prompted (or -storepass pass:... — avoid leaving it in history)
jarsigner -verify zwipe-<VERSION>.aab   # -> "jar verified."
```

(`self-signed certificate` is expected and fine for an upload key.)

## 5. Smoke-test the RELEASE build on the emulator

Build an installable universal APK from the **signed** AAB and run it — this
exercises the exact minified artifact you're uploading (R8 + edge-to-edge).

```bash
ADB="$ANDROID_HOME/platform-tools/adb"
bundletool build-apks --bundle=zwipe-<VERSION>.aab --output=/tmp/z.apks --overwrite \
  --mode=universal --ks=~/certs/zwipe-upload.jks --ks-key-alias=zwipe-upload
"$ADB" uninstall com.scadoshi.zwipe 2>/dev/null
bundletool install-apks --apks=/tmp/z.apks
"$ADB" shell am start -n com.scadoshi.zwipe/dev.dioxus.main.MainActivity
"$ADB" logcat -d | grep -iE 'FATAL|UnsatisfiedLink' | head   # must be empty
"$ADB" exec-out screencap -p > /tmp/zwipe.png                 # eyeball the layout
```

Check: app launches (no `libmain.so`/R8 crash), and login/home content isn't
clipped under the transparent status/nav bars.

**Next:** upload + roll out → [publish.md](publish.md).
