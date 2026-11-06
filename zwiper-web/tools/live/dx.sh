#!/bin/bash
# Restarts zwiper-web's dev server on :8080 and waits for its build.
ROOT=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
pkill -f 'dx[ ]serve --platform web --port 8080'; sleep 1
cd "$ROOT/zwiper-web"
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$ROOT/target} SQLX_OFFLINE=true BACKEND_URL=http://localhost:3000 RUST_LOG=info RUST_BACKTRACE=0 nohup dx serve --platform web --port 8080 --hot-reload false --open false > /tmp/webz-dx.out 2>&1 &
for i in $(seq 1 200); do grep -q 'Build completed' /tmp/webz-dx.out && break; grep -q 'ERROR' /tmp/webz-dx.out && break; sleep 3; done
grep -E 'Build completed|ERROR' /tmp/webz-dx.out | head -3 | cut -c1-300
