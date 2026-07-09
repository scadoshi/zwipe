# Android release build + Play submission

How to produce a **signed release `.aab`** and get it into the Play Console.
The Android analogue of [ios/appstore_update.md](../../ios/appstore_update.md).

First-time listing copy lives in [form_fields.md](form_fields.md). Dev-env setup
(JDK 21 gotcha, emulator) lives in [../setup.md](../setup.md).

---

## ⚠️ Gotchas (read first — these will bite every release)

1. **dx hardcodes `targetSdk = 34` and `versionCode = 1`** in its generated Gradle
   template, and **regenerates that file on every `dx bundle`**. Google requires
   `targetSdk >= 35`, and every upload needs a **unique, incrementing** `versionCode`.
   → After `dx bundle`, you must **edit the generated `build.gradle.kts` and
   repackage with Gradle directly** (see steps 3–4). Don't re-run `dx bundle`
   after editing — it wipes the edit.
2. **dx does not sign Android release builds** (only Apple codesigning exists).
   The Gradle output `.aab` is **unsigned** — you sign it yourself with the
   upload key (step 5).
3. **R8 minification is ON in release** (off in debug). It can strip WebView/JNI
   classes the app needs. **Always smoke-test the release build** (step 7) before
   rollout — a debug build passing proves nothing about the release.
4. **targetSdk 35 enables edge-to-edge enforcement.** Verify the WebView layout
   isn't drawing critical content under the status/nav bars (checked in step 7).
5. **dx regenerates `MainActivity.kt` too** (bare `class MainActivity :
   WryActivity()`), which closes the app on the OS back gesture. Re-apply the
   back-navigation patch after `dx bundle` (step 1c) or back-swipe ships broken.

---

## Prerequisites (one-time)

- **Upload keystore** at `~/certs/zwipe-upload.jks` (alias `zwipe-upload`, PKCS12).
  Created once with `keytool -genkeypair ... -keystore ~/certs/zwipe-upload.jks
  -alias zwipe-upload -keyalg RSA -keysize 2048 -validity 9125`. The **password
  lives in the password manager** — never in this repo. `~/certs/` is outside the
  repo and covered by the [mac-restore](../../ios/mac_restore.md) backup.
  *Losing the upload key is recoverable via Play's upload-key reset; losing the
  password isn't fun — keep it.*
