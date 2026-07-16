# Otag examples follow-up — audit issues + `fetch_usable_page` + dictionary follow-through

**Status: PLANNED (2026-07-15).** Hand-off doc for the next AI session.
Covers (1) every issue found in the review of the uncommitted
`otag-example-cards` Claude session work, (2) the agreed plan to fix the
pagination / image-filter empty-page bug via a shared `fetch_usable_page`
helper, and (3) dictionary row **follow-through** — explicit **Examples** /
**Use** buttons (whole row no longer clickable).

**One sentence:** finish the dictionary → examples loop: safe paging
(`fetch_usable_page`), non-row-click dict entries with **Examples** + **Use**
buttons (context-aware adopt), and keep examples browse as left/down-only
swipe.

**Related (already implemented, local/uncommitted):**

| Plan | Status |
|------|--------|
| [`otag_example_cards.md`](otag_example_cards.md) | IMPLEMENTED (uncommitted, not runtime-tested); **row UX superseded** by § Dictionary follow-through below |
| [`otag_filter_search_only.md`](otag_filter_search_only.md) | IMPLEMENTED (uncommitted) |
| [`auth_route_gate.md`](auth_route_gate.md) | IMPLEMENTED (uncommitted) |

**Primary files for this follow-up:**

- New: `zwiper/src/lib/inbound/screens/deck/card/components/fetch_usable_page.rs`
- Wire: `zwiper/.../screens/oracle_tag_examples.rs`
- Wire: `zwiper/.../screens/oracle_tag_dictionary.rs` (buttons; drop whole-row click)
- Entry-context plumbing: deck `oracle_tag_select.rs`, card filter `oracle_tags.rs`
  (Dictionary link + return intent for **Use**)
- Later consumers of paging helper (same bug family, optional second pass):
  `deck/card/add.rs` (`load_more_cards`), `deck/components/swipe_select.rs`

---

## Context for the next AI

### What shipped in the prior session (uncommitted)

1. **Auth gate** — `Bouncer` deleted; router layout `AuthGate` wraps every
   route except Login / Register / ForgotPassword. Changelog, Privacy, and
   OracleTagDictionary are now authed (were public).
2. **Filter search-only** — card filter `OracleTags` dropped the curated
   `CURATED_ORACLE_TAGS` include grid; selected chips + search only. Deck
   strategy picker still uses curated chips (untouched).
3. **Dictionary → examples browse** — currently whole-row `dict-row-tappable`
   onclick → `/oracle-tags/examples/:slug` → deck-free swipe via plain
   `search_cards`, `oracle_tags_contains_any=[slug]`, `EdhrecRank`, left=next,
   down=back, right/up inert. `BrowseAction` in `action_history.rs`.
   **Superseded for row UX:** see § Dictionary follow-through (buttons, not
   whole-row click).
4. **Post-audit polish already done by Grok session:**
   - Eyeball `RulesButton` + `CardDetailsDialog` on examples ActionBar.
   - Renamed `show_rules` → `show_details` on all swipe surfaces (add, remove,
     swipe_select, examples).

### Owner decisions already locked (do not re-litigate)

- Path A deck-free browse; **not** inventing new swipe grammar.
- **Examples browse gestures stay reduced:** only **swipe left** (next) and
  **swipe down** (back / undo). Right and up remain inert (not in
  `SwipeConfig`). No “swipe right to pick the tag.”
- Synergy stays in `CardQuery` body; `deck_id` stays on deck search path.
- No thin-tag gate; 1-card tags fine.
- `fetch_usable_page` is a **free function / module helper**, **not** a method
  on `CardStack`.
- Inject **fetch callback** + **is_usable predicate**; helper owns paging
  policy only.
- Cap consecutive filter-empty pages (e.g. **5**) per load call; true server
  empty stops immediately (1 request).
- **Dictionary rows are NOT whole-element clickable.** Two explicit buttons per
  entry; adopt is a button, never a swipe.

