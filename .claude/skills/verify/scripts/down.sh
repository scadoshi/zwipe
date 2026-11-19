#!/usr/bin/env bash
# Stops what up.sh started and drops the verify database (KEEP=1 keeps it).
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
. "$(dirname "${BASH_SOURCE[0]}")/common.sh"

if [ -f "$RUN_DIR/zerver.pid" ]; then
  pid="$(cat "$RUN_DIR/zerver.pid")"
  is_zerver_pid "$pid" && kill "$pid" 2>/dev/null || true
  rm -f "$RUN_DIR/zerver.pid"
fi
pkill -f "dx serve --port $WEB_PORT" 2>/dev/null || true

if [ "${KEEP:-0}" != 1 ]; then
  TEMPLATE="$(saved_template_url)"
  require_local "$TEMPLATE"
  PGOPTIONS="-c client_min_messages=warning" psql "$(with_db "$TEMPLATE" postgres)" -qc "drop database if exists $VERIFY_DB"
  echo "stopped; dropped $VERIFY_DB"
else
  echo "stopped; kept $VERIFY_DB"
fi
