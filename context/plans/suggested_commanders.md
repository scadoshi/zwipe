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

## Done (2026-10-26): variety in the top 10

First on-device test with real profiles: Scotty's top 10 came back all mono-blue, though he builds mostly red-white-blue and a lot of red-white. The UI is settled; the fix went into the ranking rather than injecting randomness, which would have hidden the bias instead of removing it. Three changes, all in zerver:

- **Color identity as a set.** The color term was the user's share of each of the candidate's colors, averaged, so a mono-blue candidate scored the full blue share while a Jeskai one averaged blue against red and white. It is now the best Jaccard match of the candidate's color identity against the identities of the user's commander decks (shared colors over all colors across the two; colorless against colorless is 1, colorless against colored 0), so a Jeskai candidate scores 1.0 against a Jeskai deck and a mono-blue one a third. Still weighted by `W_COLOR_AFFINITY` (0.15).
- **One profile per deck, best match.** The user profile merged every deck into one bag of tags, so decks that shared more tags pulled the whole profile their way. `deck_tag_profile` now runs once per commander deck (the commander plus its mainboard, the `MAX_PROFILED_DECKS` = 12 most recently updated decks; older decks still count for colors), the queries issued concurrently and queued on the connection pool. The tag term is the best dot product over those profiles, and the reasons' tags come from that best deck's overlap. The decks go to the one scoring query as a jsonb array of `{colors, profile}` objects; the SQL unnests it with `jsonb_array_elements ... WITH ORDINALITY` and takes `max()` per candidate. Score = best tag dot product + `W_COLOR_AFFINITY` × best color match, ordered by score desc, then name.
- **Variety cap.** `cap_color_identities` (in `domain/deck/models/deck/commander_suggestions.rs`) reorders the scored list so at most `MAX_PER_COLOR_IDENTITY` = 4 commanders of one exact color identity keep their place; the rest of that identity move behind every other candidate, in score order, so they still show further down the Swipe ranking. The service scores the whole `SCORED_POOL` of 100 before capping and cutting to the requested limit, so a top 10 has candidates to pull up.

