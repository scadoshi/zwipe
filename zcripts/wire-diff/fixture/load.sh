#!/usr/bin/env bash
# Loads cards.csv.gz into a LOCAL database that its migrations have already built, then refreshes
# the card views, so the seed and the wire diff have a catalog without the Scryfall sync.
#
#   DATABASE_URL=postgres://user@localhost:5432/zwipe_ci zcripts/wire-diff/fixture/load.sh
#
# Rows go in by column name, so a column the schema added since the fixture was made takes its
# NULL; a required column the fixture lacks fails the load, and generate.sh makes a new one.
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$DIR/../../.." && pwd)"
# shellcheck source=../../seed/guard.sh
. "$ROOT/zcripts/seed/guard.sh"
seed_guard

# Parents before the rows that point at them. A row a migration already wrote (oou_sets) stays.
TABLES="scryfall_data card_profiles oracle_tags card_oracle_tags oou_sets"
{
  echo "BEGIN;"
  echo "CREATE TEMP TABLE fixture_rows (tbl text NOT NULL, row jsonb NOT NULL) ON COMMIT DROP;"
  echo "COPY fixture_rows FROM STDIN WITH (FORMAT csv);"
  gzip -dc "$DIR/cards.csv.gz"
  echo '\.'
  for t in $TABLES; do
    echo "INSERT INTO $t SELECT (jsonb_populate_record(NULL::$t, row)).* FROM fixture_rows WHERE tbl = '$t' ON CONFLICT DO NOTHING;"
  done
  echo "COMMIT;"
  echo "REFRESH MATERIALIZED VIEW latest_cards;"
  echo "REFRESH MATERIALIZED VIEW card_signal_rollup;"
  echo "REFRESH MATERIALIZED VIEW otag_context_signal_rollup;"
  echo "ANALYZE;"
} | psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q
echo "loaded: $(psql "$DATABASE_URL" -Atqc 'select count(*) from latest_cards') cards"
