# Zite Swipe Demo — Plan

## Goal

Add an interactive swipe demo to zwipe.net that lets visitors experience the core swiping mechanic — browse real MTG cards by swiping left/right/up — without logging in or building a deck. A taste of the app on the web.

This is the first step toward eventually building the full web app into zwipe.net. For now: just cards, filters, and swiping. No auth, no deck persistence, no account.

## Current state

**zite today** is a static marketing site: landing page, about, contribute, download links, email verify/password reset flows. Built with Dioxus 0.7.4 (web platform), uses `zwipe-core` for shared types and `reqwest` for the two auth-completion API calls. No card data, no swipe components, no filter UI.

**zwiper** has the full swipe stack (`SwipeStack` component) and filter UI (`CardFilterSheet` with ~15 filter sections driven by `CardFilterBuilder`). Both are tightly coupled to:
- Authenticated sessions (`Session` / JWT bearer tokens)
- Deck context (adding cards to a deck, undo history, deck-specific filter defaults)
- The `ZwipeClient` abstraction (configured with `AppConfig.backend_url`, session-aware)

**Backend** `POST /api/card/search` requires `AuthenticatedUser` (JWT). All card endpoints sit behind the private route layer. CORS is configured via `ALLOWED_ORIGINS` env var.

## Key decisions

### 1. New public search endpoint (backend change required)

**Problem:** `search_cards` extracts `AuthenticatedUser` — no JWT, no cards. The zite has no auth flow.

**Options considered:**

| Option | Pros | Cons |
|--------|------|------|
| A. New `POST /api/public/card/search` endpoint | Clean separation, rate-limitable independently, no auth bypass | New handler + route, needs its own rate limit config |
| B. Make existing search_cards optionally authenticated | No new endpoint | Muddies the auth contract, harder to rate-limit public vs private differently |
| C. Hardcode a service-account JWT in the zite build | Zero backend changes | Security smell, JWT rotation headache, WASM bundle exposes the token |

**Decision: Option A.** New public endpoint. It reuses the exact same `CardService::search_cards` call — the handler is ~10 lines. The only differences from the private handler:
- No `AuthenticatedUser` extractor
- Aggressive per-IP rate limit via Governor (the real protection against abuse)
- Lives in `public_routes()`

### 2. CORS

`zwipe.net` must be in `ALLOWED_ORIGINS` — it likely already is since the zite's verify/reset pages already call `api.zwipe.net`. No new CORS config needed. The existing `CorsLayer` applies to all routes (public and private) uniformly.

### 3. Reuse SwipeStack component from zwiper? Copy or depend?

**Problem:** `SwipeStack` lives in `zwiper`. The zite is a separate binary crate. Options:

| Option | Pros | Cons |
|--------|------|------|
| A. Extract swipe components into zwipe-core | True code sharing | zwipe-core purity rules forbid Dioxus dependency; swipe code uses `dioxus::prelude::*` heavily |
| B. Create a shared UI crate (e.g., zwipe-ui) | Clean dependency, both crates import it | New crate, build complexity, may need feature flags for web-vs-mobile |
| C. Copy SwipeStack + gesture infra into zite | Works immediately, no dependency changes | Code duplication, drift risk |
| D. Make zite depend on zwiper as a library | Pulls in the entire app as a dependency | Way too heavy, circular concerns |

**Decision: Option C for now (copy), with Option B as the future migration path.**

Rationale: The demo needs a simplified SwipeStack — no undo, no maybeboard, just left/right swiping. The gesture infrastructure (SwipeState, SwipeConfig, Direction, Axis, TimePoint, OnTouch, OnMouse, OnSwipe) is ~500 lines and stable. Copying it means we can strip deck-specific logic and keep the zite build lean. When we later build the full web app into zwipe.net, we'll extract a shared `zwipe-ui` crate.

**What to copy (8 files from zwiper):**
- `swipe/stack.rs` — simplified (no undo/entering, only left+right)
- `swipe/state.rs` — as-is
- `swipe/config.rs` — as-is
- `swipe/direction.rs` — as-is
- `swipe/axis.rs` — as-is
- `swipe/time_point.rs` — as-is
- `swipe/ontouch.rs` — as-is
- `swipe/onmouse.rs` — as-is
- `swipe/onswipe.rs` — as-is
- Relevant CSS from `zwiper/assets/main.css` (swipe-stack classes + keyframes)

### 4. Filter UI scope for the demo

The full `CardFilterSheet` has 15 sections and depends on signals, deck context, and several client traits for populating dropdowns (artists, sets, keywords, etc.). Too heavy for a demo.

**Demo filter scope — a simple inline form, not the full sheet:**
- Name search (text input)
- Color identity (5 mana color buttons, click to toggle)
- Card type (dropdown or button group: Creature, Instant, Sorcery, Enchantment, Artifact, Land, Planeswalker)
- CMC range (two number inputs or a slider)
- Rarity (Common/Uncommon/Rare/Mythic toggle buttons)

