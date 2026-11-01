#!/bin/bash
# run.sh steps.json out-dir widths...: runs the steps at each width (dark), shots named <w>-<name>.png.
# Needs a local Chrome or Chromium for cdp.py, zerver.sh and dx.sh running first.
S=$1; OUT=$(realpath -m "$2"); shift 2
mkdir -p "$OUT"
cd "$(dirname "$0")"
for W in "$@"; do
  rm -rf /tmp/webz-profile-$W
  H=900; [ $W = 1920 ] && H=1080
  PROFILE=/tmp/webz-profile-$W python3 cdp.py http://localhost:8080/login $W $H "$S" "$OUT/$W-" > /tmp/webz-log-$W.txt 2>&1 &
done
wait
for W in "$@"; do echo "== $W"; cat /tmp/webz-log-$W.txt; done