Tests: `zerver/tests/commander_suggestions.rs` covers the set match (Jeskai over mono-blue for a Jeskai and Boros builder), the per-deck match (a red-white candidate with its deck's reasons over a tag-heavy blue deck) and the cap at limits 5, 10 and 100; `cap_color_identities` has unit tests. Still to do: re-test on device with Scotty's account and check the query time in production, where ~1,500 profiles meet up to 12 decks per request.

## Agreed, not built (2026-10-26): color chips and a profile summary

Two additions Scotty asked for after the first real test, agreed in shape and sequenced after the variety fix has been retested on his account.

**Color-identity chips that filter client-side.** The screen fetches the full ranking once on entry (100, which Swipe already asks for) and shows the first 10 as the real top 10. A row of chips, one per color identity present in the user's own decks plus "Any", filters that held list; tapping Boros shows the Boros commanders within the overall top 100, and Swipe reads the same filtered list. No server parameter and no new query: the filter is a predicate over data the client already has, so fewer requests than today, not more. A thin result ("2 Boros picks") is honest and the summary explains it; re-ranking server-side within one identity would surface weaker picks to fill ten slots, which turns suggestions back into search. The variety cap applies only when no chip is active, since a chip has asked for one identity. A full filter sheet is deliberately not offered: text and type filters replace the ranking instead of sharpening it, and color is the one axis a player knows before seeing anything.

**A profile summary header.** The suggestions are computed from a profile of the user's decks that is currently invisible; a header above the list shows it, closing the loop between "what we think you build" and "what we suggest". Numbers rather than charts: "7 of your 10 decks are red", "sacrifice outlets in 6 decks", the top colors and top oracle tags. One picture at most, a color bar sized by deck count. The header's color row IS the chip row above, so one element is both the summary and the filter, and the summary explains why the filter offers the identities it does. The data is a by-product of the scoring: the per-deck color identities and profiles the service already builds, returned once as part of the suggestions response (a small contract addition, `profile: { colors: [{color, decks}], tags: [{slug, role, decks}] }`), not a second endpoint.

Build the header without the filter first, a few hours, and see whether the chips still need to filter once the top 10 spans the user's colors on its own; the filter adds a Swipe-mode wrinkle and earns its own step.

**Watch item from the variety build:** one suggestions request now issues up to 12 per-deck profile queries concurrently against a pool of 10 connections, so a single request can briefly occupy the whole pool before its scoring query runs. Fine at current traffic; if it bites, the knobs are a `Semaphore` around the profile tasks or a smaller `MAX_PROFILED_DECKS`. Check the endpoint's production latency with ~1,500 profiles before adding the header, which would add nothing to the query but would make slowness visible on every screen entry.

## Commander profiles (decided 2026-10-25: the synergy worker writes them)

The user profile is cheap per request (the deck tag profile query runs in ~30-50 ms locally), so it is computed live and never lags. Candidate profiles are not: each comes from a payload averaging 263 cards (production 2026-10-25: 1,527 cached payloads, 86 to 445 cards each). They are precomputed into `commander_tag_profiles(commander_oracle_id, profile jsonb, computed_at)`.

**The synergy worker writes each profile in the same step that writes the commander's payload.** A commander profile depends only on its payload, and the worker is the demand-driven lane: it discovers new deck commanders, fetches them, pre-warms popular ones and refreshes stale ones on a short poll. Computing the profile there means a newly cached commander can be suggested right away. A nightly zervice view was the other candidate and was ruled out because a commander cached during the day would have no profile until that night. This also needs no zervice role change. One-time cost: a backfill pass over the existing payloads on the first run (about 400k card lookups).

**It runs as its own task in the worker, apart from the upstream fetch loop.** The fetch loop is deliberately slow (fetch delays and backoff toward the upstream source); profile building only touches our own database, so it has no rate limit and runs as fast as it can. The fetch loop hands each written or refreshed commander to the profile task over an in-process channel, which builds that profile at once. A periodic sweep catches any payload whose profile is missing or older than the payload, covering the first-run backfill, restarts and anything the channel missed. Profiles build one at a time, since the database also serves zerver's users: each is one query of roughly 30-50 ms, so the full backfill takes about a minute and a new payload has its profile moments after it lands. A failure in either task never stalls the other.

Otag data changes nightly with the Scryfall sync, so a profile reflects the otags as of its payload's last fetch or refresh. That is fresh enough: profiles roll forward with the worker's payload refresh cycle.

## Data and endpoints

- `GET /api/commander/suggestions?limit=…` returns ranked suggestions, each with the card and its breakdown (top shared tags with labels and weights, color counts). The list asks for 10, Swipe for 100.
- Dismissals: `user_commander_dismissals(user_id, oracle_id, dismissed_at)` with a 7-day TTL like deck skips, read at query time, plus routes to dismiss, undo one, and clear all.
- Start deck reuses deck creation with `commander_id`; Maybeboard reuses the commander maybeboard's save route.

## Build order

1. `commander_tag_profiles` (migration in zwipe), written by the synergy worker alongside each payload, with its backfill pass (in the worker repo).
2. Done 2026-10-25. `GET /api/commander/suggestions?limit=N` (default 10, max 100) answers `HttpCommanderSuggestions`: each suggestion carries the commander's card and its reasons (top 3 shared themes with catalog labels, and per color how many of the user's commander decks use it), plus `deck_count` (0 is the cold start). Scoring is `score_commander_suggestions` in zerver's card repository; the user profile reuses `deck_tag_profile` over every commander deck's commander and mainboard. Integration tests in `zerver/tests/commander_suggestions.rs`.
3. Done 2026-10-25. `user_commander_dismissals` with a 7-day TTL; `POST /api/commander/suggestions/dismissals` (body `{"oracle_id"}`) dismisses, `DELETE .../dismissals/{oracle_id}` undoes one, `DELETE .../dismissals` clears all and answers `{"cleared": n}`. Client methods `commander_suggestions`, `dismiss_commander`, `undismiss_commander`, `clear_commander_dismissals`.
4. The list screen, its More actions entry with Clear dismissals, and the cold-start state.
5. The `footer` prop and `SuggestionReasons`.
6. The swipe view: a mode of the shared `SwipeSelect` overlay (`screens/deck/components/swipe_select.rs`) that swipes a given list instead of searching. Moderate work in that one file.
7. Copy: a guide, the More actions labels, the hint line, a changelog entry.

## Ship

The migration and the worker first, so profiles exist before anything reads them. Then zerver, then the app in the next store build. Old apps don't call the new endpoint, so nothing gates on version.
