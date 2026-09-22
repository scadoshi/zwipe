#!/usr/bin/env bash
# Boot the project's DEFAULT iOS simulator and make it the sole active one:
# iPhone 11 Pro Max, the 6.5" device used for App Store screenshots and
# day-to-day dev. The runtime does not matter for screenshot size; the device
# model does, so this takes whatever iOS runtime is installed.
#
# `dx serve --platform ios` installs to the *booted* simulator and will not
# boot one for you. Run this once per session first.
set -euo pipefail

DEVICE="${1:-iPhone 11 Pro Max}"

# Newest iOS runtime that has the device. Xcode updates drop old runtimes, so
# pinning one (this used to pin iOS 18.6) breaks the script the first time
# Xcode upgrades.
UDID="$(xcrun simctl list devices available --json | DEVICE="$DEVICE" python3 -c '
import json, os, re, sys
want = os.environ["DEVICE"]
best = None
for runtime, devices in json.load(sys.stdin)["devices"].items():
    if "iOS" not in runtime:
        continue
    version = tuple(int(n) for n in re.findall(r"\d+", runtime.rsplit(".", 1)[-1]))
    for device in devices:
        if device["name"] == want and (best is None or version > best[0]):
            best = (version, device["udid"])
print(best[1]) if best else sys.exit(1)
')" || {
  echo "No simulator named \"$DEVICE\" on any installed iOS runtime." >&2
  echo "Installed runtimes:" >&2
  xcrun simctl list runtimes | grep -i ios >&2
  echo "Create one with zcripts/ios/sim.sh's device name, see operations/ios/simulator.md" >&2
  exit 1
}

xcrun simctl shutdown all 2>/dev/null || true
xcrun simctl boot "$UDID"

# Xcode 27 removed Simulator.app; DeviceHub is the replacement and `open -a
# Simulator` fails outright. Fall back to it so this works on both.
if ! open -a Simulator 2>/dev/null; then
  open /Applications/Xcode.app/Contents/Applications/DeviceHub.app 2>/dev/null || true
fi

echo "Booted $DEVICE ($UDID). Now: cd zwiper && dx serve --platform ios"
