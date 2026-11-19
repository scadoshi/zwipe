#!/usr/bin/env bash
# Declare the FileProvider the app shares files through, in the dx-generated
# Android project.
#
# Collection's Export hands the share sheet a real .csv: the app writes it to
# cache/shared/ and passes a content:// URI that androidx's FileProvider serves,
# with a read grant for the app the user picks. Android lets no app read
# another's private files by path, so without the provider the share would
# carry text, which the Binder transaction caps near 1 MiB, and the receiving
# app would get no file.
#
# Two edits, both safe to run again:
#   1. a <provider> in <application>, authority ${applicationId}.fileprovider,
#      not exported, granting URI permissions, pointing at @xml/file_paths
#   2. res/xml/file_paths.xml, serving only cache/shared/
#
# The share code (zwiper/src/lib/outbound/share.rs) names the same authority
# suffix and directory, and a test there reads this file to keep them equal.
#
# dx regenerates the project on every build, so this runs after it, through
# patch_bundle.sh.
#
# Usage: zcripts/android/file_provider.sh [APP_MAIN_DIR]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MAIN="${1:-$REPO_ROOT/target/dx/zwiper/release/android/app/app/src/main}"
MANIFEST="$MAIN/AndroidManifest.xml"
[ -f "$MANIFEST" ] || { echo "AndroidManifest.xml not found: $MANIFEST" >&2; exit 1; }

python3 - "$MANIFEST" <<'PY'
import sys

path = sys.argv[1]
src = open(path).read()
provider = (
    '<provider android:name="androidx.core.content.FileProvider" '
    'android:authorities="${applicationId}.fileprovider" '
    'android:exported="false" android:grantUriPermissions="true">'
    '<meta-data android:name="android.support.FILE_PROVIDER_PATHS" '
    'android:resource="@xml/file_paths" /></provider>'
)
if "androidx.core.content.FileProvider" not in src:
    if "</application>" not in src:
        sys.exit("</application> not found: did dx change its manifest template?")
    src = src.replace("</application>", provider + "</application>", 1)
    open(path, "w").write(src)
PY

mkdir -p "$MAIN/res/xml"
cat > "$MAIN/res/xml/file_paths.xml" <<'XML'
<?xml version="1.0" encoding="utf-8"?>
<paths>
    <cache-path name="shared" path="shared/" />
</paths>
XML

grep -q 'androidx.core.content.FileProvider' "$MANIFEST" \
  || { echo "file provider patch did not apply: $MANIFEST" >&2; exit 1; }
echo "Declared the FileProvider in $MANIFEST"
