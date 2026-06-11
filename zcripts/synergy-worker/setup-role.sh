#!/bin/bash
set -euo pipefail

# creates the least-privilege postgres role the synergy worker connects as.
#
# the worker (separate private service) shares zwipe's database but should
# only ever touch:
#   synergy_requests   read/write/delete  (its own work queue)
#   commander_synergy  read/insert/update (the cache it fills; upsert = insert+update)
#   decks              read-only          (demand discovery)
#   scryfall_data      read-only          (commander identity resolution)
# nothing else — no users, sessions, deck_cards, etc. a compromised or buggy
# worker cannot read accounts or touch deck contents.
#
# usage:
#   ./setup-role.sh [database] [role]
#     database  defaults to zwipe (prod). dev: pass your dev db name (e.g. zerver)
#     role      defaults to synergy_worker
#
#   SYNERGY_WORKER_PASSWORD=...  use a specific password (otherwise generated)
#   PSQL="sudo -u postgres psql" override how psql runs (default: sudo -u
#   postgres psql on linux, plain psql elsewhere)
#
# idempotent: re-running updates the password and re-applies grants.
# run AFTER the synergy tables migration — grants on missing tables fail.

DB="${1:-zwipe}"
ROLE="${2:-synergy_worker}"
PASSWORD="${SYNERGY_WORKER_PASSWORD:-$(openssl rand -hex 24)}"

if [[ -n "${PSQL:-}" ]]; then
    : # caller override
elif [[ "$OSTYPE" == "darwin"* ]]; then
    PSQL="psql"
else
    PSQL="sudo -u postgres psql"
fi

run() { $PSQL -d "$DB" -v ON_ERROR_STOP=1 "$@"; }

echo "checking synergy tables exist in '$DB'..."
MISSING=$(run -tAc "SELECT 2 - count(*) FROM pg_tables WHERE schemaname = 'public' AND tablename IN ('synergy_requests', 'commander_synergy')")
if [[ "$MISSING" != "0" ]]; then
    echo "error: synergy tables missing — run the migration first (zerver/migrations/20260611120000_create_synergy_tables.sql)"
    exit 1
fi

echo "creating role '$ROLE' (idempotent)..."
run <<SQL
DO \$\$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = '$ROLE') THEN
        CREATE ROLE $ROLE LOGIN;
    END IF;
END
\$\$;
ALTER ROLE $ROLE WITH LOGIN PASSWORD '$PASSWORD' NOSUPERUSER NOCREATEDB NOCREATEROLE;
SQL

echo "applying least-privilege grants..."
run <<SQL
GRANT CONNECT ON DATABASE $DB TO $ROLE;
GRANT USAGE ON SCHEMA public TO $ROLE;

-- start from zero so re-runs converge even if grants were once wider
REVOKE ALL ON ALL TABLES IN SCHEMA public FROM $ROLE;

-- its own work queue
GRANT SELECT, INSERT, UPDATE, DELETE ON synergy_requests TO $ROLE;
-- the cache it fills (upsert needs insert + update; never delete)
GRANT SELECT, INSERT, UPDATE ON commander_synergy TO $ROLE;
-- demand discovery + identity resolution, strictly read-only
GRANT SELECT ON decks, scryfall_data TO $ROLE;
SQL

echo
echo "grants now held by $ROLE:"
run -c "SELECT table_name, string_agg(privilege_type, ', ' ORDER BY privilege_type) AS privileges FROM information_schema.table_privileges WHERE grantee = '$ROLE' GROUP BY table_name ORDER BY table_name"

echo
echo "worker DATABASE_URL (copy into the worker's .env — shown once, not stored):"
echo "  postgres://$ROLE:$PASSWORD@127.0.0.1/$DB"
echo
echo "note: use 127.0.0.1 (tcp/password auth), not localhost — peer auth blocks"
echo "unix-socket connections for non-system users (same rule as the zwipe user)."
