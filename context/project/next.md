# Next Immediate Priorities 🎯

Planned work after completing current tasks.

---

## Testing & Stability

1. **Integration Tests** - Repository tests with real PostgreSQL, domain trait tests (FilterCards, GroupCards), frontend component tests

2. **Bug Fixes** - Layout shift after deck creation, iOS keyboard push issues (see Bugs section below)

---

## Refactoring

1. **Generics sweep on domain validators** - Replace `&str`-only implementations with `impl AsRef<str>` (or `impl Into<String>`) across domain validation entry points:
   - `ContainsBadWord` trait — currently only implemented for `&str`, should also cover `String` and `&String`
   - `Username::new()`, `DeckName::new()`, `Password::new()` etc. — accepting `impl AsRef<str>` avoids callers needing to call `.as_str()` on owned strings
   - Makes validators more ergonomic when working with owned data from deserializers, HTTP handlers, and database rows

---

## Enhancements

1. **Deck List Screen Redesign** - Better list styling, improved layout with utility bar, visual hierarchy

2. **CardFilter Enhancements (Serve Only Playable Cards)** - Continue refining default CardFilter to exclude non-playable/non-standard cards.

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

3. **Cross-Deck Card Ownership Indicator** - Highlight cards that are already in other decks:
    - Visual indicator when browsing cards for one deck (e.g., "In 2 other decks")
    - Show which decks contain the card
    - Helps users avoid over-buying duplicate cards

4. **Toast: Card in Other Decks** - When viewing a card that exists in other decks, show a toast notification:
    - Message options: "You use this card in other decks" or "You seem to like this card"
    - Only show when the card is being viewed for deck-building (add/remove context)

5. **EDHREC Integration** - Sort and filter cards by deck synergies:
    - Fetch synergy data from EDHREC API for current commander
    - Sort by "synergy score" or "popularity in decks with this commander"
    - Highlight cards frequently paired with the current commander
    - Helps players discover strong deck synergies

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