- **bundletool** (`brew install bundletool`) — for the smoke test.
- Build env exported (see [../setup.md](../setup.md)):
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
pre-2026-07-09 bug). See [`../../../plans/back_swipe_gesture.md`](../../../plans/back_swipe_gesture.md).
R8 keeps the handler (it's used), but the step-7 smoke test is the confirmation.

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

---

## 6. Upload to the Play Console

Listing + content forms must be complete (no errors) first — there is **no
separate "submit listing for review"** step; rolling out a release reviews app +
listing + content together.

1. **Test and release → Testing → Closed testing → Create new release**
   (new personal accounts must run **Closed testing**: ≥12 testers, 14 continuous
   days, before Production access. Internal testing is fine for pipeline checks
   but does **not** count toward the 14 days.)
2. First release: enroll in **Play App Signing → "Use Google-generated key"**
   (Google holds the app signing key; `zwipe-upload.jks` is just the upload key).
3. **Upload** `zwipe-<VERSION>.aab`.
4. **Release name** `<versionName> (<versionCode>)`, e.g. `1.0.9 (1)` (internal only).
   **Release notes** inside `<en-US>…</en-US>` (≤500 chars, generic "TCG" wording).
5. **Save → Review → Start rollout to closed testing.**
6. **Testers** tab → add testers, then share the opt-in link. The 14-day clock
   runs from when ≥12 are opted in.

### Recruiting testers (the gotchas)

- Closed testing is **invite-only** — only emails on your tester list (or members
  of an added **Google Group**) can join. A bare public link does nothing for
  someone not on the list.
- **For social-media recruitment, use a Google Group**, not a hand-typed email
  list: create a public group at groups.google.com ("Anyone can join"), then
  Testers → **Google Groups** → paste the group address → Save. Anyone who joins
  the group becomes an eligible tester automatically — no per-person adding. Post
  the group join link + the opt-in URL together.
- The **"Copy link" stays greyed until the release clears review and is live** on
  the track ("the link will be shown when you publish"). The opt-in URL is
  predictable, though: `https://play.google.com/apps/testing/com.scadoshi.zwipe`
  — it only works post-publish and only for list/group members.
- Also fill the **Feedback URL or email** field so testers know where to report.

---

## Native debug symbols (the "upload a symbol file" warning)

Play shows a **non-blocking warning** that the bundle has native code without
debug symbols. **Ship anyway — it's cosmetic.** AGP's `ndk { debugSymbolLevel }`
only harvests symbols from libraries *it* builds (CMake/ndk-build); it does
**not** touch a **prebuilt** `.so`, and dx drops the Rust lib straight into
`jniLibs/`. So that config is a no-op here and the warning persists regardless.

The `.so` itself is unstripped (`file …/jniLibs/arm64-v8a/libmain.so` →
"not stripped, with debug_info"), so *if* you ever need symbolicated native
crash reports, upload symbols **manually**: Play Console → App bundle explorer →
the bundle → **Upload native debug symbols** (a zip containing
`arm64-v8a/libmain.so`). Not worth it for routine releases.

## History

- **2026-07-07 — `1.4.0`, versionCode `22`** (feature batch: commander picks now
  lead with the community's most-built commanders in a fresh daily order
  (Zwipe-select popularity ordering + wildcard deep slice), partners that name
  each other auto-pair, Deck MVPs phase 1 (star up to three cards per deck), and
  share-a-deck public links from the More sheet; commander-select signal ingest
  ships dormant. First minor bump since 1.3.x; workspace version bumped 1.3.2 →
  1.4.0). Built per this recipe — `dx bundle` → `launcher_icons.sh` → gradle
  patch (compileSdk 36 / targetSdk 35 / versionCode 22) → `gradlew
  :app:bundleRelease` → jarsigner (0600 scratchpad password, deleted after).
  Artifact `zwipe-1.4.0-vc22.aab`, signed + `jar verified`. **R8/edge-to-edge
  smoke test run this round** (Pixel_9a): app launches clean (no
  `libmain.so`/R8 crash, no FATAL), login + bottom action bar clear of the
  status/nav bars. iOS counterpart: build 61. Server halves (commander
  popularity endpoint already live 2026-07-07; commander-select-signal + Deck
  MVPs additive migrations) deploy to prod before rollout.

- **2026-07-05 — `1.3.1`, versionCode `21`** (pre-auth funnel telemetry: the
  client posts anonymous session events — app_opened, register_viewed,
  register_submitted — to the new `/api/metrics/anonymous` endpoint; plus the
  server-side AppState type-erasure refactor, no behavior change). Built per
  this recipe — `dx bundle` → `launcher_icons.sh` → gradle patch (compileSdk 36 /
  targetSdk 35 / versionCode 21) → `gradlew :app:bundleRelease` → jarsigner
  (0600 scratchpad password, deleted after). Artifact `zwipe-1.3.1-vc21.aab`,
  signed + `jar verified`. iOS counterpart: build 60. Server (anonymous_events +
  daily-activity BIGINT migrations) must deploy to prod before rollout.
  *R8/edge-to-edge emulator smoke test skipped again.*

- **2026-07-03 — `1.3.0`, versionCode `20`** (filter-intent + Reset batch:
  sort/synergy-only searches now serve, `Reset` returns each screen to its
  default view, the filter dot tracks any real filter or sort, and the filter
  sheet collapses its sections on close. Supersedes vc19). Built per this recipe
  (gradle patch versionCode 20). Artifact `zwipe-1.3.0-vc20.aab`, signed +
  `jar verified`, uploaded to the Alpha track. iOS counterpart: build 59. No
  server change. *R8/edge-to-edge emulator smoke test skipped again.*

- **2026-07-02 — `1.3.0`, versionCode `19`** (per-swipe durable skips via the
  new skip/unskip endpoints; per-deck add-stack memory with MRU parking;
  CardStack refactor across the three swipe screens; image/skeleton ease-ins +
  swipe-layout spacing; stack cap 1000 → 500; profile About section with the
  website link. Supersedes 1.2.3/vc17, withdrawn from review — release notes
  folded into 1.3.0). Built per this recipe — `dx bundle` → `launcher_icons.sh`
  → gradle patch (compileSdk 36 / targetSdk 35 / versionCode 19) →
  `gradlew :app:bundleRelease` → jarsigner (0600 scratchpad password, deleted
  after). Artifact `zwipe-1.3.0-vc19.aab`, signed + `jar verified`, uploaded to
  the Alpha closed-testing track. **vc18 was built and submitted first, then
  superseded by vc19** (added the About section) before review completed. iOS
  counterpart: build 58. Server (skip endpoints, no migration) deployed to prod
  first. *R8/edge-to-edge emulator smoke test skipped again — first suspect if a
  tester device misbehaves.*

- **2026-06-23 — first Android build (`1.0.9`).** targetSdk 35 (compiled against
  API 36.1), signed with the new `zwipe-upload` key, R8 + edge-to-edge smoke-tested
  clean on Pixel_9a. **versionCode `1` burned** by an initial targetSdk-34 upload
  (rejected for the API-35 rule but still consumed the code); `2` uploaded then
  superseded by **`3`**, which shipped — Closed testing (Alpha), 176 countries,
  with the harmless native-debug-symbols warning. *Lessons: a rejected/superseded
  upload still burns its versionCode (always bump); the debug-symbols warning is
  unavoidable with dx's prebuilt Rust lib (see section above).*

- **2026-06-23 — `1.0.10`** (update-screen redesign + external-link arrows;
  first coordinated release run alongside iOS build 44). targetSdk 35, signed with
  `zwipe-upload`. **versionCode `4` burned** by an upload attempt, **`5` shipped**
  to the Alpha track. *Lesson: keep every closed-test release on the **same Alpha
  track** so the 12-tester / 14-day clock accumulates — don't create a new track
  per version (Play won't let you delete the stray ones, only rename them).*

- **2026-06-23 — `1.0.10` refresh, versionCode `6`** (commander-search "Searching…"
  indicator — the debounce-feedback fix). Same versionName `1.0.10` (no app-version
  bump); only the versionCode increments. Artifact `zwipe-1.0.10-build6.aab`,
  **submitted to the Alpha track 2026-06-23**. iOS counterpart: build 45
  (submitted to Apple review the same day).

- **2026-06-25 — `1.1.0`, versionCode `8`** (Zwipe-select, deck tags, keyword
  hinter, expanded card detail — first minor bump). Two Android-only fixes rode
  here: **session persistence** (keyring has no Android backend → was using its
  in-memory mock, so sessions died on restart; now a JSON file in internal storage
  via JNI — see `zwiper/src/lib/outbound/session.rs`) and the **real launcher icon**
  (step 1b — dx ships its default droid). versionCode **`7` was built + smoke-tested
  but never uploaded**, then a one-line metrics fix (record the SwipeSelect select
  swipe) bumped it to **`8`, submitted to the Alpha track**. Artifact
  `zwipe-1.1.0.aab`. iOS counterpart: build 48. *Lesson: an unuploaded versionCode
  can be reused — 7 was never sent to Play, so 8 is the next real number after 6.*

- **2026-06-26 — `1.1.1`, versionCode `9`** (in-app help button, import/export hints,
  the `mailto` OS-open fix). Artifact `zwipe-1.1.1.aab`, signed + R8/edge-to-edge
  smoke-tested clean on Pixel_9a, rolled out to the Alpha track. iOS counterpart:
  build 49. **Launcher-icon lesson:** the full-bleed Z (`icon-1024.png`) was getting
  its edges sliced by the adaptive-icon **circular mask** — adaptive icons are a
  108dp canvas but only the inner ~66dp is the guaranteed-visible safe zone, and a
  wide logo like the Z has bars at the very top/bottom of its bbox that land outside
  the circle. Fix: a separate **padded** source `icon-1024-android.png` (Z ≈ 47% of
  the canvas, centered, generous `#282828` padding) wired into `launcher_icons.sh`;
  iOS/web keep the full-bleed `icon-1024.png` (square icons aren't masked). Verify a
  candidate by simulating the mask: crop the foreground to the center 66.6% and
  circle-mask it before rebuilding. *Logo design polish still deferred (see `todo.md`).*

- **2026-06-28 — `1.1.2`, versionCode `10`** (filter-control consistency pass on the
  card-swipe screens). iOS counterpart: build 50.

- **2026-06-28 — `1.1.3`, versionCode `11`** (media-day release: card names while
  swiping, deck-form overhaul, expanded tags + format/power pickers, in-app privacy
  policy, under-field validation). Built per this recipe — `dx bundle` →
  `launcher_icons.sh` → gradle patch (compileSdk 36 / targetSdk 35 / versionCode 11)
  → `gradlew :app:bundleRelease` → jarsigner (password via a 0600 scratchpad file,
  deleted after). Artifact `zwipe-1.1.3.aab`, signed + `jar verified`, uploaded to the
  Alpha closed-testing track. iOS counterpart: build 51. *R8/edge-to-edge emulator
  smoke test was skipped this round (same build path as prior releases) — first
  suspect if a tester device misbehaves.*

- **2026-06-30 — `1.1.4`, versionCode `14`** (bottom-sheet flash fix + clone-nav
  fix, over an initial 52/vc12 rebuild). Artifact `zwipe-1.1.4-vc14.aab`. iOS
  counterpart: build 53.

- **2026-07-02 — `1.2.3`, versionCode `17`** (swipe memory: per-deck skip/removal
  suppressions with server-side filtering + Clear skips in the deck More sheet;
  alphabetical deck lists; profile System/version row; email-verification row
  rework; updated privacy policy. 1.2.2 skipped, versionCode 16 shipped as 1.2.1).
  Built per this recipe — `dx bundle` → `launcher_icons.sh` → gradle patch
  (compileSdk 36 / targetSdk 35 / versionCode 17) → `gradlew :app:bundleRelease` →
  jarsigner (0600 scratchpad password, deleted after). Artifact `zwipe-1.2.3.aab`,
  signed + `jar verified`, uploaded to the Alpha closed-testing track. iOS
  counterpart: build 56. Server (swipe-memory migration) deployed to prod first.
  *R8/edge-to-edge emulator smoke test skipped again — first suspect if a tester
  device misbehaves.*

- **2026-07-01 — `1.2.1`, versionCode `16`** (card rules dialog + launch-flash
  fix). Built per this recipe, published to the Alpha closed-testing track.
  iOS counterpart: build 55 (uploaded but held behind 1.2.0, ultimately
  superseded by build 56 — 1.2.1 never went to iOS review).

- **2026-06-30 — `1.2.0`, versionCode `15`** (first minor bump since 1.1.0:
  hypergeometric draw-odds, Synergy on/off toggle, power level + other-tags,
  deck tags 85→117, include/exclude filter guard, PDH commander fix, `edhrec_rank`
  index, proliferate→Counters, create/edit top-scroll fix). Built per this recipe —
  `dx bundle` → `launcher_icons.sh` → gradle patch (compileSdk 36 / targetSdk 35 /
  versionCode 15) → `gradlew :app:bundleRelease` → jarsigner (0600 scratchpad
  password, deleted after). Artifact `zwipe-1.2.0.aab`, signed + `jar verified`,
  uploaded to the Alpha closed-testing track. iOS counterpart: build 54. Server
  batch (additive migrations) deployed to prod first. *R8/edge-to-edge emulator
  smoke test skipped again — first suspect if a tester device misbehaves.*
