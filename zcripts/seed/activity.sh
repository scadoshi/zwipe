#!/usr/bin/env bash
# Seeds a busy year of Activity for one account on a LOCAL database, for
# recording the Activity clip. It replaces that account's activity (see
# activity.sql) and refuses to run against any database that isn't on this
# machine, because seeded swipes in prod would land in zwipe.net's public
# counters.
#
# Usage: DATABASE_URL=postgres://user:pass@localhost:5432/zwipe zcripts/seed/activity.sh <username-or-email> [seed]
#   seed: a number from -1 to 1 for a different year (default 0.42)
set -euo pipefail

usage() { echo "usage: DATABASE_URL=... $0 <username-or-email> [seed]" >&2; exit 2; }
[ $# -ge 1 ] || usage
ACCOUNT="$1"
SEED="${2:-0.42}"

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=guard.sh
source "$DIR/guard.sh"
seed_guard

[[ "$SEED" =~ ^-?(0(\.[0-9]+)?|1(\.0+)?|\.[0-9]+)$ ]] || { echo "seed must be a number from -1 to 1" >&2; exit 2; }

PGOPTIONS="-c zwipe.seed_user=$ACCOUNT -c zwipe.seed=$SEED" \
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -f "$DIR/activity.sql"