### Not runtime-tested

Prior session never got an emulator/device run (usage limit). Treat as
**compile-checked only** until someone swipes through the flows below.

---

## Audit issues (full list)

Severity: **P1** fix before merge · **P2** should fix soon · **P3** polish /
product · **Nit** cleanup.

### P1 — False empty / premature exhaust when client filter drops a whole page

> **Re-scoped 2026-07-15:** image presence is no longer a usability filter —
> image-less cards now render as a text "identity frame" and are shown everywhere
> (see [`no_image_card_placeholder.md`](no_image_card_placeholder.md)). So
> **examples has no client filter left → this bug can't occur there.** The helper
> below is now only for **Add / swipe_select**, whose filters still drop *common*
> in-deck / duplicate cards; its `is_usable` no longer mentions images.

**Where:** `add.rs` (`load_more_cards`) and `swipe_select.rs`. (Examples: resolved
by the placeholder plan, no helper.)

**Pattern:** fetch page → client-filter (no Large image; Add also drops
in-deck + dups) → if result empty, control flow is wrong.

| Screen | On filter-empty page | Failure mode |
|--------|----------------------|--------------|
| ~~Examples~~ | n/a — no client filter after the placeholder change | resolved |
| **Add load-more** | Sets `pagination_exhausted`, does **not** advance offset | Stops paging early; later good pages never requested (worse when whole page is already in-deck) |
| **Add initial** | Replaces stack; if empty, empty UI; no auto next page | Same “give up after one barren page” |
| **Swipe select** | Same as Add load-more (`exhausted = true`) | Same premature end |

**Agreed fix:** `fetch_usable_page` (this plan), applied to Add / swipe_select.
`is_usable` = `!in_deck && !seen` (no image clause).

**Not the same as true empty filter:** server returns `[]` on first page →
one request, “no cards.” Only multi-hit when server keeps returning rows that
all die in the client filter.

---

### P2 — No stack size cap on examples browse — **DONE 2026-07-15**

Added `peek_len()` to `CardStack`; examples `load_more` sets
`pagination_exhausted` at `MAX_CARDS_IN_STACK` (500) and ends via the "no more
cards" panel (no spam toast).

**Where:** `oracle_tag_examples.rs` only.

Add enforces `MAX_CARDS_IN_STACK` (500) + warning threshold. Examples can
`append` forever on a fat tag (e.g. broad otags). Long session = unbounded
memory.

**Fix:** mirror Add — refuse further `load_more` at 500 (toast optional). Can
land with or after `fetch_usable_page`.

---

### P3 — `ensure_fresh` failure is silent on examples — **DONE 2026-07-15**

Examples now toasts `e.to_user_message()` on `ensure_fresh` Err (auth failures
still fall through to the AuthGate redirect). The broader "every screen handles
this differently" cleanup is captured separately in
[`authed_error_handler.md`](authed_error_handler.md) (BACKLOG).

**Where:** `oracle_tag_examples.rs` load path.

```text
Err(_) => { clear loading flags; return; }  // no toast
```

User sees empty-state copy, not “session expired” / network error. Add usually
toasts.

**Fix:** toast on `ensure_fresh` Err and on search Err (search already toasts
in examples; ensure_fresh does not).

---

### P3 — Dictionary discoverability incomplete → **resolved by follow-through plan**

**Where:** `oracle_tag_dictionary.rs` + card filter.

1. **Hint dialog** still only explains letter rail + search + missing
   descriptions — never Examples / Use.
2. **As-built affordance** is whole-row `dict-row-tappable` — **owner rejected**
   for the two-button design; remove whole-row click (see § Dictionary
   follow-through).
3. **Card filter has no Dictionary link** — still needed so **Use** can write
   back into the filter. Selector already has a Dictionary link.

**Fix:** implement § Dictionary follow-through (buttons + return intent +
filter Dictionary link). Update `HintDialog` for **Examples** / **Use**.

---

## Dictionary follow-through (owner-decided UX)

