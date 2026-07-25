# Oracle-tag dictionary → example cards

**Status: IMPLEMENTED 2026-07-15 (local, uncommitted, not yet runtime-tested).
Client-only (zwiper). No backend, no schema, no wire change** — reuses the existing deck-free `search_cards` route and the frozen
`CardQuery` shape verbatim. Turns the read-only dictionary into a discovery
surface: tap a tag, swipe through real cards that carry it.

**One sentence:** dictionary rows become tappable; a tap opens a lightweight,
deck-agnostic swipe browse that serves that tag's cards EDHREC-rank-first, with a
stripped-down gesture grammar (left = next, down = back, right/up inert) because
there is no deck to collect into.

**Related:** consumes the same `CatalogCache.oracle_tags` the dictionary reads;
reuses the `SwipeStack` component and `CardStack` state from the add screen but
NOT the add screen's deck logic. Companion cleanup:
[`otag_filter_search_only.md`](otag_filter_search_only.md). Dictionary lives at
`zwiper/.../inbound/screens/oracle_tag_dictionary.rs`.

**Follow-up (audit + `fetch_usable_page` plan):**
[`otag_examples_followup.md`](otag_examples_followup.md) — hand-off for the next
AI: full issue list from post-implementation review and the agreed paging helper.

**Mount model pivoting (route → overlay):** the shipped version is a **route**
(`/oracle-tags/examples/:slug`). Owner decided the dictionary + examples should be
**in-place overlays** (preserve host state, `Use` writes live signals). This screen
will be refactored from a route to an `open`-gated overlay per
[`overlay_architecture.md`](overlay_architecture.md) and
[`dictionary_adopt_flow.md`](dictionary_adopt_flow.md). The browse *logic* below
stays; only the mounting changes.

---

## Why

Descriptions tell you what a tag *means*; examples show you what it *catches*. For
an abstract system (`removal-conditional`, `ramp-artifact`, ...) five real cards
beat any sentence we author. It also makes the dictionary a front door into the
app's core loop (swiping) instead of a dead-end reference. The plumbing already
exists — this is wiring, not new protocol.

## Wire — nothing changes (verified)

- `CardQuery` / `CardCriteria` carry **no `deck_id`**
  (`zwipe-core/.../card_filter/query.rs`). Deck-awareness lives entirely in the
  *route* (`deck_id` is a URL path param on `search_deck_cards`), never the body.
- The generic `search_cards` route (`zwiper/.../outbound/client/card/search_cards.rs`
  → server `zerver/.../handlers/card/search_card.rs`) takes a bare `CardQuery`,
  requires only an auth token, and runs it with **zero deck logic**: no in-deck
  exclusion, no synergy scoring, no suppression set. Exactly what a neutral
  "examples" browse wants.
- `CardSortKey::EdhrecRank` exists (`card_sort_key.rs`), sorts ascending with
  unranked cards last (`None → i32::MAX`). Setting it puts the most iconic cards
  for the tag first, so even a thin tag leads with its best example.

So the query is: `CardQueryBuilder` with `oracle_tags_contains_any = [slug]`,
`sort = EdhrecRank`, `ascending = true`, `limit` + `offset` for pagination →
`build()` → POST via `ClientSearchCards::search_cards`. One otag is enough intent
to pass `build_criteria()` (no `InvalidCardCriteria::Empty`).

## Gesture grammar — reduce the existing one, don't invent a new one

`SwipeStack` (`zwiper/.../interactions/swipe/stack.rs`) already exposes
`on_swipe_{left,right,up,down}` + a `SwipeConfig` of allowed directions. In the
browse there is no deck, so the deck actions have nothing to do. Keep the two
gestures that already have a deck-free meaning, drop the rest:

