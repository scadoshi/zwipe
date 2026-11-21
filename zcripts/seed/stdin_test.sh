#!/usr/bin/env bash
# Checks that seed.sh and activity.sh hand psql their SQL over stdin, with their settings as SET
# lines quoted through psql variables, so they work with a psql that runs in a container and sees
# neither the host's files nor its environment.
# Usage: zcripts/seed/stdin_test.sh
set -uo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/seed-stdin-test.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT
failures=0

# A psql that refuses -f, records its arguments and stdin per call, and never reads PGOPTIONS.
mkdir -p "$WORK/bin"
cat > "$WORK/bin/psql" <<'EOF'
#!/usr/bin/env bash
n=$(( $(ls "$CALLS" | wc -l) / 2 + 1 ))
printf '%s\n' "$@" > "$CALLS/$n.args"
cat > "$CALLS/$n.stdin"
for a in "$@"; do [ "$a" = -f ] && { echo "psql: -f given" >&2; exit 1; }; done
exit 0
EOF
chmod +x "$WORK/bin/psql"

# Runs a script with the fake psql on a local URL and PGOPTIONS cleared; prints its exit code.
run() {
  rm -rf "$WORK/calls"; mkdir -p "$WORK/calls"
  env -u PGOPTIONS PATH="$WORK/bin:$PATH" CALLS="$WORK/calls" \
    DATABASE_URL="postgres://zwipe@localhost:5432/zwipe" "$@" </dev/null >/dev/null 2>&1
  echo $?
}

check() {
  if ! eval "$2"; then
    echo "FAIL: $1" >&2
    failures=$((failures + 1))
  fi
}

calls="$WORK/calls"

[ "$(run "$DIR/seed.sh" small --reset 7)" = 0 ] || { echo "FAIL: seed.sh small --reset 7 exited nonzero" >&2; failures=$((failures + 1)); }
check "seed.sh resets over stdin" 'cmp -s "$calls/1.stdin" "$DIR/reset.sql"'
check "seed.sh passes the size as a psql variable" 'grep -qx "seed_size=small" "$calls/2.args"'
check "seed.sh passes the seed as a psql variable" 'grep -qx "seed=7" "$calls/2.args"'
check "seed.sh sets the size first" 'head -n 2 "$calls/2.stdin" | grep -qx "SET zwipe.seed_size = :'"'"'seed_size'"'"';"'
check "seed.sh sets the seed first" 'head -n 2 "$calls/2.stdin" | grep -qx "SET zwipe.seed = :'"'"'seed'"'"';"'
check "seed.sh sends seed.sql after the settings" 'tail -n +3 "$calls/2.stdin" | cmp -s - "$DIR/seed.sql"'

[ "$(run "$DIR/activity.sh" "o'brien" 0.5)" = 0 ] || { echo "FAIL: activity.sh exited nonzero" >&2; failures=$((failures + 1)); }
check "activity.sh passes the account as a psql variable" 'grep -qx "seed_user=o'"'"'brien" "$calls/1.args"'
check "activity.sh passes the seed as a psql variable" 'grep -qx "seed=0.5" "$calls/1.args"'
check "activity.sh sets the account first" 'head -n 2 "$calls/1.stdin" | grep -qx "SET zwipe.seed_user = :'"'"'seed_user'"'"';"'
check "activity.sh sets the seed first" 'head -n 2 "$calls/1.stdin" | grep -qx "SET zwipe.seed = :'"'"'seed'"'"';"'
check "activity.sh sends activity.sql after the settings" 'tail -n +3 "$calls/1.stdin" | cmp -s - "$DIR/activity.sql"'

if [ "$failures" -gt 0 ]; then
  echo "$failures failed" >&2
  exit 1
fi
echo "ok"