> **Detailed design moved to [`dictionary_adopt_flow.md`](dictionary_adopt_flow.md)**
> (2026-07-15). **Approach changed to all-overlays:** the Dictionary and Examples
> become in-place overlays (like `SwipeSelect`), not routes, so the host never
> unmounts and **Use** writes the live signal directly via an `on_use(slug)`
> callback (apply + STAY + toast). This **deletes** the old A/inbox and B/deck-store
> problem entirely. The real remaining work is an **overlay back-stack** so the OS
> back gesture closes the top overlay instead of exiting the screen (the back
> handler is not overlay-aware today). The route-based summary below is stale —
> the new doc is the source of truth.

### Problem

As-built: entire `dict-row` is clickable → examples only. Owner wants a clear
**follow-through** after browsing the dictionary: not only “see what this tag
catches,” but also **adopt** the tag into the surface they came from (deck
strategy picker or card filter). Whole-row click cannot express two actions
safely; swiping right on example cards must **not** adopt the tag (page-level
intent, wrong surface).

### Buttons on each dictionary entry (not the whole row) — **owner choice, final**

Earlier soft suggestion was “list = learn only, commit only from examples.”
**Rejected.** That forces anyone who already knows the tag (or trusts the
description) to either go back and re-search in the selector/filter, or invent
a clunky “open examples → swipe right to adopt” path. **Use on the dict row is
the right primary adopt control.**

| Button label | Action |
|--------------|--------|
| **Examples** | Navigate to `OracleTagExamples { slug }` — optional peruse. |
| **Use** | Context-aware adopt (see below). Short label; avoid “Add” / “Add tag” (collides with add-cards grammar). |

**Do not use:** Example cards, eg, cards, explore (examples side); Add, Add tag,
Use as tag, Apply, Include (adopt side — too long, filter-only, or wrong
app grammar).

**Row chrome:**

- Remove `onclick` from the row container and drop `dict-row-tappable` (or
  leave CSS unused and delete the rules).
- Row body (slug, description, parents) is **display only** — not a hit target
  for navigation. **Learn** = read the row; no click required.
- Place **Examples** and **Use** as compact controls on the row (chip /
  text-button style consistent with existing filter/selector UI). Avoid
  competing with parent chips if those stay non-interactive for now.

### Intended flow (owner)

```
Filter or deck tag selector
  → Dictionary
       - Read / learn (row text only)
       - Optional: Examples → browse cards (left = next, down = back only)
       - Use → write slug into that selector/filter
```

**Use does not require visiting Examples first.** Both buttons are first-class.
Examples is “peruse if you want”; Use is “I want this tag in my selection.”

### What **Use** does (entry context)

Same label everywhere; write-back depends on **how the dictionary was opened**:

| Opened from | **Use** does |
|-------------|--------------|
| **Deck create / edit** strategy tag selector (`oracle_tag_select`) | Add slug as a deck oracle tag selection. |
| **Card filter** oracle-tags section | Add slug as an **include** (`oracle_tags_contains_any` or current any/all mode’s include list). |
| **No adopt context** (e.g. deep link / future pure-reference entry) | Hide **Use**, or show disabled with a short reason — do not invent a default deck/filter. |

### After **Use** — navigate back vs stay + toast

**Default: apply + `go_back()` (or pop to the opener).** Matches “I found it,
put it in my selection” and lands the user on the selector/filter with the
chip visible. One deliberate action, done.

**Stay + toast** (“Added to filter” / “Added to deck tags”) is better only if
we expect multi-tag shopping from the dictionary in one sitting without
re-opening. That is a real pattern (browse letter A, use three tags). If we
want that later: **Use** applies, toast confirms, stay on dictionary; Back
returns with all applied tags. Not required for v1.

**v1 recommendation:** **apply + navigate back.** Simpler mental model, no
“did it stick?” ambiguity, no multi-select state to explain. Revisit stay+toast
if multi-pick from dictionary is a common pain.