- **Left = next card** (already "skip/dismiss" everywhere → maps perfectly).
- **Down = go back one card** (already "undo" everywhere → maps perfectly).
- **Right / Up = inert.** No deck to add/maybeboard into. Disable them in
  `SwipeConfig` (preferred — the card won't fly off), or wire no-op handlers.

Explicitly **rejected**: "right = go back" (right is the trained *positive/forward*
action; inverting it is the worst inconsistency) and "right = adopt this tag as a
filter" (a page-level intent about the *tag*, wrong to bury in a per-card gesture).
If we later want "filter a deck by this tag," it's an explicit button, not a swipe.

## Plan

### 1. Route (public, like the dictionary)
Add to `Router` (`zwiper/.../inbound/router.rs`), no `Bouncer` wrapper to match
the dictionary's public stance:
```
#[route("/oracle-tags/examples/:slug")]
OracleTagExamples { slug: String },
```

### 2. Make dictionary rows tappable
In `oracle_tag_dictionary.rs`, the `dict-row` (lines ~160-177) currently has no
`onclick`. Add one that pushes `Router::OracleTagExamples { slug: t.slug }`. Give
the row an affordance so it reads as tappable (cursor/active state, maybe a small
chevron or "examples" caption) — decide against the existing `dict-row` styling in
`zwipe-components` CSS. Keep the parents chips non-interactive for now.

### 3. New browse screen `OracleTagExamples { slug }`
A fresh, lean component — do **not** reuse `Add` (it's 1000+ lines of deck/undo/
signal logic). It needs only:
- Local `CardStack<()>` (or a plain cursor `Signal<usize>` + `Vec<Card>`), seeded
  by fetching page 0.
- Fetch loop: `CardQueryBuilder` as above, paginate by `offset += limit` when the
  cursor nears the end of the loaded set (mirror add.rs's offset paging, minus the
  `AddStackCache` parking — a browse doesn't need parked stacks).
- Render `SwipeStack { cards: <window from cursor>, config, entering, ... }`.
  - `on_swipe_left`: advance cursor (next).
  - `on_swipe_down`: rewind cursor one (back); no-op at the top.
  - `on_swipe_right` / `on_swipe_up`: inert (or disabled in `SwipeConfig`).
- Header via `ScreenHeader` showing the tag's human label
  (`label_for`-style lookup from the catalog, fall back to slug) + a short
  subtitle ("Example cards"). `ActionBar` with a Back button.
- End state: cursor past the last card with nothing left to fetch → a calm "No
  more cards" panel (per owner: a 1-card tag ending immediately is fine, no
  threshold gate needed).
- First-load skeleton + a failed-fetch toast, matching the dictionary's patterns.

### 4. No side effects
No suppression posts, no add-stack writes, no swipe signal, no deck reads. The
browse is pure read. Confirm the generic route's `synergy` stays unset (it's
ignored on this path anyway).

## Open questions / decisions

- **Logged-out access — handled by the auth gate.** Depends on
  [`auth_route_gate.md`](auth_route_gate.md): once auth is a router layout, the
  dictionary and this browse route both sit under `AuthGate`, so a valid `Session`
  is guaranteed in context and `search_cards` always carries a token — no per-screen
  guard here. If that refactor hasn't landed yet, the same holds in practice today
  (the dictionary's only inbound links are behind auth), so this is safe either way;
  the gate just makes it structural instead of incidental.
- **Row affordance styling.** ~~Whole-row tap~~ **Superseded** — see
  [`otag_examples_followup.md`](otag_examples_followup.md) § Dictionary
  follow-through: whole row **not** clickable; per-entry buttons **Examples** +
  **Use** (context-aware adopt). Examples browse stays left/down swipe only.
- **Pagination depth.** Reasonable `limit` (e.g. 30–60) and stop paging past some
  sane ceiling; EDHREC-rank order means later pages are low-value anyway.
- **Landscape / flip.** Reuse whatever `FlippableCardImage` behavior `SwipeStack`
  already gives; no extra work expected.

## As-built notes (deviations from the plan above)

- **Chose A (real swipe feed), deck-free.** New screen
  `screens/oracle_tag_examples.rs` (`OracleTagExamples { slug }`), route
  `/oracle-tags/examples/:slug` under `AuthGate`. Reuses `SwipeStack` +
  `CardStack<BrowseAction>` (new field-less linear action in `action_history.rs`),
  not the `Add` screen.
- **Gestures:** `SwipeConfig` allows only `Left` + `Down` (verified `state.rs`
  gates commit on `allowed_directions`), so right/up return to center. Left =
  `advance_after_commit` (records `BrowseAction::Next`, paginates near the end);
  down = `go_back` (`pop_action` + `step_back`, enters from the left). The
  end-of-stack panel keeps the add screen's down-swipe-to-go-back touch handler.
- **Query:** plain `search_cards` with `oracle_tags_contains_any=[slug]`,
  `is_token(false)`, `sort=EdhrecRank` (ascending default → most popular first),
  `limit=25`, offset pagination.
- **Kept lean:** shows `CardInfoDisplay` under the card; **no** CardDetailsDialog /
  RulesButton / PrintingSheet, no Filter/Refresh in the ActionBar (Back only). No
  thin-tag gate (per decision — a 1-card tag just ends). No dedup (single-tag
  paginated search doesn't overlap pages); only image-less cards are filtered.
- **Dictionary rows** got `dict-row-tappable` + whole-row `onclick` → examples.
  **To be replaced** by explicit **Examples** / **Use** buttons (row not
  clickable) — [`otag_examples_followup.md`](otag_examples_followup.md).
- **Gestures on examples (locked):** left = next, down = back only; right/up
  inert. No swipe-to-adopt-tag.
- **Logged-out:** moot — the route is under `AuthGate` (auth_route_gate shipped),
  so a session is guaranteed.
- **Slug-change caveat:** `tag_slug` seeded via `use_signal` on mount; direct
  example→example nav wouldn't refetch, but that's not a reachable path (the
  dictionary sits between, forcing a remount).
- **Eyeball:** `RulesButton` + `CardDetailsDialog` added on examples ActionBar
  post as-built; signal named `show_details` (not `show_rules`).

## Not doing

- No swipe-right (or any swipe) to pick/adopt a tag — **Use** is a dictionary
  row button (and optionally an examples ActionBar control later), never a
  gesture.
- No "swipe these into a deck" hand-off in v1.
- No new server route, no unauth endpoint, no `CardQuery` field.
- No changes to the deck-aware `search_deck_cards` path.
