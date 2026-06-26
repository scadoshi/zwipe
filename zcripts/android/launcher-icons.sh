#!/usr/bin/env bash
# Regenerate Android launcher icons from the Zwipe source icon, replacing dx's
# default placeholder (the green Android droid).
#
# dx regenerates the Android Gradle project's res/ on EVERY `dx bundle`, so run
# this AFTER `dx bundle` and BEFORE the Gradle repackage. See
# context/operations/android/play-store-submission/build-and-submit.md.
#
# Usage: zcripts/android/launcher-icons.sh [RES_DIR]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SRC="$REPO_ROOT/zwiper/assets/favicon/icon-1024.png"
RES="${1:-$REPO_ROOT/target/dx/zwipe/release/android/app/app/src/main/res}"
BG="#282828" # icon background color (matches the source icon)

[ -f "$SRC" ] || { echo "source icon not found: $SRC" >&2; exit 1; }
[ -d "$RES" ] || { echo "res dir not found: $RES" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# --- Legacy launcher icons (mipmap-*/ic_launcher.webp), full-bleed square ---
for entry in mdpi:48 hdpi:72 xhdpi:96 xxhdpi:144 xxxhdpi:192; do
  d="${entry%%:*}"; px="${entry##*:}"
  magick "$SRC" -resize "${px}x${px}" "$tmp/l.png"
  cwebp -quiet "$tmp/l.png" -o "$RES/mipmap-$d/ic_launcher.webp"
done

# --- Adaptive foreground (drawable-*/ic_launcher_foreground.png), 108dp, full-bleed ---
# Drop dx's default droid vector so the density PNGs resolve instead.
rm -f "$RES/drawable-v24/ic_launcher_foreground.xml"
for entry in mdpi:108 hdpi:162 xhdpi:216 xxhdpi:324 xxxhdpi:432; do
  d="${entry%%:*}"; px="${entry##*:}"
  mkdir -p "$RES/drawable-$d"
  magick "$SRC" -resize "${px}x${px}" "$RES/drawable-$d/ic_launcher_foreground.png"
done

# --- Adaptive background: solid color matching the icon background ---
cat > "$RES/drawable/ic_launcher_background.xml" <<EOF
<?xml version="1.0" encoding="utf-8"?>
<vector xmlns:android="http://schemas.android.com/apk/res/android"
    android:width="108dp"
    android:height="108dp"
    android:viewportWidth="108"
    android:viewportHeight="108">
    <path
        android:fillColor="$BG"
        android:pathData="M0,0h108v108h-108z" />
</vector>
EOF

echo "Launcher icons regenerated from $(basename "$SRC") into $RES"
