#!/usr/bin/env bash
# Latency probe — measures backend vs tunnel latency for representative endpoints.
#
# Usage:
#   bash probe.sh           # measure both localhost and public
#   bash probe.sh local     # localhost only (run on the server)
#   bash probe.sh public    # public hostname only (run anywhere)
#
# Reads credentials from `.env` (next to this script) so secrets stay out of git.
# The `.env` is gitignored — copy `.env.example` to `.env` and fill in real values.
# Can still be overridden by exporting ZWIPE_TEST_USER / ZWIPE_TEST_PASS in your shell.

set -eu

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source .env if present. set -a auto-exports anything defined.
if [ -f "$SCRIPT_DIR/.env" ]; then
    set -a
    # shellcheck disable=SC1091
    . "$SCRIPT_DIR/.env"
    set +a
fi

USER="${ZWIPE_TEST_USER:-test}"
PASS="${ZWIPE_TEST_PASS:-}"
LOCAL_BASE="http://localhost:3000"
PUBLIC_BASE="https://api.zwipe.net"

if [ -z "$PASS" ]; then
    echo "ZWIPE_TEST_PASS not set."
    echo "   Option 1: cp $SCRIPT_DIR/.env.example $SCRIPT_DIR/.env  (then fill in)"
    echo "   Option 2: ZWIPE_TEST_PASS='<your-test-password>' bash $0"
    exit 1
fi

# Login always via the public endpoint so it works whether we're on
# the server or a remote machine. Login is one-time, the ~200ms hop is fine.
TOKEN=$(curl -s -X POST "$PUBLIC_BASE/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"identifier\":\"$USER\",\"password\":\"$PASS\"}" |
    jq -r '.access_token.value')

if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
    echo "Login failed; got empty token. Raw response:"
    curl -s -X POST "$PUBLIC_BASE/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"identifier\":\"$USER\",\"password\":\"$PASS\"}"
    echo ""
    exit 1
fi
echo "Token acquired (${#TOKEN} chars)"

# Probe one endpoint, 5 samples. Prints status, body size, total time.
probe() {
    local label=$1 base=$2 method=$3 path=$4 body=$5
    echo ""
    echo "── $label  $method $path ──"
    for i in 1 2 3 4 5; do
        if [ "$method" = "GET" ]; then
            curl -s -o /tmp/zwipe-probe-resp.json \
                -w "  %{http_code}  %{size_download}b  %{time_total}s\n" \
                -H "Authorization: Bearer $TOKEN" \
                "$base$path"
        else
            curl -s -o /tmp/zwipe-probe-resp.json \
                -w "  %{http_code}  %{size_download}b  %{time_total}s\n" \
                -X "$method" \
                -H "Authorization: Bearer $TOKEN" \
                -H "Content-Type: application/json" \
                "$base$path" \
                -d "$body"
        fi
        # rate-limit breather — search endpoint has a GovernorLayer
        sleep 0.4
    done
}

MODE="${1:-both}"

run_against() {
    local label=$1 base=$2
    probe "$label" "$base" GET "/health/server" ""
    probe "$label" "$base" GET "/health/database" ""
    probe "$label" "$base" POST "/api/card/search" '{"name_contains":"krenko"}'
    probe "$label" "$base" GET "/api/deck" ""
}

case "$MODE" in
local) run_against "LOCAL" "$LOCAL_BASE" ;;
public) run_against "PUBLIC" "$PUBLIC_BASE" ;;
both | *)
    run_against "LOCAL" "$LOCAL_BASE"
    run_against "PUBLIC" "$PUBLIC_BASE"
    ;;
esac

echo ""
echo "─ last response body sample ─"
head -c 300 /tmp/zwipe-probe-resp.json
echo ""
