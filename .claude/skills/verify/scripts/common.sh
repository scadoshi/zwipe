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

# Refuses any database URL that isn't this machine, as zcripts/seed/activity.sh does.
require_local() {
  local url="$1" rest host
  rest="${url#*://}"
  rest="${rest#*@}"
  case "$rest" in
    \[*) host="${rest#[}"; host="${host%%]*}" ;;
    *) host="${rest%%[:/?]*}" ;;
  esac
  case "$host" in
    ""|localhost|127.0.0.1|::1) ;;
    *) echo "refusing: $host is not this machine; verify only runs against a local database" >&2; exit 1 ;;
  esac
  if [[ "$url" =~ [?\&]host=([^\&]*) ]]; then
    case "${BASH_REMATCH[1]}" in
      localhost|127.0.0.1|::1|/*) ;;
      *) echo "refusing: host=${BASH_REMATCH[1]} is not this machine" >&2; exit 1 ;;
    esac
  fi
}

# The database name in a URL, and the same URL pointed at another database.
db_name() { local base="${1%%\?*}"; echo "${base##*/}"; }
with_db() {
  local base="${1%%\?*}" query=""
  [ "$base" != "$1" ] && query="?${1#*\?}"
  echo "${base%/*}/$2$query"
}
