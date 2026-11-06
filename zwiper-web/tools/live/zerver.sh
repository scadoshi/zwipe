#!/bin/bash
# Runs zerver from this checkout on the local dev DB, with the browser origin allowed and a dummy
# Resend key so no email goes out. ZERVER_ENV points at the env file (default: zerver/.env here).
ROOT=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
set -a; source "${ZERVER_ENV:-$ROOT/zerver/.env}"; set +a
export ALLOWED_ORIGINS=http://localhost:8080,http://127.0.0.1:8080 RESEND_API_KEY=re_dummy_not_a_key LOG_DIR=/tmp/webz-zerver-logs SQLX_OFFLINE=true
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$ROOT/target}
cd "$ROOT"
exec cargo run -q --bin zerver
