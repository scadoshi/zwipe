# Android

`dx bundle` regenerates the whole Android project every time, so these scripts patch what it gets wrong after each bundle.

- `patch_bundle.sh` runs every patch below in one call. It's the one to remember.
- `back_handler.sh` makes the OS back gesture and button drive the app's router instead of closing the app.
- `manifest.sh` fixes two crash and UX bugs in the generated `AndroidManifest.xml`.
- `wry_activity.sh` rewrites the generated `WryActivity.kt` for the vendored tao.
- `launcher_icons.sh` replaces dx's placeholder launcher icon with Zwipe's.
- `debug_apk.sh` builds a debug APK that actually launches: `dx build`, the patches, then a Gradle repackage.

Store builds and submission steps are in `zwipe-context operations/android/`.
