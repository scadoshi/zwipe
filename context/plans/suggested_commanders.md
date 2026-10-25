# Suggested commanders

**Status: AGREED, not started (2026-10-25).** Shape settled with Scotty on 2026-10-25; the one open call is how commander profiles are precomputed (see Open question). Backlog origin: "Commanders You'd Like" in `../progress/backlog.md`, built on the tier-2 deck tag profile that shipped in `8deb24ed`.

## Problem

Zwipe helps you fill a deck once you've picked a commander, but not pick the next one. Players with a few decks built have already told us what they like to play; we can turn that into a short, explained list of commanders worth trying.

## What the user sees

- **Entry point:** a "Suggested commanders" action in the decks list's More actions sheet (`zwiper/src/lib/inbound/screens/deck/list.rs`, next to "Commander maybeboard", which is its only action today).
- **The list:** the top 10 suggestions, styled like the commander maybeboard (`screens/deck/commander_maybeboard.rs`): one card row per commander, tap to expand. Under the list, a hint line: "Your top 10. Tap Swipe to see more suggestions."
- **Swipe:** the action bar's Swipe button opens a swipe stack that starts at the top 10 and keeps going down the same ranking, to 100. Right starts a new deck with that commander, up saves it to the commander maybeboard, left dismisses it, down undoes.
- **In the list:** an expanded row carries the same actions as buttons (Start deck, Maybeboard, Not for me), through the row's existing `extra_actions` slot the way the commander maybeboard adds Create deck and Remove.
- **Dismissals refill:** "Not for me" or a left swipe hides the commander and pulls the next one up, so the list always shows the best 10 not yet turned down.
- **Cold start:** with no deck that has a commander and cards, the screen stays empty with "Build a deck and we'll suggest commanders that fit how you play." No popularity list standing in for personal picks.

## Why it was suggested

Every reason comes straight from the score terms the server computed, never written separately. The server returns each suggestion's breakdown and the client turns it into plain lines:

- Themes: "Shares your top themes: tokens, sacrifice, goblins", using `oracle_tags.label` for the shared tags with the most weight.
- Colors: "You build in red: 3 of your 4 decks."

No raw percentages: a small profile makes them swing, and a named theme or a count says more to a player.

**Where it renders:** a `SuggestionReasons` block in the card's details body, shown when a list row expands and in the swipe screen's eyeball dialog. Collapsed rows and the card on the stack look exactly as they do everywhere else.

**How, without forking shared UI:** one new optional `footer: Option<Element>` prop on `components::CardDetails` (`components/src/card_details.rs`), rendered after the role chips, and forwarded through `components::CardRow`, zwiper's `CardRow` wrapper (`screens/deck/card/components/card_row.rs`) and `CardDetailsDialog` (`screens/deck/card/components/card_info.rs`). It defaults to `None`, so every other screen renders byte-identical. It follows the `extra_actions` precedent: an optional element the screen supplies.

## Scoring

- **User profile:** the tag profiles of all the user's decks with a commander, merged, each deck weighted by its card count and recency. Same weighting as the deck tag profile (count × rarity, `cycle-*` excluded), so it reuses `deck_tag_profile`'s query shape.
- **Candidates:** commanders with a synergy payload (`commander_synergy`), each described by the tag profile of its payload's cards plus its own tags. The payload says far more about what a commander does than its handful of own tags.
- **Score:** overlap of the user profile with each candidate's profile, plus a color-affinity term (how often the candidate's colors appear across the user's decks). Later: partial credit for sibling tags (shared `oracle_tags.parent_ids`; 3,671 of 4,588 tags have parents).
- **Excluded:** commanders the user already has a deck with, has in the commander maybeboard, or dismissed.

## Open question: precomputing commander profiles

The user profile is cheap per request (the deck tag profile query runs in ~30-50 ms locally). Candidate profiles are not: each comes from a ~250-card payload, across every cached commander. They have to be precomputed. Options:

1. **Materialized view refreshed nightly by zervice (recommended).** `commander_tag_profiles(commander_oracle_id, profile jsonb)`, refreshed in zervice's step 4 alongside `latest_cards` and the rollups. Same pattern as the existing views. Cost: one production step, re-running `zcripts/server/sql/zervice_role.sql` with the new view added to its ownership loop, since `REFRESH` needs ownership.
2. **Written by the synergy worker.** It already writes `commander_synergy`, so it could write each commander's profile alongside the payload. No zervice grant, and profiles stay fresh with the payload. Cost: the work lands in the private worker repo and it starts reading otag data.
3. **Computed lazily by zerver and cached in a table it owns.** No grants and no second repo, but the first request per commander pays the cost and the cache needs its own freshness rule.

## Data and endpoints

- `GET /api/commander/suggestions?limit=…` returns ranked suggestions, each with the card and its breakdown (top shared tags with labels and weights, color counts). The list asks for 10, Swipe for 100.
- Dismissals: `user_commander_dismissals(user_id, oracle_id, dismissed_at)` with a TTL like deck skips (30 days to start), read at query time, plus `POST`/`DELETE` routes for dismiss and undo.
- Start deck reuses deck creation with `commander_id`; Maybeboard reuses the commander maybeboard's save route.

## Build order

1. Commander profiles (the open question) and the scoring endpoint with its breakdown, with integration tests over seeded payloads and otags.
2. Dismissals table and routes.
3. The list screen and its More actions entry, with the cold-start state.
4. The `footer` prop and `SuggestionReasons`.
5. The swipe view: a mode of the shared `SwipeSelect` overlay (`screens/deck/components/swipe_select.rs`) that swipes a given list instead of searching. Moderate work in that one file.
6. Copy: a guide, the More actions label, the hint line, a changelog entry.

## Ship

Server first (zerver deploy, a migration, and one zervice-role step if option 1), then the app in the next store build. Old apps don't call the new endpoint, so nothing gates on version.
