#!/usr/bin/env bash
# Compares what two zervers answer to the requests shipped clients send: main (origin/main) and
# a candidate commit, each on its own copy of one seeded local database. Fails unless every
# difference is listed in approved.json. Local databases only.
#
#   zcripts/wire-diff/run.sh [candidate-ref] [main-ref]     (defaults: HEAD, origin/main)
#
# It builds committed trees: uncommitted changes in the checkout aren't part of the candidate.
#
# Env: VERIFY_TEMPLATE_URL (the dev database to copy, as the verify skill takes it),
# MAIN_PORT / CAND_PORT (3201 / 3202), WIRE_DIR (/tmp/wire-diff), KEEP=1 keeps the databases.
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
CAND_REF="${1:-HEAD}"
MAIN_REF="${2:-origin/main}"
WIRE_DIR="${WIRE_DIR:-/tmp/wire-diff}"
MAIN_PORT="${MAIN_PORT:-3201}"
CAND_PORT="${CAND_PORT:-3202}"
# The verify skill's template lookup and local-only guard.
. "$ROOT/zcripts/development/stack_common.sh"
started=$SECONDS
step() { echo "[$((SECONDS - started))s] $*"; }

TEMPLATE="$(template_url)"
require_local "$TEMPLATE"
TEMPLATE_DB="$(db_name "$TEMPLATE")"
ADMIN="$(with_db "$TEMPLATE" postgres)"
SEEDED="zwipe_wire_seed"
psql_admin() { PGOPTIONS="-c client_min_messages=warning" psql "$ADMIN" -v ON_ERROR_STOP=1 -qc "$1"; }

# How to build a template that matches main, as CI does: main's migrations, then the card fixture.
template_recipe() {
  local url
  url="$(with_db "$TEMPLATE" zwipe_wire_template)"
  cat >&2 <<RECIPE
The template has to be main's schema. Build one from $MAIN_REF's migrations and the card fixture, as CI does:
  psql "$ADMIN" -c 'create database zwipe_wire_template'
  mkdir -p /tmp/wire-main && git archive $MAIN_REF zerver/migrations | tar -x -C /tmp/wire-main
  DATABASE_URL="$url" sqlx migrate run --source /tmp/wire-main/zerver/migrations
  DATABASE_URL="$url" zcripts/wire-diff/fixture/load.sh
then rerun with VERIFY_TEMPLATE_URL="$url".
RECIPE
}

# Runs $1's migrations on $2, and on a failure shows sqlx's own words and the recipe.
migrate() {
  if ! (cd "$WIRE_DIR/$1" && DATABASE_URL="$2" sqlx migrate run --source zerver/migrations) \
    >"$WIRE_DIR/migrate-$1.out" 2>&1; then
    echo "$1's migrations failed on $(db_name "$2"):" >&2
    tail -5 "$WIRE_DIR/migrate-$1.out" >&2
    template_recipe
    exit 1
  fi
}

