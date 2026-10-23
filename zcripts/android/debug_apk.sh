#!/usr/bin/env bash
# Build a debug APK that launches: `dx build`, then the post-bundle patches,
# then a Gradle repackage.
#
# A bare `dx build --platform android` APK dies on launch with
# UnsatisfiedLinkError (WryActivity.create): dx writes the stock activity and
# the vendored tao needs wry_activity.sh. dx regenerates the Gradle project on
# every build, so the patches run after it each time.
#
# Prints the APK path on the last line. Needs JAVA_HOME and ANDROID_HOME, as in
# context/operations/android/emulator.md section 0.
#
# Usage: zcripts/android/debug_apk.sh
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../.." && pwd)"
APP_DIR="$REPO_ROOT/target/dx/zwipe/debug/android/app"

: "${JAVA_HOME:?set JAVA_HOME to the Android Studio JBR (emulator.md section 0)}"
: "${ANDROID_HOME:?set ANDROID_HOME (emulator.md section 0)}"

(cd "$REPO_ROOT/zwiper" && dx build --platform android)

"$HERE/patch_bundle.sh" "$APP_DIR"

(cd "$APP_DIR" && ./gradlew :app:assembleDebug --console=plain -q)

echo "$APP_DIR/app/build/outputs/apk/debug/app-debug.apk"
