# Suggested commanders

**Status: AGREED, not started (2026-10-25).** Shape settled with Scotty on 2026-10-25, including where commander profiles are computed (the synergy worker, see Commander profiles). Backlog origin: "Commanders You'd Like" in `../progress/backlog.md`, built on the tier-2 deck tag profile that shipped in `8deb24ed`.

## Problem

Zwipe helps you fill a deck once you've picked a commander, but not pick the next one. Players with a few decks built have already told us what they like to play; we can turn that into a short, explained list of commanders worth trying.

## What the user sees

- **Entry point:** a "Suggested commanders" action in the decks list's More actions sheet (`zwiper/src/lib/inbound/screens/deck/list.rs`, next to "Commander maybeboard", which is its only action today).
- **The list:** the top 10 suggestions, styled like the commander maybeboard (`screens/deck/commander_maybeboard.rs`): one card row per commander, tap to expand. Under the list, a hint line: "Your top 10. Tap Swipe to see more suggestions."
- **Swipe:** the action bar's Swipe button opens a swipe stack that starts at the top 10 and keeps going down the same ranking, to 100. Right starts a new deck with that commander, up saves it to the commander maybeboard, left dismisses it, down undoes.
- **In the list:** an expanded row carries the same actions as buttons (Start deck, Maybeboard, Not for me), through the row's existing `extra_actions` slot the way the commander maybeboard adds Create deck and Remove.
- **Dismissals refill:** "Not for me" or a left swipe hides the commander for 7 days, the same as a deck skip, and pulls the next one up, so the list always shows the best 10 not yet turned down. A left swipe means "not right now", not "never".
- **Clear dismissals:** the suggestions screen's More sheet has "Clear dismissals", like Clear skips on a deck, bringing every dismissed commander back at once.
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

## Commander profiles (decided 2026-10-25: the synergy worker writes them)

The user profile is cheap per request (the deck tag profile query runs in ~30-50 ms locally), so it is computed live and never lags. Candidate profiles are not: each comes from a payload averaging 263 cards (production 2026-10-25: 1,527 cached payloads, 86 to 445 cards each). They are precomputed into `commander_tag_profiles(commander_oracle_id, profile jsonb, computed_at)`.

**The synergy worker writes each profile in the same step that writes the commander's payload.** A commander profile depends only on its payload, and the worker is the demand-driven lane: it discovers new deck commanders, fetches them, pre-warms popular ones and refreshes stale ones on a short poll. Computing the profile there means a newly cached commander can be suggested right away. A nightly zervice view was the other candidate and was ruled out because a commander cached during the day would have no profile until that night. This also needs no zervice role change. One-time cost: a backfill pass over the existing payloads on the first run (about 400k card lookups).

Otag data changes nightly with the Scryfall sync, so a profile reflects the otags as of its payload's last fetch or refresh. That is fresh enough: profiles roll forward with the worker's payload refresh cycle.

## Data and endpoints

- `GET /api/commander/suggestions?limit=…` returns ranked suggestions, each with the card and its breakdown (top shared tags with labels and weights, color counts). The list asks for 10, Swipe for 100.
- Dismissals: `user_commander_dismissals(user_id, oracle_id, dismissed_at)` with a 7-day TTL like deck skips, read at query time, plus routes to dismiss, undo one, and clear all.
- Start deck reuses deck creation with `commander_id`; Maybeboard reuses the commander maybeboard's save route.

## Build order

1. `commander_tag_profiles` (migration in zwipe), written by the synergy worker alongside each payload, with its backfill pass (in the worker repo).
2. The scoring endpoint with its breakdown, with integration tests over seeded profiles.
3. Dismissals table and routes, including clear all.
4. The list screen, its More actions entry with Clear dismissals, and the cold-start state.
5. The `footer` prop and `SuggestionReasons`.
6. The swipe view: a mode of the shared `SwipeSelect` overlay (`screens/deck/components/swipe_select.rs`) that swipes a given list instead of searching. Moderate work in that one file.
7. Copy: a guide, the More actions labels, the hint line, a changelog entry.

## Ship

The migration and the worker first, so profiles exist before anything reads them. Then zerver, then the app in the next store build. Old apps don't call the new endpoint, so nothing gates on version.
