#!/usr/bin/env bash
# Put the app icons into the simulator build and reinstall it, so the App icon
# sheet has alternates to switch to and the simulator's home screen shows the
# pick. `dx serve --platform ios` installs its bundle with no icons at all,
# and every full rebuild writes a fresh bundle, so run this again after one.
# Hot reloads keep the icons.
#
# Usage: zcripts/ios/sim_icons.sh [SIMULATOR]   # a udid or name; default booted
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SIM="${1:-booted}"
APP="$REPO_ROOT/target/dx/zwipe/debug/ios/Zwipe.app"
[ -d "$APP" ] || { echo "no simulator build at $APP; run dx serve --platform ios in zwiper first" >&2; exit 1; }

BUNDLE_ID="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$APP/Info.plist")"
"$REPO_ROOT/zcripts/ios/icons.sh" "$APP"

xcrun simctl terminate "$SIM" "$BUNDLE_ID" 2>/dev/null || true
xcrun simctl install "$SIM" "$APP"
xcrun simctl launch "$SIM" "$BUNDLE_ID" > /dev/null
echo "installed $BUNDLE_ID with its icons on simulator $SIM"
