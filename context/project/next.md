# Next Immediate Priorities 🎯

Planned work after completing current tasks.

---

## Testing & Stability

1. **Integration Tests** - Repository tests with real PostgreSQL (longer-term — requires real DB infrastructure)
   - Unit testing phase complete: filter_cards (24 tests), group_cards (15 tests), copy_max (9 tests), quantity, SwipeState (32 tests)
   - Remaining gap: outbound SQLx repositories have no test coverage (integration tests only viable path)

2. **Bug Fixes** - Layout shift after deck creation, iOS keyboard push issues (see Bugs section below)

---

---

## Enhancements

### Deck Composition & Card Management

1. **Multi-Copy Add Flow** - Right now swiping right always adds exactly 1 copy. For standard decks (CopyMax=4) the user should be able to add up to 4 copies in one action.

   **Scope:**
   - UI: After swipe-right, show a quantity picker (1–CopyMax) before confirming the add, or allow repeat swipe to increment
   - Backend: `AddDeckCard` / `CreateDeckCard` already stores a `Quantity` — the quantity just needs to be driven by user input rather than hardcoded to 1
   - Validation (see CopyMax Enforcement below)

2. **CopyMax Enforcement (Frontend + Backend)** - Currently copy limits are stored in the deck but not actively enforced during add/update operations.

   **Scope:**
   - Backend: `AddDeckCard` and `UpdateDeckCard` domain types must check the deck's `copy_max` and reject quantities that exceed it (singleton=1, standard=4). Backend is the source of truth — always defend here.
   - Frontend: Assert the same rule before making the request so UX is immediate. Frontend asserts, backend defends.
   - Migration: `CreateDeckCard` and `UpdateDeckCard` flows need to fetch the deck's `copy_max` and thread it into validation.

3. **Change Quantity in View Deck Screen** - Users need a way to adjust the quantity of a card already in their deck without removing and re-adding it.

   **Scope:**
   - Add inline quantity control to `ViewDeckCard` expandable card rows (+ / − buttons or a stepper)
   - Wire to `UpdateDeckCard` endpoint
   - Respect CopyMax enforcement (see above)
   - Optimistic update in local state; revert on error

4. **Deck Metrics View** - Aggregate stats for the current deck, useful for evaluating deck balance.

   **Scope:**
   - **Total card count** — sum of all quantities
   - **Type distribution** — creature / instant / sorcery / land / etc. counts (reuse `GroupByOption::CardType` classification)
   - **Mana curve** — CMC histogram (0 / 1 / 2 / 3 / 4 / 5 / 6+), reuse `GroupByOption::Cmc` classification
   - **Color distribution** — pie or bar breakdown by color identity
   - All computed in-memory from `Vec<Card>` already loaded in `ViewDeckCard` — no extra server round-trips

   **Implementation note:** The `GroupCards` trait already partitions by type and CMC — metrics are just `.len()` calls on those groups.

### Card Intelligence

5. **Card Keyword Categories (Import Pipeline)** - Tag cards during the Scryfall import with meaningful strategic categories that are hard to derive at query time.

   **Proposed categories:** burn, recursion, flyers, tokens, ramp, removal, counterspells, draw, tutors, board wipes, lifegain

   **Scope:**
   - Add a `keyword_categories: Vec<String>` (or JSONB) column to `card_profiles` (or a separate `card_categories` table)
   - During `zervice` upsert: scan `oracle_text` and `keywords` fields with pattern matching rules per category (e.g., "deals damage" → burn, "flying" keyword → flyers, "create a token" → tokens)
   - Expose as a filter option in `CardFilter` and add a frontend chip multi-select
   - Categories are only as good as the rules — iterate over time

   **Why import-time vs query-time:** Oracle text search is fuzzy; category tagging at import gives indexed, consistent labels the frontend can filter on without complex text queries.

6. **EDHREC Integration** - Synergy scores for the current commander's deck pool.

   **Scope:**
   - Hit EDHREC API (external, not via zervice) with the commander name/id to fetch top synergy cards + scores
   - Store nothing — fetch on demand when user opens AddDeckCard for a commander deck
   - Expose a "EDHREC synergy" sort option in the filter that reorders the card list by synergy score
   - Cards not in EDHREC data sort to the bottom
   - **Note:** Cannot use zervice for this — EDHREC is a read-only external API hit from the frontend client

### UI & Polish

7. **Deck List Screen Redesign** - Better list styling, improved layout with utility bar, visual hierarchy

8. **CardFilter Enhancements (Serve Only Playable Cards)** - Continue refining default CardFilter to exclude non-playable/non-standard cards.

### Pending Improvements
   - **tri-toggle labels** - Improve clarity of boolean filter options
     - Current: "show / hide / neither"
     - Proposed: "show / hide / any" (or "no filter")
     - Applies to: playable, digital, oversized, promo, content_warning filters

   - **Language filter refinement** - Hide language selector when using OracleCards
     - Backend infrastructure complete and ready
     - Frontend: Remove language chip UI from config.rs when OracleCards enabled
     - Keeps all backend support for future language needs

