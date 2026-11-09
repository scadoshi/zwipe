#!/usr/bin/env bash
# Renders the App Store product page headers from header.html: 5244x2950 (16:9)
# and 3840x1646 (21:9), opaque RGB PNGs, since App Store Connect refuses any
# alpha. Needs Chrome (or chrome-headless-shell) and ImageMagick.
#
# Usage: ./render.sh   # from anywhere; writes the PNGs beside this script
# CHROME overrides the browser path; BG and INK the theme (default Gruvbox dark).
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BG="${BG:-282828}"
INK="${INK:-ebdbb2}"
CHROME="${CHROME:-$(for c in \
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
  "$HOME"/.cache/ms-playwright/chromium_headless_shell-*/chrome-headless-shell-linux64/chrome-headless-shell \
  chromium google-chrome; do command -v "$c" >/dev/null 2>&1 && { echo "$c"; break; }; [ -x "$c" ] && { echo "$c"; break; }; done)}"
[ -n "$CHROME" ] || { echo "no Chrome found; set CHROME" >&2; exit 1; }

# The same tile size in both: 36 tiles across the 16:9 header's width, and the
# 21:9 one at that size too.
render() {
  local w=$1 h=$2 cols=$3 out=$4 tmp
  tmp="$(mktemp -d)"
  "$CHROME" --headless --disable-gpu --hide-scrollbars --force-device-scale-factor=1 \
    --user-data-dir="$tmp/profile" --window-size="$w,$h" --virtual-time-budget=2000 \
    --screenshot="$tmp/shot.png" "file://$DIR/header.html?w=$w&h=$h&cols=$cols&bg=$BG&ink=$INK" >/dev/null 2>&1
  magick "$tmp/shot.png" -alpha off -define png:color-type=2 "$DIR/$out"
  rm -rf "$tmp"
  echo "$out: $(magick identify -format '%wx%h %[channels]' "$DIR/$out")"
}

render 5244 2950 36 header-16x9-5244x2950.png
render 3840 1646 26.36 header-21x9-3840x1646.png
