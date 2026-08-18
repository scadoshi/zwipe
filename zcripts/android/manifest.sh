#!/usr/bin/env bash
# Patch the dx-generated AndroidManifest.xml for two crash/UX bugs that the
# default manifest causes. See context/plans/android_resume_crash.md.
#
# 1. launchMode="singleTask" — without it MainActivity defaults to `standard`,
#    so any EXPLICIT start of the component while an instance already exists
#    (notification tap, another app, an app shortcut, the Play Store's "Open"
#    button after an update) creates a SECOND Activity in the live process.
#    The app is a NativeActivity, so that second creation re-runs wry/tao's
#    native init, and ndk_context::initialize_android_context panics on
#    `assert!(previous.is_none())`. singleTask reuses the existing instance via
#    onNewIntent instead. Reproduced and verified on device 2026-08-17: the
#    same `am start` panics without it and is clean with the instance reused.
#
# 2. uiMode|locale|density|fontScale added to configChanges — dx generates
#    only orientation|screenLayout|screenSize|keyboardHidden, so a system
#    dark/light switch (or a locale, display-size or font-size change)
#    RECREATES the Activity. That path does reach onDestroy, where
#    back_handler.sh's process kill fires, so the app silently vanishes
#    mid-session. Auto dark mode at sunset does this on a schedule.
#
# dx REGENERATES AndroidManifest.xml on every `dx bundle`, so run this AFTER
# `dx bundle` and BEFORE the Gradle repackage — the same window as
# launcher_icons.sh and back_handler.sh. `patch_bundle.sh` runs all three.
#
# Usage: zcripts/android/manifest.sh [ANDROID_MANIFEST_XML]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MANIFEST="${1:-$REPO_ROOT/target/dx/zwipe/release/android/app/app/src/main/AndroidManifest.xml}"

[ -f "$MANIFEST" ] || { echo "AndroidManifest.xml not found: $MANIFEST" >&2; exit 1; }

CONFIG_CHANGES="orientation|screenLayout|screenSize|keyboardHidden|uiMode|locale|density|fontScale"

python3 - "$MANIFEST" "$CONFIG_CHANGES" <<'PY'
import re, sys

path, config_changes = sys.argv[1], sys.argv[2]
src = open(path).read()

# The MainActivity open tag, however dx has wrapped its attributes.
m = re.search(r'<activity\b[^>]*android:name="dev\.dioxus\.main\.MainActivity"[^>]*>', src)
if not m:
    sys.exit("MainActivity <activity> tag not found — did dx change its manifest template?")
tag = m.group(0)
patched = tag

# 1. configChanges: replace whatever dx emitted with our fuller set.
if 'android:configChanges="' in patched:
    patched = re.sub(r'android:configChanges="[^"]*"',
                     f'android:configChanges="{config_changes}"', patched)
else:
    patched = patched.replace("<activity", f'<activity android:configChanges="{config_changes}"', 1)

# 2. launchMode: add it if absent, correct it if dx ever starts emitting one.
if 'android:launchMode="' in patched:
    patched = re.sub(r'android:launchMode="[^"]*"', 'android:launchMode="singleTask"', patched)
else:
    patched = patched.replace("<activity", '<activity android:launchMode="singleTask"', 1)

if patched != tag:
    open(path, "w").write(src.replace(tag, patched, 1))

print("launchMode=singleTask, configChanges=" + config_changes)
PY

echo "Patched MainActivity attributes into $MANIFEST"