This covers the most common use case ("show me red creatures with CMC 3-5") without needing the full filter infra. All of these map directly to `CardFilterBuilder` setters. No dropdown population endpoints needed.

### 5. Card display

The SwipeStack renders `card.scryfall_data.image_uris.large` as an `<img>`. That's a Scryfall CDN URL — no auth needed, works from any origin. Card images will load directly in the browser from Scryfall's servers.

Below the swipe stack, show a minimal card info bar:
- Card name
- Type line
- Mana cost (text, not symbols for v1)

### 6. Page structure

New route: `/demo` or `/try` — a dedicated page, not embedded in the home page.

Home page gets a new CTA button: **"try it now"** linking to the demo page. This replaces or supplements the "download on app store" focus for web visitors.

Layout:
```
┌─────────────────────────────┐
│ Nav (existing)              │
├─────────────────────────────┤
│ Filter bar (collapsible)    │
│ [name] [colors] [type] ... │
├─────────────────────────────┤
│                             │
│      ┌───────────────┐      │
│      │               │      │
│      │  Card Image   │      │
│      │  (SwipeStack) │      │
│      │               │      │
│      └───────────────┘      │
│                             │
│  Card Name · Type Line      │
│  ← skip    add →    ↑ maybe│
│                             │
├─────────────────────────────┤
│ "like this? download the    │
│  app to build real decks"   │
│ [app store] [play store]    │
├─────────────────────────────┤
│ Footer                      │
└─────────────────────────────┘
```

The swipe direction labels (skip/add/maybe) are just visual hints — in demo mode, all three directions simply advance to the next card. No deck is modified.

## File-by-file plan

All paths relative to repo root.

### Backend changes (zerver)

#### 1. Public search handler

**New file:** `zerver/src/lib/inbound/http/handlers/card/public_search_card.rs`

```rust
/// Unauthenticated card search for the web demo.
/// Hard-caps limit at 20 to prevent bulk scraping.
pub async fn public_search_cards<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(mut body): Json<CardFilter>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError>
where
    CS: CardService,
{
    // Clamp limit to prevent abuse
    body.limit = body.limit.min(PUBLIC_SEARCH_LIMIT);
    state
        .card_service
        .search_cards(&body)
        .await
        .map_err(ApiError::from)
        .map(|cards| (StatusCode::OK, Json(cards)))
}
```

Note: `CardFilter.limit` is currently private (set via builder). We'll need to either:
- Add a `pub fn clamp_limit(&mut self, max: u32)` method to `CardFilter` in zwipe-core
- Or create a wrapper that rebuilds the filter with the capped limit

The cleaner approach is a `clamp_limit` method on `CardFilter` since it doesn't violate zwipe-core purity (no new dependencies).

**Edit:** `zerver/src/lib/inbound/http/handlers/card/mod.rs` — add `pub mod public_search_card;`

#### 2. Public route registration

**Edit:** `zerver/src/lib/inbound/http/routes.rs` — inside `public_routes()`, add a `/api/public` nest:

```rust
.nest(
    "/api/public",
    Router::new().nest(
        "/card",
        Router::new().route(
            "/search",
            post(public_search_cards)
                .layer(GovernorLayer::new(public_card_search_config)),
        ),
    ),
)
```

Rate limit: 10 req/min per IP (1 request every 6 seconds, burst of 10).

#### 3. Path constant

**Edit:** `zwipe-core/src/http/paths.rs` — add:

```rust
pub fn public_search_cards_route() -> String {
    "/api/public/card/search".to_string()
}
```

#### 4. CardFilter limit clamp

**Edit:** `zwipe-core/src/domain/card/models/search_card/card_filter/mod.rs` — add a method:

```rust
/// Caps the result limit to `max`. Used by the public search endpoint
/// to prevent bulk data extraction.
pub fn clamp_limit(&mut self, max: u32) {
    self.limit = self.limit.min(max);
}
```

#### 5. SQLx prepare

```bash
cargo sqlx prepare --workspace
```

