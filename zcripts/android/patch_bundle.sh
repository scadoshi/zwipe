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
#                                 (context/plans/android_resume_crash.md)
#
# The manifest bug survived five releases partly because this was a checklist.
# Call this one script instead; add new patches here rather than to the docs.
#
# Usage: zcripts/android/patch_bundle.sh
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "== launcher icons =="
"$HERE/launcher_icons.sh"

echo "== back navigation =="
"$HERE/back_handler.sh"

echo "== manifest (launchMode + configChanges) =="
"$HERE/manifest.sh"

echo
echo "All post-bundle patches applied. Next: edit build.gradle.kts (targetSdk /"
echo "versionCode) and repackage with ./gradlew :app:bundleRelease."
