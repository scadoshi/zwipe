#!/usr/bin/env bash
# Every patch the dx-generated Android project needs, in one call.
#
# `dx bundle` regenerates the whole android/ tree each time — res/,
# MainActivity.kt, AndroidManifest.xml — so each of these has to be re-applied
# AFTER `dx bundle` and BEFORE the Gradle repackage. They were three separate
# commands to remember, and forgetting one ships a broken release quietly:
#
#   launcher_icons.sh  missing -> stock dioxus icon on the launcher
#   back_handler.sh    missing -> OS back exits the app instead of navigating
#   manifest.sh        missing -> the ndk-context double-init crash returns,
#                                 and a system theme change closes the app
#                                 (context/plans/archive/android_ndk_context_crash.md)
#   launcher label     missing -> launcher shows "Zwiper" (dx names it after
#                                 the binary, not Dioxus.toml)
#   wry_activity.sh    missing -> UnsatisfiedLinkError in onCreate on every
#                                 device: the stock activity calls tao 0.34's
#                                 native names and the vendored tao is 0.37
#
# The manifest bug survived five releases partly because this was a checklist.
# Call this one script instead; add new patches here rather than to the docs.
#
# APP_DIR is the generated Gradle project; it defaults to the release one that
# `dx bundle --release` writes. debug_apk.sh passes the debug one.
#
# Usage: zcripts/android/patch_bundle.sh [APP_DIR]
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../.." && pwd)"
APP_DIR="${1:-$REPO_ROOT/target/dx/zwiper/release/android/app}"
MAIN="$APP_DIR/app/src/main"

[ -d "$MAIN" ] || { echo "generated Android project not found: $APP_DIR" >&2; exit 1; }

echo "== launcher icons =="
"$HERE/launcher_icons.sh" "$MAIN/res"

echo "== back navigation =="
"$HERE/back_handler.sh" "$MAIN/kotlin/dev/dioxus/main/MainActivity.kt"

echo "== manifest (launchMode + configChanges + allowBackup) =="
"$HERE/manifest.sh" "$MAIN/AndroidManifest.xml"

echo "== WryActivity (tao 0.37 entry points) =="
"$HERE/wry_activity.sh" "$MAIN/kotlin/dev/dioxus/main/WryActivity.kt"

# dx sets app_name from the binary name (`zwiper` -> "Zwiper"), and Dioxus.toml
# has no Android override, so put the launcher label back to "Zwipe".
echo "== launcher label =="
STRINGS="$MAIN/res/values/strings.xml"
sed -i.bak 's#<string name="app_name">[^<]*</string>#<string name="app_name">Zwipe</string>#' "$STRINGS"
rm -f "$STRINGS.bak"
grep -q '<string name="app_name">Zwipe</string>' "$STRINGS" \
  || { echo "launcher label patch did not apply: $STRINGS" >&2; exit 1; }

echo
echo "All post-bundle patches applied. Next: edit build.gradle.kts (targetSdk /"
echo "versionCode) and repackage with ./gradlew :app:bundleRelease."
