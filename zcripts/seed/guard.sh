# Sourced by the seed scripts: exits unless DATABASE_URL points at a database on
# this machine, through a local socket or localhost. Seeded rows in prod would
# show in zwipe.net's public counters and in real users' feeds.
#
# It accepts only a postgres:// or postgresql:// URL whose host, every host=
# parameter and libpq's PGHOST all name this machine. hostaddr=, service=, a
# percent-encoded keyword, and PGHOSTADDR or PGSERVICE are refused outright,
# since each can send libpq somewhere the URL doesn't name.

seed_guard_refuse() { echo "refusing: $1; the seed only runs against a database on this machine" >&2; exit 1; }

# Whether $1 names this machine: empty (the default socket), localhost, a
# loopback address, or a socket directory.
seed_guard_local_host() {
  case "$1" in
    ""|localhost|127.0.0.1|::1|/*) return 0 ;;
    *) return 1 ;;
  esac
}

seed_guard() {
  : "${DATABASE_URL:?set DATABASE_URL to the local database}"
  case "$DATABASE_URL" in
    postgres://*|postgresql://*) ;;
    *) seed_guard_refuse "DATABASE_URL is not a postgres:// URL" ;;
  esac
  [ -z "${PGHOSTADDR:-}" ] || seed_guard_refuse "PGHOSTADDR is set"
  [ -z "${PGSERVICE:-}" ] || seed_guard_refuse "PGSERVICE is set"

  local rest query host
  rest="${DATABASE_URL#*://}"
  query=""
  case "$rest" in *\?*) query="${rest#*\?}" ;; esac
  rest="${rest%%\?*}"
  rest="${rest%%/*}"
  case "$rest" in *@*) rest="${rest##*@}" ;; esac
  case "$rest" in *,*) seed_guard_refuse "the URL lists several hosts" ;; esac
  case "$rest" in
    \[*) host="${rest#[}"; host="${host%%]*}" ;;
    *) host="${rest%%:*}" ;;
  esac
  seed_guard_local_host "$host" || seed_guard_refuse "$host is not this machine"
  # With no host in the URL, libpq falls back to PGHOST.
  if [ -z "$host" ]; then
    case "${PGHOST:-}" in *,*) seed_guard_refuse "PGHOST lists several hosts" ;; esac
    seed_guard_local_host "${PGHOST:-}" || seed_guard_refuse "PGHOST=${PGHOST} is not this machine"
  fi

  local param key value params
  # read splits on & without glob expansion, so a * or ? stays as written.
  IFS='&' read -ra params <<< "$query"
  for param in "${params[@]}"; do
    key="${param%%=*}"
    value="${param#*=}"
    # libpq percent-decodes keywords too, so an encoded key could be host or
    # hostaddr in disguise.
    case "$key" in *%*) seed_guard_refuse "the query has a percent-encoded keyword" ;; esac
    case "$key" in
      host)
        case "$value" in *,*|*%2[cC]*) seed_guard_refuse "host= lists several hosts" ;; esac
        value="${value//%2[fF]//}"
        seed_guard_local_host "$value" || seed_guard_refuse "host=$value is not this machine"
        ;;
      hostaddr) seed_guard_refuse "hostaddr= is set" ;;
      service) seed_guard_refuse "service= is set" ;;
    esac
  done
}