(Probably no new queries, since we're reusing `CardService::search_cards` which already has its query cached. But run it to be safe.)

### Frontend changes (zite)

#### 6. Swipe gesture infrastructure

**New directory:** `zite/src/swipe/`

Copy from `zwiper/src/lib/inbound/components/interactions/swipe/`:
- `mod.rs` — module hub (simplified re-exports)
- `state.rs` — `SwipeState` (as-is)
- `config.rs` — `SwipeConfig` (as-is)
- `direction.rs` — `Direction` enum (as-is)
- `axis.rs` — `Axis` enum (as-is)
- `time_point.rs` — `TimePoint` (as-is)
- `ontouch.rs` — `OnTouch` trait (as-is)
- `onmouse.rs` — `OnMouse` trait (as-is)
- `onswipe.rs` — `OnSwipe` trait (as-is)

These files have no zwiper-specific dependencies — they use only `dioxus::prelude::*`, `chrono`, and `dioxus::html::geometry`.

#### 7. Demo SwipeStack component

**New file:** `zite/src/swipe/stack.rs`

Simplified version of zwiper's `SwipeStack`:
- Props: `cards: Vec<Card>`, `config: SwipeConfig`, `on_swipe: EventHandler<(Direction, Card)>`
- No `entering` signal (no undo in demo)
- All three exit directions (left/right/up) fire `on_swipe` and advance
- Down-swipe disabled (not in `SwipeConfig.allowed_directions`)
- Same overlay exit animation pattern
- Same peeking stack visual

#### 8. Demo page component

**New file:** `zite/src/pages/demo.rs`

State:
- `filter_builder: Signal<CardFilterBuilder>` — drives the filter form
- `cards: Signal<Vec<Card>>` — current card buffer
- `current_index: Signal<usize>` — position in buffer
- `is_loading: Signal<bool>` — loading state
- `error: Signal<Option<String>>` — error display

Behavior:
1. On mount (and on filter change), build a `CardFilter` from the builder, POST to `{API_BASE}/api/public/card/search`, store results in `cards`, reset `current_index` to 0
2. Pass `cards[current_index..]` to `SwipeStack`
3. On swipe callback: increment `current_index`. When approaching end of buffer (e.g., 5 cards remaining), fetch next page (offset += limit) and append
4. Filter form sits above the stack, collapsible

Filter form elements (inline, not a sheet):
- Text input for name
- 5 color buttons (W/U/B/R/G) that toggle `color_identity_within`
- Card type selector
- CMC min/max inputs
- Rarity toggle buttons

#### 9. Route registration

**Edit:** `zite/src/main.rs` — add to `Route` enum:

```rust
#[route("/demo")]
Demo {},
```

Add `Demo` to the `pages` module imports.

#### 10. Home page CTA

**Edit:** `zite/src/pages/home.rs` — add a prominent button/link to `/demo`:

```rust
div { class: "demo-cta",
    Link { to: Route::Demo {}, class: "demo-btn", "try swiping now" }
}
```

Place between the tagline and the store buttons, or as a new hero-level element.

#### 11. Swipe CSS

**Edit:** `zite/assets/style.css` — copy the swipe-stack CSS block from `zwiper/assets/main.css` (lines 793-939):
- `.swipe-stack` container
- `.swipe-stack-card` base + `.top` variant
- All `@keyframes card-stack-enter-*` (may not need enter keyframes if no undo, but keep for completeness)
- All `@keyframes card-stack-exit-*`
- `.swipe-stack-exiting.*` animation bindings

Plus new styles for:
- `.demo-page` layout (flexbox column, card centered)
- `.demo-filter` bar
- `.demo-card-info` bar below the stack
- `.demo-cta` button styling
- Color identity buttons (mana colors)
- Mobile responsive breakpoints

#### 12. Dependencies

**Edit:** `zite/Cargo.toml` — add `chrono` (needed by `TimePoint` in the swipe gesture code):

```toml
chrono = { workspace = true }
```

`dioxus`, `serde`, `reqwest`, `zwipe-core`, and `web-sys` are already present.

## Data flow

```
User lands on /demo
│
├── DemoPage mounts
│   ├── CardFilterBuilder::default() (is_playable=true, language=en, etc.)
│   ├── Initial filter: set_is_token(false), set_limit(20)
│   ├── But builder.is_empty() returns true with only defaults...
│   │
│   └── DECISION: For the demo, we need a non-empty initial filter.
│       Options:
│       a) Start with a random color identity pre-selected
│       b) Start with a random card type pre-selected
│       c) Add a "browse all" mode that skips the is_empty check
│       d) Pre-populate with name_contains = "" (empty string counts as set?)
│
│       → Option (c) is cleanest: the public endpoint should accept
│         an empty filter and return random/shuffled cards. This means
│         either the public handler bypasses the is_empty validation,
│         or we use a separate "browse" builder method.
│
│       → Actually: the public handler receives a CardFilter (already built).
│         The is_empty check is in CardFilterBuilder::build(). For the demo,
│         we can set any trivial filter like set_is_playable(true) — but
│         that's already the default and doesn't count as non-empty.
│
│       → Simplest: add set_name_contains("") or a dedicated
│         CardFilterBuilder::for_browse() that sets a flag making build()
│         succeed even when "empty". OR: just pick a default like
│         color_identity_within = all 5 colors (WUBRG), which is functionally
│         "all cards" but satisfies is_empty().
│
├── POST /api/public/card/search with CardFilter
│   └── Returns Vec<Card> (up to 20)
│
├── cards signal populated, SwipeStack renders top 10
│
├── User swipes right on card A
│   ├── SwipeStack: exiting_overlay.push(A), on_swipe fires
│   ├── DemoPage: current_index += 1
│   ├── If current_index > cards.len() - 5: fetch next page
│   └── New top card is interactive immediately
│
├── User changes a filter (e.g., selects Red)
│   ├── filter_builder updated via setter
│   ├── Debounce 300ms (avoid spamming on rapid toggling)
│   ├── Build new CardFilter, POST to API
│   ├── Replace cards buffer, reset current_index to 0
│   └── SwipeStack re-renders with new cards
│
└── User reaches end of results
    └── Show "no more cards" message + "reset filters" button
```

### The "empty filter" problem

`CardFilterBuilder::build()` returns `Err(InvalidCardFilter::Empty)` when only defaults are set. For the demo's initial load, we need cards without the user having set any filter.

**Recommended solution:** Pre-select a default filter state that's visually interesting and satisfies `is_empty()`:
- Default to `color_identity_within` = all 5 colors (shows everything)
- Or default to a popular card type like Creature

This avoids modifying `CardFilterBuilder`'s validation. The demo page starts with pre-toggled filter state that the user can then modify.

## Pagination strategy

- `limit = 20` per request (hard-capped by public endpoint)
- `offset` increments by 20 on each fetch
- Prefetch next page when user is within 5 cards of buffer end
- On filter change: reset offset to 0, clear buffer, fresh fetch
- Total card pool is ~35k+ — pagination will last a very long time

## Security considerations

- **Rate limiting:** Aggressive per-IP rate limit on the public search endpoint. This is the primary protection against database abuse. Tighter than the authenticated search — exact numbers TBD but something like 5 req/30s per IP.
- **No auth tokens in WASM:** The zite never handles JWTs for the demo flow.
- **CORS:** `zwipe.net` is already in `ALLOWED_ORIGINS` (verify/reset already work). No changes needed.
- **No write operations:** The public endpoint is read-only. No card/deck mutations possible.
- **Scryfall images:** Loaded directly from Scryfall CDN (`cards.scryfall.io`). No proxying needed.

## Deployment order

1. Backend: add public search endpoint, deploy zerver
2. Verify CORS allows `zwipe.net`
3. Frontend: build and deploy zite with demo page

## Future: from demo to full web app

This demo is the seed for the full web app at zwipe.net. The progression:

1. **This plan:** Anonymous card browsing + swipe demo
2. **Auth on web:** Add login/register to zite, store session in localStorage
3. **Deck building on web:** Reuse deck creation/editing flows from zwiper
4. **Shared UI crate:** Extract `SwipeStack`, gesture infra, and filter components into `zwipe-ui` crate that both zwiper and zite depend on
5. **Full parity:** zite becomes the web version of zwiper

Each step builds on the previous. The swipe infra copied in step 1 gets replaced by the shared crate in step 4.

## Open questions for you

1. **Route name:** `/demo` or `/try` or something else?
2. **Default filter state:** Pre-select all colors (shows everything), or start with a specific color/type for visual impact?
3. **Swipe labels:** In the demo, left/right/up all just advance. Should we label them "skip/like/maybe" to preview the app's mental model, or just "swipe to browse"?
4. **Card info display:** Just name + type line, or also show mana cost / power/toughness / oracle text?
5. **CTA placement:** "Download the app" prompt — below the swipe stack always visible, or only after N swipes as a gentle nudge?

## Files changed summary

**New files:**
- `zerver/src/lib/inbound/http/handlers/card/public_search_card.rs`
- `zite/src/swipe/mod.rs`
- `zite/src/swipe/stack.rs`
- `zite/src/swipe/state.rs`
- `zite/src/swipe/config.rs`
- `zite/src/swipe/direction.rs`
- `zite/src/swipe/axis.rs`
- `zite/src/swipe/time_point.rs`
- `zite/src/swipe/ontouch.rs`
- `zite/src/swipe/onmouse.rs`
- `zite/src/swipe/onswipe.rs`
- `zite/src/pages/demo.rs`

**Edited files:**
- `zwipe-core/src/http/paths.rs` — add `public_search_cards_route()`
- `zwipe-core/src/domain/card/models/search_card/card_filter/mod.rs` — add `clamp_limit()` method
- `zerver/src/lib/inbound/http/handlers/card/mod.rs` — register public handler module
- `zerver/src/lib/inbound/http/routes.rs` — add public card search route with rate limit
- `zite/src/main.rs` — add `/demo` route
- `zite/src/pages/mod.rs` — register demo module
- `zite/src/pages/home.rs` — add "try swiping" CTA
- `zite/assets/style.css` — swipe stack styles + demo page layout
- `zite/Cargo.toml` — add `chrono` dependency
