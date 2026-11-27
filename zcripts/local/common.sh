# Shared by up.sh and down.sh: where a run keeps its state, and the local-only guard.

# A run is keyed by its zerver port, which no two runs on one machine can share: its work dir
# and its database carry the port, so one run's down.sh never stops another's zerver or drops
# its database.
ZERVER_PORT="${ZERVER_PORT:-3100}"
WEB_PORT="${WEB_PORT:-8100}"
RUN_DIR="${VERIFY_DIR:-/tmp/zwipe-verify-$ZERVER_PORT}"
VERIFY_DB="${VERIFY_DB:-zwipe_verify_$ZERVER_PORT}"

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

# The template this run was copied from, for down.sh: VERIFY_TEMPLATE_URL, else the one up.sh
# saved in the run's dir, else template_url's default. A run on a non-default Postgres is then
# torn down from the same server it was made on.
saved_template_url() {
  if [ -n "${VERIFY_TEMPLATE_URL:-}" ]; then
    echo "$VERIFY_TEMPLATE_URL"
  elif [ -s "$RUN_DIR/template.url" ]; then
    cat "$RUN_DIR/template.url"
  else
    template_url
  fi
}

# The migration versions read on stdin (a database's applied ones, one per line) that the
# migrations folder $1 doesn't hold, joined by commas: what another branch's migrations left in a
# database. Prints nothing when every one is in the folder.
extra_migrations() {
  local known
  known="$(ls "$1" | cut -d_ -f1)"
  grep -vxF -e "" -f <(echo "$known") | paste -sd, - || true
}

# Whether $1 is a running zerver's pid. down.sh checks before it stops one, since a pid file can
# outlive its process and the pid can come back as anything.
is_zerver_pid() {
  [ -n "$1" ] || return 1
  case "$(ps -o command= -p "$1" 2>/dev/null)" in
    */zerver | */zerver\ * | zerver | zerver\ *) return 0 ;;
    *) return 1 ;;
  esac
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
    # shellcheck source=../seed/guard.sh
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

# The URL with this machine's user added when it names none, in its user info or a user=
# parameter: sqlx, unlike psql, doesn't fall back to the login name on every platform.
with_user() {
  local rest="${1#*://}" query=""
  rest="${rest%%\?*}"
  rest="${rest%%/*}"
  case "$1" in *\?*) query="&${1#*\?}" ;; esac
  case "$rest" in *@*) echo "$1"; return ;; esac
  case "$query" in *"&user="*) echo "$1"; return ;; esac
  local user="${USER:-$(whoami)}"
  case "$1" in
    *\?*) echo "$1&user=$user" ;;
    *) echo "$1?user=$user" ;;
  esac
}

# Exits unless $1, the API the web build calls, is an http:// URL on this machine: a check's
# sign-in then never reaches a real server.
require_local_backend() {
  local rest host
  case "$1" in
    http://*) rest="${1#http://}" ;;
    *) echo "refusing: BACKEND_URL=$1 is not an http:// URL on this machine" >&2; exit 1 ;;
  esac
  rest="${rest%%/*}"
  case "$rest" in *@*) echo "refusing: BACKEND_URL=$1 carries user info" >&2; exit 1 ;; esac
  case "$rest" in
    \[*) host="${rest%%]*}]" ;;
    *) host="${rest%%:*}" ;;
  esac
  case "$host" in
    localhost|127.0.0.1|"[::1]") ;;
    *) echo "refusing: BACKEND_URL=$1 is not this machine" >&2; exit 1 ;;
  esac
}