### Set Type Filter (Phase 2)
   - **set_type filter** - Filter by set classification
     - Domain: `SetType` enum or string filter
     - Default: hide `funny`, `memorabilia`, `token` set types
     - Not exposed on frontend initially

### Legality Filter (Phase 3 - Complex)
   - **legality/format filter** - Filter by format legality
     - Uses existing `Legality` and `LegalityKind` enums
     - Requires special UI handling (format + legal status)
     - Deferred - needs design work

9. **Cross-Deck Card Ownership Indicator** - Highlight cards already in other decks when browsing.
    - **Note:** Cards already in the current deck won't be served by the add-card query (filtered server-side), so this is only relevant for cards that appear in *other* decks. Lower priority.

10. **EDHREC Integration** - See Card Intelligence section above.

---

## Infrastructure

1. **Dockerized Backend Dev Environment** - Container for zerver + zervice + postgres so devs don't need to install anything locally

   **Goal:** `docker compose up` spins up the full backend stack ready to develop against

   **Scope:**
   - `Dockerfile.dev` for zerver/zervice (Arch Linux base, Rust toolchain, sqlx-cli)
   - `docker-compose.yml` wiring the app container + postgres container together
   - Postgres container replaces local postgres install (handle migrations on startup)
   - Mount source code as a volume so changes reflect without rebuilding the image
   - `.env` generation handled by compose (no manual setup)
   - Frontend (zwiper) stays native — connects to container via `BACKEND_URL=http://127.0.0.1:3000` unchanged

   **Out of scope:** zwiper/Dioxus (requires GUI libs, stays on host)

   **Companion script:** `zcripts/denv/{platform}/setup-frontend.sh` — frontend-only setup for devs using the Docker backend. Installs Dioxus GUI deps (webkit2gtk, gtk3, etc.), dioxus-cli, and writes `zwiper/.env` pointing at the container. Skips postgres, sqlx-cli, and all backend setup entirely.

---

## Bugs

1. **Layout Shift After Deck Creation** - Content shifted up ~5px (sometimes down) after saving new deck, persists across navigation

   **Reproduction steps:**
   1. Navigate to Deck List screen
   2. Click "create deck"
   3. Fill out deck information (name, commander, copy max)
   4. Click "save deck"
   5. **BUG:** ViewDeck screen renders with content shifted up ~5px (occasionally down)
   6. Click "back" to return to Deck List
   7. **BUG PERSISTS:** Decks clip into page header, appears one deck is missing
   8. Click "back" to Home, then forward to Deck List
   9. **BUG RESOLVED:** Layout returns to normal

   **Observations:**
   - Bug triggered specifically by clicking "save" on CreateDeck
   - Layout shift affects ViewDeck screen immediately after save
   - Same layout issue persists when navigating to DeckList
   - Content typically shifts UP but sometimes shifts DOWN
   - Only clears when navigating completely away and returning
   - Not related to data fetching (deck appears in list)
   - Possibly related to: CSS state pollution, signal state, commander image loading, save/navigation timing

   **Investigated (not the cause):**
   - ViewDeck padding-top: 8vh vs others using 4rem (tried changing, not the issue)
   - DeckList use_effect resource restart (intentional for fresh data)

   **Next steps to investigate:**
   - is_saving signal cleanup
   - Commander image load timing
   - DOM state between navigation transitions
   - Spinner/loading state CSS artifacts

2. **iOS Keyboard Pushes Content Down** - Content shifts down 5-10px when keyboard appears, particularly noticeable with enlarged card images

   **Reproduction steps:**
   1. Navigate to Add Deck Card screen (or any screen with text input)
   2. Click into a text filter field (e.g., card name search)
   3. **BUG:** iOS keyboard appears and pushes all content down 5-10px
   4. Card images and other content visibly shift downward
   5. Dismiss keyboard
   6. Content returns to original position

   **Observations:**
   - Long-standing bug, only recently noticeable after enlarging card images to `max-height: 400px`
   - Affects screens with text input fields (filters, search bars, etc.)
   - Viewport height changes when iOS keyboard appears, causing layout reflow
   - Related to container sizing using `h-screen` (100vh) and flexbox centering
   - Likely caused by `sticky top-0` positioning + `justify-content: center` + `h-screen`
   - When viewport shrinks (keyboard appears), centered content recalculates position

   **Context:**
   - Card images recently increased from `35vh` to `400px` max-height
   - Makes the layout shift much more visible than before
   - Verified in old commits - bug existed previously but was subtle
   - Add Deck Card screen uses: `class="sticky top-0 left-0 h-screen flex flex-col items-center overflow-y-auto"`

   **Potential solutions to investigate:**
   - Remove `sticky top-0` positioning (let content flow naturally)
   - Remove `justify-content: center` (prevents recalculation on viewport resize)
   - Use fixed pixel `height` instead of `h-screen` (100vh)
   - Use `position: fixed` instead of `sticky` for card container
   - Apply iOS-specific viewport fix: `height: 100dvh` (dynamic viewport height)

   **Requirements:**
   - Add Deck Card screen should NOT be scrollable
   - Only the card itself should move (swipe gestures)
   - Container should remain fixed while keyboard is open
