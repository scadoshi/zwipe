#!/usr/bin/env bash
# Compile the app icons into an asset catalog and reference them from
# Info.plist: the gruvbox primary from zwiper/assets/favicon, plus one
# alternate per theme from zwiper/assets/app-icons (zcripts/ios/icon.py
# renders those). dx does not run actool, so without this the phone shows a
# blank icon and Apple rejects the upload.
#
# Run after every release build and before signing; build.md step 3.
#
# Usage: zcripts/ios/icons.sh [APP_BUNDLE]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP="${1:-$REPO_ROOT/target/dx/zwipe/release/ios/Zwipe.app}"
ICONS="$REPO_ROOT/zwiper/assets/favicon"
[ -d "$APP" ] || { echo "no app bundle at $APP" >&2; exit 1; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
SET="$WORK/Assets.xcassets/AppIcon.appiconset"
mkdir -p "$SET"

echo '{"info":{"version":1,"author":"xcode"}}' > "$WORK/Assets.xcassets/Contents.json"
cat > "$SET/Contents.json" <<'JSON'
{
  "images": [
    {"size":"20x20","idiom":"iphone","scale":"2x","filename":"icon-40.png"},
    {"size":"20x20","idiom":"iphone","scale":"3x","filename":"icon-60.png"},
    {"size":"29x29","idiom":"iphone","scale":"2x","filename":"icon-60.png"},
    {"size":"29x29","idiom":"iphone","scale":"3x","filename":"icon-87.png"},
    {"size":"40x40","idiom":"iphone","scale":"2x","filename":"icon-80.png"},
    {"size":"40x40","idiom":"iphone","scale":"3x","filename":"icon-120.png"},
    {"size":"60x60","idiom":"iphone","scale":"2x","filename":"icon-120.png"},
    {"size":"60x60","idiom":"iphone","scale":"3x","filename":"icon-180.png"},
    {"size":"1024x1024","idiom":"ios-marketing","scale":"1x","filename":"icon-1024.png"}
  ],
  "info":{"version":1,"author":"xcode"}
}
JSON

for px in 40 60 80 87 120 180 1024; do
  cp "$ICONS/icon-$px.png" "$SET/"
done

# One single-size set per theme, named AppIcon-<theme>, which is the name
# zwiper/src/lib/outbound/app_icon.rs asks UIKit for.
ALTERNATES=()
for png in "$REPO_ROOT"/zwiper/assets/app-icons/*.png; do
  name="AppIcon-$(basename "$png" .png)"
  alt="$WORK/Assets.xcassets/$name.appiconset"
  mkdir -p "$alt"
  cp "$png" "$alt/icon.png"
  cat > "$alt/Contents.json" <<'JSON'
{
  "images": [
    {"size":"1024x1024","idiom":"universal","platform":"ios","filename":"icon.png"}
  ],
  "info":{"version":1,"author":"xcode"}
}
JSON
  ALTERNATES+=(--alternate-app-icon "$name")
done

actool --compile "$APP" --platform iphoneos --minimum-deployment-target 16.0 \
  --app-icon AppIcon "${ALTERNATES[@]}" \
  --output-partial-info-plist "$WORK/partial.plist" \
  "$WORK/Assets.xcassets" > /dev/null

PLIST="$APP/Info.plist"
# Rewrite rather than Add so a rebuilt bundle can be patched twice.
/usr/libexec/PlistBuddy -c "Delete :CFBundleIcons" "$PLIST" 2>/dev/null || true
/usr/libexec/PlistBuddy \
  -c "Add :CFBundleIcons dict" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon dict" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon:CFBundleIconFiles array" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon:CFBundleIconFiles:0 string AppIcon60x60" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon:CFBundleIconName string AppIcon" \
  -c "Add :CFBundleIcons:CFBundleAlternateIcons dict" \
  "$PLIST"
for ((i = 1; i < ${#ALTERNATES[@]}; i += 2)); do
  name="${ALTERNATES[$i]}"
  /usr/libexec/PlistBuddy \
    -c "Add :CFBundleIcons:CFBundleAlternateIcons:$name dict" \
    -c "Add :CFBundleIcons:CFBundleAlternateIcons:$name:CFBundleIconName string $name" \
    "$PLIST"
done

[ -f "$APP/Assets.car" ] || { echo "actool produced no Assets.car" >&2; exit 1; }
echo "icons: Assets.car, primary + $(( ${#ALTERNATES[@]} / 2 )) alternates"
