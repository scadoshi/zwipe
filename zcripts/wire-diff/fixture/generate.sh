#!/usr/bin/env bash
# Writes cards.csv.gz, the card catalog the CI wire-diff job seeds from, out of a LOCAL database
# that holds the synced catalog. Regenerate after a migration changes a card table:
#
#   DATABASE_URL=postgres://user@localhost:5432/zwipe zcripts/wire-diff/fixture/generate.sh
#
# Each line is a table's name and one row as JSON. load.sh puts them back. It holds Scryfall's
# card data only: the Commander tables the synergy worker fills stay out.
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$DIR/../../.." && pwd)"
# shellcheck source=../../development/guard.sh
. "$ROOT/zcripts/development/guard.sh"
seed_guard

# One table's rows, in key order, as CSV lines of (table, row).
rows() { echo "COPY (SELECT '$1', row_to_json(t)::text FROM ($2) t) TO STDOUT WITH (FORMAT csv);"; }

{
  cat "$DIR/select.sql"
  rows scryfall_data "SELECT sd.* FROM scryfall_data sd JOIN fixture_cards f ON f.id = sd.id ORDER BY sd.id"
  rows card_profiles "SELECT cp.* FROM card_profiles cp JOIN fixture_cards f ON f.id = cp.scryfall_data_id ORDER BY cp.scryfall_data_id"
  rows oracle_tags "SELECT ot.* FROM oracle_tags ot JOIN fixture_tags f ON f.id = ot.id ORDER BY ot.id"
  rows card_oracle_tags "SELECT cot.* FROM card_oracle_tags cot WHERE cot.oracle_id IN (SELECT oracle_id FROM fixture_cards) AND cot.oracle_tag IN (SELECT ot.slug FROM oracle_tags ot JOIN fixture_tags f ON f.id = ot.id) ORDER BY cot.oracle_id, cot.oracle_tag, cot.source"
  rows oou_sets "SELECT * FROM oou_sets ORDER BY code"
} | psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -qAt | gzip -n -9 >"$DIR/cards.csv.gz.tmp"
mv "$DIR/cards.csv.gz.tmp" "$DIR/cards.csv.gz"
echo "wrote $DIR/cards.csv.gz: $(gzip -dc "$DIR/cards.csv.gz" | wc -l) rows, $(du -h "$DIR/cards.csv.gz" | cut -f1)"
