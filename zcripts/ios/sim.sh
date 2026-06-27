#!/usr/bin/env bash
# Boot the project's DEFAULT iOS simulator and make it the sole active one:
# iPhone 11 Pro Max on iOS 18.6 — the 6.5" device used for App Store
# screenshots and day-to-day dev.
#
# `dx serve --ios` targets the *active* simulator, so booting this as the only
# active sim makes it the default — no --device flag needed. Run this once per
# session before `dx serve --ios`.
set -euo pipefail

# Resolve the UDID by name + runtime (there are two "iPhone 11 Pro Max" sims —
# one on iOS 18.6, one on a newer runtime — so match the 18.6 one explicitly).
UDID="$(xcrun simctl list devices available --json | python3 -c '
import json, sys
for runtime, devs in json.load(sys.stdin)["devices"].items():
    if "iOS-18-6" in runtime:
        for d in devs:
            if d["name"] == "iPhone 11 Pro Max":
                print(d["udid"]); sys.exit(0)
sys.exit(1)
')" || {
  echo "iPhone 11 Pro Max (iOS 18.6) not found. Create it:" >&2
  echo '  xcrun simctl create "iPhone 11 Pro Max" \' >&2
  echo '    "com.apple.CoreSimulator.SimDeviceType.iPhone-11-Pro-Max" \' >&2
  echo '    "com.apple.CoreSimulator.SimRuntime.iOS-18-6"' >&2
  exit 1
}

xcrun simctl shutdown all 2>/dev/null || true
xcrun simctl boot "$UDID"
open -a Simulator
echo "Booted iPhone 11 Pro Max (iOS 18.6, 6.5\") [$UDID] — dx serve --ios will target it."
