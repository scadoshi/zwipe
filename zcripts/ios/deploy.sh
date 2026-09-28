#!/usr/bin/env bash
# Build zwiper for a real iPhone and install it. No backup step: every deck
# lives on the server, so a reinstall has nothing on the phone to lose.
#
# Usage:
#   zcripts/ios/deploy.sh                    # debug build against prod
#   zcripts/ios/deploy.sh --release          # store-parity build
#   zcripts/ios/deploy.sh --device matthew   # when more than one phone is on
#   BACKEND_URL=http://192.168.1.5:3000 zcripts/ios/deploy.sh   # a local server
#
# The profile picks the output directory as well as the compiler flags, so
# this installs from the directory it just built into. Building --release by
# hand and then installing debug/ios/Zwipe.app ships the previous debug bundle,
# which looks exactly like a build that ignored your changes.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=device.sh
. "$REPO_ROOT/zcripts/ios/device.sh"

PROFILE=debug
WANT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --release) PROFILE=release; shift ;;
    --device) WANT="${2:?--device needs a name}"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

find_device "$WANT"
echo "device: $DEVICE_NAME"

# Baked into the binary by zwiper/build.rs. Prod unless told otherwise, since
# a dev client against the live server is what gets tested on a phone.
export BACKEND_URL="${BACKEND_URL:-https://api.zwipe.net}"
echo "backend: $BACKEND_URL"

cd "$REPO_ROOT/zwiper"
if [ "$PROFILE" = release ]; then
  dx build --release --platform ios --device true
else
  dx build --platform ios --device true
fi

APP="$REPO_ROOT/target/dx/zwipe/$PROFILE/ios/Zwipe.app"
[ -d "$APP" ] || { echo "no app bundle at $APP" >&2; exit 1; }

# --id matters: with two phones connected ios-deploy otherwise picks one on
# its own.
ios-deploy --id "$UDID" --bundle "$APP"
echo "installed $PROFILE build on $DEVICE_NAME"
