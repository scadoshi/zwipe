#!/bin/bash
# The per-commit gate for zwiper-web, run from anywhere in the checkout (dx build too with "dx").
cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)" || exit 1
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$PWD/target} SQLX_OFFLINE=true BACKEND_URL=http://localhost:3000 RUST_LOG=info RUST_BACKTRACE=0
set -o pipefail
step() { echo "== $1"; shift; "$@" > /tmp/webz-gate-step.log 2>&1 || { tail -40 /tmp/webz-gate-step.log; echo "FAIL"; exit 1; }; }
step fmt cargo +nightly fmt --check
step clippy cargo clippy --workspace --all-targets -- -D warnings
step clippy-wasm cargo clippy -p zwiper-web --target wasm32-unknown-unknown -- -D warnings
step test cargo test -p zwiper-web
step doc env RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p zwiper-web
if [ "$1" = dx ]; then cd zwiper-web; step dx dx build --platform web; fi
echo GREEN
