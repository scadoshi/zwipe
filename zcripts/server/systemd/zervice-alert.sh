#!/usr/bin/env bash
# Emails a zervice failure alert via Resend. Wired via OnFailure= on
# zervice.service; reuses the credentials already in the server's .env
# (RESEND_API_KEY / RESEND_EMAIL_FROM / SUPPORT_EMAIL_ADDRESS) — no new
# secrets, nothing in the repo.
set -euo pipefail
for var in RESEND_API_KEY RESEND_EMAIL_FROM SUPPORT_EMAIL_ADDRESS; do
    if [ -z "${!var:-}" ]; then
        echo "$var unset; no alert sent"
        exit 0
    fi
done
ZLOGS=$(journalctl -u zervice.service -n 15 --no-pager -o cat | sed -e 's/\x1b\[[0-9;]*m//g' | tail -c 1500)
export ZLOGS
python3 << 'PYEOF'
import json, os, urllib.request

body = {
    "from": os.environ["RESEND_EMAIL_FROM"],
    "to": [os.environ["SUPPORT_EMAIL_ADDRESS"]],
    "subject": "zervice FAILED on zerver-prod",
    "text": "The nightly zervice run exited non-zero. Last journal lines:\n\n"
    + os.environ.get("ZLOGS", "(no journal output)")
    + "\n\nInspect: systemctl status zervice / journalctl -u zervice",
}
req = urllib.request.Request(
    "https://api.resend.com/emails",
    data=json.dumps(body).encode(),
    headers={
        "Content-Type": "application/json",
        "Authorization": "Bearer " + os.environ["RESEND_API_KEY"],
        "User-Agent": "zwipe-zervice-alert/1.0",
    },
)
with urllib.request.urlopen(req, timeout=10) as resp:
    print("resend accepted:", resp.status)
PYEOF
