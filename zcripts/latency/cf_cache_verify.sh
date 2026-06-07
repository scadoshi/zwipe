#!/usr/bin/env bash
# Cloudflare cache verification — confirms edge caching is working for the
# immutable card endpoints configured in the CF dashboard (Caching → Cache Rules).
#
# Usage:
#   bash cf_cache_verify.sh
#
# These endpoints are public (no auth required), so no .env credentials needed.
#
# Expected output: first request to each endpoint shows `cf-cache-status: MISS`,
# second request within TTL shows `HIT`. Any `DYNAMIC` or `BYPASS` means the
# CF Cache Rule isn't matching that path — usually a rule expression typo.

set -eu

BASE="https://api.zwipe.net"
echo ""

# Endpoints expected to be cached by the CF Cache Rule.
ENDPOINTS=(
    "/api/card/sets"
    "/api/card/types"
    "/api/card/keywords"
    "/api/card/oracle-words"
    "/api/card/artists"
    "/api/card/languages"
)

PASS=0
FAIL=0

for endpoint in "${ENDPOINTS[@]}"; do
    echo "── $endpoint ──"
    statuses=()
    for i in 1 2; do
        status=$(curl -sI "$BASE$endpoint" |
            grep -i '^cf-cache-status:' |
            awk '{print $2}' | tr -d '\r\n')
        statuses+=("$status")
        printf "  request %d: cf-cache-status: %s\n" "$i" "${status:-<missing>}"
        # tiny breather so we're not hammering CF
        sleep 0.2
    done
    # Verdict: pass if second request is HIT (or first MISS + second HIT).
    if [ "${statuses[1]}" = "HIT" ]; then
        echo "  PASS — cached"
        PASS=$((PASS + 1))
    else
        echo "  FAIL — expected HIT on second request, got '${statuses[1]}'"
        FAIL=$((FAIL + 1))
    fi
    echo ""
done

echo "─── summary ───"
echo "  passed: $PASS / ${#ENDPOINTS[@]}"
echo "  failed: $FAIL / ${#ENDPOINTS[@]}"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