for port in "$MAIN_PORT" "$CAND_PORT"; do
  if (exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null; then
    echo "port $port is taken; set MAIN_PORT / CAND_PORT" >&2
    exit 1
  fi
done

cleanup() {
  for side in main cand; do
    [ -f "$WIRE_DIR/$side.pid" ] && kill "$(cat "$WIRE_DIR/$side.pid")" 2>/dev/null || true
    rm -f "$WIRE_DIR/$side.pid"
  done
  if [ "${KEEP:-0}" != 1 ]; then
    for db in "$SEEDED" zwipe_wire_main zwipe_wire_cand; do psql_admin "drop database if exists $db" || true; done
  fi
}
trap cleanup EXIT

git -C "$ROOT" fetch -q origin
mkdir -p "$WIRE_DIR"
for side in main cand; do
  ref="$MAIN_REF"; [ "$side" = cand ] && ref="$CAND_REF"
  sha="$(git -C "$ROOT" rev-parse --short "$ref")"
  step "export $side ($ref, $sha)"
  rm -rf "${WIRE_DIR:?}/$side"
  mkdir -p "$WIRE_DIR/$side"
  git -C "$ROOT" archive "$sha" | tar -x -C "$WIRE_DIR/$side"
  # git archive stamps files with the commit's time, which cargo would read as older than a
  # build of another commit in the same target dir: a new commit gets fresh times, so it rebuilds.
  if [ "$(cat "$WIRE_DIR/target-$side.sha" 2>/dev/null)" != "$sha" ]; then
    find "$WIRE_DIR/$side" -type f -exec touch {} +
  fi
  step "build $side's zerver"
  (cd "$WIRE_DIR/$side" && SQLX_OFFLINE=true CARGO_TARGET_DIR="$WIRE_DIR/target-$side" cargo build -q --bin zerver)
  echo "$sha" >"$WIRE_DIR/target-$side.sha"
done

step "seeded database from $TEMPLATE_DB"
others="$(psql "$ADMIN" -Atqc "select count(*) from pg_stat_activity where datname = '$TEMPLATE_DB'")"
if [ "$others" != "0" ]; then
  echo "can't copy $TEMPLATE_DB: $others connection(s) are open on it; close them (a local zerver, a psql session) and rerun" >&2
  exit 1
fi
# A template that ran another branch's migrations can't take main's: name them and say how to
# build one that matches.
extra="$({ psql "$TEMPLATE" -Atqc "select version from _sqlx_migrations order by version" 2>/dev/null || true; } |
  extra_migrations "$WIRE_DIR/main/zerver/migrations")"
if [ -n "$extra" ]; then
  echo "$TEMPLATE_DB ran migrations $MAIN_REF doesn't have: $extra." >&2
  template_recipe
  exit 1
fi
psql_admin "drop database if exists $SEEDED"
psql_admin "create database $SEEDED template \"$TEMPLATE_DB\""
SEEDED_URL="$(with_db "$TEMPLATE" "$SEEDED")"
migrate main "$SEEDED_URL"
# Main's own seed when it has one, so the seed matches the schema it runs on.
SEED_SH="$WIRE_DIR/main/zcripts/development/seed.sh"
[ -x "$SEED_SH" ] || SEED_SH="$ROOT/zcripts/development/seed.sh"
DATABASE_URL="$SEEDED_URL" "$SEED_SH" small --reset >/dev/null

for side in main cand; do
  db="zwipe_wire_$side"
  port="$MAIN_PORT"; [ "$side" = cand ] && port="$CAND_PORT"
  url="$(with_db "$TEMPLATE" "$db")"
  step "$side: copy, its own migrations, zerver on :$port"
  psql_admin "drop database if exists $db"
  psql_admin "create database $db template $SEEDED"
  migrate "$side" "$url"
  (
    cd "$WIRE_DIR"
    env JWT_SECRET="$(openssl rand -hex 32)" DATABASE_URL="$url" BIND_ADDRESS="127.0.0.1:$port" \
      ALLOWED_ORIGINS="http://wire-diff.localhost" RUST_LOG=info RUST_BACKTRACE=1 LOG_DIR="$WIRE_DIR/logs-$side" \
      RESEND_API_KEY=local-wire-diff RESEND_EMAIL_FROM=support@zwipe.net MIN_CLIENT_VERSION=0.0.0 \
      nohup "$WIRE_DIR/target-$side/debug/zerver" >"$WIRE_DIR/$side.out" 2>&1 &
    echo $! >"$WIRE_DIR/$side.pid"
  )
done
for port in "$MAIN_PORT" "$CAND_PORT"; do
  for _ in $(seq 1 60); do curl -sf "http://127.0.0.1:$port/health" >/dev/null && break; sleep 1; done
  curl -sf "http://127.0.0.1:$port/health" >/dev/null || { echo "zerver on :$port didn't come up; see $WIRE_DIR/*.out" >&2; exit 1; }
done

step "replay"
node "$ROOT/zcripts/wire-diff/replay.mjs" "http://127.0.0.1:$MAIN_PORT" "http://127.0.0.1:$CAND_PORT"
