#!/usr/bin/env bash
# Checks guard.sh: every local form passes, everything else is refused.
# Usage: zcripts/seed/guard_test.sh
set -uo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
failures=0

# Runs the guard in a clean subshell with DATABASE_URL=$1 and any extra
# environment in the rest of the arguments, under seed.sh's own `set -euo
# pipefail`; prints pass or refused.
verdict() {
  local url="$1"; shift
  if env -u PGHOST -u PGHOSTADDR -u PGSERVICE "$@" DATABASE_URL="$url" \
    bash -c 'set -euo pipefail; source "$0"; seed_guard' "$DIR/guard.sh" 2>/dev/null; then
    echo pass
  else
    echo refused
  fi
}

expect() {
  local want="$1"; shift
  local got
  got="$(verdict "$@")"
  if [ "$got" != "$want" ]; then
    echo "FAIL: $* -> $got, want $want" >&2
    failures=$((failures + 1))
  fi
}

expect pass "postgres://zwipe@localhost:55432/zwipe"
expect pass "postgresql://zwipe:secret@127.0.0.1/zwipe"
expect pass "postgres://zwipe@[::1]:5432/zwipe"
expect pass "postgres:///zwipe"
expect pass "postgres:///zwipe?host=/run/postgresql"
expect pass "postgres:///zwipe?host=%2Frun%2Fpostgresql"
expect pass "postgres://localhost/zwipe?sslmode=disable&host=localhost"
expect pass "postgres:///zwipe" PGHOST=localhost
expect pass "postgres:///zwipe" PGHOST=/run/postgresql

expect refused "postgres://zwipe@db.example.com/zwipe"
expect refused "postgres://zwipe@10.0.0.5:5432/zwipe"
expect refused "postgres://zwipe@localhost.example.com/zwipe"
expect refused "postgres://zwipe@localhost,db.example.com/zwipe"
expect refused "postgres://localhost/zwipe?host=db.example.com"
expect refused "postgres:///zwipe?host=localhost,db.example.com"
expect refused "postgres:///zwipe?host=localhost%2Cdb.example.com"
expect refused "postgres://localhost/zwipe?hostaddr=10.0.0.5"
expect refused "postgres://localhost/zwipe?service=prod"
expect refused "postgres://localhost/zwipe?%68ost=db.example.com"
expect refused "postgres://localhost/zwipe?%68ostaddr=10.0.0.5"
expect refused "postgres://localhost/zwipe?%73ervice=prod"
expect refused "postgres://localhost/zwipe?host=*"
expect refused "postgres://localhost/zwipe?a=b
&host=db.example.com"
expect refused "$(printf 'postgres://localhost/zwipe?a=b\t&host=db.example.com')"
expect refused "postgres:///zwipe" PGHOST=db.example.com
expect refused "postgres://localhost/zwipe" PGHOSTADDR=10.0.0.5
expect refused "postgres://localhost/zwipe" PGSERVICE=prod
expect refused "host=localhost dbname=zwipe"
expect refused "mysql://localhost/zwipe"
expect refused ""

if [ "$failures" -ne 0 ]; then
  echo "$failures guard check(s) failed" >&2
  exit 1
fi
echo "guard: all checks pass"
