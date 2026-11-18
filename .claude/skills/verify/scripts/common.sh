# Shared by up.sh and down.sh: where a run keeps its state, and the local-only guard.

RUN_DIR="${VERIFY_DIR:-/tmp/zwipe-verify}"
ZERVER_PORT="${ZERVER_PORT:-3100}"
WEB_PORT="${WEB_PORT:-8100}"
VERIFY_DB="zwipe_verify"

# The database the verify copy is made from: VERIFY_TEMPLATE_URL, else zerver/.env's
# DATABASE_URL, else the dev setup's default.
template_url() {
  if [ -n "${VERIFY_TEMPLATE_URL:-}" ]; then
    echo "$VERIFY_TEMPLATE_URL"
  elif [ -f "$ROOT/zerver/.env" ] && grep -q '^DATABASE_URL=' "$ROOT/zerver/.env"; then
    grep '^DATABASE_URL=' "$ROOT/zerver/.env" | tail -1 | cut -d= -f2-
  else
    echo "postgres:///zerver?user=$(whoami)"
  fi
}

# Refuses any database URL that isn't this machine, through the seed scripts' shared guard
# (zcripts/seed/guard.sh): its host, every host= and PGHOST must be local, and hostaddr=,
# service=, PGHOSTADDR, PGSERVICE, percent-encoded keywords and control characters are refused.
require_local() {
  [ -f "$ROOT/zcripts/seed/guard.sh" ] || {
    echo "zcripts/seed/guard.sh is missing; this skill needs thinker's seed (feat/seed) merged" >&2
    exit 1
  }
  (
    export DATABASE_URL="$1"
    # shellcheck source=../../../../zcripts/seed/guard.sh
    . "$ROOT/zcripts/seed/guard.sh"
    seed_guard
  ) || exit 1
}

# The database name in a URL, and the same URL pointed at another database.
db_name() { local base="${1%%\?*}"; echo "${base##*/}"; }
with_db() {
  local base="${1%%\?*}" query=""
  [ "$base" != "$1" ] && query="?${1#*\?}"
  echo "${base%/*}/$2$query"
}
