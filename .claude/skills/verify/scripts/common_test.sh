#!/usr/bin/env bash
# Checks common.sh: a database URL with no user gets this machine's user, and the web build's
# backend must be this machine.
# Usage: .claude/skills/verify/scripts/common_test.sh
set -uo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$DIR/common.sh"
failures=0

fail() {
  echo "FAIL: $*" >&2
  failures=$((failures + 1))
}

user_is() {
  local got
  got="$(USER=alice with_user "$1")"
  [ "$got" = "$2" ] || fail "with_user $1 -> $got, want $2"
}

user_is "postgres:///zerver" "postgres:///zerver?user=alice"
user_is "postgres://localhost:5432/zerver" "postgres://localhost:5432/zerver?user=alice"
user_is "postgres:///zerver?host=/tmp" "postgres:///zerver?host=/tmp&user=alice"
user_is "postgres://bob@localhost/zerver" "postgres://bob@localhost/zerver"
user_is "postgres://bob:secret@localhost/zerver" "postgres://bob:secret@localhost/zerver"
user_is "postgres:///zerver?user=bob" "postgres:///zerver?user=bob"
user_is "postgres:///zerver?sslmode=disable&user=bob" "postgres:///zerver?sslmode=disable&user=bob"

backend() {
  if (require_local_backend "$1") 2>/dev/null; then echo pass; else echo refused; fi
}

backend_is() {
  local got
  got="$(backend "$1")"
  [ "$got" = "$2" ] || fail "require_local_backend $1 -> $got, want $2"
}

backend_is "http://localhost:3100" pass
backend_is "http://127.0.0.1:3100" pass
backend_is "http://[::1]:3100" pass
backend_is "http://localhost" pass
backend_is "https://api.zwipe.net" refused
backend_is "http://localhost.example.com:3100" refused
backend_is "http://localhost@api.zwipe.net" refused
backend_is "http://127.0.0.1.nip.io:3100" refused
backend_is "localhost:3100" refused
backend_is "" refused

if [ "$failures" -ne 0 ]; then
  echo "$failures common check(s) failed" >&2
  exit 1
fi
echo "common: all checks pass"