Implementation sketch:

- Pass a **return intent** when pushing Dictionary (query param, route enum
  variant, or a short-lived signal/context set by the opener before push).
- On **Use**: apply slug to the right store (deck form state / `FilterStore` /
  selector selection), then `navigator.go_back()` (or pop to the known route).

### Rejected alternatives

| Idea | Why not |
|------|---------|
| No Use on dict — go back and search in selector | Clunky; dictionary discovery dies at the last mile |
| Row click → examples → swipe right adopts | Wrong surface (swipe is per-card); fights left/down-only grammar; easy mis-swipe |
| Use only on examples ActionBar | Forces a trip through examples to adopt a tag you already understand |

Optional later (not v1): also put **Use** on the examples ActionBar for
“I looked at cards, now commit” without returning to the list. Dict-row **Use**
remains the primary.

### Examples screen — gestures unchanged

Still only:

- **Left** = next card  
- **Down** = go back one card (undo)  
- **Right / Up** = inert (not allowed in `SwipeConfig`)

Eyeball (`RulesButton` / card details) stays; still not a swipe.

### Dictionary link on card filter (prerequisite for filter **Use**)

Card filter `OracleTags` still lacks a Dictionary entry point (backlog /
[`progress/todo.md`](../progress/todo.md)). Add a **Dictionary** control
parity with the deck strategy picker so users can open the dict with
**filter** return intent and **Use** write-back.

### Hint copy

Update dictionary `HintDialog` roughly:

- Letter rail + search (keep).
- **Examples** shows real cards for that tag.
- **Use** applies the tag to the deck or filter you came from (when available).
- Drop any “tap the row” wording.

### Verify (follow-through)

- [ ] Row body not clickable; only **Examples** / **Use** hit targets.
- [ ] **Examples** → browse; left/down work; right/up do not commit.
- [ ] From deck selector: **Use** returns with tag selected.
- [ ] From card filter (once link exists): **Use** returns with include set.
- [ ] Without context: **Use** hidden or inert (no wrong write).
- [ ] Hint text matches the two-button model.

---

### P3 / product — Auth gate now requires login for privacy + changelog

**Where:** `router.rs` + `auth_gate.rs`.

Intentional: only Login / Register / ForgotPassword are public. Previously
Changelog, PrivacyPolicy, OracleTagDictionary were reachable with no session
(dictionary intentionally public-like).

**Impact:** normal Profile → Privacy path still works when logged in. In-app
deep link to `/privacy` while logged out redirects to login. Fine if store
legal lives on the website; flag if anyone relied on unauth in-app privacy.

**Not a code bug** — confirm product intent; no change unless owner wants
Privacy (and maybe Changelog) public again under `AuthGate` exceptions.

---

### Nit — Stale doc comment on filter OracleTags

**Where:** `filter/oracle_tags.rs` component doc (~line 69):

> “curated default grid + full-catalog search…”

Module-level docs were updated; the `#[component]` doc was not.

**Fix:** one-line doc rewrite to search-only + selected chips.

---

### Nit / polish — Examples UI

- No subtitle “Example cards” under header (plan mentioned it; as-built
  dropped it). Optional.
- No Filter / Refresh in ActionBar (by design for v1 lean browse).
- Eyeball already added; no printings sheet (optional later).
- Slug via `use_signal` on mount — example→example without remount won’t
  refetch; dictionary sits between so OK. Document only.

---

### Process — Uncommitted + untested

- Three feature plans claim IMPLEMENTED but work is **local uncommitted**.
- No device/simulator QA yet.
- Progress / todo / overview not updated for this batch (only the three plan
  files + this follow-up).

**Before merge:** cargo fmt/clippy, manual QA checklist at bottom of this
doc, then commit(s) as owner prefers (one commit vs split by feature).

---

## Plan: `fetch_usable_page`

### Goal

