#!/usr/bin/env bash
# Checks stack_common.sh: a database URL with no user gets this machine's user, and the web build's
# backend must be this machine.
# Usage: zcripts/development/stack_common_test.sh
set -uo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=stack_common.sh
. "$DIR/stack_common.sh"
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

# Each run's state is its own, keyed by its zerver port, so one run's down.sh never touches
# another's server or database.
state_is() {
  local got
  got="$(env -u VERIFY_DIR -u VERIFY_DB ZERVER_PORT="$1" bash -c ". '$DIR/stack_common.sh'; echo \"\$RUN_DIR \$VERIFY_DB\"")"
  [ "$got" = "$2" ] || fail "state for port $1 -> $got, want $2"
}
state_is 3100 "/tmp/zwipe-verify-3100 zwipe_verify_3100"
state_is 3200 "/tmp/zwipe-verify-3200 zwipe_verify_3200"
got="$(VERIFY_DIR=/tmp/elsewhere VERIFY_DB=mine bash -c ". '$DIR/stack_common.sh'; echo \"\$RUN_DIR \$VERIFY_DB\"")"
[ "$got" = "/tmp/elsewhere mine" ] || fail "VERIFY_DIR and VERIFY_DB override -> $got"

# down.sh drops the database up.sh copied, from the template up.sh saved, unless
# VERIFY_TEMPLATE_URL names one.
scratch="$(mktemp -d)"
RUN_DIR="$scratch"
echo "postgres://me@localhost:55432/dev" >"$scratch/template.url"
got="$(env -u VERIFY_TEMPLATE_URL bash -c ". '$DIR/stack_common.sh'; RUN_DIR='$scratch'; saved_template_url")"
[ "$got" = "postgres://me@localhost:55432/dev" ] || fail "saved template -> $got"
got="$(VERIFY_TEMPLATE_URL=postgres:///other bash -c ". '$DIR/stack_common.sh'; RUN_DIR='$scratch'; saved_template_url")"
[ "$got" = "postgres:///other" ] || fail "VERIFY_TEMPLATE_URL wins -> $got"

# down.sh stops a pid only when it is a zerver: a reused pid, or another tool's, is left alone.
# A sleep named as a zerver: macOS kills a copied system binary at launch (its signature), so
# the name is set with exec -a instead of a copy.
(exec -a "$scratch/zerver" sleep 30) &
zerver_pid=$!
sleep 30 &
other_pid=$!
is_zerver_pid "$zerver_pid" || fail "a zerver's pid reads as not a zerver"
is_zerver_pid "$other_pid" && fail "a sleep's pid reads as a zerver"
is_zerver_pid "" && fail "an empty pid reads as a zerver"
kill "$zerver_pid" "$other_pid" 2>/dev/null
rm -rf "$scratch"

# A database's applied migrations that a migrations folder doesn't hold: another branch's.
migrations="$(mktemp -d)"
touch "$migrations/20260101000000_one.sql" "$migrations/20260102000000_two.sql"
got="$(printf '20260101000000\n20260102000000\n' | extra_migrations "$migrations")"
[ -z "$got" ] || fail "a database matching the folder -> $got, want nothing"
got="$(printf '20260101000000\n20261119120000\n20261119130000\n' | extra_migrations "$migrations")"
[ "$got" = "20261119120000,20261119130000" ] || fail "another branch's migrations -> $got"
got="$(printf '' | extra_migrations "$migrations")"
[ -z "$got" ] || fail "a database with none applied -> $got"
rm -rf "$migrations"

if [ "$failures" -ne 0 ]; then
  echo "$failures common check(s) failed" >&2
  exit 1
fi
echo "common: all checks pass"
