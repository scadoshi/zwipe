#!/usr/bin/env bash
# Boots zwipe locally for a check: a throwaway copy of this machine's dev database, migrated
# and seeded, zerver on :3100 and the web app on :8100. Local only. Undo with down.sh.
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
. "$(dirname "${BASH_SOURCE[0]}")/common.sh"
mkdir -p "$RUN_DIR/logs"
started=$SECONDS
step() { echo "[$((SECONDS - started))s] $*"; }

TEMPLATE="$(with_user "$(template_url)")"
require_local "$TEMPLATE"
# down.sh tears this run down from the same server.
echo "$TEMPLATE" >"$RUN_DIR/template.url"
TEMPLATE_DB="$(db_name "$TEMPLATE")"
ADMIN="$(with_db "$TEMPLATE" postgres)"
VERIFY_URL="$(with_db "$TEMPLATE" "$VERIFY_DB")"
[ "$TEMPLATE_DB" != "$VERIFY_DB" ] || { echo "the template can't be $VERIFY_DB itself" >&2; exit 1; }

for port in "$ZERVER_PORT" "$WEB_PORT"; do
  if (exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null; then
    echo "port $port is taken (an earlier run? down.sh stops it); set ZERVER_PORT / WEB_PORT for others" >&2
    exit 1
  fi
done

step "postgres"
if ! psql "$ADMIN" -Atqc 'select 1' >/dev/null 2>&1; then
  if command -v brew >/dev/null; then
    brew services start "$(brew list --formula | grep -m1 '^postgresql')" >/dev/null
    for _ in $(seq 1 20); do psql "$ADMIN" -Atqc 'select 1' >/dev/null 2>&1 && break; sleep 1; done
  fi
  psql "$ADMIN" -Atqc 'select 1' >/dev/null 2>&1 || {
    echo "postgres isn't answering at $ADMIN; start it (brew services start postgresql@18, sudo systemctl start postgresql, or docker start <container>) and rerun" >&2
    exit 1
  }
fi

step "copy $TEMPLATE_DB to $VERIFY_DB"
others="$(psql "$ADMIN" -Atqc "select count(*) from pg_stat_activity where datname = '$TEMPLATE_DB'")"
if [ "$others" != "0" ]; then
  echo "can't copy $TEMPLATE_DB: $others connection(s) are open on it, and Postgres copies a database only while nothing is connected. Who:" >&2
  psql "$ADMIN" -Atqc "select coalesce(nullif(application_name, ''), 'unnamed') || ' (pid ' || pid || ')' from pg_stat_activity where datname = '$TEMPLATE_DB'" >&2
  echo "Stop them (a local zerver: pkill -f target/debug/zerver; a psql session: quit it; an editor's" >&2
  echo "rust-analyzer keeps sqlx's query! connections open: pkill -f rust-analyzer-proc-macro-srv," >&2
  echo "which it restarts on demand) and rerun." >&2
  exit 1
fi
PGOPTIONS="-c client_min_messages=warning" psql "$ADMIN" -v ON_ERROR_STOP=1 -qc "drop database if exists $VERIFY_DB" -c "create database $VERIFY_DB template \"$TEMPLATE_DB\""

step "migrations"
# A dev database that ran another branch's migrations carries tables this branch has never seen,
# so a check could pass against a schema the branch won't ship with: refuse it, naming them.
extra="$(psql "$VERIFY_URL" -Atqc "select version from _sqlx_migrations order by version" |
  grep -vxF -f <(ls "$ROOT/zerver/migrations" | cut -d_ -f1) || true)"
if [ -n "$extra" ]; then
  PGOPTIONS="-c client_min_messages=warning" psql "$ADMIN" -qc "drop database if exists $VERIFY_DB"
  echo "$TEMPLATE_DB ran migrations this branch doesn't have: $(echo $extra | tr ' ' ',')." >&2
  echo "They come from another branch; rebuild the dev database from this one (or point VERIFY_TEMPLATE_URL at one that matches) and rerun." >&2
  exit 1