A small shared helper that, given a page fetch and a usability predicate,
returns a page of **swipeable** cards by skipping barren pages (server
non-empty, client filter empty) up to a cap — without coupling to
`CardStack` or to a specific HTTP route.

### Ownership — decisions (final)

| Owns | Does **not** own |
|------|------------------|
| Paging loop, offset math, exhausted flag, empty-filter skip cap | `CardStack` (caller appends) |
| | Network client / session |
| | Query construction |
| | Definition of usable (caller passes `Fn(&Card) -> bool`) |

**Not** a method on `CardStack`. `CardStack` stays pure UI state
(cards / cursor / history / entering). Network + “what is usable” stay at
the screen (or a thin adapter).

### API sketch

File: `zwiper/src/lib/inbound/screens/deck/card/components/fetch_usable_page.rs`
(mod export from `components/mod.rs` next to `card_stack`).

```rust
/// Result of walking one or more server pages until usable cards appear.
pub struct UsablePage {
    /// Cards that passed `is_usable` (may be empty if exhausted / cap hit).
    pub cards: Vec<Card>,
    /// Offset to use for the *next* `fetch_usable_page` call.
    pub next_offset: u32,
    /// True when the server returned an empty page, or we hit the consecutive
    /// filter-empty cap with nothing left to try in this stretch.
    pub exhausted: bool,
}

/// Default: how many consecutive filter-empty (but server-nonempty) pages to
/// skip before giving up on this load call.
pub const MAX_EMPTY_FILTER_PAGES: u32 = 5;

/// Fetch until we have usable cards, the server is empty, or we skip
/// `max_empty_filter_pages` barren pages.
///
/// Control flow:
/// - server `[]` → exhausted, stop (1 request for a true empty filter)
/// - after `is_usable`, non-empty → return those cards, advance offset by
///   one page limit per page visited
/// - after `is_usable`, empty → bump offset, count skip; if skip >= cap →
///   exhausted (or exhausted only when cards also empty — see note below)
pub async fn fetch_usable_page<E, F, Fut>(
    start_offset: u32,
    page_limit: u32,
    max_empty_filter_pages: u32,
    is_usable: impl Fn(&Card) -> bool,
    mut fetch: F,
) -> Result<UsablePage, E>
where
    F: FnMut(u32) -> Fut,
    Fut: std::future::Future<Output = Result<Vec<Card>, E>>,
{
    // implement loop as agreed
}
```

**Offset rule:** always advance `next_offset` by `page_limit` for every
**server** page that returned non-empty (including filter-empty ones), so we
never re-fetch the same barren page. On server empty, `next_offset` stays at
the offset that returned empty (or start_offset if first page empty — either
is fine if exhausted is true).

**Exhausted when cap hit:** if stack already has cards and mid-browse hits 5
barren pages, set `exhausted = true` so near-end threshold does not spin the
same loop forever. User can leave/re-enter. Prefer simple: **always** set
`exhausted` on cap or server empty.

### `is_usable` predicate (no image clause)

Image is no longer a usability factor anywhere. The predicate Add / swipe_select
pass in is deck-based, e.g. `!in_deck(c) && !already_seen(c)`. Examples doesn't use
the helper at all (no filter).

### Wire into Add / swipe_select

Replace each screen's load-more success path with:

1. `ensure_fresh` — on Err, toast + clear loading.
2. `fetch_usable_page(offset, PAGE_LIMIT, MAX_EMPTY_FILTER_PAGES, is_usable, |off| { build query; search_deck_cards })`.
3. On Ok: `current_offset = page.next_offset`, `pagination_exhausted = page.exhausted`, if `!page.cards.is_empty() { stack.append(...) }`.

Each `load_more` call gets its own skip budget of `MAX_EMPTY_FILTER_PAGES`.

### Stack cap (P2, same PR or immediate follow)

In `load_more` before fetch: if `stack.len() >= MAX_CARDS_IN_STACK`, toast +
return (import const from `action_history`).

### Out of scope for first implementation (unless free)

