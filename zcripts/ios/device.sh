#!/usr/bin/env bash
# Shared helper for the iOS scripts. Source it, don't run it.

# Sets UDID and DEVICE_NAME. With a selector, matches that name or UDID;
# without one, requires exactly one iPhone on the cable.
#
# Only phones whose tunnel is "connected" count. A phone this Mac has paired
# before still shows in the list as "disconnected" when it is not plugged in,
# and treating it as a candidate made a one-phone day look like two.
#
# Xcode 27 (devicectl JSON v5) moved everything under "properties" and stopped
# sending "reality" for real hardware, so anything not "simulated" is a phone.
# The old top-level fields are read as a fallback for older Xcodes.
#
# Xcode 27 also stopped opening the tunnel when a phone is plugged in; it only
# comes up when a command talks to the phone. So a wired phone that lists as
# "disconnected" gets one `device info details` call to wake it, then the list
# is read again. A phone that is really unplugged just times out and stays out.

# Prints "udid tunnel name" for each wired iPhone matching the selector.
_wired_iphones() {
  local want="$1" json
  json="$(mktemp)"
  if ! xcrun devicectl list devices --quiet --json-output "$json" > /dev/null 2>&1 \
      || [ ! -s "$json" ]; then
    rm -f "$json"
    echo "devicectl could not list devices. Run: xcrun devicectl list devices" >&2
    return 1
  fi
  python3 -c '
import json, sys
want = sys.argv[2] if len(sys.argv) > 2 else ""
for d in json.load(open(sys.argv[1]))["result"]["devices"]:
    props = d.get("properties", {})
    hw = props.get("hardware") or d.get("hardwareProperties", {})
    old_conn = d.get("connectionProperties", {})
    conn = props.get("connection") or {}
    if hw.get("reality") == "simulated" or hw.get("deviceType") != "iPhone":
        continue
    if (conn.get("transportType") or old_conn.get("transportType")) != "wired":
        continue
    tunnel = conn.get("state") or old_conn.get("tunnelState") or "?"
    udid = hw.get("udid", "")
    name = (props.get("state") or {}).get("name") or d.get("deviceProperties", {}).get("name", "?")
    if want and want.lower() not in name.lower() and want.lower() != udid.lower():
        continue
    print(udid, tunnel, name)
' "$json" "$want"
  rm -f "$json"
}

find_device() {
  local want="${1:-}" found count listed udid tunnel name
  listed="$(_wired_iphones "$want")" || return 1
  while read -r udid tunnel name; do
    [ -n "$udid" ] && [ "$tunnel" != "connected" ] || continue
    xcrun devicectl device info details --device "$udid" --timeout 10 \
      > /dev/null 2>&1 || true
  done <<< "$listed"
  listed="$(_wired_iphones "$want")" || return 1
  found="$(printf '%s\n' "$listed" | awk '$2 == "connected" { $2 = ""; sub(/  /, " "); print }')"
  count="$(printf '%s' "$found" | grep -c . || true)"
  if [ "$count" -eq 0 ]; then
    if [ -n "$want" ]; then
      echo "No connected iPhone matches \"$want\"." >&2
    else
      echo "No iPhone on the cable. Plug it in, unlock it, and trust the Mac." >&2
    fi
    echo "Known to this Mac:" >&2
    xcrun devicectl list devices 2>/dev/null | grep -iv "simulated" >&2 || true
    return 1
  fi
  if [ "$count" -gt 1 ]; then
    echo "More than one iPhone connected. Name one with --device:" >&2
    printf '%s\n' "$found" | sed 's/^/  /' >&2
    return 1
  fi
  UDID="$(printf '%s' "$found" | awk '{print $1}')"
  DEVICE_NAME="$(printf '%s' "$found" | cut -d' ' -f2-)"
}
