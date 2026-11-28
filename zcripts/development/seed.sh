#!/usr/bin/env bash
# Seeds accounts, decks, deck cards and swipes on a LOCAL database for testing
# a branch before it merges: seed_tester gets a busy year of activity, the
# numbered accounts the swipes that built their decks, and the pooled signal
# tables the sum of them all. Refuses any database that isn't on this machine.
#
# Usage: DATABASE_URL=postgres://user@localhost:5432/zwipe zcripts/development/seed.sh <small|load|reset> [--reset] [seed]
#   small    the three test accounts and 40 users
#   load     the three test accounts and 100,000 users
#   reset    removes every seeded account and stops
#   --reset  removes the seeded accounts first, then seeds
#   seed     a whole number for a different, repeatable world (default 42)
set -euo pipefail

usage() { echo "usage: DATABASE_URL=... $0 <small|load|reset> [--reset] [seed]" >&2; exit 2; }
[ $# -ge 1 ] || usage
SIZE="$1"; shift
RESET=false
SEED=42
for arg in "$@"; do
  case "$arg" in
    --reset) RESET=true ;;
    *) [[ "$arg" =~ ^[0-9]{1,9}$ ]] || usage; SEED="$arg" ;;
  esac
done
case "$SIZE" in small|load|reset) ;; *) usage ;; esac

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=guard.sh
source "$DIR/guard.sh"
seed_guard

# The SQL goes over stdin and the settings as SET lines, quoted by psql, so a psql that runs in
# a container, seeing neither this machine's files nor its environment, works too.
if [ "$SIZE" = reset ] || [ "$RESET" = true ]; then
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q < "$DIR/seed_reset.sql"
  echo "removed the seeded accounts"
fi
[ "$SIZE" = reset ] && exit 0

{
  echo "SET zwipe.seed_size = :'seed_size';"
  echo "SET zwipe.seed = :'seed';"
  cat "$DIR/seed.sql"
} | psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -v seed_size="$SIZE" -v seed="$SEED"

# seed_tester's year, its random generator seeded from the world's seed.
ACTIVITY_SEED="$(printf '0.%03d' $((SEED % 1000)))"
{
  echo "SET zwipe.seed_user = :'seed_user';"
  echo "SET zwipe.seed = :'seed';"
  cat "$DIR/seed_activity.sql"
} | psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -v seed_user=seed_tester -v seed="$ACTIVITY_SEED"

psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q < "$DIR/seed_pools.sql"
