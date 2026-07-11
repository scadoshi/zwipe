# Oracle tags (otags) — a community-maintained functional tagging system

**Status: HORIZON / IDEA (captured 2026-07-11 from owner vision). Not started. This
doc records the concept and the payoff; it is intentionally NOT fully researched —
the open questions at the bottom are for a follow-up AI/session to resolve before
building.**

## One sentence

Ingest Scryfall's **oracle tags** (otags) — hundreds of community-maintained functional
tags on cards (e.g. `otag:removal`, `otag:ramp`, `otag:card-advantage`, `otag:tutor`) —
correlate every card to its otags via a daily backend sync, let players select the otags
relevant to a deck's strategy, and use that formalized, community-accurate tagging as a
new axis for algorithmic card serving (alongside the synergy worker, MVPs, and swipe
signal) — including unlocking non-EDH formats.

## Why this is big

Today our functional categorization is **`mechanical_categories`** — derived by our own
**heuristics/regex** (`domain/card/models/mechanical_category`). That's ~73% accurate,
brittle, and **we have to maintain it**. otags are:

- **Community-maintained** on Scryfall (the Tagger project) — more accurate, and *not our
  maintenance burden*. Corrections flow to us for free via the daily sync.
- **The real tags that show against cards**, so they align our internal categorization
  with what players actually see / search elsewhere.
- **A formalized vocabulary** we can align **deck tags** to. The deck-tags feature we
  invented (`deck_tag` / `deck_other_tag`) overlaps this directly — otags give those tags
  a real, card-level backing.

This is a strict improvement over heuristic categories on accuracy, maintenance, and
alignment. (Decision to make during build: does otags **replace**, **complement**, or
**seed** `mechanical_categories`? See open questions.)

## The data pipeline

- otags are a **bulk-syncable** dataset from Scryfall. Run it **once a day via `zervice`**
  (our existing background sync/classify job) so every card carries its full otag set.
- Store a card → otags correlation (new table, e.g. `card_otags`, keyed by `oracle_id`).
- Because otags ride on `oracle_id` and our **Archidekt/text import already resolves to
  oracle ids**, imported decks automatically inherit their cards' otags.

## Deck-level: player-selected strategy otags

Hundreds of otags exist, and a single card falls under many — dumping all of them in a
chart is overwhelming. So:

- Let the player **select the otags relevant to their deck** (its strategy). This *is*
  effectively the deck-tags feature, now backed by real card tags — **merge / reconcile
  with `deck_tag`**.
- **Distribution view:** show the deck's otag distribution, but scoped to the
  selected/relevant otags (or the top-N most common in the deck), not the full firehose.
- Selected deck otags become a **serving signal**: they declare the deck's intended
  direction, which the algorithm can lean into.

## Why it supercharges serving (the real payoff)

Today serving leans on: the commander's synergy map (scraped by the synergy worker) +
pooled swipe signal (`commander_card_signal`) + band shuffle. otags add a **formalized,
community-accurate correlation axis** on top:

1. **Commander + deck otags → community-correlated cards.** Serve cards whose otags match
   the deck's declared strategy, using correlations the *community* maintains, not our
   heuristics.
2. **Most-popular otags for a commander** → derive candidate cards even before the player
   sets tags.
3. **MVP otags:** when a player stars a card as an MVP (deck-mvps feature), read *its*
   otags and serve cards that overlap — "more like your favorites."
4. **Blend it all:** otag correlation + synergy-worker scores + swipe signal + MVP otags →
   a richer serving algorithm than any single source.

## The format unlock (marketable)

Serving today is basically Commander-only because it pivots on the commander (color
identity + synergy). otags + color identity generalize that to **any format**:

- If a deck has a **color identity** + **selected otags**, it behaves "commander-like" for
  serving purposes — we have something to pivot from even with no commander.
- Then **collect swipe correlation data keyed by (format, color identity, otag set)**:
  every left/right swipe in that context builds correlation data *for that format*.
- Over time this lets us **serve synergistic cards for non-EDH formats** (Standard, Modern,
  Pioneer, etc.) — a capability we currently don't have and **can market** as a
  differentiator.

Priority note: **Commander dominates usage by a wide margin**, so land otags there first
(where the data is) and let the non-EDH format serving accrue data over time.

## How it fits existing systems

- **Deck tags** (`deck_tag`, `deck_other_tag`): otags should back / merge with these.
- **`mechanical_categories`** (heuristic): otags likely supersede or reseed these.
- **Synergy worker** (cache-first EDHREC-style synergy): complementary — synergy = "played
  together for this commander," otags = "functional role." Blend, don't replace.
- **`commander_card_signal` / swipe signal**: otags add a keying dimension (format, CI,
  otag set) to broaden signal collection beyond commander.
- **MVPs** (deck-mvps): otags of starred cards drive "more like this."
- **Import** (Archidekt/text): oracle-id resolution means imports inherit otags for free.

## Open questions (resolve before building — this is the research a follow-up owns)

1. **Data source + access (the critical one).** Scryfall otags come from the **Tagger**
   project. They're queryable via the `otag:` search syntax and a (largely undocumented)
   **Tagger GraphQL API**, but are **NOT part of the standard bulk-data download**. Figure
   out the reliable way to get the *full* card→otag mapping at scale: Tagger API vs.
   iterating `otag:` searches, rate limits, completeness, and **Scryfall ToS / attribution
   / caching rules** (mirror the care taken with the EDHREC synergy layer — keep candid
   sourcing details out of the public repo where appropriate).
2. **Storage schema**: `card_otags` (oracle_id → otag[]) + `deck_otags` (deck-selected
   subset). Indexing for "cards with otag X in color identity Y."
3. **Deck-tag reconciliation**: do selected otags *become* deck tags, or a parallel field?
   Avoid two overlapping tagging UIs.
4. **UI**: otag selection (searchable, hundreds of options), the deck distribution chart
   (scoped, not overwhelming), and the format+CI+otags "commander-like" setup for non-EDH.
5. **Serving algorithm**: how to weight otag correlation vs synergy score vs swipe signal
   vs MVP otags; cold-start behavior.
6. **`mechanical_categories` fate**: replace, complement, or seed.
7. **Volume/perf**: hundreds of otags × 110k+ cards × many-per-card — matview/rollup
   design like `card_signal_rollup`.

## Sequencing (rough)

1. Nail the **data source** (open question 1) — nothing else starts until otags can be
   fetched reliably.
2. Schema + daily `zervice` sync → `card_otags`.
3. Deck otag selection (reconciled with deck tags) + distribution view.
4. Serving: fold otags into the existing serve as a new term.
5. Non-EDH format serving + (format, CI, otag) swipe-signal collection.
