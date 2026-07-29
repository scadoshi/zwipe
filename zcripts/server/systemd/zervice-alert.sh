#!/usr/bin/env bash
# Posts a zervice failure alert to Discord. Wired via OnFailure= on
# zervice.service; the webhook URL lives in the server's .env only
# (ZERVICE_ALERT_WEBHOOK) — never in the repo.
set -euo pipefail
if [ -z "${ZERVICE_ALERT_WEBHOOK:-}" ]; then
    echo "ZERVICE_ALERT_WEBHOOK unset; no alert sent"
    exit 0
fi
ZLOGS=$(journalctl -u zervice.service -n 15 --no-pager -o cat | tail -c 1500)
export ZLOGS
python3 - "$ZERVICE_ALERT_WEBHOOK" << 'PYEOF'
import json, os, sys, urllib.request

url = sys.argv[1]
logs = os.environ.get("ZLOGS", "(no journal output)")
body = {"content": f"zervice FAILED on zerver-prod\n```\n{logs}\n```"}
req = urllib.request.Request(
    url,
    data=json.dumps(body).encode(),
    headers={"Content-Type": "application/json"},
)
urllib.request.urlopen(req, timeout=10)
PYEOF
