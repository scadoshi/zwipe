#!/usr/bin/env bash
# Seeds accounts, decks and deck cards on a LOCAL database for testing a branch
# before it merges. Refuses any database that isn't on this machine.
#
# Usage: DATABASE_URL=postgres://user@localhost:5432/zwipe zcripts/seed/seed.sh <small|load|reset> [--reset] [seed]
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

if [ "$SIZE" = reset ] || [ "$RESET" = true ]; then
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -f "$DIR/reset.sql"
  echo "removed the seeded accounts"
fi
[ "$SIZE" = reset ] && exit 0

PGOPTIONS="-c zwipe.seed_size=$SIZE -c zwipe.seed=$SEED" \
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -f "$DIR/seed.sql"
