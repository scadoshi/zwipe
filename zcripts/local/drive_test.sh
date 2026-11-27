#!/usr/bin/env bash
# Checks drive.mjs types nothing into a page until the page's API calls all go to this machine.
# Serves three local pages: one that calls its own /api/, one that calls an API elsewhere and one
# that calls none. Needs node and Chrome.
# Usage: zcripts/local/drive_test.sh
set -uo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="$(mktemp -d)"
SERVE_PORT="${SERVE_PORT:-8199}"
failures=0

cat >"$WORK/serve.mjs" <<'SERVE'
import { createServer } from "node:http";
// 127.0.0.2 is this machine too, but not a name the driver accepts, so it stands in for a remote
// API without any traffic leaving.
const pages = {
  "/local": "fetch('/api/ping')",
  "/away": "fetch('http://127.0.0.2:9/api/ping').catch(() => {})",
  "/quiet": "",
};
createServer((req, res) => {
  if (req.url === "/api/ping") return res.end("{}");
  const script = pages[req.url];
  if (script === undefined) return res.writeHead(404).end();
  res.setHeader("content-type", "text/html");
  res.end(`<input id="f" autofocus><script>${script}</script>`);
}).listen(Number(process.argv[2]), "127.0.0.1");
SERVE
node "$WORK/serve.mjs" "$SERVE_PORT" &
server=$!
trap 'kill $server 2>/dev/null; rm -rf "$WORK"' EXIT
sleep 1

echo '[{"type": "typed"}, {"eval": "document.getElementById(\"f\").value", "label": "field"}]' >"$WORK/steps.json"

# Runs the driver on page $1; passes when it exits $2 and the field holds $3.
check() {
  local out code
  out="$(node "$DIR/drive.mjs" "http://127.0.0.1:$SERVE_PORT/$1" 1440 "$WORK/steps.json" 2>&1)"
  code=$?
  if [ "$code" != "$2" ]; then
    echo "FAIL: /$1 exited $code, want $2: $out" >&2
    failures=$((failures + 1))
  elif [ -n "$3" ] && ! grep -qF "field => \"$3\"" <<<"$out"; then
    echo "FAIL: /$1 field isn't $3: $out" >&2
    failures=$((failures + 1))
  elif [ -z "$3" ] && grep -q "field =>" <<<"$out"; then
    echo "FAIL: /$1 ran on past the refusal: $out" >&2
    failures=$((failures + 1))
  fi
}

check local 0 typed
check away 2 ""
check quiet 2 ""

if [ "$failures" -ne 0 ]; then
  echo "$failures drive check(s) failed" >&2
  exit 1
fi
echo "drive: all checks pass"
