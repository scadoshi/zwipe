#!/usr/bin/env bash
#
# archidekt_probe.sh — prototype Archidekt deck importer.
#
# Fetches a public Archidekt deck via its open JSON API, parses it into the
# shape Zwipe needs (exact printing per card, quantity, command zone), then
# resolves every printing UID against the local `scryfall_data` table and
# reports coverage. Pure prototype / spec — does NOT write to the DB.
#
# Usage: ./archidekt_probe.sh <archidekt-deck-url-or-id>
# Requires: curl, jq, psql. DB defaults to the local zerver DB.

set -euo pipefail

DB="${DATABASE_URL:-postgres:///zerver?user=$USER}"
UA="ZwipeTCG/0.1 (+https://zwipe.net; deck-import prototype)"

arg="${1:?usage: archidekt_probe.sh <archidekt-deck-url-or-id>}"
deck_id="$(printf '%s' "$arg" | grep -oE '[0-9]+' | head -1)"
[ -n "$deck_id" ] || { echo "could not extract a numeric deck id from: $arg" >&2; exit 1; }

raw="$(mktemp)"
trap 'rm -f "$raw"' EXIT
code="$(curl -s -A "$UA" -o "$raw" -w '%{http_code}' "https://archidekt.com/api/decks/${deck_id}/")"
[ "$code" = "200" ] || { echo "archidekt returned HTTP $code for deck $deck_id" >&2; exit 1; }

# --- parse ------------------------------------------------------------------
# Premier categories = the command zone(s) (isPremier=true, e.g. "Commander").
# Excluded categories = includedInDeck=false (maybeboard / sideboard / Attraction).
# A card lands in the deck unless it carries an excluded category; it lands in
# the command zone if it carries a premier category.
#
# Emits one TSV row per included card: uid<TAB>qty<TAB>zone<TAB>name
parsed="$(jq -r '
  (.categories // []) as $cats
  | ([ $cats[] | select(.isPremier == true)     | .name ]) as $premier
  | ([ $cats[] | select(.includedInDeck == false) | .name ]) as $excluded
  | .cards[]
  | . as $c
  | ($c.categories // []) as $cc
  | select( any($cc[]; . as $n | $excluded | index($n)) | not )
  | ( if any($cc[]; . as $n | $premier | index($n)) then "command" else "deck" end ) as $zone
  | [ $c.card.uid, ($c.quantity|tostring), $zone, $c.card.oracleCard.name ] | @tsv
' "$raw")"

deck_name="$(jq -r '.name' "$raw")"
deck_format="$(jq -r '.deckFormat' "$raw")"

# --- resolve against local scryfall_data ------------------------------------
uids="$(printf '%s\n' "$parsed" | cut -f1 | sort -u)"
uid_array="$(printf '%s\n' "$uids" | sed "s/.*/'&'/" | paste -sd,)"
resolved_set="$(psql "$DB" -tAc \
  "SELECT id FROM scryfall_data WHERE id = ANY(ARRAY[$uid_array]::uuid[]);" | sort -u)"

total_lines="$(printf '%s\n' "$parsed" | grep -c . || true)"
total_uids="$(printf '%s\n' "$uids" | grep -c . || true)"
resolved_uids="$(printf '%s\n' "$resolved_set" | grep -c . || true)"
total_qty="$(printf '%s\n' "$parsed" | awk -F'\t' '{s+=$2} END{print s+0}')"

echo "============================================================"
echo "  Deck: $deck_name   (id $deck_id, format $deck_format)"
echo "============================================================"
echo "  lines (distinct cards): $total_lines"
echo "  total quantity:         $total_qty"
echo "  unique printings:       $total_uids"
echo "  resolved by UID:        $resolved_uids"
echo "  misses:                 $((total_uids - resolved_uids))"
echo
echo "  Command zone:"
printf '%s\n' "$parsed" | awk -F'\t' '$3=="command"{printf "    - %s\n", $4}'
echo

# --- report unresolved (the cards an importer would surface to the user) ----
misses="$(comm -23 <(printf '%s\n' "$uids") <(printf '%s\n' "$resolved_set") | grep . || true)"
if [ -n "$misses" ]; then
  echo "  UNRESOLVED printings (would fall back to name lookup):"
  printf '%s\n' "$misses" | while read -r u; do
    name="$(printf '%s\n' "$parsed" | awk -F'\t' -v u="$u" '$1==u{print $4; exit}')"
    echo "    - $u  ($name)"
  done
  echo
else
  echo "  ✓ All printings resolved directly by Scryfall UID."
  echo
fi