- Refactoring Add / swipe_select onto `fetch_usable_page` (same API, richer
  predicate). File the work as “apply helper to add + swipe_select” after
  examples is proven.
- Printings sheet on examples eyeball.
- Optional **Use** on the examples ActionBar (dict-row **Use** is enough for v1).
- Making Privacy public again.

**In scope for this follow-up (not optional):** dictionary **Examples** + **Use**
buttons, drop whole-row click, return intent, filter Dictionary link, hint copy.

### Verify

- Unit-test the helper with a fake `fetch` queue (no HTTP):
  - page0 empty → exhausted, cards empty, 1 call
  - page0 all unusable, page1 usable → cards from page1, 2 calls
  - 5 unusable pages → exhausted, 5 calls, no 6th
  - first page usable → 1 call
- Manual: popular tag (images), thin tag, nonsense slug empty, left/down
  gestures, eyeball details, auth redirect logged out.

---

## Suggested implementation order

1. **No-image placeholder** ([`no_image_card_placeholder.md`](no_image_card_placeholder.md)):
   identity frame in `FlippableCardImage` + drop the image clause everywhere. This
   resolves examples P1 (no filter → no barren page).
2. Examples: ensure_fresh toast (P3) + stack cap (P2). Gestures stay left/down only.
3. **Dictionary follow-through:** drop whole-row click; add **Examples** +
   **Use** buttons; return-intent plumbing from selector; Dictionary link +
   intent from card filter; hint copy.
4. `fetch_usable_page.rs` + unit tests; wire into **Add / swipe_select**
   (in-deck/dup `is_usable`).
5. Nits: filter OracleTags component doc.
6. Runtime QA (simulator/device) — full checklist below. Commit.

---

## Manual QA checklist (whole uncommitted batch)

**Auth gate**

- [ ] Logged out: open app / deep link home → redirect Login.
- [ ] Login / Register / ForgotPassword render without redirect.
- [ ] Logged in: Home, decks, profile, dictionary, examples all work.
- [ ] Confirm Privacy / Changelog only from Profile when authed (as intended).

**Filter search-only**

- [ ] Oracle tags include: no curated grid; search → add chip; remove chip; any/all; clear.
- [ ] Exclude section unchanged behavior.
- [ ] Deck strategy oracle-tag picker still shows curated chips.

**Dictionary follow-through**

- [ ] Dict row body not clickable; **Examples** and **Use** are the only actions.
- [ ] **Examples** → examples screen for that slug; header label correct.
- [ ] From deck strategy Dictionary: **Use** returns with tag selected.
- [ ] From card filter Dictionary: **Use** returns with include filter set.
- [ ] Hint mentions **Examples** / **Use**, not “tap the row.”

**Examples browse**

- [ ] Left advances; down undoes; right/up do not commit (only functional swipes).
- [ ] End panel “No more cards” + down goes back.
- [ ] Zero-hit tag empty copy.
- [ ] Eyeball opens CardDetails; closes cleanly.
- [ ] After `fetch_usable_page`: no false empty when intermediate pages lack images (hard to force — rely on unit tests).
- [ ] Long browse does not grow past stack cap without warning.

---

## Quick reference — issue → fix map

| ID | Issue | Fix |
|----|--------|-----|
| P1 | Filter-empty page stalls / false exhaust | Examples: no-image placeholder (drop filter). Add/swipe_select: `fetch_usable_page` |
| P2 | Unbounded examples stack | `MAX_CARDS_IN_STACK` on load_more |
| P3 | Silent ensure_fresh | Toast on Err |
| P3 | Dictionary follow-through | **Examples** + **Use** buttons; row not clickable; return intent |
| P3 | No filter → Dictionary link | Required for filter **Use**; parity with selector |
| P3 | Privacy/changelog now authed | Product confirm only |
| Nit | Stale OracleTags component doc | One-line rewrite |
| Process | Untested / uncommitted | QA checklist + commit |

---

## As-built notes

_(fill when implementing)_