fi
if ! (cd "$ROOT" && DATABASE_URL="$VERIFY_URL" sqlx migrate run --source zerver/migrations) \
  >"$RUN_DIR/migrate.out" 2>&1; then
  echo "the migrations failed:" >&2
  tail -5 "$RUN_DIR/migrate.out" >&2
  exit 1
fi

step "seed (zcripts/seed/seed.sh small --reset)"
DATABASE_URL="$VERIFY_URL" "$ROOT/zcripts/seed/seed.sh" small --reset

step "build zerver"
(cd "$ROOT" && SQLX_OFFLINE=true cargo build -q --bin zerver)
step "start zerver on :$ZERVER_PORT"
(
  cd "$RUN_DIR"
  env JWT_SECRET="$(openssl rand -hex 32)" DATABASE_URL="$VERIFY_URL" \
    BIND_ADDRESS="127.0.0.1:$ZERVER_PORT" \
    ALLOWED_ORIGINS="http://localhost:$WEB_PORT,http://127.0.0.1:$WEB_PORT" \
    RUST_LOG=info RUST_BACKTRACE=1 LOG_DIR="$RUN_DIR/logs" \
    RESEND_API_KEY=local-verify RESEND_EMAIL_FROM=support@zwipe.net MIN_CLIENT_VERSION=0.0.0 \
    nohup "$ROOT/target/debug/zerver" >"$RUN_DIR/zerver.out" 2>&1 &
  echo $! >"$RUN_DIR/zerver.pid"
)
for _ in $(seq 1 60); do curl -sf "http://127.0.0.1:$ZERVER_PORT/health" >/dev/null && break; sleep 1; done
curl -sf "http://127.0.0.1:$ZERVER_PORT/health" >/dev/null || { echo "zerver didn't come up; see $RUN_DIR/zerver.out" >&2; exit 1; }

step "check account"
# A localcheck the dev database already holds has a password nobody here knows: it goes first.
psql "$VERIFY_URL" -qc "delete from users where username = 'localcheck'"
curl -s -o /dev/null -X POST "http://127.0.0.1:$ZERVER_PORT/api/auth/register" -H 'content-type: application/json' \
  -d '{"username":"localcheck","email":"localcheck@localhost.test","password":"Verify-local-pass-1!"}'
verified="$(psql "$VERIFY_URL" -Atqc "update users set email_verified_at = coalesce(email_verified_at, now()) where username = 'localcheck' returning 1")"
[ "$verified" = 1 ] || { echo "couldn't make the localcheck account; see $RUN_DIR/zerver.out" >&2; exit 1; }

step "start the web app on :$WEB_PORT (dx serve)"
# The build bakes this in, and the process env wins over zwiper-web/.env. The check only trips on
# a later edit to this line; drive.mjs's own check of the page's API is the one a sign-in relies on.
BACKEND="http://localhost:$ZERVER_PORT"
require_local_backend "$BACKEND"
(
  cd "$ROOT/zwiper-web"
  BACKEND_URL="$BACKEND" RUST_LOG=info RUST_BACKTRACE=1 \
    nohup dx serve --port "$WEB_PORT" --hot-reload false --watch false --open false >"$RUN_DIR/dx.out" 2>&1 &
)
for _ in $(seq 1 180); do grep -aq "Build completed" "$RUN_DIR/dx.out" && break; sleep 5; done
grep -aq "Build completed" "$RUN_DIR/dx.out" || { echo "the web build didn't finish; see $RUN_DIR/dx.out" >&2; exit 1; }

step "up"
cat <<SUMMARY
web     http://localhost:$WEB_PORT/login
api     http://127.0.0.1:$ZERVER_PORT
db      $VERIFY_URL
login   localcheck / Verify-local-pass-1!
logs    $RUN_DIR (zerver.out, dx.out)
stop    $(dirname "${BASH_SOURCE[0]}")/down.sh
SUMMARY
